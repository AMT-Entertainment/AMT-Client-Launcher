use std::path::PathBuf;

use tokio::fs;
use tokio_tar::Builder;
use async_compression::tokio::write::GzipEncoder;
use tracing::info;

use crate::SERVERS_DIR;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldInfo {
    pub name: String,
    pub size_bytes: u64,
    pub region_count: u32,
    pub player_data_count: u32,
    pub has_level_dat: bool,
    pub last_modified: String,
    pub path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldBackup {
    pub filename: String,
    pub size_bytes: u64,
    pub world_name: String,
    pub created_at: String,
}

pub struct WorldManager {
    pub server_id: String,
}

impl WorldManager {
    pub fn new(server_id: &str) -> Self {
        Self {
            server_id: server_id.to_string(),
        }
    }

    fn server_dir(&self) -> PathBuf {
        SERVERS_DIR.join(&self.server_id)
    }

    fn world_backups_dir(&self) -> PathBuf {
        self.server_dir().join("world_backups")
    }

    pub async fn ensure_world_backups_dir(&self) -> Result<(), String> {
        let dir = self.world_backups_dir();
        fs::create_dir_all(&dir)
            .await
            .map_err(|e| format!("Failed to create world backups dir: {}", e))
    }

    pub async fn list_worlds(&self) -> Result<Vec<WorldInfo>, String> {
        let server_dir = self.server_dir();
        let mut worlds = Vec::new();

        let mut dir = fs::read_dir(&server_dir).await
            .map_err(|e| format!("Failed to read server dir: {}", e))?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| format!("Read error: {}", e))? {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "backups" || name == "world_backups"
                || name == "mods" || name == "plugins" || name == "cache" || name == "logs"
                || name == "crash-reports" || name == "stats" || name == "data"
                || name == "libraries" || name == "versions"
            {
                continue;
            }

            let region_path = path.join("region");
            let level_dat = path.join("level.dat");
            if !region_path.exists() && !level_dat.exists() { continue; }

            let size_bytes = calculate_dir_size(&path).await;

            let region_count = if region_path.exists() {
                let mut count = 0u32;
                if let Ok(mut rd) = fs::read_dir(&region_path).await {
                    while let Ok(Some(e)) = rd.next_entry().await {
                        if e.file_name().to_string_lossy().ends_with(".mca") { count += 1; }
                    }
                }
                count
            } else { 0 };

            let playerdata_path = path.join("playerdata");
            let player_data_count = if playerdata_path.exists() {
                let mut count = 0u32;
                if let Ok(mut pd) = fs::read_dir(&playerdata_path).await {
                    while let Ok(Some(_)) = pd.next_entry().await { count += 1; }
                }
                count
            } else { 0 };

            let last_modified = if level_dat.exists() {
                if let Ok(meta) = fs::metadata(&level_dat).await {
                    if let Ok(modified) = meta.modified() {
                        let datetime: chrono::DateTime<chrono::Local> = modified.into();
                        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                    } else { "unknown".to_string() }
                } else { "unknown".to_string() }
            } else { "unknown".to_string() };

            worlds.push(WorldInfo {
                name: name.clone(),
                size_bytes,
                region_count,
                player_data_count,
                has_level_dat: level_dat.exists(),
                last_modified,
                path: path.to_string_lossy().to_string(),
            });
        }

        worlds.sort_by(|a, b| {
            if a.name == "world" { std::cmp::Ordering::Less }
            else if b.name == "world" { std::cmp::Ordering::Greater }
            else { a.name.cmp(&b.name) }
        });

        Ok(worlds)
    }

    pub async fn delete_world(&self, world_name: &str) -> Result<(), String> {
        let world_path = self.server_dir().join(world_name);
        if !world_path.exists() {
            return Err(format!("World '{}' not found", world_name));
        }
        if !world_path.is_dir() {
            return Err(format!("'{}' is not a directory", world_name));
        }
        if world_name == "." || world_name == ".." || world_name.starts_with('.') {
            return Err("Cannot delete hidden directories".to_string());
        }

        fs::remove_dir_all(&world_path).await
            .map_err(|e| format!("Failed to delete world '{}': {}", world_name, e))?;

        info!("Deleted world '{}' for server {}", world_name, self.server_id);
        Ok(())
    }

    pub async fn backup_world(&self, world_name: &str) -> Result<WorldBackup, String> {
        let world_path = self.server_dir().join(world_name);
        if !world_path.exists() || !world_path.is_dir() {
            return Err(format!("World '{}' not found", world_name));
        }

        self.ensure_world_backups_dir().await?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let safe_name = world_name.replace(' ', "_").replace('/', "_");
        let backup_filename = format!("world_{}_{}.tar.gz", safe_name, timestamp);
        let backup_path = self.world_backups_dir().join(&backup_filename);

        let file = fs::File::create(&backup_path).await
            .map_err(|e| format!("Failed to create backup file: {}", e))?;
        let encoder = GzipEncoder::new(file);
        let mut builder = Builder::new(encoder);

        // Add all files from the world directory with the world name as prefix
        let mut entries = Vec::new();
        collect_world_files(&world_path, world_name, &mut entries).await
            .map_err(|e| format!("Failed to collect world files: {}", e))?;

        for (archive_name, file_path) in &entries {
            let data = fs::read(file_path).await
                .map_err(|e| format!("Failed to read {}: {}", archive_name, e))?;
            let mut header = tokio_tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_entry_type(tokio_tar::EntryType::Regular);
            header.set_mode(0o644);
            builder.append_data(&mut header, archive_name, &*data).await
                .map_err(|e| format!("Failed to add {}: {}", archive_name, e))?;
        }

        builder.finish().await
            .map_err(|e| format!("Failed to finalize tar: {}", e))?;

        let metadata = fs::metadata(&backup_path).await
            .map_err(|e| format!("Failed to read backup metadata: {}", e))?;

        let backup = WorldBackup {
            filename: backup_filename.clone(),
            size_bytes: metadata.len(),
            world_name: world_name.to_string(),
            created_at: timestamp,
        };

        info!("Created world backup '{}' for world '{}'", backup_filename.clone(), world_name);
        Ok(backup)
    }

    pub async fn list_world_backups(&self) -> Result<Vec<WorldBackup>, String> {
        let backups_dir = self.world_backups_dir();
        if !backups_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = Vec::new();
        let mut dir = fs::read_dir(&backups_dir).await
            .map_err(|e| format!("Failed to read world backups dir: {}", e))?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| format!("Read error: {}", e))? {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "gz") {
                let filename = entry.file_name().to_string_lossy().to_string();

                // Parse world name from filename: world_{name}_{timestamp}.tar.gz
                let world_name = filename
                    .strip_prefix("world_")
                    .and_then(|s| {
                        // Find second-to-last underscore
                        let underscore_positions: Vec<_> = s.match_indices('_').map(|(i, _)| i).collect();
                        if underscore_positions.len() >= 2 {
                            let last_underscore = underscore_positions[underscore_positions.len() - 1];
                            Some(s[..last_underscore].to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                let created_at = filename
                    .strip_prefix("world_")
                    .and_then(|s| {
                        s.strip_suffix(".tar.gz")
                            .and_then(|s| {
                                s.rfind('_').map(|i| s[i+1..].to_string())
                            })
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                let metadata = fs::metadata(&path).await.ok();

                backups.push(WorldBackup {
                    filename,
                    size_bytes: metadata.map(|m| m.len()).unwrap_or(0),
                    world_name,
                    created_at,
                });
            }
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    pub async fn restore_world_by_name(&self, world_name: &str) -> Result<(), String> {
        // Find latest backup for this world
        let backups = self.list_world_backups().await?;
        let matching: Vec<_> = backups.into_iter()
            .filter(|b| b.world_name == world_name)
            .collect();

        if matching.is_empty() {
            return Err(format!("No backups found for world '{}'", world_name));
        }

        self.restore_world_backup(&matching[0].filename).await
    }

    pub async fn restore_world_backup(&self, backup_filename: &str) -> Result<(), String> {
        let backup_path = self.world_backups_dir().join(backup_filename);
        if !backup_path.exists() {
            return Err(format!("World backup not found: {}", backup_filename));
        }

        // Extract world name from filename
        let world_name = backup_filename
            .strip_prefix("world_")
            .and_then(|s| {
                let underscore_positions: Vec<_> = s.match_indices('_').map(|(i, _)| i).collect();
                if underscore_positions.len() >= 2 {
                    let last_underscore = underscore_positions[underscore_positions.len() - 1];
                    Some(s[..last_underscore].to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "world".to_string());

        let server_dir = self.server_dir();
        let world_target = server_dir.join(&world_name);

        if world_target.exists() {
            fs::remove_dir_all(&world_target).await
                .map_err(|e| format!("Failed to remove existing world: {}", e))?;
        }

        // Extract backup directly (tar.gz will create the world_name dir)
        let file = fs::File::open(&backup_path).await
            .map_err(|e| format!("Failed to open backup: {}", e))?;
        let decoder = async_compression::tokio::bufread::GzipDecoder::new(tokio::io::BufReader::new(file));
        let mut archive = tokio_tar::Archive::new(decoder);
        archive.unpack(&server_dir).await
            .map_err(|e| format!("Failed to restore world: {}", e))?;

        info!("Restored world backup '{}' to '{}'", backup_filename, world_name);
        Ok(())
    }

    pub async fn delete_world_backup(&self, backup_filename: &str) -> Result<(), String> {
        let backup_path = self.world_backups_dir().join(backup_filename);
        if !backup_path.exists() {
            return Err(format!("World backup not found: {}", backup_filename));
        }
        fs::remove_file(&backup_path).await
            .map_err(|e| format!("Failed to delete world backup: {}", e))?;
        Ok(())
    }
}

async fn calculate_dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(mut dir) = fs::read_dir(path).await {
        while let Ok(Some(entry)) = dir.next_entry().await {
            let p = entry.path();
            if let Ok(meta) = fs::metadata(&p).await {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() {
                    total += Box::pin(calculate_dir_size(&p)).await;
                }
            }
        }
    }
    total
}

async fn collect_world_files(
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
            Box::pin(collect_world_files(&path, &format!("{}/{}", prefix, name), entries)).await?;
        } else {
            entries.push((format!("{}/{}", prefix, name), path));
        }
    }
    Ok(())
}
