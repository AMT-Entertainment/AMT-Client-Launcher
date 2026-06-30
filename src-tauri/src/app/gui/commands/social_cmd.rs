use crate::HTTP_CLIENT;
use serde::{Deserialize, Serialize};

fn get_social_api() -> String {
    std::env::var("SOCIAL_API_URL").unwrap_or_else(|_| {
        "https://amt-client-backend.onrender.com".to_string()
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct SocialUser {
    pub uuid: String,
    pub minecraft_username: String,
    pub badge: String,
    pub equipped_cape: Option<String>,
    pub youtube_link: String,
    pub joined_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct SocialPost {
    pub id: String,
    pub author_uuid: String,
    pub author_username: String,
    pub author_badge: String,
    pub content: String,
    pub post_type: String,
    pub attachment_data: Option<serde_json::Value>,
    pub created_at: String,
    pub likes: i32,
    pub liked_by_me: bool,
    pub hashtags: Vec<String>,
    pub mentions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FeedResponse {
    pub posts: Vec<SocialPost>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreatePostBody {
    author_uuid: String,
    content: String,
    post_type: Option<String>,
    attachment_data: Option<serde_json::Value>,
}

// ── Profile ──

#[tauri::command]
pub(crate) async fn social_register_user(
    uuid: String,
    minecraft_username: String,
    badge: String,
) -> Result<(), String> {
    let url = format!("{}/api/social/register", get_social_api());
    let body = serde_json::json!({
        "uuid": uuid,
        "minecraft_username": minecraft_username,
        "badge": badge
    });

    let resp = HTTP_CLIENT
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to register: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Registration failed: {}", resp.status()));
    }
    Ok(())
}

#[tauri::command]
pub(crate) async fn social_get_user_profile(uuid: String) -> Result<Option<SocialUser>, String> {
    let url = format!("{}/api/social/users/{}", get_social_api(), uuid);

    let resp = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user: {e}"))?;

    if resp.status() == 404 {
        return Ok(None);
    }

    let user = resp
        .json::<SocialUser>()
        .await
        .map_err(|e| format!("Failed to parse user: {e}"))?;
    Ok(Some(user))
}

#[tauri::command]
pub(crate) async fn social_update_profile(
    uuid: String,
    youtube_link: Option<String>,
    badge: Option<String>,
) -> Result<(), String> {
    let url = format!("{}/api/social/users/{}/update", get_social_api(), uuid);

    let body = serde_json::json!({
        "youtube_link": youtube_link,
        "badge": badge,
    });

    let resp = HTTP_CLIENT
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to update profile: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Profile update failed: {}", resp.status()));
    }
    Ok(())
}

// ── Posts ──

#[tauri::command]
pub(crate) async fn social_create_post(
    author_uuid: String,
    content: String,
    post_type: Option<String>,
    attachment_data: Option<serde_json::Value>,
) -> Result<SocialPost, String> {
    let url = format!("{}/api/social/posts", get_social_api());

    let body = CreatePostBody {
        author_uuid,
        content,
        post_type,
        attachment_data,
    };

    let resp = HTTP_CLIENT
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to create post: {e}"))?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Failed to create post: {text}"));
    }

    resp.json::<SocialPost>()
        .await
        .map_err(|e| format!("Failed to parse post: {e}"))
}

#[tauri::command]
pub(crate) async fn social_get_feed(
    tag: Option<String>,
    user: Option<String>,
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<FeedResponse, String> {
    let mut url = format!("{}/api/social/feed?", get_social_api());
    if let Some(t) = &tag {
        url.push_str(&format!("tag={}&", t));
    }
    if let Some(u) = &user {
        url.push_str(&format!("user={}&", u));
    }
    if let Some(s) = &search {
        url.push_str(&format!("search={}&", urlencoding::encode(&s)));
    }
    if let Some(l) = limit {
        url.push_str(&format!("limit={}&", l));
    }
    if let Some(o) = offset {
        url.push_str(&format!("offset={}&", o));
    }

    let resp = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch feed: {e}"))?;

    resp.json::<FeedResponse>()
        .await
        .map_err(|e| format!("Failed to parse feed: {e}"))
}

#[tauri::command]
pub(crate) async fn social_like_post(post_id: String, user_uuid: String) -> Result<bool, String> {
    let url = format!("{}/api/social/posts/{}/like", get_social_api(), post_id);

    let resp = HTTP_CLIENT
        .post(&url)
        .json(&serde_json::json!({ "user_uuid": user_uuid }))
        .send()
        .await
        .map_err(|e| format!("Failed to like post: {e}"))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    Ok(data["liked"].as_bool().unwrap_or(false))
}

#[tauri::command]
pub(crate) async fn social_get_hashtags() -> Result<Vec<serde_json::Value>, String> {
    let url = format!("{}/api/social/hashtags", get_social_api());

    let resp = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch hashtags: {e}"))?;

    resp.json::<Vec<serde_json::Value>>()
        .await
        .map_err(|e| format!("Failed to parse hashtags: {e}"))
}


