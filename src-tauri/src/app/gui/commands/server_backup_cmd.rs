use crate::app::backup::BackupManager;

#[tauri::command]
pub(crate) async fn server_backup_create(server_id: String) -> Result<crate::app::backup::BackupInfo, String> {
    let manager = BackupManager::new(&server_id);
    manager.create_backup().await
}

#[tauri::command]
pub(crate) async fn server_backup_list(server_id: String) -> Result<Vec<crate::app::backup::BackupInfo>, String> {
    let manager = BackupManager::new(&server_id);
    manager.list_backups().await
}

#[tauri::command]
pub(crate) async fn server_backup_restore(server_id: String, backup_filename: String) -> Result<(), String> {
    // Check if server is running
    let running_servers = crate::app::gui::commands::server_cmd::get_running_servers().await;
    if running_servers.contains(&server_id) {
        return Err("Cannot restore backup while server is running. Stop the server first.".to_string());
    }

    let manager = BackupManager::new(&server_id);
    manager.restore_backup(&backup_filename).await
}

#[tauri::command]
pub(crate) async fn server_backup_delete(server_id: String, backup_filename: String) -> Result<(), String> {
    let manager = BackupManager::new(&server_id);
    manager.delete_backup(&backup_filename).await
}

#[tauri::command]
pub(crate) async fn server_backup_info(server_id: String, backup_filename: String) -> Result<crate::app::backup::BackupInfo, String> {
    let manager = BackupManager::new(&server_id);
    let backups = manager.list_backups().await?;
    backups.into_iter().find(|b| b.filename == backup_filename)
        .ok_or_else(|| format!("Backup not found: {}", backup_filename))
}
