use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthSession {
  pub session: SessionInfo,
  pub user: UserInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
  pub id: String,
  pub user_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
  pub id: String,
  pub email: Option<String>,
  pub name: Option<String>,
}

pub async fn get_session(client: &ApiClient) -> Result<Option<AuthSession>, ApiError> {
  match client.get::<AuthSession>("/auth/get-session").await {
    Ok(session) => Ok(Some(session)),
    Err(ApiError::Http { status: 401, .. }) => Ok(None),
    Err(e) => Err(e),
  }
}
