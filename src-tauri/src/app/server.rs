use std::path::PathBuf;
use std::sync::Arc;

use chrono;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::info;

use crate::{HTTP_CLIENT, SERVERS_DIR};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerType {
    Vanilla,
    Paper,
    Fabric,
    Forge,
    NeoForge,
    Mohist,
    CatServer,
    Folia,
    Spigot,
    Purpur,
}

impl ServerType {
    pub fn label(&self) -> &str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::Paper => "Paper",
            Self::Fabric => "Fabric",
            Self::Forge => "Forge",
            Self::NeoForge => "NeoForge",
            Self::Mohist => "Mohist",
            Self::CatServer => "CatServer",
            Self::Folia => "Folia",
            Self::Spigot => "Spigot",
            Self::Purpur => "Purpur",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    #[serde(default = "default_server_id")]
    pub id: String,
    pub name: String,
    pub server_type: ServerType,
    pub mc_version: String,
    pub loader_version: Option<String>,
    pub ram_mb: u64,
    pub port: u16,
    pub motd: String,
    pub max_players: u32,
    pub online_mode: bool,
    pub pvp: bool,
    pub difficulty: String,
    pub gamemode: String,
    pub whitelist: Vec<String>,
    pub ops: Vec<String>,
    pub icon_path: Option<String>,
    pub mod_ids: Vec<String>,
    pub auto_sharing: bool,
    
    // JVM arguments
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub jvm_extra_args: String,
    
    // World settings
    pub world_name: String,
    pub level_seed: Option<String>,
    pub level_type: String,
    pub generator_settings: Option<String>,
    pub force_gamemode: bool,
    pub allow_flight: bool,
    pub allow_nether: bool,
    pub spawn_monsters: bool,
    pub spawn_animals: bool,
    pub spawn_npcs: bool,
    pub view_distance: u32,
    pub simulation_distance: u32,
    pub entity_broadcast_range: u32,
    pub tick_distance: u32,
    
    // Server properties
    pub enable_rcon: bool,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub enable_query: bool,
    pub query_port: u16,
    pub resource_pack: Option<String>,
    pub resource_pack_sha1: Option<String>,
    pub require_resource_pack: bool,
    pub broadcast_rcon_to_ops: bool,
    pub broadcast_console_to_ops: bool,
    pub spawn_protection: u32,
    pub max_world_size: u32,
    pub network_compression_threshold: u32,
    pub max_tick_time: u64,
    pub use_aikar_flags: bool,
    
    // Backup settings
    pub auto_backup: bool,
    pub backup_interval_hours: u32,
    pub max_backups: u32,
    pub backup_on_start: bool,
    pub backup_on_stop: bool,
    
    // Plugin/Mod management
    pub plugin_ids: Vec<String>,
    pub mod_loader: ModLoader,
    
    // Advanced
    pub server_jar_url: Option<String>,
    pub custom_server_jar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ModLoader {
    #[default]
    None,
    Paper,
    Fabric,
    Forge,
    NeoForge,
    Mohist,
    CatServer,
    Folia,
    Spigot,
    Purpur,
}

impl ModLoader {
    pub fn label(&self) -> &str {
        match self {
            Self::None => "None",
            Self::Paper => "Paper",
            Self::Fabric => "Fabric",
            Self::Forge => "Forge",
            Self::NeoForge => "NeoForge",
            Self::Mohist => "Mohist",
            Self::CatServer => "CatServer",
            Self::Folia => "Folia",
            Self::Spigot => "Spigot",
            Self::Purpur => "Purpur",
        }
    }
}

fn default_server_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "My AMT Server".to_string(),
            server_type: ServerType::Paper,
            mc_version: "26.1.2".to_string(),
            loader_version: None,
            ram_mb: 2048,
            port: 25565,
            motd: "Hosted on AMT Launcher".to_string(),
            max_players: 20,
            online_mode: true,
            pvp: true,
            difficulty: "easy".to_string(),
            gamemode: "survival".to_string(),
            whitelist: vec![],
            ops: vec![],
            icon_path: None,
            mod_ids: vec![],
            auto_sharing: false,
            
            // JVM arguments
            jvm_args: vec![],
            jvm_extra_args: String::new(),
            
