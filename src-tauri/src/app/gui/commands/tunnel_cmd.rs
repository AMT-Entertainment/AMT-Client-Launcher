use std::sync::Arc;

use tokio::sync::Mutex;

use crate::app::tunnel::{TunnelClient, TunnelConfig};

// Global active tunnels
static ACTIVE_TUNNELS: once_cell::sync::Lazy<
    Arc<Mutex<std::collections::HashMap<String, TunnelClient>>>,
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(std::collections::HashMap::new())));

#[tauri::command]
pub(crate) async fn tunnel_start(
    server_id: String,
    local_port: u16,
    relay_host: Option<String>,
    relay_port: Option<u16>,
) -> Result<String, String> {
    let config = TunnelConfig {
        enabled: true,
        local_port,
        relay_host: relay_host.unwrap_or_else(|| "127.0.0.1".to_string()),
        relay_port: relay_port.unwrap_or(42070),
    };

    let client = TunnelClient::new(server_id.clone(), config);
    let public_addr = client.start().await?;

    let mut tunnels = ACTIVE_TUNNELS.lock().await;
    tunnels.insert(server_id, client);

    Ok(public_addr)
}

#[tauri::command]
pub(crate) async fn tunnel_stop(server_id: String) -> Result<(), String> {
    let mut tunnels = ACTIVE_TUNNELS.lock().await;
    if let Some(client) = tunnels.remove(&server_id) {
        client.stop().await;
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn tunnel_status(server_id: String) -> Result<bool, String> {
    let tunnels = ACTIVE_TUNNELS.lock().await;
    Ok(tunnels.contains_key(&server_id))
}
