use crate::app::github;
use crate::app::options::AMTOptions;

#[tauri::command]
pub(crate) async fn github_auth_url(amt_options: AMTOptions) -> Result<String, String> {
    if amt_options.github_client_id.is_empty() {
        return Err("GitHub Client ID not configured. Please set it in Settings > Accounts.".to_string());
    }
    Ok(github::get_auth_url(&amt_options.github_client_id))
}

#[tauri::command]
pub(crate) async fn github_exchange_code(code: String, amt_options: AMTOptions) -> Result<String, String> {
    if amt_options.github_client_id.is_empty() || amt_options.github_client_secret.is_empty() {
        return Err("GitHub OAuth not configured. Please set Client ID and Secret in Settings > Accounts.".to_string());
    }
    github::exchange_code(&code, &amt_options.github_client_id, &amt_options.github_client_secret).await
}

#[tauri::command]
pub(crate) async fn github_fetch_user(token: String) -> Result<github::GithubUser, String> {
    github::fetch_user(&token).await
}

#[tauri::command]
pub(crate) async fn github_dispatch(
    token: String,
    event_type: String,
    payload: serde_json::Value,
    repo_owner: String,
    repo_name: String,
) -> Result<(), String> {
    github::trigger_dispatch(&token, &repo_owner, &repo_name, &event_type, payload).await
}