            // World settings
            world_name: "world".to_string(),
            level_seed: None,
            level_type: "minecraft:normal".to_string(),
            generator_settings: None,
            force_gamemode: false,
            allow_flight: false,
            allow_nether: true,
            spawn_monsters: true,
            spawn_animals: true,
            spawn_npcs: true,
            view_distance: 10,
            simulation_distance: 10,
            entity_broadcast_range: 100,
            tick_distance: 4,
            
            // Server properties
            enable_rcon: false,
            rcon_port: 25575,
            rcon_password: String::new(),
            enable_query: false,
            query_port: 25565,
            resource_pack: None,
            resource_pack_sha1: None,
            require_resource_pack: false,
            broadcast_rcon_to_ops: true,
            broadcast_console_to_ops: true,
            spawn_protection: 16,
            max_world_size: 29999984,
            network_compression_threshold: 256,
            max_tick_time: 60000,
            use_aikar_flags: true,
            
            // Backup settings
            auto_backup: false,
            backup_interval_hours: 24,
            max_backups: 10,
            backup_on_start: false,
            backup_on_stop: true,
            
            // Plugin/Mod management
            plugin_ids: vec![],
            mod_loader: ModLoader::None,
            
            // Advanced
            server_jar_url: None,
            custom_server_jar: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub id: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub port: u16,
    pub online_players: Vec<String>,
    pub uptime_secs: u64,
    pub memory_mb: u64,
    pub max_memory_mb: u64,
    pub cpu_percent: f32,
    pub tps: f32,
    pub player_count: u32,
    pub max_players: u32,
    pub version: String,
    pub motd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerLogLine {
    pub line: String,
    pub timestamp: String,
}

pub struct ServerInstance {
    pub config: ServerConfig,
    pub process: Option<Child>,
    pub log: Arc<Mutex<Vec<String>>>,
    pub started_at: Option<std::time::Instant>,
}

impl ServerInstance {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            process: None,
            log: Arc::new(Mutex::new(Vec::new())),
            started_at: None,
        }
    }

    pub fn server_dir(&self) -> PathBuf {
        SERVERS_DIR.join(&self.config.id)
    }

    fn jar_path(&self) -> PathBuf {
        self.server_dir().join("server.jar")
    }
}

