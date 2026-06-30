use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tracing::{info, warn};

const DEFAULT_RELAY_HOST: &str = "127.0.0.1";
const DEFAULT_RELAY_PORT: u16 = 42070;
const TUNNEL_VERSION: &str = "amt-v1";

pub struct TunnelConfig {
    pub enabled: bool,
    pub relay_host: String,
    pub relay_port: u16,
    pub local_port: u16,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            relay_host: DEFAULT_RELAY_HOST.to_string(),
            relay_port: DEFAULT_RELAY_PORT,
            local_port: 25565,
        }
    }
}

pub struct TunnelClient {
    config: TunnelConfig,
    server_id: String,
    running: Arc<Mutex<bool>>,
}

impl TunnelClient {
    pub fn new(server_id: String, config: TunnelConfig) -> Self {
        Self {
            config,
            server_id,
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) -> Result<String, String> {
        let running = self.running.clone();
        *running.lock().await = true;

        let relay_addr = format!("{}:{}", self.config.relay_host, self.config.relay_port);
        let local_port = self.config.local_port;
        let server_id = self.server_id.clone();
        let running_clone = running.clone();

        info!("Connecting to tunnel relay at {}", relay_addr);

        // Try UPnP first
        let upnp_success = try_upnp_mapping(local_port).await;
        if upnp_success {
            info!("UPnP port mapping successful on port {}", local_port);
            let public_ip = get_public_ip().await.unwrap_or_else(|| "127.0.0.1".to_string());
            return Ok(format!("{}:{}", public_ip, local_port));
        }

        warn!("UPnP failed, falling back to TCP tunnel relay");

        // Connect to relay
        let mut stream = TcpStream::connect(&relay_addr)
            .await
            .map_err(|e| format!("Failed to connect to relay: {}", e))?;

        // Send registration
        let register_msg = format!(
            "REGISTER {} {} {}\n",
            TUNNEL_VERSION, server_id, local_port
        );
        stream
            .write_all(register_msg.as_bytes())
            .await
            .map_err(|e| format!("Failed to register with relay: {}", e))?;

        // Read assigned public address
        let mut buf = vec![0u8; 1024];
        let n = stream
            .read(&mut buf)
            .await
            .map_err(|e| format!("Failed to read relay response: {}", e))?;

        let response = String::from_utf8_lossy(&buf[..n]).to_string();
        info!("Tunnel relay response: {}", response);

        if let Some(public_addr) = response.strip_prefix("OK ") {
            let public_addr = public_addr.trim().to_string();
            let (mut reader, writer) = stream.into_split();
            let running = running_clone.clone();

            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                while *running.lock().await {
                    match reader.read(&mut buf).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            let msg = String::from_utf8_lossy(&buf[..n]);
                            if msg.starts_with("CONNECT ") {
                                if let Ok(mut relay) = TcpStream::connect(&relay_addr).await {
                                    let forward_msg =
                                        format!("FORWARD {} {}\n", server_id, local_port);
                                    let _ = relay.write_all(forward_msg.as_bytes()).await;
                                    if let Ok(local) =
                                        TcpStream::connect(format!("127.0.0.1:{}", local_port)).await
                                    {
                                        let (mut r_relay, mut w_relay) = relay.into_split();
                                        let (mut r_local, mut w_local) = local.into_split();
                                        let _ = tokio::join!(
                                            tokio::io::copy(&mut r_relay, &mut w_local),
                                            tokio::io::copy(&mut r_local, &mut w_relay),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                drop(writer);
            });

            return Ok(public_addr);
        }

        Err(format!("Relay registration failed: {}", response))
    }

    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;

        remove_upnp_mapping(self.config.local_port).await;
    }
}

async fn try_upnp_mapping(port: u16) -> bool {
    match tokio::task::spawn_blocking(move || {
        match igd::search_gateway(Default::default()) {
            Ok(gateway) => {
                let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
                match gateway.add_any_port(igd::PortMappingProtocol::TCP, addr, 0, "AMT Client Server") {
                    Ok(_) => {
                        info!("UPnP: Successfully mapped port {}", port);
                        true
                    }
                    Err(e) => {
                        warn!("UPnP: Failed to map port {}: {}", port, e);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("UPnP: No gateway found: {}", e);
                false
            }
        }
    })
    .await
    {
        Ok(result) => result,
        Err(e) => {
            warn!("UPnP task failed: {}", e);
            false
        }
    }
}

async fn remove_upnp_mapping(port: u16) {
    tokio::task::spawn_blocking(move || {
        if let Ok(gateway) = igd::search_gateway(Default::default()) {
            let _ = gateway.remove_port(igd::PortMappingProtocol::TCP, port);
            info!("UPnP: Removed port mapping for {}", port);
        }
    })
    .await
    .ok();
}

async fn get_public_ip() -> Option<String> {
    match crate::HTTP_CLIENT
        .get("https://api.ipify.org")
        .header("User-Agent", "AMT-Client/0.1.0")
        .send()
        .await
    {
        Ok(resp) => resp.text().await.ok(),
        Err(_) => None,
    }
}

/// Simple TCP relay server (for self-hosting)
/// Protocol:
///   REGISTER <version> <server_id> <local_port>  →  OK <public_addr>
///   FORWARD <server_id> <local_port>              →  CONNECT <peer_id>
pub async fn run_relay_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Tunnel relay listening on port {}", port);

    let tunnels: Arc<Mutex<std::collections::HashMap<String, TcpStream>>> =
        Arc::new(Mutex::new(std::collections::HashMap::new()));

    loop {
        let (mut stream, addr) = listener.accept().await?;
        let tunnels = tunnels.clone();

        tokio::spawn(async move {
            let peer_id = uuid::Uuid::new_v4().to_string();
            let mut buf = vec![0u8; 1024];

            match stream.read(&mut buf).await {
                Ok(0) | Err(_) => return,
                Ok(n) => {
                    let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                    let parts: Vec<&str> = msg.trim().split_whitespace().collect();

                    match parts.first().copied() {
                        Some("REGISTER") if parts.len() >= 3 => {
                            let server_id = parts[2].to_string();
                            info!("Relay: {} registered tunnel '{}'", addr, server_id);
                            let response = format!("OK {}:{}\n", addr.ip(), parts.get(3).unwrap_or(&"25565"));
                            let mut stream = stream;
                            let _ = stream.write_all(response.as_bytes()).await;
                            tunnels.lock().await.insert(server_id, stream);
                        }
                        Some("FORWARD") if parts.len() >= 2 => {
                            let server_id = parts[1].to_string();
                            let mut tunnels_lock = tunnels.lock().await;

                            if let Some(mut tunnel_stream) = tunnels_lock.remove(&server_id) {
                                let connect_msg = format!("CONNECT {}\n", peer_id);
                                if tunnel_stream.write_all(connect_msg.as_bytes()).await.is_ok() {
                                    info!("Relay: forwarded connection to {}", server_id);
                                    let (mut r_t, mut w_t) = tunnel_stream.into_split();
                                    let (mut r_s, mut w_s) = stream.into_split();
                                    let _ = tokio::join!(
                                        tokio::io::copy(&mut r_t, &mut w_s),
                                        tokio::io::copy(&mut r_s, &mut w_t),
                                    );
                                }
                            } else {
                                let _ = stream.write_all(b"ERROR tunnel not found\n").await;
                            }
                        }
                        _ => {
                            let _ = stream.write_all(b"ERROR Unknown command\n").await;
                        }
                    }
                }
            }
        });
    }
}
