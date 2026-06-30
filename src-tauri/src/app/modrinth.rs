use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::HTTP_CLIENT;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub hits: Vec<SearchHit>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub project_type: String,
    pub slug: String,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub follows: u64,
    pub date_created: String,
    pub date_modified: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub description: String,
    pub body_url: Option<String>,
    pub body: Option<String>,
    pub icon_url: Option<String>,
    pub project_type: String,
    pub downloads: u64,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub slug: String,
    pub followers: u64,
    pub date_created: String,
    pub date_modified: String,
    pub license: Option<License>,
    pub client_side: String,
    pub server_side: String,
    pub gallery: Vec<GalleryImage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GalleryImage {
    pub url: String,
    pub raw_url: Option<String>,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: String,
    pub ordering: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<VersionFile>,
    pub dependencies: Vec<Dependency>,
    pub date_published: String,
    pub version_type: String, // release, beta, alpha
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFile {
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub dependency_type: String, // required, optional, incompatible, embedded
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModrinthProjectType {
    Mod,
    Shader,
    ResourcePack,
    ModPack,
    DataPack,
}

impl ModrinthProjectType {
    pub fn facet(&self) -> &str {
        match self {
            Self::Mod => "mod",
            Self::Shader => "shader",
            Self::ResourcePack => "resourcepack",
            Self::ModPack => "modpack",
            Self::DataPack => "datapack",
        }
    }

    pub fn install_dir(&self) -> &str {
        match self {
            Self::Mod => "mods",
            Self::Shader => "shaderpacks",
            Self::ResourcePack => "resourcepacks",
            Self::ModPack => "modpacks",
            Self::DataPack => "datapacks",
        }
    }
}

/// Search Modrinth projects
pub async fn search(
    query: &str,
    project_type: &ModrinthProjectType,
    limit: u32,
    offset: u32,
) -> Result<SearchResult, String> {
    let facets = serde_json::json!([[format!("project_type:{}", project_type.facet())]]);
    let facets_json = serde_json::to_string(&facets).map_err(|e| format!("Failed to encode facets: {}", e))?;

    let url = format!(
        "{}/search?query={}&facets={}&limit={}&offset={}",
        MODRINTH_API,
        urlencoding(&query),
        urlencoding(&facets_json),
        limit,
        offset
    );

    let response = HTTP_CLIENT
        .get(&url)
        .header("User-Agent", "AMT-Client/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to search Modrinth: {}", e))?;

    let result: SearchResult = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse search results: {}", e))?;

    Ok(result)
}

/// Get project details
pub async fn get_project(project_id: &str) -> Result<Project, String> {
    let url = format!("{}/project/{}", MODRINTH_API, project_id);

    let response = HTTP_CLIENT
        .get(&url)
        .header("User-Agent", "AMT-Client/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to get project: {}", e))?;

    let project: Project = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse project: {}", e))?;

    Ok(project)
}

/// Get all versions for a project
pub async fn get_versions(project_id: &str) -> Result<Vec<Version>, String> {
    let url = format!("{}/project/{}/version", MODRINTH_API, project_id);

    let response = HTTP_CLIENT
        .get(&url)
        .header("User-Agent", "AMT-Client/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to get versions: {}", e))?;

    let versions: Vec<Version> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse versions: {}", e))?;

    Ok(versions)
}

/// Get versions filtered by game version and loader
pub async fn get_filtered_versions(
    project_id: &str,
    game_version: &str,
    loader: &str,
) -> Result<Vec<Version>, String> {
    let url = format!(
        "{}/project/{}/version?game_versions={}&loaders={}",
        MODRINTH_API,
        project_id,
        urlencoding(&serde_json::to_string(&[game_version]).unwrap()),
        urlencoding(&serde_json::to_string(&[loader]).unwrap()),
    );

    let response = HTTP_CLIENT
        .get(&url)
        .header("User-Agent", "AMT-Client/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to get filtered versions: {}", e))?;

    let versions: Vec<Version> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse versions: {}", e))?;

    Ok(versions)
}

/// Download a file from a URL and return the bytes
pub async fn download_file(url: &str) -> Result<Vec<u8>, String> {
    let response = HTTP_CLIENT
        .get(url)
        .header("User-Agent", "AMT-Client/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to download file: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read file bytes: {}", e))?
        .to_vec();

    Ok(bytes)
}

/// Dependency resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyResolution {
    pub project_id: String,
    pub project_title: String,
    pub version_id: String,
    pub version_number: String,
    pub filename: String,
    pub file_url: String,
    pub dependency_type: String,
    pub dependencies: Vec<DependencyResolution>,
}

/// Resolve all dependencies for a given version ID
pub async fn resolve_dependencies(
    version_id: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
    max_depth: u32,
) -> Result<DependencyResolution, String> {
    let mut visited = HashSet::new();
    resolve_deps_recursive(version_id, game_version, loader, &mut visited, 0, max_depth).await
}