/// Download the appropriate server JAR based on type
pub async fn download_server_jar(config: &ServerConfig) -> Result<PathBuf, String> {
    let server_dir = SERVERS_DIR.join(&config.id);
    tokio::fs::create_dir_all(&server_dir)
        .await
        .map_err(|e| format!("Failed to create server dir: {}", e))?;

    let jar_path = server_dir.join("server.jar");

    // If custom JAR URL is provided, use that
    if config.custom_server_jar {
        if let Some(url) = &config.server_jar_url {
            return download_jar_from_url(url, &jar_path, &server_dir, config).await;
        }
    }

    let url = match config.server_type {
        ServerType::Vanilla => {
            // Fetch version manifest to get server.jar URL
            let manifest: serde_json::Value = HTTP_CLIENT
                .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
                .header("User-Agent", "AMT-Client/0.1.0")
                .send()
                .await
                .map_err(|e| format!("Failed to fetch version manifest: {}", e))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse manifest: {}", e))?;

            let version_entry = manifest["versions"]
                .as_array()
                .ok_or("No versions array")?
                .iter()
                .find(|v| v["id"] == config.mc_version)
                .ok_or(format!("Version {} not found", config.mc_version))?
                .clone();

            let version_url = version_entry["url"]
                .as_str()
                .ok_or("No URL in version entry")?
                .to_string();

            let version_data: serde_json::Value = HTTP_CLIENT
                .get(&version_url)
                .header("User-Agent", "AMT-Client/0.1.0")
                .send()
                .await
                .map_err(|e| format!("Failed to fetch version data: {}", e))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse version data: {}", e))?;

            let server_jar = version_data["downloads"]["server"]
                .as_object()
                .ok_or("No server download")?
                .clone();
            let sha1 = server_jar["sha1"].as_str().ok_or("No sha1")?;
            format!(
                "https://piston-data.mojang.com/v1/objects/{}/server.jar",
                sha1
            )
        }
        ServerType::Paper => {
            // Get latest build for version
            let builds: serde_json::Value = HTTP_CLIENT
                .get(&format!(
                    "https://api.papermc.io/v2/projects/paper/versions/{}",
                    config.mc_version
                ))
                .header("User-Agent", "AMT-Client/0.1.0")
                .send()
                .await
                .map_err(|e| format!("Failed to fetch Paper builds: {}", e))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse Paper builds: {}", e))?;

            let latest_build = builds["builds"]
                .as_array()
                .and_then(|b| b.last())
                .ok_or("No Paper builds found")?;

            format!(
                "https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}/downloads/paper-{}-{}.jar",
                config.mc_version, latest_build, config.mc_version, latest_build
            )
        }
        ServerType::Purpur => {
            // Get latest build for version from Purpur API
            let builds: serde_json::Value = HTTP_CLIENT
                .get(&format!(
                    "https://api.purpurmc.org/v2/purpur/{}/builds",
                    config.mc_version
                ))
                .header("User-Agent", "AMT-Client/0.1.0")
                .send()
                .await
                .map_err(|e| format!("Failed to fetch Purpur builds: {}", e))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse Purpur builds: {}", e))?;

            let latest_build = builds["builds"]
                .as_array()
                .and_then(|b| b.last())
                .ok_or("No Purpur builds found")?;

            format!(
                "https://api.purpurmc.org/v2/purpur/{}/{}/download/purpur-{}-{}.jar",
                config.mc_version, latest_build, config.mc_version, latest_build
            )
        }
        ServerType::Spigot => {
            // Spigot uses BuildTools - we'll use a direct download from a mirror
            // For now, we'll use the SpigotMC API or a reliable mirror
            format!(
                "https://cdn.getbukkit.org/spigot/spigot-{}.jar",
                config.mc_version
            )
        }
        ServerType::Folia => {
            // Get latest build for version from PaperMC API (Folia is a Paper project)
            let builds: serde_json::Value = HTTP_CLIENT
                .get(&format!(
                    "https://api.papermc.io/v2/projects/folia/versions/{}",
                    config.mc_version
                ))
                .header("User-Agent", "AMT-Client/0.1.0")
                .send()
                .await
                .map_err(|e| format!("Failed to fetch Folia builds: {}", e))?
                .json()
                .await
                .map_err(|e| format!("Failed to parse Folia builds: {}", e))?;

            let latest_build = builds["builds"]
                .as_array()
                .and_then(|b| b.last())
                .ok_or("No Folia builds found")?;

            format!(
                "https://api.papermc.io/v2/projects/folia/versions/{}/builds/{}/downloads/folia-{}-{}.jar",
                config.mc_version, latest_build, config.mc_version, latest_build
            )
        }
        ServerType::Fabric => {
            let loader = config
                .loader_version
                .as_deref()
                .unwrap_or("0.16.10");
            format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}/{}/server/jar",
                config.mc_version, loader
            )
        }
        ServerType::Forge => {
            let loader = config
                .loader_version
                .as_deref()
                .unwrap_or("latest");
            format!(
                "https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
                config.mc_version, loader, config.mc_version, loader
            )
        }
        ServerType::NeoForge => {
            let loader = config
                .loader_version
                .as_deref()
                .unwrap_or("latest");
            format!(
                "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}-{}/neoforge-{}-{}-installer.jar",
                config.mc_version, loader, config.mc_version, loader
            )
        }
        ServerType::Mohist => {
            // Mohist API for latest build
            format!(
                "https://mohistmc.com/api/v2/projects/mohist/{}/builds/latest/download",
                config.mc_version
            )
        }
        ServerType::CatServer => {
            // CatServer API
            format!(
                "https://catmc.org/api/v2/catserver/{}/download",
                config.mc_version
            )
        }
    };

    info!("Downloading server JAR from {}", url);

    let response = HTTP_CLIENT
        .get(&url)
        .header("User-Agent", "AMT-Client/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to download server JAR: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?
        .to_vec();

    tokio::fs::write(&jar_path, &bytes)
        .await
        .map_err(|e| format!("Failed to write JAR: {}", e))?;

    // For Forge, run the installer
    if config.server_type == ServerType::Forge {
        let jar_str = jar_path
            .to_str()
            .ok_or_else(|| "Forge installer JAR path contains non-UTF-8 characters".to_string())?;
        let status = Command::new("java")
            .args([
                "-jar",
                jar_str,
                "--installServer",
            ])
            .current_dir(&server_dir)
            .status()
            .await
            .map_err(|e| format!("Failed to run Forge installer: {}", e))?;

        if !status.success() {
            return Err("Forge installer failed".to_string());
        }

        // The installer creates a different JAR name
        // Look for the forge-universal JAR
        let mut dir = tokio::fs::read_dir(&server_dir)
            .await
            .map_err(|e| format!("Failed to read server dir: {}", e))?;

        let mut found = false;
        while let Some(entry) = dir.next_entry().await.map_err(|e| format!("Failed to read server dir: {}", e))? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".jar") && name.contains("forge") && !name.contains("installer") {
                let new_path = server_dir.join("server.jar");
                tokio::fs::copy(entry.path(), &new_path)
                    .await
                    .map_err(|e| format!("Failed to copy Forge JAR: {}", e))?;
                found = true;
                break;
            }
        }

        if !found {
            return Err("Forge JAR not found after installation".to_string());
        }
    }
    
    // For NeoForge, run the installer
    if config.server_type == ServerType::NeoForge {
        let jar_str = jar_path
            .to_str()
            .ok_or_else(|| "NeoForge installer JAR path contains non-UTF-8 characters".to_string())?;
        let status = Command::new("java")
            .args([
                "-jar",
                jar_str,
                "--installServer",
            ])
            .current_dir(&server_dir)
            .status()
            .await
            .map_err(|e| format!("Failed to run NeoForge installer: {}", e))?;
        
        if !status.success() {
            return Err("NeoForge installer failed".to_string());
        }
        
        // Look for the neoforge JAR
        let mut dir = tokio::fs::read_dir(&server_dir)
            .await
            .map_err(|e| format!("Failed to read server dir: {}", e))?;
        
        let mut found = false;
        while let Some(entry) = dir.next_entry().await.map_err(|e| format!("Failed to read server dir: {}", e))? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".jar") && name.contains("neoforge") && !name.contains("installer") {
                let new_path = server_dir.join("server.jar");
                tokio::fs::copy(entry.path(), &new_path)
                    .await
                    .map_err(|e| format!("Failed to copy NeoForge JAR: {}", e))?;
                found = true;
                break;
            }
        }
        
        if !found {
            return Err("NeoForge JAR not found after installation".to_string());
        }
    }
    
    // For Mohist and CatServer, they should download as ready-to-run JARs
    // No additional installation needed

    Ok(jar_path)
}

