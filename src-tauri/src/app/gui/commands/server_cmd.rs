use std::sync::Arc;

use sysinfo::{ProcessExt, System, SystemExt};
use tokio::sync::Mutex;
use tracing::info;

// Helper to convert u32 PID to platform-specific Pid type
fn pid_to_syspid(pid: u32) -> sysinfo::Pid {
    sysinfo::Pid::from(pid as usize)
}

use crate::app::server::{self, ServerConfig, ServerInstance, ServerStatus};

// Global map of running server instances
static RUNNING_SERVERS: once_cell::sync::Lazy<
    Arc<Mutex<std::collections::HashMap<String, Arc<Mutex<ServerInstance>>>>>,
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(std::collections::HashMap::new())));

// Player history tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerHistoryEntry {
    pub player_name: String,
    pub action: String, // "join" or "leave"
    pub timestamp: String,
}

static PLAYER_HISTORY: once_cell::sync::Lazy<
    Arc<Mutex<std::collections::HashMap<String, Vec<PlayerHistoryEntry>>>>
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(std::collections::HashMap::new())));

// Metrics history for real-time charts (last 60 samples)
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub timestamp: String,
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub tps: f32,
    pub player_count: u32,
}

static METRICS_HISTORY: once_cell::sync::Lazy<
    Arc<Mutex<std::collections::HashMap<String, Vec<MetricsSnapshot>>>>
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(std::collections::HashMap::new())));

pub(crate) async fn get_running_servers() -> Vec<String> {
    let running = RUNNING_SERVERS.lock().await;
    running.keys().cloned().collect()
}

#[tauri::command]
pub(crate) async fn server_list() -> Result<Vec<ServerConfig>, String> {
    server::load_servers().await;
    Ok(server::load_servers().await)
}

#[tauri::command]
pub(crate) async fn server_create(config: ServerConfig) -> Result<ServerConfig, String> {
    let mut servers = server::load_servers().await;
    let mut cfg = config;

    // Generate a new ID
    cfg.id = uuid::Uuid::new_v4().to_string();

    // Download server JAR
    info!("Downloading server JAR for {} ({})", cfg.name, cfg.server_type.label());
    server::download_server_jar(&cfg).await?;

    servers.push(cfg.clone());
    server::save_servers(&servers).await?;

    Ok(cfg)
}

#[tauri::command]
pub(crate) async fn server_delete(server_id: String) -> Result<(), String> {
    // Stop if running
    {
        let running = RUNNING_SERVERS.lock().await;
        if let Some(instance) = running.get(&server_id) {
            let instance = instance.clone();
            drop(running);
            server::stop_server(&instance).await?;
        }
    }

    let mut servers = server::load_servers().await;
    servers.retain(|s| s.id != server_id);
    server::save_servers(&servers).await
}

#[tauri::command]
pub(crate) async fn server_update(config: ServerConfig) -> Result<(), String> {
    let mut servers = server::load_servers().await;
    if let Some(existing) = servers.iter_mut().find(|s| s.id == config.id) {
        *existing = config;
    }
    server::save_servers(&servers).await
}

#[tauri::command]
pub(crate) async fn server_start(server_id: String) -> Result<(), String> {
    let servers = server::load_servers().await;
    let config = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?
        .clone();

    let instance = Arc::new(Mutex::new(ServerInstance::new(config)));
    server::start_server(&instance).await?;

    let mut running = RUNNING_SERVERS.lock().await;
    running.insert(server_id, instance);

    Ok(())
}