async fn resolve_deps_recursive(
    version_id: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
    visited: &mut HashSet<String>,
    depth: u32,
    max_depth: u32,
) -> Result<DependencyResolution, String> {
    if depth > max_depth {
        return Err("Max dependency depth exceeded".to_string());
    }

    if visited.contains(version_id) {
        return Err("Circular dependency detected".to_string());
    }
    visited.insert(version_id.to_string());

    // Fetch the version data
    let url = format!("{}/version/{}", MODRINTH_API, version_id);
    let response = HTTP_CLIENT
        .get(&url)
        .header("User-Agent", "AMT-Client/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch version: {}", e))?;

    let version: Version = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse version: {}", e))?;

    let primary_file = version.files.iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or("No files in version")?;

    // Get project info for the title
    let project = get_project(&version.project_id).await?;

    let mut child_deps = Vec::new();

    // Process each dependency
    for dep in &version.dependencies {
        if dep.dependency_type == "incompatible" {
            continue;
        }

        // If dependency has a version_id, resolve that version
        if let Some(dep_version_id) = &dep.version_id {
            match Box::pin(resolve_deps_recursive(
                dep_version_id,
                game_version,
                loader,
                visited,
                depth + 1,
                max_depth,
            )).await {
                Ok(resolved) => child_deps.push(resolved),
                Err(_) => continue, // Skip unresolvable deps
            }
        }
        // If dependency only has project_id, find the best matching version
        else if let Some(dep_project_id) = &dep.project_id {
            match find_best_version_for_dep(dep_project_id, game_version, loader).await {
                Ok(best_version) => {
                    match Box::pin(resolve_deps_recursive(
                        &best_version.id,
                        game_version,
                        loader,
                        visited,
                        depth + 1,
                        max_depth,
                    )).await {
                        Ok(resolved) => child_deps.push(resolved),
                        Err(_) => continue,
                    }
                }
                Err(_) => continue,
            }
        }
    }

    Ok(DependencyResolution {
        project_id: version.project_id.clone(),
        project_title: project.title.clone(),
        version_id: version.id.clone(),
        version_number: version.version_number.clone(),
        filename: primary_file.filename.clone(),
        file_url: primary_file.url.clone(),
        dependency_type: if depth == 0 { "root".to_string() } else { "required".to_string() },
        dependencies: child_deps,
    })
}

async fn find_best_version_for_dep(
    project_id: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
) -> Result<Version, String> {
    let versions = get_versions(project_id).await?;

    // Filter by game version and loader if specified
    let filtered: Vec<Version> = versions.into_iter()
        .filter(|v| {
            let gv_match = game_version.map_or(true, |gv| v.game_versions.contains(&gv.to_string()));
            let l_match = loader.map_or(true, |l| v.loaders.contains(&l.to_string()));
            gv_match && l_match
        })
        .collect();

    // Return the most recent release
    for v in &filtered {
        if v.version_type == "release" {
            return Ok(Version::clone(v));
        }
    }

    // Fallback to any version
    filtered.into_iter().next().ok_or_else(|| format!("No version found for project {}", project_id))
}

/// Get all files that need to be downloaded for a version (including dependencies)
pub async fn resolve_download_list(
    version_id: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
) -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();
    let mut visited = HashSet::new();

    collect_downloads_recursive(version_id, game_version, loader, &mut visited, &mut files, 0).await?;

    Ok(files)
}

async fn collect_downloads_recursive(
    version_id: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
    visited: &mut HashSet<String>,
    files: &mut Vec<(String, String)>,
    depth: u32,
) -> Result<(), String> {
    if depth > 10 || visited.contains(version_id) {
        return Ok(());
    }
    visited.insert(version_id.to_string());

    let url = format!("{}/version/{}", MODRINTH_API, version_id);
    let response = HTTP_CLIENT
        .get(&url)
        .header("User-Agent", "AMT-Client/0.1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch version: {}", e))?;

    let version: Version = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse version: {}", e))?;

    for file in &version.files {
        files.push((file.filename.clone(), file.url.clone()));
    }

    for dep in &version.dependencies {
        if dep.dependency_type == "incompatible" || dep.dependency_type == "embedded" {
            continue;
        }

        let dep_version_id = if let Some(dep_vid) = &dep.version_id {
            dep_vid.clone()
        } else if let Some(dep_pid) = &dep.project_id {
            match find_best_version_for_dep(dep_pid, game_version, loader).await {
                Ok(v) => v.id,
                Err(_) => continue,
            }
        } else {
            continue;
        };

        Box::pin(collect_downloads_recursive(
            &dep_version_id, game_version, loader, visited, files, depth + 1,
        )).await?;
    }

    Ok(())
}

fn urlencoding(input: &str) -> String {
    urlencoding::encode(input).to_string()
}