async fn download_jar_from_url(url: &str, jar_path: &PathBuf, _server_dir: &PathBuf, _config: &ServerConfig) -> Result<PathBuf, String> {
    info!("Downloading custom server JAR from {}", url);

    let response = HTTP_CLIENT
        .get(url)
        .header("User-Agent", "AMT-Client/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to download server JAR: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?
        .to_vec();

    tokio::fs::write(jar_path, &bytes)
        .await
        .map_err(|e| format!("Failed to write JAR: {}", e))?;

    Ok(jar_path.clone())
}

/// Launch a server process
pub async fn start_server(
    instance: &Arc<Mutex<ServerInstance>>,
) -> Result<(), String> {
    let mut inst = instance.lock().await;

    let server_dir = inst.server_dir();
    let jar_path = inst.jar_path();

    if !jar_path.exists() {
        return Err("Server JAR not found. Create the server first.".to_string());
    }

    // Accept EULA
    let eula_path = server_dir.join("eula.txt");
    if !eula_path.exists() {
        tokio::fs::write(&eula_path, "eula=true\n")
            .await
            .map_err(|e| format!("Failed to write eula.txt: {}", e))?;
    }

    // Write server.properties with all config options
    let props = build_server_properties(&inst.config);
    tokio::fs::write(server_dir.join("server.properties"), props)
        .await
        .map_err(|e| format!("Failed to write server.properties: {}", e))?;

    // Write whitelist
    if !inst.config.whitelist.is_empty() {
        let whitelist_json: Vec<serde_json::Value> = inst
            .config
            .whitelist
            .iter()
            .map(|name| serde_json::json!({"name": name, "uuid": ""}))
            .collect();
        tokio::fs::write(
            server_dir.join("whitelist.json"),
            serde_json::to_string_pretty(&whitelist_json).unwrap(),
        )
        .await
        .map_err(|e| format!("Failed to write whitelist: {}", e))?;
    }

    // Write ops
    if !inst.config.ops.is_empty() {
        let ops_json: Vec<serde_json::Value> = inst
            .config
            .ops
            .iter()
            .map(|name| serde_json::json!({"name": name, "uuid": "", "level": 4}))
            .collect();
        tokio::fs::write(
            server_dir.join("ops.json"),
            serde_json::to_string_pretty(&ops_json).unwrap(),
        )
        .await
        .map_err(|e| format!("Failed to write ops: {}", e))?;
    }

    // Copy server icon if set
    if let Some(icon_path) = &inst.config.icon_path {
        let icon_bytes = tokio::fs::read(icon_path)
            .await
            .map_err(|e| format!("Failed to read icon: {}", e))?;
        tokio::fs::write(server_dir.join("server-icon.png"), icon_bytes)
            .await
            .map_err(|e| format!("Failed to write icon: {}", e))?;
    }

    // Build JVM arguments
    let mut jvm_args = vec![];
    
    // Memory settings
    jvm_args.push(format!("-Xmx{}M", inst.config.ram_mb));
    jvm_args.push(format!("-Xms{}M", (inst.config.ram_mb / 2).max(512)));
    
    // Aikar's flags for better GC performance
    if inst.config.use_aikar_flags {
        jvm_args.extend([
            "-XX:+UseG1GC".to_string(),
            "-XX:+ParallelRefProcEnabled".to_string(),
            "-XX:MaxGCPauseMillis=200".to_string(),
            "-XX:+UnlockExperimentalVMOptions".to_string(),
            "-XX:+DisableExplicitGC".to_string(),
            "-XX:+AlwaysPreTouch".to_string(),
            "-XX:G1NewSizePercent=30".to_string(),
            "-XX:G1MaxNewSizePercent=40".to_string(),
            "-XX:G1HeapRegionSize=8M".to_string(),
            "-XX:G1ReservePercent=20".to_string(),
            "-XX:G1HeapWastePercent=5".to_string(),
            "-XX:G1MixedGCCountTarget=4".to_string(),
            "-XX:InitiatingHeapOccupancyPercent=15".to_string(),
            "-XX:G1MixedGCLiveThresholdPercent=90".to_string(),
            "-XX:G1RSetUpdatingPauseTimePercent=5".to_string(),
            "-XX:SurvivorRatio=32".to_string(),
            "-XX:+PerfDisableSharedMem".to_string(),
            "-XX:MaxTenuringThreshold=1".to_string(),
            "-Dusing.aikars.flags=https://mcflags.emc.gs".to_string(),
            "-Daikars.new.flags=true".to_string(),
        ]);
    }
    
    // Custom JVM args
    jvm_args.extend(inst.config.jvm_args.clone());
    
    // Extra JVM args from string (split by whitespace)
    for arg in inst.config.jvm_extra_args.split_whitespace() {
        if !arg.is_empty() {
            jvm_args.push(arg.to_string());
        }
    }

    // Server jar args
    let server_args = vec!["nogui".to_string()];

    let jar_str = jar_path
        .to_str()
        .ok_or_else(|| "Server JAR path contains non-UTF-8 characters".to_string())?;
    let mut child = Command::new("java")
        .args(&jvm_args)
        .arg("-jar")
        .arg(jar_str)
        .args(&server_args)
        .current_dir(&server_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to start server: {}", e))?;

    // Read stdout
    let log = inst.log.clone();
    if let Some(stdout) = child.stdout.take() {
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                let mut log_lock = log.lock().await;
                log_lock.push(format!("[{}] {}", timestamp, line));
                if log_lock.len() > 10000 {
                    log_lock.drain(0..5000);
                }
            }
        });
    }
    
    // Also read stderr
    let log = inst.log.clone();
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                let mut log_lock = log.lock().await;
                log_lock.push(format!("[{}] [STDERR] {}", timestamp, line));
                if log_lock.len() > 10000 {
                    log_lock.drain(0..5000);
                }
            }
        });
    }

    inst.process = Some(child);
    inst.started_at = Some(std::time::Instant::now());

    Ok(())
}

