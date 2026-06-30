use crate::app::modrinth;
use crate::LAUNCHER_DIRECTORY;

#[tauri::command]
pub(crate) async fn server_mods_search(query: String, limit: u32, offset: u32) -> Result<modrinth::SearchResult, String> {
    modrinth::search(&query, &modrinth::ModrinthProjectType::Mod, limit, offset).await
}

#[tauri::command]
pub(crate) async fn server_mods_get_project(project_id: String) -> Result<modrinth::Project, String> {
    modrinth::get_project(&project_id).await
}

#[tauri::command]
pub(crate) async fn server_mods_get_versions(project_id: String) -> Result<Vec<modrinth::Version>, String> {
    modrinth::get_versions(&project_id).await
}

#[tauri::command]
pub(crate) async fn server_mods_install(
    server_id: String,
    file_url: String,
    filename: String,
) -> Result<(), String> {
    let bytes = modrinth::download_file(&file_url).await?;

    let server_dir = LAUNCHER_DIRECTORY
        .data_dir()
        .join("servers")
        .join(&server_id);
    let mods_dir = server_dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir)
        .await
        .map_err(|e| format!("Failed to create mods dir: {}", e))?;

    let target_path = mods_dir.join(&filename);
    tokio::fs::write(&target_path, bytes)
        .await
        .map_err(|e| format!("Failed to write mod file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub(crate) async fn server_mods_list(server_id: String) -> Result<Vec<String>, String> {
    let server_dir = LAUNCHER_DIRECTORY
        .data_dir()
        .join("servers")
        .join(&server_id);
    let mods_dir = server_dir.join("mods");

    if !mods_dir.exists() {
        return Ok(vec![]);
    }

    let mut mods = Vec::new();
    let mut dir = tokio::fs::read_dir(&mods_dir)
        .await
        .map_err(|e| format!("Failed to read mods dir: {}", e))?;

    while let Some(entry) = dir.next_entry().await.map_err(|e| format!("Read error: {}", e))? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".jar") {
            mods.push(name);
        }
    }

    Ok(mods)
}

#[tauri::command]
pub(crate) async fn server_mods_delete(server_id: String, filename: String) -> Result<(), String> {
    let mods_dir = LAUNCHER_DIRECTORY
        .data_dir()
        .join("servers")
        .join(&server_id)
        .join("mods")
        .join(&filename);

    tokio::fs::remove_file(&mods_dir)
        .await
        .map_err(|e| format!("Failed to delete mod: {}", e))?;

    Ok(())
}
