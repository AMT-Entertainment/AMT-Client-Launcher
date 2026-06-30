use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::LAUNCHER_DIRECTORY;

use super::encryption;

/// A user profile (linked to GitHub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub github_username: String,
    pub display_name: String,
    pub avatar_url: String,
    pub minecraft_uuid: Option<String>,
    pub bio: String,
    pub joined_at: String,
}

/// A post in the social feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub author_avatar: String,
    pub content: String,
    pub image_url: Option<String>,
    pub post_type: PostType,
    pub created_at: String,
    pub likes: u32,
    pub comments: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PostType {
    Text,
    Image,
    ServerInvite(ServerInviteData),
    CosmeticsShare(CosmeticsShareData),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerInviteData {
    pub server_id: String,
    pub server_name: String,
    pub server_type: String,
    pub mc_version: String,
    pub address: String,
    pub player_count: u32,
    pub max_players: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CosmeticsShareData {
    pub cape_id: Option<String>,
    pub badge_text: Option<String>,
    pub description: String,
}

/// A direct message (content is encrypted at rest)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    /// Encrypted content (AES-256-GCM, base64-encoded)
    pub content: String,
    pub created_at: String,
    pub read: bool,
}

impl DirectMessage {
    /// Get decrypted content using the recipient's key
    pub fn decrypted_content(&self, profile_id: &str) -> String {
        encryption::decrypt(&self.content, profile_id).unwrap_or_else(|e| {
            warn!("Failed to decrypt DM content: {}", e);
            "[encrypted]".to_string()
        })
    }
}

/// Friend relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendShip {
    pub user_id: String,
    pub friend_id: String,
    pub status: FriendStatus,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FriendStatus {
    Pending,
    Accepted,
    Blocked,
}

/// Full social data for a user
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SocialData {
    pub profile: Option<UserProfile>,
    pub friends: Vec<FriendShip>,
    pub posts: Vec<Post>,
    pub messages: Vec<DirectMessage>,
    pub feed: Vec<Post>,
}

impl SocialData {
    pub fn path() -> PathBuf {
        LAUNCHER_DIRECTORY
            .data_dir()
            .join("social_data.json")
    }

    pub async fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            return Self::default();
        }
        match tokio::fs::read_to_string(&path).await {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub async fn save(&self) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        tokio::fs::write(Self::path(), content)
            .await
            .map_err(|e| format!("Failed to save social data: {}", e))
    }
}

/// Create a new post locally
pub async fn create_post(
    content: String,
    image_url: Option<String>,
    post_type: PostType,
) -> Result<Post, String> {
    let mut data = SocialData::load().await;
    let profile = data.profile.as_ref().ok_or("No profile set")?;

    let post = Post {
        id: uuid::Uuid::new_v4().to_string(),
        author_id: profile.id.clone(),
        author_name: profile.display_name.clone(),
        author_avatar: profile.avatar_url.clone(),
        content,
        image_url,
        post_type,
        created_at: Utc::now().to_rfc3339(),
        likes: 0,
        comments: 0,
    };

    data.posts.insert(0, post.clone());
    data.feed.insert(0, post.clone());
    data.save().await?;

    Ok(post)
}

/// Send a direct message with encrypted content
pub async fn send_message(to_username: &str, content: String) -> Result<DirectMessage, String> {
    let mut data = SocialData::load().await;
    let profile = data.profile.as_ref().ok_or("No profile set")?;

    // Initialize encryption keys for this profile
    encryption::init_profile_keys(&profile.id).await?;

    // Encrypt the message content
    let encrypted = encryption::encrypt(&content, &profile.id)?;

    let msg = DirectMessage {
        id: uuid::Uuid::new_v4().to_string(),
        from_id: profile.id.clone(),
        to_id: to_username.to_string(),
        content: encrypted,
        created_at: Utc::now().to_rfc3339(),
        read: false,
    };

    data.messages.push(msg.clone());
    data.save().await?;

    Ok(msg)
}

/// Get messages with a specific user
pub async fn get_conversation(with_username: &str) -> Result<Vec<DirectMessage>, String> {
    let data = SocialData::load().await;

    let mut msgs: Vec<DirectMessage> = data
        .messages
        .into_iter()
        .filter(|m| m.from_id == with_username || m.to_id == with_username)
        .collect();

    msgs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(msgs)
}

/// Set or update the local user profile
pub async fn set_profile(profile: UserProfile) -> Result<(), String> {
    // Initialize encryption keys
    encryption::init_profile_keys(&profile.id).await?;

    let mut data = SocialData::load().await;
    data.profile = Some(profile);
    data.save().await
}

/// Add a friend request
pub async fn add_friend(friend_username: &str) -> Result<(), String> {
    let mut data = SocialData::load().await;
    let profile = data.profile.as_ref().ok_or("No profile set")?;

    // Check if already friends
    if data
        .friends
        .iter()
        .any(|f| f.friend_id == friend_username && f.status == FriendStatus::Accepted)
    {
        return Err("Already friends".to_string());
    }

    data.friends.push(FriendShip {
        user_id: profile.id.clone(),
        friend_id: friend_username.to_string(),
        status: FriendStatus::Pending,
        created_at: Utc::now().to_rfc3339(),
    });

    data.save().await
}

/// Accept a friend request
pub async fn accept_friend(friend_username: &str) -> Result<(), String> {
    let mut data = SocialData::load().await;

    if let Some(friend) = data
        .friends
        .iter_mut()
        .find(|f| f.friend_id == friend_username && f.status == FriendStatus::Pending)
    {
        friend.status = FriendStatus::Accepted;
    }

    data.save().await
}

/// Create a server invite post
pub async fn create_server_invite(
    server_id: String,
    server_name: String,
    server_type: String,
    mc_version: String,
    address: String,
    player_count: u32,
    max_players: u32,
) -> Result<Post, String> {
    let content = format!(
        "🚀 Join my {} server! {} is running {} on {}",
        server_type, server_name, mc_version, address
    );

    create_post(
        content,
        None,
        PostType::ServerInvite(ServerInviteData {
            server_id,
            server_name,
            server_type,
            mc_version,
            address,
            player_count,
            max_players,
        }),
    )
    .await
}