/// Build server.properties from config
fn build_server_properties(config: &ServerConfig) -> String {
    let mut props = Vec::new();
    
    // Basic settings
    props.push(format!("server-port={}", config.port));
    props.push(format!("motd={}", config.motd));
    props.push(format!("max-players={}", config.max_players));
    props.push(format!("online-mode={}", config.online_mode));
    props.push(format!("pvp={}", config.pvp));
    props.push(format!("difficulty={}", config.difficulty));
    props.push(format!("gamemode={}", config.gamemode));
    props.push(format!("level-name={}", config.world_name));
    
    // World settings
    if let Some(seed) = &config.level_seed {
        props.push(format!("level-seed={}", seed));
    }
    props.push(format!("level-type={}", config.level_type));
    if let Some(gen) = &config.generator_settings {
        props.push(format!("generator-settings={}", gen));
    }
    props.push(format!("force-gamemode={}", config.force_gamemode));
    props.push(format!("allow-flight={}", config.allow_flight));
    props.push(format!("allow-nether={}", config.allow_nether));
    props.push(format!("spawn-monsters={}", config.spawn_monsters));
    props.push(format!("spawn-animals={}", config.spawn_animals));
    props.push(format!("spawn-npcs={}", config.spawn_npcs));
    props.push(format!("view-distance={}", config.view_distance));
    props.push(format!("simulation-distance={}", config.simulation_distance));
    props.push(format!("entity-broadcast-range-percentage={}", config.entity_broadcast_range));
    props.push(format!("tick-distance={}", config.tick_distance));
    
    // RCON
    props.push(format!("enable-rcon={}", config.enable_rcon));
    props.push(format!("rcon.port={}", config.rcon_port));
    props.push(format!("rcon.password={}", config.rcon_password));
    
    // Query
    props.push(format!("enable-query={}", config.enable_query));
    props.push(format!("query.port={}", config.query_port));
    
    // Resource pack
    if let Some(rp) = &config.resource_pack {
        props.push(format!("resource-pack={}", rp));
    }
    if let Some(sha1) = &config.resource_pack_sha1 {
        props.push(format!("resource-pack-sha1={}", sha1));
    }
    props.push(format!("require-resource-pack={}", config.require_resource_pack));
    
    // Broadcast
    props.push(format!("broadcast-rcon-to-ops={}", config.broadcast_rcon_to_ops));
    props.push(format!("broadcast-console-to-ops={}", config.broadcast_console_to_ops));
    
    // Protection & limits
    props.push(format!("spawn-protection={}", config.spawn_protection));
    props.push(format!("max-world-size={}", config.max_world_size));
    props.push(format!("network-compression-threshold={}", config.network_compression_threshold));
    props.push(format!("max-tick-time={}", config.max_tick_time));
    
    props.join("\n") + "\n"
}

