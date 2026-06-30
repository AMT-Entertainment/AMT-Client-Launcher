use std::path::PathBuf;

use tokio::fs;
use tokio_tar::Builder;
use async_compression::tokio::write::GzipEncoder;
use tracing::info;

use crate::SERVERS_DIR;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupInfo {
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub world_count: u32,
    pub mod_count: u32,
    pub plugin_count: u32,
    pub has_config: bool,
}

pub struct BackupManager {
    pub server_id: String,
    pub max_backups: u32,
}

impl BackupManager {
    pub fn new(server_id: &str) -> Self {
        Self {
            server_id: server_id.to_string(),
            max_backups: 10,
        }
    }

    pub fn with_max_backups(server_id: &str, max_backups: u32) -> Self {
        Self {
            server_id: server_id.to_string(),
            max_backups,
        }
    }

    fn server_dir(&self) -> PathBuf {
        SERVERS_DIR.join(&self.server_id)
    }

    fn backups_dir(&self) -> PathBuf {
        self.server_dir().join("backups")
    }

    pub async fn ensure_backups_dir(&self) -> Result<(), String> {
        let dir = self.backups_dir();
        fs::create_dir_all(&dir)
            .await
            .map_err(|e| format!("Failed to create backups dir: {}", e))
    }

    pub async fn create_backup(&self) -> Result<BackupInfo, String> {
        self.ensure_backups_dir().await?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_filename = format!("backup_{}.tar.gz", timestamp);
        let backup_path = self.backups_dir().join(&backup_filename);

        let server_dir = self.server_dir();
        let file = fs::File::create(&backup_path).await
            .map_err(|e| format!("Failed to create backup file: {}", e))?;
        let encoder = GzipEncoder::new(file);
        let mut builder = Builder::new(encoder);

        // Count and add server.properties
        let props_path = server_dir.join("server.properties");
        if props_path.exists() {
            let data = fs::read_to_string(&props_path).await
                .map_err(|e| format!("Failed to read server.properties: {}", e))?;
            let mut header = tokio_tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_entry_type(tokio_tar::EntryType::Regular);
            header.set_mode(0o644);
            builder.append_data(&mut header, "server.properties", data.as_bytes()).await
                .map_err(|e| format!("Failed to add server.properties: {}", e))?;
        }

        // Add eula.txt
        let eula_path = server_dir.join("eula.txt");
        if eula_path.exists() {
            let data = fs::read_to_string(&eula_path).await
                .map_err(|e| format!("Failed to read eula.txt: {}", e))?;
            let mut header = tokio_tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_entry_type(tokio_tar::EntryType::Regular);
            header.set_mode(0o644);
            builder.append_data(&mut header, "eula.txt", data.as_bytes()).await
                .map_err(|e| format!("Failed to add eula.txt: {}", e))?;
        }

        // Add critical config files
        for critical_file in &["ops.json", "whitelist.json", "banned-players.json", "banned-ips.json", "bukkit.yml", "paper.yml", "spigot.yml", "permissions.yml"] {
            let path = server_dir.join(critical_file);
            if path.exists() {
                let data = fs::read(&path).await
                    .map_err(|e| format!("Failed to read {}: {}", critical_file, e))?;
                let mut header = tokio_tar::Header::new_gnu();
                header.set_size(data.len() as u64);
                header.set_entry_type(tokio_tar::EntryType::Regular);
                header.set_mode(0o644);
                builder.append_data(&mut header, critical_file, &*data).await
                    .map_err(|e| format!("Failed to add {}: {}", critical_file, e))?;
            }
        }

        // Discover and add world directories
        let mut world_count = 0u32;
        let mut dir = fs::read_dir(&server_dir).await
            .map_err(|e| format!("Failed to read server dir: {}", e))?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| format!("Read error: {}", e))? {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }

            let region_path = path.join("region");
            let level_dat = path.join("level.dat");
            if !region_path.exists() && !level_dat.exists() { continue; }

