use std::path::PathBuf;

use path_absolutize::Absolutize;
use tokio::fs;
use tracing::info;

use crate::LAUNCHER_DIRECTORY;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified: String,
    pub extension: String,
}

fn server_dir(server_id: &str) -> PathBuf {
    LAUNCHER_DIRECTORY
        .data_dir()
        .join("servers")
        .join(server_id)
}

fn resolve_path(server_id: &str, relative_path: &str) -> Result<PathBuf, String> {
    let base = server_dir(server_id);
    let target = base.join(relative_path);

    // Security: prevent path traversal
    let base_canonical = base.absolutize()
        .map_err(|e| format!("Failed to resolve base path: {}", e))?;
    let target_canonical = target.absolutize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;

    if !target_canonical.starts_with(&base_canonical) {
        return Err("Path traversal detected".to_string());
    }

    Ok(target_canonical.to_path_buf())
}

#[tauri::command]
pub(crate) async fn server_files_list(server_id: String, path: String) -> Result<Vec<FileEntry>, String> {
    let dir_path = resolve_path(&server_id, &path)?;

    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir_path.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }

    let mut entries = Vec::new();
    let mut dir = fs::read_dir(&dir_path).await
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    while let Some(entry) = dir.next_entry().await.map_err(|e| format!("Read error: {}", e))? {
        let name = entry.file_name().to_string_lossy().to_string();
        let full_path = entry.path();
        let metadata = fs::metadata(&full_path).await.ok();
        let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);

        let modified = metadata.as_ref().and_then(|m| m.modified().ok())
            .map(|t| {
                let dt: chrono::DateTime<chrono::Local> = t.into();
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            })
            .unwrap_or_else(|| "unknown".to_string());

        let extension = if is_dir {
            String::new()
        } else {
            std::path::Path::new(&name)
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default()
        };

        entries.push(FileEntry {
            name,
            path: full_path.to_string_lossy().to_string(),
            is_dir,
            size_bytes: metadata.map(|m| m.len()).unwrap_or(0),
            modified,
            extension,
        });
    }

    // Sort: directories first, then files, alphabetically
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            b.is_dir.cmp(&a.is_dir)
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    Ok(entries)
}

#[tauri::command]
pub(crate) async fn server_files_read(server_id: String, path: String) -> Result<String, String> {
    let file_path = resolve_path(&server_id, &path)?;

    if !file_path.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    if file_path.is_dir() {
        return Err(format!("Cannot read a directory: {}", path));
    }

    let content = fs::read_to_string(&file_path).await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    Ok(content)
}

#[tauri::command]
pub(crate) async fn server_files_write(server_id: String, path: String, content: String) -> Result<(), String> {
    let file_path = resolve_path(&server_id, &path)?;

    // Create parent directories if they don't exist
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).await
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }

    fs::write(&file_path, &content).await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    info!("Wrote file: {}", file_path.display());
    Ok(())
}

#[tauri::command]
pub(crate) async fn server_files_delete(server_id: String, path: String) -> Result<(), String> {
    let file_path = resolve_path(&server_id, &path)?;

    if !file_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if file_path.is_dir() {
        fs::remove_dir_all(&file_path).await
            .map_err(|e| format!("Failed to delete directory: {}", e))?;
    } else {
        fs::remove_file(&file_path).await
            .map_err(|e| format!("Failed to delete file: {}", e))?;
    }

    info!("Deleted: {}", file_path.display());
    Ok(())
}

#[tauri::command]
pub(crate) async fn server_files_rename(server_id: String, old_path: String, new_path: String) -> Result<(), String> {
    let src = resolve_path(&server_id, &old_path)?;
    let dst = resolve_path(&server_id, &new_path)?;

    if !src.exists() {
        return Err(format!("Source does not exist: {}", old_path));
    }

    fs::rename(&src, &dst).await
        .map_err(|e| format!("Failed to rename: {}", e))?;

    info!("Renamed {} to {}", src.display(), dst.display());
    Ok(())
}

#[tauri::command]
pub(crate) async fn server_files_mkdir(server_id: String, path: String) -> Result<(), String> {
    let dir_path = resolve_path(&server_id, &path)?;

    fs::create_dir_all(&dir_path).await
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    Ok(())
}
