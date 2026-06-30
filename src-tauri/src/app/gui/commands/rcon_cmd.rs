use crate::app::rcon::{RconConnection, RconPlayer, RconStatus};

#[tauri::command]
pub(crate) async fn rcon_connect(server_id: String, host: String, port: u16, password: String) -> Result<(), String> {
    let mut conn = RconConnection::new(&host, port, &password);
    conn.connect().await?;
    // Store in global RCON connections map
    let mut connections = RCON_CONNECTIONS.lock().await;
    connections.insert(server_id, conn);
    Ok(())
}

#[tauri::command]
pub(crate) async fn rcon_disconnect(server_id: String) -> Result<(), String> {
    let mut connections = RCON_CONNECTIONS.lock().await;
    if let Some(mut conn) = connections.remove(&server_id) {
        conn.disconnect().await;
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn rcon_send_command(server_id: String, command: String) -> Result<String, String> {
    let mut connections = RCON_CONNECTIONS.lock().await;
    let conn = connections.get_mut(&server_id).ok_or("RCON not connected")?;
    conn.send_command(&command).await
}

#[tauri::command]
pub(crate) async fn rcon_player_list(server_id: String) -> Result<Vec<RconPlayer>, String> {
    let mut connections = RCON_CONNECTIONS.lock().await;
    let conn = connections.get_mut(&server_id).ok_or("RCON not connected")?;
    conn.get_player_list().await
}

#[tauri::command]
pub(crate) async fn rcon_player_info(server_id: String, player_name: String) -> Result<String, String> {
    let mut connections = RCON_CONNECTIONS.lock().await;
    let conn = connections.get_mut(&server_id).ok_or("RCON not connected")?;
    conn.get_player_info(&player_name).await
}

#[tauri::command]
pub(crate) async fn rcon_kick(server_id: String, player_name: String, reason: String) -> Result<String, String> {
    let mut connections = RCON_CONNECTIONS.lock().await;
    let conn = connections.get_mut(&server_id).ok_or("RCON not connected")?;
    conn.kick_player(&player_name, &reason).await
}

#[tauri::command]
pub(crate) async fn rcon_ban(server_id: String, player_name: String, reason: String) -> Result<String, String> {
    let mut connections = RCON_CONNECTIONS.lock().await;
    let conn = connections.get_mut(&server_id).ok_or("RCON not connected")?;
    conn.ban_player(&player_name, &reason).await
}

#[tauri::command]
pub(crate) async fn rcon_pardon(server_id: String, player_name: String) -> Result<String, String> {
    let mut connections = RCON_CONNECTIONS.lock().await;
    let conn = connections.get_mut(&server_id).ok_or("RCON not connected")?;
    conn.pardon_player(&player_name).await
}

#[tauri::command]
pub(crate) async fn rcon_op(server_id: String, player_name: String) -> Result<String, String> {
    let mut connections = RCON_CONNECTIONS.lock().await;
    let conn = connections.get_mut(&server_id).ok_or("RCON not connected")?;
    conn.op_player(&player_name).await
}

#[tauri::command]
pub(crate) async fn rcon_deop(server_id: String, player_name: String) -> Result<String, String> {
    let mut connections = RCON_CONNECTIONS.lock().await;
    let conn = connections.get_mut(&server_id).ok_or("RCON not connected")?;
    conn.deop_player(&player_name).await
}

#[tauri::command]
pub(crate) async fn rcon_status(server_id: String) -> Result<Option<RconStatus>, String> {
    let connections = RCON_CONNECTIONS.lock().await;
    if connections.contains_key(&server_id) {
        Ok(Some(RconStatus {
            connected: true,
            address: "RCON".to_string(),
        }))
    } else {
        Ok(None)
    }
}

use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use once_cell::sync::Lazy;

pub(crate) static RCON_CONNECTIONS: Lazy<Arc<Mutex<HashMap<String, RconConnection>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

