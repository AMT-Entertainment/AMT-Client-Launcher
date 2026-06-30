use std::{collections::HashMap, path::Path};

use crate::minecraft::java::DistributionSelection;
use crate::minecraft::auth::MinecraftAccount;
use anyhow::Result;
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::info;

#[derive(Serialize, Deserialize)]
pub(crate) struct Options {
    #[serde(rename = "start")]
    pub start_options: StartOptions,
    #[serde(rename = "version")]
    pub version_options: VersionOptions,
    #[serde(rename = "launcher")]
    pub launcher_options: LauncherOptions,
    #[serde(rename = "premium")]
    pub premium_options: PremiumOptions,
    #[serde(rename = "amt_options")]
    pub amt_options: AMTOptions,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct StartOptions {
    #[serde(rename = "account")]
    pub minecraft_account: Option<MinecraftAccount>,
    #[serde(rename = "accounts", default)]
    pub accounts: Vec<MinecraftAccount>,
    #[serde(rename = "activeAccountIndex", default)]
    pub active_account_index: usize,
    #[serde(rename = "customDataPath", default)]
    pub custom_data_path: String,
    #[serde(rename = "javaDistribution", default)]
    pub java_distribution: DistributionSelection,
    #[serde(rename = "jvmArgs", default)]
    pub jvm_args: Option<Vec<String>>,
    #[serde(rename = "memory", default = "default_memory")]
    pub memory: u64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct VersionOptions {
    #[serde(rename = "branchName")]
    pub branch_name: Option<String>,
    #[serde(rename = "buildId", default)]
    pub build_id: i32,
    #[serde(rename = "options", default)]
    pub options: HashMap<String, BranchOptions>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct LauncherOptions {
    #[serde(rename = "firstRun", default)]
    pub first_run: bool,
    #[serde(rename = "showNightlyBuilds")]
    pub show_nightly_builds: bool,
    #[serde(rename = "concurrentDownloads")]
    pub concurrent_downloads: u32,
    #[serde(rename = "keepLauncherOpen", default = "default_keep_open")]
    pub keep_launcher_open: bool,
    #[serde(rename = "sessionToken", default = "random_token")]
    pub session_token: String,
    #[serde(rename = "vanillaMode", default)]
    pub vanilla_mode: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PremiumOptions {
    #[serde(rename = "skipAdvertisement", default)]
    pub skip_advertisement: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AMTOptions {
    #[serde(rename = "accentColor", default = "default_accent")]
    pub accent_color: String,
    #[serde(rename = "badge", default = "default_badge")]
    pub badge: String,
    #[serde(rename = "displayName", default)]
    pub display_name: String,
    #[serde(rename = "badgeVisibility", default)]
    pub badge_visibility: BadgeVisibility,
    #[serde(rename = "equippedCape", default)]
    pub equipped_cape: Option<String>,
    #[serde(rename = "githubToken", default)]
    pub github_token: String,
    #[serde(rename = "glassOpacity", default = "default_glass_opacity")]
    pub glass_opacity: f64,
    #[serde(rename = "fontSize", default = "default_font_size")]
    pub font_size: String,
    #[serde(rename = "backgroundPreset", default)]
    pub background_preset: String,
    #[serde(rename = "profileVisibility", default)]
    pub profile_visibility: String,
    #[serde(rename = "backendUrl", default = "default_backend_url")]
    pub backend_url: String,
    #[serde(rename = "githubRepoOwner", default = "default_github_owner")]
    pub github_repo_owner: String,
    #[serde(rename = "githubRepoName", default = "default_github_repo")]
    pub github_repo_name: String,
    #[serde(rename = "githubClientId", default)]
    pub github_client_id: String,
    #[serde(rename = "githubClientSecret", default)]
    pub github_client_secret: String,
    #[serde(rename = "language", default = "default_language")]
    pub language: String,
    #[serde(rename = "themePreset", default)]
    pub theme_preset: String,
    #[serde(rename = "glassBlur", default = "default_glass_blur")]
    pub glass_blur: f64,
    #[serde(rename = "bgBlur", default)]
    pub bg_blur: bool,
    #[serde(rename = "animations", default = "default_true")]
    pub animations: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BadgeVisibility {
    #[serde(rename = "launcher", default = "default_true")]
    pub launcher: bool,
    #[serde(rename = "inGame", default = "default_true")]
    pub in_game: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct BranchOptions {
    #[serde(rename = "modStates", default)]
    pub mod_states: HashMap<String, bool>,
    #[serde(rename = "customModStates", default)]
    pub custom_mod_states: HashMap<String, bool>,
}

fn default_accent() -> String {
    "#ACC4DE".to_string()
}

fn default_badge() -> String {
    "AMT".to_string()
}

fn default_glass_opacity() -> f64 {
    0.05
}

fn default_font_size() -> String {
    "M".to_string()
}

fn default_true() -> bool {
    true
}

fn default_backend_url() -> String {
    "https://amt-entertainment.github.io/AMT-Client-Backend".to_string()
}

fn default_github_owner() -> String {
    "AMT-Entertainment".to_string()
}

fn default_github_repo() -> String {
    "AMT-Client-Backend".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

fn default_glass_blur() -> f64 {
    24.0
}

impl Default for BadgeVisibility {
    fn default() -> Self {
        Self {
            launcher: true,
            in_game: true,
        }
    }
}

    impl Default for AMTOptions {
    fn default() -> Self {
        Self {
            accent_color: default_accent(),
            badge: default_badge(),
            display_name: String::new(),
            badge_visibility: BadgeVisibility::default(),
            equipped_cape: None,
            github_token: String::new(),
            glass_opacity: default_glass_opacity(),
            font_size: default_font_size(),
            background_preset: String::new(),
            profile_visibility: "public".to_string(),
            backend_url: default_backend_url(),
            github_repo_owner: default_github_owner(),
            github_repo_name: default_github_repo(),
            github_client_id: String::new(),
            github_client_secret: String::new(),
            language: default_language(),
            theme_preset: String::new(),
            glass_blur: default_glass_blur(),
            bg_blur: false,
            animations: true,
        }
    }
}

impl Options {
    pub async fn load(app_data: &Path) -> Result<Self> {
        let file_content = fs::read(app_data.join("options.json")).await?;

        if let Ok(mut options) = serde_json::from_slice::<Self>(&file_content) {
            // Migrate single account to multi-account list
            if options.start_options.accounts.is_empty() {
                if let Some(account) = options.start_options.minecraft_account.take() {
                    options.start_options.accounts.push(account);
                    options.start_options.active_account_index = 0;
                }
            }
            info!("Successfully loaded options from file");
            return Ok(options);
        }

        if let Ok(legacy) = serde_json::from_slice::<LegacyOptions>(&file_content) {
            info!("Successfully loaded legacy options from file");
            return Ok(Self::from_legacy(legacy));
        }

        Ok(serde_json::from_slice::<Self>(&file_content)?)
    }

    fn from_legacy(legacy: LegacyOptions) -> Self {
        let accounts = legacy.current_account
            .map(|a| vec![a])
            .unwrap_or_default();
        Self {
            start_options: StartOptions {
                custom_data_path: legacy.custom_data_path,
                java_distribution: DistributionSelection::default(),
                minecraft_account: None,
                accounts,
                active_account_index: 0,
                jvm_args: None,
                memory: 4096,
            },
            version_options: VersionOptions {
                branch_name: None,
                build_id: -1,
                options: legacy.branch_options,
            },
            launcher_options: LauncherOptions {
                first_run: false,
                keep_launcher_open: legacy.keep_launcher_open,
                show_nightly_builds: legacy.show_nightly_builds,
                concurrent_downloads: legacy.concurrent_downloads as u32,
                session_token: random_token(),
                vanilla_mode: false,
            },
            premium_options: PremiumOptions {
                skip_advertisement: legacy.skip_advertisement,
            },
            amt_options: AMTOptions::default(),
        }
    }

    pub async fn store(&self, app_data: &Path) -> Result<()> {
        fs::write(app_data.join("options.json"), serde_json::to_string(&self)?).await?;
        Ok(())
    }
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            minecraft_account: None,
            accounts: Vec::new(),
            active_account_index: 0,
            java_distribution: DistributionSelection::default(),
            custom_data_path: String::new(),
            jvm_args: None,
            memory: 4096,
        }
    }
}

impl Default for VersionOptions {
    fn default() -> Self {
        Self {
            branch_name: None,
            build_id: -1,
            options: HashMap::new(),
        }
    }
}

fn default_keep_open() -> bool { true }

impl Default for LauncherOptions {
    fn default() -> Self {
        Self {
            first_run: true,
            show_nightly_builds: false,
            keep_launcher_open: true,
            concurrent_downloads: 10,
            session_token: random_token(),
            vanilla_mode: false,
        }
    }
}

impl Default for PremiumOptions {
    fn default() -> Self {
        Self {
            skip_advertisement: false,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            start_options: StartOptions::default(),
            version_options: VersionOptions::default(),
            launcher_options: LauncherOptions::default(),
            premium_options: PremiumOptions::default(),
            amt_options: AMTOptions::default(),
        }
    }
}

fn default_memory() -> u64 {
    4096
}

fn random_token() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 16)
}

#[derive(Deserialize)]
#[allow(unused)]
pub(crate) struct LegacyOptions {
    #[serde(rename = "keepLauncherOpen")]
    pub keep_launcher_open: bool,
    #[serde(rename = "customDataPath", default)]
    pub custom_data_path: String,
    #[serde(rename = "showNightlyBuilds")]
    pub show_nightly_builds: bool,
    #[serde(rename = "memoryPercentage")]
    pub memory_percentage: i32,
    #[serde(rename = "customJavaPath", default)]
    pub custom_java_path: String,
    #[serde(rename = "selectedBranch")]
    pub selected_branch: Option<String>,
    #[serde(rename = "selectedBuild")]
    pub selected_build: Option<i32>,
    #[serde(rename = "skipAdvertisement", default)]
    pub skip_advertisement: bool,
    #[serde(rename = "currentAccount")]
    pub current_account: Option<MinecraftAccount>,
    #[serde(rename = "branchOptions", default)]
    pub branch_options: HashMap<String, BranchOptions>,
    #[serde(rename = "concurrentDownloads")]
    pub concurrent_downloads: i32,
}