            world_count += 1;
            append_dir_to_tar(&mut builder, &path, &name).await
                .map_err(|e| format!("Failed to add world '{}': {}", name, e))?;
        }

        // Add mods directory
        let mods_path = server_dir.join("mods");
        let mod_count = if mods_path.exists() {
            let mut count = 0u32;
            if let Ok(mut md) = fs::read_dir(&mods_path).await {
                while let Ok(Some(entry)) = md.next_entry().await {
                    if entry.file_name().to_string_lossy().ends_with(".jar") {
                        count += 1;
                    }
                }
            }
            if count > 0 {
                append_dir_to_tar(&mut builder, &mods_path, "mods").await
                    .map_err(|e| format!("Failed to add mods: {}", e))?;
            }
            count
        } else { 0 };

        // Add plugins directory
        let plugins_path = server_dir.join("plugins");
        let plugin_count = if plugins_path.exists() {
            let mut count = 0u32;
            if let Ok(mut pd) = fs::read_dir(&plugins_path).await {
                while let Ok(Some(entry)) = pd.next_entry().await {
                    if entry.file_name().to_string_lossy().ends_with(".jar") {
                        count += 1;
                    }
                }
            }
            if count > 0 {
                append_dir_to_tar(&mut builder, &plugins_path, "plugins").await
                    .map_err(|e| format!("Failed to add plugins: {}", e))?;
            }
            count
        } else { 0 };

        // Finalize
        builder.finish().await
            .map_err(|e| format!("Failed to finalize tar: {}", e))?;

        let metadata = fs::metadata(&backup_path).await
            .map_err(|e| format!("Failed to read backup metadata: {}", e))?;

        let backup_info = BackupInfo {
            filename: backup_filename,
            size_bytes: metadata.len(),
            created_at: timestamp,
            world_count,
            mod_count,
            plugin_count,
            has_config: true,
        };

        self.prune_old_backups(self.max_backups).await?;

        info!("Created backup {} ({} bytes, {} worlds, {} mods, {} plugins)",
            backup_info.filename, backup_info.size_bytes, world_count, mod_count, plugin_count);

        Ok(backup_info)
    }

    pub async fn list_backups(&self) -> Result<Vec<BackupInfo>, String> {
        self.ensure_backups_dir().await?;
        let backups_dir = self.backups_dir();
        let mut backups = Vec::new();

        let mut dir = fs::read_dir(&backups_dir).await
            .map_err(|e| format!("Failed to read backups dir: {}", e))?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| format!("Read error: {}", e))? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "gz") {
                let filename = entry.file_name().to_string_lossy().to_string();
                let created_at = filename
                    .strip_prefix("backup_")
                    .and_then(|s| s.strip_suffix(".tar.gz"))
                    .unwrap_or("unknown")
                    .to_string();
                let metadata = fs::metadata(&path).await.ok();

                backups.push(BackupInfo {
                    filename,
                    size_bytes: metadata.map(|m| m.len()).unwrap_or(0),
                    created_at,
                    world_count: 0,
                    mod_count: 0,
                    plugin_count: 0,
                    has_config: true,
                });
            }
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    pub async fn restore_backup(&self, backup_filename: &str) -> Result<(), String> {
        let backup_path = self.backups_dir().join(backup_filename);
        if !backup_path.exists() {
            return Err(format!("Backup not found: {}", backup_filename));
        }

        let server_dir = self.server_dir();

        // Extract tar.gz to server directory, overwriting
        let file = fs::File::open(&backup_path).await
            .map_err(|e| format!("Failed to open backup: {}", e))?;
        let decoder = async_compression::tokio::bufread::GzipDecoder::new(tokio::io::BufReader::new(file));
        let mut archive = tokio_tar::Archive::new(decoder);

        archive.unpack(&server_dir).await
            .map_err(|e| format!("Failed to restore backup: {}", e))?;

        info!("Restored backup {}", backup_filename);
        Ok(())
    }

    pub async fn delete_backup(&self, backup_filename: &str) -> Result<(), String> {
        let backup_path = self.backups_dir().join(backup_filename);
        if !backup_path.exists() {
            return Err(format!("Backup not found: {}", backup_filename));
        }
        fs::remove_file(&backup_path).await
            .map_err(|e| format!("Failed to delete backup: {}", e))?;
        info!("Deleted backup {}", backup_filename);
        Ok(())
    }

    pub async fn prune_old_backups(&self, max_backups: u32) -> Result<(), String> {
        let backups = self.list_backups().await?;
        if backups.len() <= max_backups as usize {
            return Ok(());
        }

        let to_delete = backups.len() - max_backups as usize;
        for backup in backups.iter().rev().take(to_delete) {
            self.delete_backup(&backup.filename).await?;
        }

        Ok(())
    }
}

async fn append_dir_to_tar(
    builder: &mut Builder<GzipEncoder<tokio::fs::File>>,
    dir_path: &std::path::Path,
    archive_prefix: &str,
) -> Result<(), String> {
    let mut entries = Vec::new();
    collect_files(dir_path, archive_prefix, &mut entries).await?;

    for (archive_path, file_path) in &entries {
        let data = fs::read(file_path).await
            .map_err(|e| format!("Failed to read {}: {}", archive_path, e))?;

        let mut header = tokio_tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_entry_type(tokio_tar::EntryType::Regular);
        header.set_mode(0o644);
        header.set_mtime(0);

        builder.append_data(&mut header, archive_path, &*data).await
            .map_err(|e| format!("Failed to add {}: {}", archive_path, e))?;
    }

    Ok(())
}

async fn collect_files(
    dir: &std::path::Path,
    prefix: &str,
    entries: &mut Vec<(String, PathBuf)>,
) -> Result<(), String> {
    let mut rd = fs::read_dir(dir).await
        .map_err(|e| format!("Failed to read dir: {}", e))?;
    while let Some(entry) = rd.next_entry().await.map_err(|e| format!("Read error: {}", e))? {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            Box::pin(collect_files(&path, &format!("{}/{}", prefix, name), entries)).await?;
        } else {
            entries.push((format!("{}/{}", prefix, name), path));
        }
    }
    Ok(())
}
