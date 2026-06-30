use tauri::{Emitter, Window};
use tracing::info;

use crate::minecraft::auth::MinecraftAccount;

#[tauri::command]
pub(crate) async fn login_microsoft(window: Window) -> Result<MinecraftAccount, String> {
    let account = MinecraftAccount::auth_msa(|_uri, code| {
        let _ = window.emit("microsoft_code", code);
    })
        .await
        .map_err(|e| format!("{}", e))?;

    Ok(account)
}

#[tauri::command]
pub(crate) async fn refresh(account_data: MinecraftAccount) -> Result<MinecraftAccount, String> {
    info!("Refreshing account...");
    let account = account_data
        .refresh()
        .await
        .map_err(|e| format!("unable to refresh: {:?}", e))?;
    info!(
        "Account was refreshed - username {}",
        account.get_username()
    );
    Ok(account)
}

#[tauri::command]
pub(crate) async fn logout(account_data: MinecraftAccount) -> Result<(), String> {
    account_data
        .logout()
        .await
        .map_err(|e| format!("unable to logout: {:?}", e))
}

#[tauri::command]
pub(crate) async fn client_account_update(
    _client: crate::app::client_api::Client,
    account: Option<serde_json::Value>,
) -> Result<Option<serde_json::Value>, String> {
    Ok(account)
}
