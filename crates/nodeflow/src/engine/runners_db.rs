use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use super::expression::resolve_string;
use super::runner::{NodeRunContext, NodeRunner, NodeRunnerRegistry};
use super::runner_helpers::{first_input_item, make_expr_ctx, properties_lookup};
use super::types::{NodeExecutionError, NodeItem, NodeRunOutcome};

pub fn register_db_runners(registry: &mut NodeRunnerRegistry) {
  registry.register("sqlite_query", Arc::new(SqliteRunner));
}

pub struct SqliteRunner;

#[async_trait]
impl NodeRunner for SqliteRunner {
  async fn run(&self, ctx: NodeRunContext<'_>) -> Result<NodeRunOutcome, NodeExecutionError> {
    let props = ctx.node.properties.clone();
    let expr_ctx = make_expr_ctx(ctx.exec, first_input_item(&ctx.inputs), &ctx.node.id);
    let database_path = resolve_string(&properties_lookup(&props, "database_path"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving database_path: {error}")))?;
    let query = resolve_string(&properties_lookup(&props, "query"), &expr_ctx, ctx.credentials)
      .map_err(|error| NodeExecutionError::Runtime(format!("resolving query: {error}")))?;
    if database_path.is_empty() {
      return Err(NodeExecutionError::Runtime("sqlite_query: `database_path` is empty".to_string()));
    }
    if query.trim().is_empty() {
      return Err(NodeExecutionError::Runtime("sqlite_query: `query` is empty".to_string()));
    }

    let path = PathBuf::from(&database_path);
    let rows = tokio::task::spawn_blocking(move || sqlite_execute(&path, &query))
      .await
      .map_err(|error| NodeExecutionError::Runtime(format!("sqlite task join: {error}")))?
      .map_err(|error| NodeExecutionError::Runtime(format!("sqlite: {error}")))?;

    let count = rows.len();
    let mut outputs = HashMap::new();
    outputs.insert("rows".to_string(), rows.into_iter().map(NodeItem::from_json).collect());
    Ok(NodeRunOutcome { outputs, logs: vec![format!("SQLite returned {count} rows")] })
  }
}

fn sqlite_execute(path: &PathBuf, query: &str) -> anyhow::Result<Vec<Value>> {
  let connection = rusqlite::Connection::open(path)?;
  let trimmed = query.trim();
  let upper = trimmed.to_uppercase();

  if upper.starts_with("SELECT") || upper.starts_with("WITH") {
    let mut statement = connection.prepare(trimmed)?;
    let column_names: Vec<String> = statement.column_names().into_iter().map(ToOwned::to_owned).collect();
    let rows_iter = statement.query_map([], |row| {
      let mut map = serde_json::Map::new();
      for (index, name) in column_names.iter().enumerate() {
        let value: rusqlite::types::Value = row.get(index)?;
        map.insert(name.clone(), sqlite_value_to_json(value));
      }
      Ok(Value::Object(map))
    })?;
    let mut rows = Vec::new();
    for row in rows_iter {
      rows.push(row?);
    }
    Ok(rows)
  } else {
    let changed = connection.execute(trimmed, [])?;
    Ok(vec![json!({ "changes": changed })])
  }
}

fn sqlite_value_to_json(value: rusqlite::types::Value) -> Value {
  match value {
    rusqlite::types::Value::Null => Value::Null,
    rusqlite::types::Value::Integer(int) => Value::Number(int.into()),
    rusqlite::types::Value::Real(float) => serde_json::Number::from_f64(float).map(Value::Number).unwrap_or(Value::Null),
    rusqlite::types::Value::Text(text) => Value::String(text),
    rusqlite::types::Value::Blob(bytes) => {
      use base64::Engine as _;
      Value::String(base64::engine::general_purpose::STANDARD.encode(bytes))
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sqlite_executes_select_and_insert() {
    let path = PathBuf::from("/tmp").join(format!("nodeflow-sqlite-test-{}.db", uuid::Uuid::new_v4()));
    let setup = sqlite_execute(&path, "CREATE TABLE items(id INTEGER, name TEXT)").unwrap();
    assert_eq!(setup[0]["changes"], 0);
    sqlite_execute(&path, "INSERT INTO items(id, name) VALUES (1, 'alpha')").unwrap();
    let rows = sqlite_execute(&path, "SELECT id, name FROM items").unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["name"], "alpha");
    let _ = std::fs::remove_file(path);
  }
}
