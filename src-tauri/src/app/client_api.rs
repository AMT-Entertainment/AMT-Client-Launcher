/*
 * This file is part of LiquidLauncher (https://github.com/CCBlueX/LiquidLauncher)
 *
 * Copyright (c) 2015 - 2025 CCBlueX
 *
 * LiquidLauncher is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * LiquidLauncher is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with LiquidLauncher. If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::BTreeMap;

use crate::minecraft::java::JavaDistribution;
use crate::utils::get_maven_artifact_path;
use crate::HTTP_CLIENT;
use anyhow::{Error, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::{debug, debug_span, error, info, warn};

/// Default API endpoint url (used as fallback if no custom URL is configured)
pub const DEFAULT_API_URL: &str = "https://amt-entertainment.github.io/AMT-Client-Backend";

pub const API_V1: &str = "api/v1";
pub const API_V3: &str = "api/v3";

#[derive(Serialize, Deserialize)]
pub struct Client {
    url: String,
    // To show a warning to the user when using a non-secure connection,
    // we need to pass this information to the frontend.
    is_secure: bool,
    session_token: String
}

impl Client {
    pub fn new(host: &str, session_token: String) -> Self {
        Self {
            url: host.to_string(),
            is_secure: host.starts_with("https://"),
            session_token
        }
    }

    /// Finds the first available API endpoint
    /// and returns a [Client] instance with the endpoint set.
    ///
    /// If `custom_url` is provided, it is tried first. Falls back to [DEFAULT_API_URL].
    ///
    /// Returns [String] as error with technical information if no API endpoint is reachable.
    pub async fn lookup(session_token: String, custom_url: Option<String>) -> Result<Self, String> {
        let span = debug_span!("api_lookup");
        let _guard = span.enter();

        let mut urls: Vec<String> = Vec::new();

        // Custom URL first if provided
        if let Some(url) = custom_url {
            if !url.is_empty() {
                urls.push(url);
            }
        }

        // Fallback to default
        urls.push(DEFAULT_API_URL.to_string());

        // AMT Client will show a technical information section in the error dialog,
        // when the API endpoint is not reachable.
        // This is to help the user to understand the issue.
        let mut technical_information = String::new();

        info!(parent: &span, "Looking up available API endpoints");
        for endpoint in &urls {
            if !technical_information.is_empty() {
                technical_information.push('\n');
            }

            let is_secure = endpoint.starts_with("https://");
            if !is_secure {
                warn!(parent: &span, "Falling back to Non-SSL '{}' endpoint.", endpoint);
            }

            // Probe a known endpoint to verify the API is functional
            let probe_url = format!("{}/api/v1/version/branches.json", endpoint);
            let is_success = HTTP_CLIENT
                .get(&probe_url)
                .send()
                .await
                .map_err(|err| {
                    let err = Into::<Error>::into(err);
                    technical_information.push_str(&format!(
                        "Failed to connect to API endpoint '{}': {:?}\n",
                        endpoint, err
                    ));
                    error!(
                        parent: &span,
                        "Failed to connect to API endpoint '{}': {:?}",
                        endpoint, err
                    );
                    err
                })
                .is_ok_and(|r| {
                    let status = r.status();
                    let is_success = status.is_success();
                    if !is_success {
                        technical_information.push_str(&format!(
                            "API endpoint '{}' returned status code: {}\n",
                            endpoint, status
                        ));
                        error!(
                            parent: &span,
                            "API endpoint '{}' returned status code: {}",
                            endpoint, status
                        );
                    }
                    is_success
                });

            if is_success {
                debug!(parent: &span, "API endpoint '{}' is available", endpoint);
                return Ok(Self::new(endpoint, session_token));
            }
        }

        Err(technical_information)
    }

    /// Check if the API endpoint is secure
    pub fn is_secure(&self) -> bool {
        self.is_secure
    }

    pub fn url(&self) -> &str {
        &self.url
    }
    
    pub fn session_token(&self) -> &str {
        &self.session_token
    }

    /// Request all blog posts
    pub async fn blog_posts(&self, page: u32) -> Result<PaginatedResponse<BlogPost>> {
        self.request_from_endpoint(API_V3, &format!("blog?page={}", page)).await
    }

    /// Request all available branches
    pub async fn branches(&self) -> Result<Branches> {
        self.request_from_endpoint(API_V1, "version/branches").await
    }

    /// Request all builds of branch
    pub async fn builds_by_branch(&self, branch: &str, release: bool) -> Result<Vec<Build>> {
        self.request_from_endpoint(API_V1, &if release {
            format!("version/builds/{}/release", branch)
        } else {
            format!("version/builds/{}", branch)
        })
        .await
    }

    /// Request launch manifest of specific build
    pub async fn fetch_launch_manifest(&self, build_id: u32) -> Result<LaunchManifest> {
        self.request_from_endpoint(API_V1, &format!("version/launch/{}", build_id))
            .await
    }

    /// Request list of downloadable mods for mc_version and used subsystem
    pub async fn fetch_mods(&self, mc_version: &str, subsystem: &str) -> Result<Vec<LoaderMod>> {
        self.request_from_endpoint(API_V1, &format!("version/mods/{}/{}", mc_version, subsystem))
            .await
    }

    /// Request changelog of specified build
    pub async fn fetch_changelog(&self, build_id: u32) -> Result<Changelog> {
        self.request_from_endpoint(API_V1, &format!("version/changelog/{}", build_id))
            .await
    }

    /// Request JSON formatted data from launcher API
    /// Appends `.json` suffix since the backend is served as static files.
    pub async fn request_from_endpoint<T: DeserializeOwned>(&self, api_version: &str, endpoint: &str) -> Result<T> {
        let path = if endpoint.ends_with(".json") {
            endpoint.to_string()
        } else {
            format!("{}.json", endpoint)
        };
        Ok(HTTP_CLIENT
            .get(format!("{}/{}/{}", self.url, api_version, path))
            .header("X-Session-Token", &self.session_token)
            .send()
            .await?
            .error_for_status()?
            .json::<T>()
            .await?)
    }

}

#[derive(Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: Pagination,
}

#[derive(Serialize, Deserialize)]
pub struct Pagination {
    pub current: u32,
    pub pages: u32,
    pub items: u32,
}

#[derive(Serialize, Deserialize)]
pub struct BlogPost {
    #[serde(rename = "postId")]
    pub post_id: u32,
    #[serde(rename = "postUid")]
    pub post_uid: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub date: NaiveDateTime,
    #[serde(rename = "bannerText")]
    pub banner_text: String,
    #[serde(rename = "bannerImageUrl")]
    pub banner_image_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Branches {
    #[serde(rename = "defaultBranch")]
    pub default_branch: String,
    pub branches: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Changelog {
    pub build: Build,
    pub changelog: String,
}

///
/// JSON struct of Build
///
#[derive(Serialize, Deserialize, Clone)]
pub struct Build {
    #[serde(rename = "buildId")]
    pub build_id: u32,
    #[serde(rename = "commitId")]
    pub commit_id: String,
    pub branch: String,
    pub subsystem: String,
    #[serde(rename = "lbVersion")]
    pub lb_version: String,
    #[serde(rename = "mcVersion")]
    pub mc_version: String,
    pub release: bool,
    pub date: DateTime<Utc>,
    pub message: String,
    pub url: String,
    #[serde(rename = "jreDistribution", default)]
    pub jre_distribution: JavaDistribution,
    #[serde(rename = "jreVersion")]
    pub jre_version: u32,
    #[serde(flatten)]
    pub subsystem_specific_data: SubsystemSpecificData,
}

///
/// Subsystem-specific data
/// This can be used for any subsystem, but for now it is only implemented for Fabric.
/// It has to be turned into an Enum to be able to decide on it's own for specific data, but for now this is not required.
///
#[derive(Serialize, Deserialize, Clone)]
pub struct SubsystemSpecificData {
    // Additional data
    #[serde(rename = "fabricApiVersion")]
    pub fabric_api_version: String,
    #[serde(rename = "fabricLoaderVersion")]
    pub fabric_loader_version: String,
    #[serde(rename = "kotlinVersion")]
    pub kotlin_version: String,
    #[serde(rename = "kotlinModVersion")]
    pub kotlin_mod_version: String,
}

///
/// JSON struct of Launch Manifest
///
#[derive(Deserialize)]
pub struct LaunchManifest {
    pub build: Build,
    pub subsystem: LoaderSubsystem,
    pub mods: Vec<LoaderMod>,
    pub repositories: BTreeMap<String, String>,
}

///
/// JSON struct of mod
///
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoaderMod {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    #[serde(alias = "default")]
    pub enabled: bool,
    pub name: String,
    pub source: ModSource,
}

///
/// JSON struct of ModSource (the method to be used for downloading the mod)
///
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type")]
pub enum ModSource {
    #[serde(rename = "skip")]
    #[serde(rename_all = "camelCase")]
    SkipAd {
        artifact_name: String,
        url: String,
        #[serde(default)]
        extract: bool,
    },
    #[serde(rename = "repository")]
    #[serde(rename_all = "camelCase")]
    Repository {
        repository: String,
        artifact: String,
    },
    #[serde(rename = "local")]
    #[serde(rename_all = "camelCase")]
    Local { file_name: String },
}

impl ModSource {
    pub fn get_path(&self) -> Result<String> {
        Ok(match self {
            ModSource::SkipAd { artifact_name, .. } => format!("{}.jar", artifact_name),
            ModSource::Repository {
                repository: _repository,
                artifact,
            } => get_maven_artifact_path(artifact)?,
            ModSource::Local { file_name } => file_name.clone(),
        })
    }
}

///
/// JSON struct of subsystem
///
#[derive(Deserialize)]
#[serde(tag = "name")]
pub enum LoaderSubsystem {
    #[serde(rename = "fabric", rename_all = "camelCase")]
    Fabric {
        manifest: String,
        mod_directory: String,
    },
    #[serde(rename = "forge", rename_all = "camelCase")]
    Forge {
        manifest: String,
        mod_directory: String,
    },
}


