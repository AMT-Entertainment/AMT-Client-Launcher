use std::sync::{Arc, Mutex};

use commands::*;
use tauri::Window;

pub type ShareableWindow = Arc<Mutex<Window>>;

pub struct RunnerInstance {
    pub terminator: tokio::sync::oneshot::Sender<()>,
}

pub struct AppState {
    pub runner_instance: Arc<Mutex<Option<RunnerInstance>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            runner_instance: Arc::new(Mutex::new(None)),
        }
    }
}

mod commands;

/// Runs the GUI and returns when the window is closed.
pub fn gui_main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            setup_client,
            check_system,
            sys_memory,
            get_options,
            store_options,
            request_branches,
            request_builds,
            request_mods,
            run_client,
            login_microsoft,
            logout,
            refresh,
            fetch_blog_posts,
            fetch_changelog,
            clear_data,
            clear_all_data,
            default_data_folder_path,
            terminate,
            get_launcher_version,
            get_custom_mods,
            install_custom_mod,
            delete_custom_mod,
            github_auth_url,
            github_exchange_code,
            github_fetch_user,
            github_dispatch,
            modrinth_search,
            modrinth_get_project,
            modrinth_get_versions,
            modrinth_get_filtered_versions,
            modrinth_install,
            modrinth_resolve_dependencies,
            modrinth_resolve_download_list,
            server_list,
            server_create,
            server_delete,
            server_update,
            server_start,
            server_stop,
            server_status,
            server_logs,
            server_send_command,
            server_share_info,
            server_is_any_running,
            server_player_history,
            server_metrics,
            server_backup_create,
            server_backup_list,
            server_backup_restore,
            server_backup_delete,
            server_backup_info,
            server_worlds_list,
            server_world_delete,
            server_world_backup,
            server_world_backups_list,
            server_world_backup_restore,
            server_world_backup_delete,
            server_files_list,
            server_files_read,
            server_files_write,
            server_files_delete,
            server_files_rename,
            server_files_mkdir,
            tunnel_start,
            tunnel_stop,
            tunnel_status,
            server_mods_search,
            server_mods_get_project,
            server_mods_get_versions,
            server_mods_install,
            server_mods_list,
            server_mods_delete,
            rcon_connect,
            rcon_disconnect,
            rcon_send_command,
            rcon_player_list,
            rcon_player_info,
            rcon_kick,
            rcon_ban,
            rcon_pardon,
            rcon_op,
            rcon_deop,
            rcon_status,
            social_register_user,
            social_get_user_profile,
            social_update_profile,
            social_create_post,
            social_get_feed,
            social_like_post,
            social_get_hashtags,
            client_account_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
