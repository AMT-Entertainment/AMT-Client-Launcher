use crate::app::modrinth;

#[tauri::command]
pub(crate) async fn modrinth_search(
    query: String,
    project_type: String,
    limit: u32,
    offset: u32,
) -> Result<modrinth::SearchResult, String> {
    let ptype = match project_type.to_lowercase().as_str() {
        "shader" => modrinth::ModrinthProjectType::Shader,
        "resourcepack" => modrinth::ModrinthProjectType::ResourcePack,
        "modpack" => modrinth::ModrinthProjectType::ModPack,
        "datapack" => modrinth::ModrinthProjectType::DataPack,
        _ => modrinth::ModrinthProjectType::Mod,
    };
    modrinth::search(&query, &ptype, limit, offset).await
}

#[tauri::command]
pub(crate) async fn modrinth_get_project(project_id: String) -> Result<modrinth::Project, String> {
    modrinth::get_project(&project_id).await
}

#[tauri::command]
pub(crate) async fn modrinth_get_versions(project_id: String) -> Result<Vec<modrinth::Version>, String> {
    modrinth::get_versions(&project_id).await
}

#[tauri::command]
pub(crate) async fn modrinth_get_filtered_versions(
    project_id: String,
    game_version: String,
    loader: String,
) -> Result<Vec<modrinth::Version>, String> {
    modrinth::get_filtered_versions(&project_id, &game_version, &loader).await
}

#[tauri::command]
pub(crate) async fn modrinth_install(
    _project_id: String,
    _version_id: String,
    file_url: String,
    filename: String,
    install_dir: String,
) -> Result<(), String> {
    // Download the file
    let bytes = modrinth::download_file(&file_url).await?;

    // Determine install path (relative to launcher data dir)
    let data_dir = crate::LAUNCHER_DIRECTORY.data_dir().to_path_buf();
    let target_dir = data_dir.join("gameDir").join(&install_dir);
    tokio::fs::create_dir_all(&target_dir)
        .await
        .map_err(|e| format!("Failed to create install directory: {}", e))?;

    let target_path = target_dir.join(&filename);
    tokio::fs::write(&target_path, bytes)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub(crate) async fn modrinth_resolve_dependencies(
    version_id: String,
    game_version: Option<String>,
    loader: Option<String>,
    max_depth: Option<u32>,
) -> Result<modrinth::DependencyResolution, String> {
    modrinth::resolve_dependencies(
        &version_id,
        game_version.as_deref(),
        loader.as_deref(),
        max_depth.unwrap_or(5),
    ).await
}

#[tauri::command]
pub(crate) async fn modrinth_resolve_download_list(
    version_id: String,
    game_version: Option<String>,
    loader: Option<String>,
) -> Result<Vec<(String, String)>, String> {
    modrinth::resolve_download_list(
        &version_id,
        game_version.as_deref(),
        loader.as_deref(),
    ).await
}