/// Send a command to the server console
pub async fn send_command(
    instance: &Arc<Mutex<ServerInstance>>,
    command: &str,
) -> Result<(), String> {
    let mut inst = instance.lock().await;

    if let Some(ref mut process) = inst.process {
        if let Some(stdin) = process.stdin.as_mut() {
            stdin
                .write_all(format!("{}\n", command).as_bytes())
                .await
                .map_err(|e| format!("Failed to send command: {}", e))?;
            return Ok(());
        }
    }

    Err("Server is not running".to_string())
}

/// Stop a server gracefully
pub async fn stop_server(instance: &Arc<Mutex<ServerInstance>>) -> Result<(), String> {
    // Try graceful stop first
    send_command(instance, "stop").await?;

    let mut inst = instance.lock().await;
    if let Some(ref mut process) = inst.process {
        let pid = process.id();
        // Wait up to 10 seconds for graceful shutdown
        tokio::select! {
            result = process.wait() => {
                if let Ok(status) = result {
                    info!("Server (PID {}) stopped with status: {}", pid.unwrap_or(0), status);
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
                info!("Server (PID {}) did not stop gracefully, killing", pid.unwrap_or(0));
                let _ = process.kill().await;
            }
        }
    }

    inst.process = None;
    inst.started_at = None;
    Ok(())
}

/// Get the server's public connection info
pub fn get_connection_info(config: &ServerConfig) -> serde_json::Value {
    serde_json::json!({
        "host": "localhost",
        "port": config.port,
        "motd": config.motd,
        "version": config.mc_version,
        "server_type": config.server_type.label(),
        "id": config.id,
        "name": config.name,
    })
}

/// Saved server configs
pub type SavedServers = Vec<ServerConfig>;

pub fn servers_path() -> PathBuf {
    SERVERS_DIR.join("servers.json")
}

pub async fn load_servers() -> Vec<ServerConfig> {
    let path = servers_path();
    if !path.exists() {
        return vec![];
    }
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => vec![],
    }
}

pub async fn save_servers(servers: &[ServerConfig]) -> Result<(), String> {
    tokio::fs::create_dir_all(SERVERS_DIR.as_path())
        .await
        .map_err(|e| format!("Failed to create servers directory: {}", e))?;
    let content = serde_json::to_string_pretty(servers).map_err(|e| e.to_string())?;
    tokio::fs::write(servers_path(), content)
        .await
        .map_err(|e| format!("Failed to save servers: {}", e))
}