#[tauri::command]
pub(crate) async fn server_stop(server_id: String) -> Result<(), String> {
    let mut running = RUNNING_SERVERS.lock().await;
    if let Some(instance) = running.remove(&server_id) {
        drop(running);
        server::stop_server(&instance).await?;
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn server_status(
    server_id: String,
) -> Result<Option<ServerStatus>, String> {
    let running = RUNNING_SERVERS.lock().await;
    if let Some(instance) = running.get(&server_id) {
        let inst = instance.lock().await;
        let uptime = inst
            .started_at
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);

        let pid = inst.process.as_ref().and_then(|p| p.id());
        
        // Get memory and CPU usage
        let (memory_mb, max_memory_mb, cpu_percent, tps) = if let Some(pid) = pid {
            let mut sys = System::new_all();
            sys.refresh_processes_specifics(sysinfo::ProcessRefreshKind::everything());
            let sys_pid = pid_to_syspid(pid);
            if let Some(process) = sys.process(sys_pid) {
                let mem: u64 = process.memory();
                let mem_mb = mem / 1024 / 1024;
                let cpu = process.cpu_usage();
                let tps = estimate_tps_from_logs(&inst).await;
                (mem_mb, inst.config.ram_mb, cpu, tps)
            } else {
                (0, inst.config.ram_mb, 0.0, 20.0)
            }
        } else {
            (0, inst.config.ram_mb, 0.0, 20.0)
        };

        // Try to get player count from logs
        let (player_count, online_players) = estimate_players(&inst).await;
        
        // Track player history from log parsing
        track_player_history(&inst, &server_id).await;

        // Record metrics snapshot
        record_metrics(&server_id, cpu_percent, memory_mb, tps, player_count).await;

        Ok(Some(ServerStatus {
            id: inst.config.id.clone(),
            running: inst.process.is_some(),
            pid,
            port: inst.config.port,
            online_players,
            uptime_secs: uptime,
            memory_mb,
            max_memory_mb,
            cpu_percent,
            tps,
            player_count,
            max_players: inst.config.max_players,
            version: inst.config.mc_version.clone(),
            motd: inst.config.motd.clone(),
        }))
    } else {
        Ok(None)
    }
}

async fn track_player_history(inst: &ServerInstance, server_id: &str) {
    let log_lock = inst.log.lock().await;
    let mut history = PLAYER_HISTORY.lock().await;
    let entries = history.entry(server_id.to_string()).or_default();

    for line in log_lock.iter() {
        // Check for join messages
        if let Some(name) = extract_player_name(line, "joined the game") {
            if !entries.iter().any(|e| e.player_name == name && e.action == "join") {
                entries.push(PlayerHistoryEntry {
                    player_name: name,
                    action: "join".to_string(),
                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                });
            }
        }
        // Check for leave messages
        if let Some(name) = extract_player_name(line, "left the game") {
            if !entries.iter().any(|e| e.player_name == name && e.action == "leave") {
                entries.push(PlayerHistoryEntry {
                    player_name: name,
                    action: "leave".to_string(),
                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                });
            }
        }
    }

    // Keep only last 1000 entries
    if entries.len() > 1000 {
        entries.drain(0..entries.len() - 1000);
    }
}

async fn estimate_players(inst: &ServerInstance) -> (u32, Vec<String>) {
    let log_lock = inst.log.lock().await;
    let recent_logs: Vec<&String> = log_lock.iter().rev().take(200).collect();
    
    let mut players = std::collections::HashSet::new();
    for line in &recent_logs {
        if let Some(name) = extract_player_name(line, "joined the game") {
            players.insert(name);
        }
    }
    
    // Remove players who left
    for line in &recent_logs {
        if let Some(name) = extract_player_name(line, "left the game") {
            players.remove(&name);
        }
    }

    // Filter out entries from "left" messages
    let online: Vec<String> = players.into_iter().collect();
    let count = online.len() as u32;
    (count, online)
}

async fn estimate_tps_from_logs(inst: &ServerInstance) -> f32 {
    let log_lock = inst.log.lock().await;
    let recent_logs: Vec<&String> = log_lock.iter().rev().take(50).collect();

    for line in recent_logs {
        // Try to find TPS from JSON-style log messages
        if let Some(tps_start) = line.find("\"tps\":") {
            let after = &line[tps_start + 6..];
            if let Some(end) = after.find(&[',', '}', ' '][..]) {
                if let Ok(tps) = after[..end].trim().parse::<f32>() {
                    return tps;
                }
            }
        }
        // Try to find TPS from text log messages
        if let Some(pos) = line.find("TPS: ") {
            let after = &line[pos + 5..];
            if let Some(end) = after.find(&[',', ' ', ']', '}'][..]) {
                if let Ok(tps) = after[..end].trim().parse::<f32>() {
                    return tps;
                }
            }
        }
        // Try to find "Overall TPS: " pattern
        if let Some(pos) = line.find("Overall TPS: ") {
            let after = &line[pos + 13..];
            if let Some(end) = after.find(&[',', ' ', ']', '}'][..]) {
                if let Ok(tps) = after[..end].trim().parse::<f32>() {
                    return tps;
                }
            }
        }
    }
    20.0
}

fn extract_player_name(line: &str, pattern: &str) -> Option<String> {
    if let Some(idx) = line.find(pattern) {
        let before = &line[..idx];
        if let Some(last_space) = before.rfind(' ') {
            let name = before[last_space+1..].trim().to_string();
            if !name.is_empty() && name != "*" && name != ":" && name != "[" {
                return Some(name);
            }
        }
    }
    None
}

#[tauri::command]
pub(crate) async fn server_player_history(server_id: String) -> Result<Vec<PlayerHistoryEntry>, String> {
    let history = PLAYER_HISTORY.lock().await;
    let entries = history.get(&server_id).cloned().unwrap_or_default();
    Ok(entries)
}

#[tauri::command]
pub(crate) async fn server_metrics(server_id: String) -> Result<Vec<MetricsSnapshot>, String> {
    let metrics = METRICS_HISTORY.lock().await;
    let history = metrics.get(&server_id).cloned().unwrap_or_default();
    Ok(history)
}

async fn record_metrics(server_id: &str, cpu: f32, memory: u64, tps: f32, players: u32) {
    let mut metrics = METRICS_HISTORY.lock().await;
    let history = metrics.entry(server_id.to_string()).or_default();

    history.push(MetricsSnapshot {
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        cpu_percent: cpu,
        memory_mb: memory,
        tps,
        player_count: players,
    });

    // Keep last 120 entries (about 4 minutes at 2s intervals)
    if history.len() > 120 {
        history.drain(0..history.len() - 120);
    }
}

#[tauri::command]
pub(crate) async fn server_logs(
    server_id: String,
    since: usize,
) -> Result<Vec<String>, String> {
    let running = RUNNING_SERVERS.lock().await;
    if let Some(instance) = running.get(&server_id) {
        let inst = instance.lock().await;
        let log_lock = inst.log.lock().await;
        let logs = if since > 0 && since <= log_lock.len() {
            log_lock[since - 1..].to_vec()
        } else {
            log_lock.clone()
        };
        Ok(logs)
    } else {
        Ok(vec!["Server is not running".to_string()])
    }
}

#[tauri::command]
pub(crate) async fn server_send_command(
    server_id: String,
    command: String,
) -> Result<(), String> {
    let running = RUNNING_SERVERS.lock().await;
    if let Some(instance) = running.get(&server_id) {
        let instance = instance.clone();
        drop(running);
        server::send_command(&instance, &command).await
    } else {
        Err("Server is not running".to_string())
    }
}

#[tauri::command]
pub(crate) async fn server_share_info(server_id: String) -> Result<serde_json::Value, String> {
    let servers = server::load_servers().await;
    let config = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or("Server not found")?;

    Ok(server::get_connection_info(config))
}

#[tauri::command]
pub(crate) async fn server_is_any_running() -> Result<bool, String> {
    let running = RUNNING_SERVERS.lock().await;
    Ok(!running.is_empty())
}
