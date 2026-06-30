use crate::app::world::WorldManager;

#[tauri::command]
pub(crate) async fn server_worlds_list(server_id: String) -> Result<Vec<crate::app::world::WorldInfo>, String> {
    let manager = WorldManager::new(&server_id);
    manager.list_worlds().await
}

#[tauri::command]
pub(crate) async fn server_world_delete(server_id: String, world_name: String) -> Result<(), String> {
    let manager = WorldManager::new(&server_id);
    manager.delete_world(&world_name).await
}

#[tauri::command]
pub(crate) async fn server_world_backup(server_id: String, world_name: String) -> Result<crate::app::world::WorldBackup, String> {
    let manager = WorldManager::new(&server_id);
    manager.backup_world(&world_name).await
}

#[tauri::command]
pub(crate) async fn server_world_backups_list(server_id: String) -> Result<Vec<crate::app::world::WorldBackup>, String> {
    let manager = WorldManager::new(&server_id);
    manager.list_world_backups().await
}

#[tauri::command]
pub(crate) async fn server_world_backup_restore(server_id: String, backup_filename: String) -> Result<(), String> {
    let manager = WorldManager::new(&server_id);
    manager.restore_world_backup(&backup_filename).await
}

#[tauri::command]
pub(crate) async fn server_world_backup_delete(server_id: String, backup_filename: String) -> Result<(), String> {
    let manager = WorldManager::new(&server_id);
    manager.delete_world_backup(&backup_filename).await
}
