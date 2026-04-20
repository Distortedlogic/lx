use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::crypto::{EncryptedBlob, decrypt, encrypt};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CredentialRecord {
  pub id: String,
  pub namespace: String,
  pub kind: String,
  pub label: String,
  #[serde(default)]
  pub detail: Option<String>,
  #[serde(default)]
  pub data: Value,
}

#[derive(Clone)]
pub struct CredentialStore {
  inner: Arc<RwLock<Inner>>,
}

struct Inner {
  path: PathBuf,
  records: HashMap<String, CredentialRecord>,
  passphrase: Option<String>,
}

impl CredentialStore {
  pub fn file_backed() -> Result<Self> {
    let passphrase = std::env::var("NODEFLOW_CREDENTIAL_KEY").ok();
    Self::with_path_and_passphrase(default_credentials_path(), passphrase)
  }

  pub fn with_path(path: PathBuf) -> Result<Self> {
    Self::with_path_and_passphrase(path, None)
  }

  pub fn with_path_and_passphrase(path: PathBuf, passphrase: Option<String>) -> Result<Self> {
    let records = if path.exists() {
      load_records(&path, passphrase.as_deref()).with_context(|| format!("failed to load credentials from `{}`", path.display()))?
    } else {
      HashMap::new()
    };
    Ok(Self { inner: Arc::new(RwLock::new(Inner { path, records, passphrase })) })
  }

  pub fn in_memory() -> Self {
    Self { inner: Arc::new(RwLock::new(Inner { path: PathBuf::from("/dev/null"), records: HashMap::new(), passphrase: None })) }
  }

  pub fn get(&self, id: &str) -> Option<CredentialRecord> {
    self.read().records.get(id).cloned()
  }

  pub fn list(&self) -> Vec<CredentialRecord> {
    let mut records: Vec<_> = self.read().records.values().cloned().collect();
    records.sort_by(|left, right| left.id.cmp(&right.id));
    records
  }

  pub fn put(&self, record: CredentialRecord) -> Result<()> {
    let mut guard = self.write();
    guard.records.insert(record.id.clone(), record);
    save_records(&guard.path, &guard.records, guard.passphrase.as_deref())
  }

  pub fn remove(&self, id: &str) -> Result<bool> {
    let mut guard = self.write();
    let removed = guard.records.remove(id).is_some();
    if removed {
      save_records(&guard.path, &guard.records, guard.passphrase.as_deref())?;
    }
    Ok(removed)
  }

  fn read(&self) -> std::sync::RwLockReadGuard<'_, Inner> {
    self.inner.read().expect("credential store read lock poisoned")
  }

  fn write(&self) -> std::sync::RwLockWriteGuard<'_, Inner> {
    self.inner.write().expect("credential store write lock poisoned")
  }
}

fn default_credentials_path() -> PathBuf {
  let base = dirs::data_local_dir().or_else(|| dirs::home_dir().map(|home| home.join(".local").join("share"))).unwrap_or_else(|| PathBuf::from("/tmp"));
  base.join("nodeflow").join("credentials.json")
}

fn load_records(path: &PathBuf, passphrase: Option<&str>) -> Result<HashMap<String, CredentialRecord>> {
  let payload = fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
  if let Some(passphrase) = passphrase
    && let Ok(blob) = serde_json::from_str::<EncryptedBlob>(&payload)
  {
    let bytes = decrypt(&blob, passphrase)?;
    let json = String::from_utf8(bytes).context("credentials utf-8")?;
    return serde_json::from_str(&json).context("failed to parse decrypted credentials");
  }
  serde_json::from_str(&payload).with_context(|| format!("failed to parse `{}`", path.display()))
}

fn save_records(path: &PathBuf, records: &HashMap<String, CredentialRecord>, passphrase: Option<&str>) -> Result<()> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).with_context(|| format!("failed to create `{}`", parent.display()))?;
  }
  let payload = serde_json::to_string_pretty(records).context("failed to serialize credentials")?;
  if let Some(passphrase) = passphrase {
    let blob = encrypt(payload.as_bytes(), passphrase)?;
    let encrypted = serde_json::to_string_pretty(&blob).context("failed to serialize encrypted blob")?;
    return fs::write(path, encrypted).with_context(|| format!("failed to write `{}`", path.display()));
  }
  fs::write(path, payload).with_context(|| format!("failed to write `{}`", path.display()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  fn temp_path(name: &str) -> PathBuf {
    PathBuf::from("/tmp").join(format!("nodeflow-credentials-{}-{}.json", name, uuid::Uuid::new_v4()))
  }

  #[test]
  fn round_trip_put_get_list_remove() {
    let path = temp_path("round-trip");
    let store = CredentialStore::with_path(path.clone()).unwrap();

    store
      .put(CredentialRecord {
        id: "cred-http".to_string(),
        namespace: "workflow".to_string(),
        kind: "http_api".to_string(),
        label: "HTTP API".to_string(),
        detail: Some("test".to_string()),
        data: json!({ "base_url": "https://api.example.com", "api_key": "secret" }),
      })
      .unwrap();

    let fetched = store.get("cred-http").unwrap();
    assert_eq!(fetched.data["api_key"], "secret");

    let listed = store.list();
    assert_eq!(listed.len(), 1);

    assert!(store.remove("cred-http").unwrap());
    assert!(store.get("cred-http").is_none());

    let _ = fs::remove_file(&path);
  }

  #[test]
  fn reloading_file_backed_store_sees_earlier_records() {
    let path = temp_path("reload");
    {
      let store = CredentialStore::with_path(path.clone()).unwrap();
      store
        .put(CredentialRecord {
          id: "cred-a".to_string(),
          namespace: "workflow".to_string(),
          kind: "http_api".to_string(),
          label: "A".to_string(),
          detail: None,
          data: json!({}),
        })
        .unwrap();
    }
    let reopened = CredentialStore::with_path(path.clone()).unwrap();
    assert!(reopened.get("cred-a").is_some());
    let _ = fs::remove_file(&path);
  }

  #[test]
  fn encrypted_credentials_round_trip() {
    let path = temp_path("encrypted");
    {
      let store = CredentialStore::with_path_and_passphrase(path.clone(), Some("test-key".to_string())).unwrap();
      store
        .put(CredentialRecord {
          id: "cred-enc".to_string(),
          namespace: "workflow".to_string(),
          kind: "http_api".to_string(),
          label: "Encrypted".to_string(),
          detail: None,
          data: json!({ "api_key": "ultra-secret" }),
        })
        .unwrap();
    }
    let payload = fs::read_to_string(&path).unwrap();
    assert!(!payload.contains("ultra-secret"));

    let reopened = CredentialStore::with_path_and_passphrase(path.clone(), Some("test-key".to_string())).unwrap();
    assert_eq!(reopened.get("cred-enc").unwrap().data["api_key"], "ultra-secret");
    let _ = fs::remove_file(&path);
  }
}
