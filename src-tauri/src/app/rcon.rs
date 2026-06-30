use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::info;

const RCON_PACKET_AUTH: i32 = 3;
const RCON_PACKET_COMMAND: i32 = 2;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RconPlayer {
    pub name: String,
    pub uuid: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RconStatus {
    pub connected: bool,
    pub address: String,
}

pub struct RconConnection {
    stream: Option<TcpStream>,
    address: String,
    password: String,
    request_id: i32,
}

impl RconConnection {
    pub fn new(address: &str, port: u16, password: &str) -> Self {
        Self {
            stream: None,
            address: format!("{}:{}", address, port),
            password: password.to_string(),
            request_id: 0,
        }
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        let mut stream = TcpStream::connect(&self.address)
            .await
            .map_err(|e| format!("Failed to connect to RCON at {}: {}", self.address, e))?;

        self.request_id += 1;
        let auth_packet = self.build_packet(self.request_id, RCON_PACKET_AUTH, &self.password);
        stream.write_all(&auth_packet).await
            .map_err(|e| format!("Failed to send RCON auth: {}", e))?;

        let response = self.read_packet(&mut stream).await?;

        if response.request_id == -1 {
            return Err("RCON authentication failed: wrong password".to_string());
        }

        self.stream = Some(stream);
        info!("Connected to RCON at {}", self.address);
        Ok(())
    }

    pub async fn send_command(&mut self, command: &str) -> Result<String, String> {
        self.request_id += 1;
        let packet = self.build_packet(self.request_id, RCON_PACKET_COMMAND, command);

        let stream = self.stream.as_mut()
            .ok_or("RCON not connected".to_string())?;
        stream.write_all(&packet).await
            .map_err(|e| format!("Failed to send RCON command: {}", e))?;

        // Read response directly without calling self.read_packet to avoid borrow conflict
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await
            .map_err(|e| format!("Failed to read packet length: {}", e))?;

        let packet_len = i32::from_le_bytes(len_buf) as usize;
        if packet_len < 8 || packet_len > 65536 {
            return Err(format!("Invalid RCON packet length: {}", packet_len));
        }

        let mut data = vec![0u8; packet_len];
        stream.read_exact(&mut data).await
            .map_err(|e| format!("Failed to read packet data: {}", e))?;

        let body_end = data[8..].iter().position(|&b| b == 0).unwrap_or(data.len() - 8);
        let body = String::from_utf8_lossy(&data[8..8 + body_end]).to_string();

        Ok(body)
    }

    pub async fn get_player_list(&mut self) -> Result<Vec<RconPlayer>, String> {
        let response = self.send_command("list").await?;
        let mut players = Vec::new();

        if let Some(colon_idx) = response.find(':') {
            let names_part = response[colon_idx + 1..].trim();
            if !names_part.is_empty() {
                for name in names_part.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                    players.push(RconPlayer {
                        name: name.to_string(),
                        uuid: String::new(),
                    });
                }
            }
        }

        Ok(players)
    }

    pub async fn get_player_info(&mut self, player_name: &str) -> Result<String, String> {
        self.send_command(&format!("data get entity {}", player_name)).await
    }

    pub async fn kick_player(&mut self, player_name: &str, reason: &str) -> Result<String, String> {
        self.send_command(&format!("kick {} {}", player_name, reason)).await
    }

    pub async fn ban_player(&mut self, player_name: &str, reason: &str) -> Result<String, String> {
        self.send_command(&format!("ban {} {}", player_name, reason)).await
    }

    pub async fn pardon_player(&mut self, player_name: &str) -> Result<String, String> {
        self.send_command(&format!("pardon {}", player_name)).await
    }

    pub async fn op_player(&mut self, player_name: &str) -> Result<String, String> {
        self.send_command(&format!("op {}", player_name)).await
    }

    pub async fn deop_player(&mut self, player_name: &str) -> Result<String, String> {
        self.send_command(&format!("deop {}", player_name)).await
    }

    pub async fn whitelist_add(&mut self, player_name: &str) -> Result<String, String> {
        self.send_command(&format!("whitelist add {}", player_name)).await
    }

    pub async fn whitelist_remove(&mut self, player_name: &str) -> Result<String, String> {
        self.send_command(&format!("whitelist remove {}", player_name)).await
    }

    pub async fn save_all(&mut self) -> Result<String, String> {
        self.send_command("save-all").await
    }

    pub async fn disconnect(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
        info!("Disconnected from RCON at {}", self.address);
    }

    fn build_packet(&self, request_id: i32, packet_type: i32, body: &str) -> Vec<u8> {
        let body_bytes = body.as_bytes();
        let payload_len = 4 + 4 + body_bytes.len() + 2;
        let mut packet = Vec::with_capacity(4 + payload_len);

        packet.extend_from_slice(&(payload_len as i32).to_le_bytes());
        packet.extend_from_slice(&request_id.to_le_bytes());
        packet.extend_from_slice(&packet_type.to_le_bytes());
        packet.extend_from_slice(body_bytes);
        packet.extend_from_slice(&[0u8, 0u8]); // null terminator + padding

        packet
    }

    async fn read_packet(&self, stream: &mut TcpStream) -> Result<RconPacket, String> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await
            .map_err(|e| format!("Failed to read packet length: {}", e))?;

        let packet_len = i32::from_le_bytes(len_buf) as usize;
        if packet_len < 8 || packet_len > 65536 {
            return Err(format!("Invalid RCON packet length: {}", packet_len));
        }

        let mut data = vec![0u8; packet_len];
        stream.read_exact(&mut data).await
            .map_err(|e| format!("Failed to read packet data: {}", e))?;

        let request_id = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let body_end = data[8..].iter().position(|&b| b == 0).unwrap_or(data.len() - 8);
        let body = String::from_utf8_lossy(&data[8..8 + body_end]).to_string();

        Ok(RconPacket { request_id, body })
    }
}

#[allow(dead_code)]
struct RconPacket {
    request_id: i32,
    body: String,
}
