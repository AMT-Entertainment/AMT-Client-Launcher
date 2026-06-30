<script>
    import {onMount} from "svelte";
    import {invoke} from "@tauri-apps/api/core";
    import SocialPostCard from "./SocialPostCard.svelte";

    export let options;

    // ── Identity (locked to Minecraft account) ──
    $: activeAccount = options.start?.accounts?.[options.start?.activeAccountIndex ?? 0] || null;
    $: myUuid = activeAccount?.profile?.id || activeAccount?.uuid || '';
    $: myUsername = activeAccount?.profile?.name || activeAccount?.name || 'Unknown';
    $: myBadge = options?.amt_options?.badge || "AMT";
    $: backendUrl = options?.amt_options?.backendUrl || "https://amt-entertainment.github.io/AMT-Client-Backend";

    // ── State ──
    let activeTab = "feed";
    let feed = [];
    let myProfile = null;
    let editingYoutube = false;
    let youtubeLinkDraft = "";
    let loading = true;
    let posting = false;
    let error = "";
    let serverOnline = true;

    // Create post
    let showComposer = false;
    let postContent = "";
    let postAttachment = null;
    let selectedHashtag = null;

    // Trending
    let trendingHashtags = [];

    // Search
    let searchQuery = "";
    let searchResults = null;
    let searching = false;

    // Server share
    let amtServers = [];
    let showServerPicker = false;
    let customServerIp = "";
    let customServerName = "";
    let customServerVersion = "";
    let serverShareMode = "amt"; // "amt" or "custom"

    // Badge attach
    let attachBadge = false;

    const tabs = [
        { id: "feed", icon: "☰", label: "Feed" },
        { id: "popular", icon: "★", label: "Popular" },
        { id: "trending", icon: "▲", label: "Trending" },
        { id: "profile", icon: "◎", label: "Profile" },
    ];

    onMount(async () => {
        await loadFeed();
        await loadTrending();
        await registerMe();
    });

    function markOffline(e) {
        serverOnline = false;
        return e;
    }

    async function registerMe() {
        if (!myUuid) return;
        try {
            await invoke("social_register_user", {
                uuid: myUuid,
                minecraftUsername: myUsername,
                badge: myBadge,
            });
        } catch (e) {
            console.warn("Register failed (maybe already registered):", e);
            serverOnline = false;
        }
    }

    async function loadFeed(tag) {
        serverOnline = true;
        loading = true;
        error = "";
        selectedHashtag = tag || null;
        try {
            const result = await invoke("social_get_feed", {
                tag: tag || null,
                user: null,
                search: null,
                limit: 100,
                offset: null,
            });
            feed = result.posts || [];
        } catch (e) {
            error = String(e);
            feed = [];
            serverOnline = false;
        }
        loading = false;
    }

    async function loadTrending() {
        serverOnline = true;
        try {
            trendingHashtags = await invoke("social_get_hashtags");
        } catch (e) {
            console.warn("Failed to load hashtags:", e);
            trendingHashtags = [];
            serverOnline = false;
        }
    }

    async function loadMyProfile() {
        if (!myUuid) return;
        serverOnline = true;
        try {
            myProfile = await invoke("social_get_user_profile", { uuid: myUuid });
        } catch (e) {
            console.warn("Failed to load profile:", e);
            myProfile = null;
            serverOnline = false;
        }
    }

    async function doCreatePost() {
        if (!postContent.trim() && !postAttachment) return;
        posting = true;
        error = "";
        try {
            const newPost = await invoke("social_create_post", {
                authorUuid: myUuid,
                content: postContent.trim(),
                postType: postAttachment?.type || "text",
                attachmentData: postAttachment?.data || null,
            });
            feed = [newPost, ...feed];
            postContent = "";
            postAttachment = null;
            showComposer = false;
            attachBadge = false;
            await loadTrending();
        } catch (e) {
            error = String(e);
        }
        posting = false;
    }

    async function doLike(postId) {
        if (!myUuid) return;
        try {
            const liked = await invoke("social_like_post", {
                postId,
                userUuid: myUuid,
            });
            const post = feed.find(p => p.id === postId);
            if (post) {
                post.likes = liked ? post.likes + 1 : Math.max(0, post.likes - 1);
                post.liked_by_me = liked;
            }
        } catch (e) {
            console.warn("Like failed:", e);
        }
    }

    function handleSearch() {
        if (!searchQuery.trim()) {
            searchResults = null;
            loadFeed();
            return;
        }
        searching = true;
        invoke("social_get_feed", {
            tag: null,
            user: null,
            search: searchQuery.trim(),
            limit: 50,
            offset: null,
        }).then(result => {
            searchResults = result.posts || [];
            searching = false;
        }).catch(e => {
            error = String(e);
            searching = false;
        });
    }

    function openServerShare() {
        invoke("server_list").then(servers => {
            amtServers = servers || [];
        }).catch(() => {
            amtServers = [];
        });
        showServerPicker = true;
        serverShareMode = "amt";
    }

    function attachAmtServer(server) {
        postAttachment = {
            type: "server_invite",
            data: {
                server_id: server.id,
                server_name: server.name,
                server_type: server.server_type,
                mc_version: server.mc_version,
                address: `localhost:${server.port}`,
                player_count: 0,
                max_players: server.max_players || 20,
            }
        };
        showServerPicker = false;
    }

    function attachCustomServer() {
        if (!customServerIp.trim() || !customServerName.trim()) return;
        postAttachment = {
            type: "server_invite",
            data: {
                server_id: "custom",
                server_name: customServerName.trim(),
                server_type: "Custom",
                mc_version: customServerVersion.trim() || "Unknown",
                address: customServerIp.trim(),
                player_count: 0,
                max_players: 0,
            }
        };
        customServerIp = "";
        customServerName = "";
        customServerVersion = "";
        showServerPicker = false;
    }

    function attachBadgeShare() {
        postAttachment = {
            type: "badge_showcase",
            data: {
                badge: myBadge,
                minecraft_username: myUsername,
            }
        };
        attachBadge = true;
    }

    function attachCapeShare() {
        postAttachment = {
            type: "cosmetics_share",
            data: {
                cape_id: options?.amt_options?.equippedCape || null,
                badge_text: myBadge,
            }
        };
    }

    function formatTime(iso) {
        if (!iso) return "";
        const d = new Date(iso);
        if (isNaN(d.getTime())) return "";
        const now = new Date();
        const diffMs = now - d;
        const mins = Math.floor(diffMs / 60000);
        if (mins < 1) return "just now";
        if (mins < 60) return `${mins}m ago`;
        const hours = Math.floor(mins / 60);
        if (hours < 24) return `${hours}h ago`;
        const days = Math.floor(hours / 24);
        if (days < 7) return `${days}d ago`;
        return d.toLocaleDateString();
    }
</script>

<div class="social-hub">
    <!-- Header with search -->
    <div class="social-header">
        <div class="search-bar">
            <input
                type="text"
                placeholder="Search posts..."
                bind:value={searchQuery}
                on:keydown={(e) => e.key === 'Enter' && handleSearch()}
            />
            <button class="search-btn" on:click={handleSearch}>Search</button>
        </div>
        <button class="create-post-btn" on:click={() => { showComposer = true; postAttachment = null; attachBadge = false; }}>
            + New Post
        </button>
    </div>

    <!-- Tabs -->
    <div class="tabs">
        {#each tabs as tab}
            <button
                class="tab"
                class:active={activeTab === tab.id}
                on:click={() => { activeTab = tab.id; if (tab.id === "feed") loadFeed(); if (tab.id === "profile") loadMyProfile(); }}
            >
                <span class="tab-icon">{tab.icon}</span>
                <span class="tab-label">{tab.label}</span>
            </button>
        {/each}
    </div>

    <!-- Error -->
    {#if error}
        <div class="error-banner">{error}</div>
    {/if}
    {#if !serverOnline}
        <div class="offline-banner">Social server offline — posts & profiles are unavailable</div>
    {/if}

    <!-- Hash filters -->
    {#if activeTab === "feed" && trendingHashtags.length > 0}
        <div class="hashtag-filters">
            {#if selectedHashtag}
                <button class="hashtag-chip active" on:click={() => loadFeed()}>
                    #<b>{selectedHashtag}</b> ✕
                </button>
            {/if}
            {#each trendingHashtags.slice(0, 8) as ht}
                {#if ht.tag !== selectedHashtag}
                    <button class="hashtag-chip" on:click={() => loadFeed(ht.tag)}>
                        #<b>{ht.tag}</b> <span class="ht-count">{ht.count}</span>
                    </button>
                {/if}
            {/each}
        </div>
    {/if}

    <!-- Tab Content -->
    <div class="tab-content">
        {#if activeTab === "feed" || activeTab === "popular" || activeTab === "trending"}
            {#if loading}
                <div class="loading">
                    <div class="amt-skeleton" style="height:90px;width:100%;margin-bottom:12px;"></div>
                    <div class="amt-skeleton" style="height:90px;width:100%;margin-bottom:12px;"></div>
                    <div class="amt-skeleton" style="height:90px;width:80%;"></div>
                </div>
            {:else if searchResults !== null}
                <div class="search-results-header">Search: "{searchQuery}" ({searchResults.length} results)</div>
                {#each searchResults as post}
                    <SocialPostCard {post} on:like={() => doLike(post.id)} on:tag={(e) => loadFeed(e.detail)} />
                {:else}
                    <div class="empty-state">No results found</div>
                {/each}
            {:else if feed.length === 0}
                <div class="empty-state">
                    <p>No posts yet. Be the first to share something!</p>
                </div>
            {:else}
                {#each feed as post}
                    <SocialPostCard {post} on:like={() => doLike(post.id)} on:tag={(e) => loadFeed(e.detail)} />
                {/each}
            {/if}
        {:else if activeTab === "profile"}
            <div class="profile-section">
                <div class="profile-card">
                    <img
                        class="profile-avatar"
                        src="https://mc-heads.net/avatar/{myUuid}/100"
                        alt={myUsername}
                        on:error={(e) => e.target.src = "https://mc-heads.net/avatar/MHF_Steve/100"}
                    />
                    <div class="profile-info">
                        <h2>{myUsername}</h2>
                        <div class="profile-badge">
                            <span class="badge-chip">{myBadge}</span>
                            <span class="badge-note">Badge is set in Cosmetics → Badge</span>
                        </div>
                    </div>
                </div>

                <div class="profile-links">
                    {#if myProfile?.youtube_link}
                        <a class="yt-link" href={myProfile.youtube_link} target="_blank" rel="noopener">
                            ▶ YouTube Channel
                        </a>
                    {:else}
                        <span class="yt-link missing">No YouTube channel linked</span>
                    {/if}
                    <button class="edit-yt-btn" on:click={() => { editingYoutube = true; youtubeLinkDraft = myProfile?.youtube_link || ''; }}>
                        ✎ {myProfile?.youtube_link ? 'Edit' : 'Add'} YouTube
                    </button>
                </div>

                {#if editingYoutube}
                    <div class="yt-editor">
                        <input
                            type="url"
                            class="yt-input"
                            placeholder="https://youtube.com/@yourchannel"
                            bind:value={youtubeLinkDraft}
                        />
                        <div class="yt-actions">
                            <button class="yt-save" on:click={async () => {
                                try {
                                    await invoke("social_update_profile", {
                                        uuid: myUuid,
                                        youtubeLink: youtubeLinkDraft || null,
                                        badge: null,
                                    });
                                    if (myProfile) myProfile.youtube_link = youtubeLinkDraft;
                                    editingYoutube = false;
                                } catch (e) {
                                    error = String(e);
                                }
                            }}>Save</button>
                            <button class="yt-cancel" on:click={() => { editingYoutube = false; }}>Cancel</button>
                        </div>
                    </div>
                {/if}

                <div class="profile-stats">
                    <div class="stat">
                        <span class="stat-value">{feed.filter(p => p.author_uuid === myUuid).length}</span>
                        <span class="stat-label">Posts</span>
                    </div>
                    <div class="stat">
                        <span class="stat-value">{feed.filter(p => p.author_uuid === myUuid).reduce((s, p) => s + p.likes, 0)}</span>
                        <span class="stat-label">Likes Received</span>
                    </div>
                </div>

                <h3 style="margin: 16px 0 8px; font-size: 13px; color: rgba(255,255,255,0.6);">My Posts</h3>
                {#each feed.filter(p => p.author_uuid === myUuid) as post}
                    <SocialPostCard {post} on:like={() => doLike(post.id)} on:tag={(e) => loadFeed(e.detail)} />
                {:else}
                    <div class="empty-state" style="padding: 20px;">No posts yet</div>
                {/each}
            </div>
        {/if}
    </div>
</div>

<!-- Composer Modal -->
{#if showComposer}
    <div class="amt-modal-overlay" role="dialog" aria-modal="true" on:click={() => { if (!posting) showComposer = false; }} on:keydown={(e) => e.key === 'Escape' && (showComposer = false)}>
        <div class="amt-modal" style="max-width:520px;" on:click|stopPropagation>
            <div class="amt-modal-header">
                <h2>Create Post</h2>
                <button class="amt-modal-close" on:click={() => showComposer = false}>✕</button>
            </div>
            <div class="amt-modal-body" style="display:flex;flex-direction:column;gap:12px;">
                <!-- Author identity -->
                <div class="composer-identity">
                    <img class="composer-avatar" src="https://mc-heads.net/avatar/{myUuid}/32" alt="" />
                    <span class="composer-name">{myUsername}</span>
                    <span class="badge-chip small">{myBadge}</span>
                </div>

                <textarea
                    class="composer-input"
                    placeholder="What's on your mind? Use @username and #hashtags..."
                    bind:value={postContent}
                    rows="4"
                ></textarea>

                <!-- Attachment preview -->
                {#if postAttachment}
                    <div class="attachment-preview">
                        {#if postAttachment.type === "server_invite"}
                            <div class="attach-card">
                                <span class="attach-icon">⛁</span>
                                <span><b>{postAttachment.data.server_name}</b> — {postAttachment.data.server_type} ({postAttachment.data.mc_version})</span>
                                <span class="attach-addr">{postAttachment.data.address}</span>
                                <button class="attach-remove" on:click={() => postAttachment = null}>✕</button>
                            </div>
                        {:else if postAttachment.type === "badge_showcase"}
                            <div class="attach-card">
                                <span class="attach-icon">◈</span>
                                <span>Badge: <b>{postAttachment.data.badge}</b></span>
                                <button class="attach-remove" on:click={() => { postAttachment = null; attachBadge = false; }}>✕</button>
                            </div>
                        {:else if postAttachment.type === "cosmetics_share"}
                            <div class="attach-card">
                                <span class="attach-icon">◈</span>
                                <span>Cape showcase</span>
                                <button class="attach-remove" on:click={() => postAttachment = null}>✕</button>
                            </div>
                        {/if}
                    </div>
                {/if}

                <!-- Attachment buttons -->
                <div class="attach-tools">
                    <button class="tool-btn" on:click={openServerShare} title="Share a server">
                        ⛁ Server
                    </button>
                    <button class="tool-btn" on:click={attachBadgeShare} disabled={attachBadge} title="Showcase your badge">
                        ◈ Badge
                    </button>
                    <button class="tool-btn" on:click={attachCapeShare} title="Share your equipped cape">
                        ◇ Cape
                    </button>
                </div>

                <button
                    class="publish-btn"
                    on:click={doCreatePost}
                    disabled={(!postContent.trim() && !postAttachment) || posting}
                >
                    {posting ? "Posting..." : "Post"}
                </button>
            </div>
        </div>
    </div>
{/if}

<!-- Server Picker Modal -->
{#if showServerPicker}
    <div class="amt-modal-overlay" role="dialog" aria-modal="true" on:click={() => showServerPicker = false} on:keydown={(e) => e.key === 'Escape' && (showServerPicker = false)}>
        <div class="amt-modal" style="max-width:440px;" on:click|stopPropagation>
            <div class="amt-modal-header">
                <h2>Share a Server</h2>
                <button class="amt-modal-close" on:click={() => showServerPicker = false}>✕</button>
            </div>
            <div class="amt-modal-body" style="display:flex;flex-direction:column;gap:12px;">
                <div class="share-mode-tabs">
                    <button class="mode-tab" class:active={serverShareMode === "amt"} on:click={() => serverShareMode = "amt"}>My AMT Servers</button>
                    <button class="mode-tab" class:active={serverShareMode === "custom"} on:click={() => serverShareMode = "custom"}>Custom Server</button>
                </div>

                {#if serverShareMode === "amt"}
                    {#if amtServers.length === 0}
                        <p style="color:rgba(255,255,255,0.4);font-size:13px;">No AMT servers created yet. Go to Servers to create one.</p>
                    {:else}
                        {#each amtServers as server}
                            <button class="server-option" on:click={() => attachAmtServer(server)}>
                                <span class="srv-name">{server.name}</span>
                                <span class="srv-meta">{server.server_type} | {server.mc_version}</span>
                            </button>
                        {/each}
                    {/if}
                {:else}
                    <input class="custom-input" type="text" placeholder="Server Name" bind:value={customServerName} />
                    <input class="custom-input" type="text" placeholder="IP:Port (e.g. play.example.com:25565)" bind:value={customServerIp} />
                    <input class="custom-input" type="text" placeholder="Minecraft Version (optional)" bind:value={customServerVersion} />
                    <button class="publish-btn" on:click={attachCustomServer} disabled={!customServerName.trim() || !customServerIp.trim()}>
                        Attach to Post
                    </button>
                {/if}
            </div>
        </div>
    </div>
{/if}

<style>
    .social-hub {
        display: flex;
        flex-direction: column;
        height: 100%;
        gap: 12px;
    }

    .social-header {
        display: flex;
        gap: 8px;
        align-items: center;
    }

    .search-bar {
        flex: 1;
        display: flex;
        gap: 4px;
    }

    .search-bar input {
        flex: 1;
        background: rgba(255,255,255,0.06);
        border: 1px solid rgba(255,255,255,0.08);
        border-radius: 8px;
        padding: 8px 12px;
        color: white;
        font-size: 13px;
        outline: none;
    }

    .search-bar input:focus {
        border-color: rgba(172,196,222,0.4);
    }

    .search-btn {
        background: rgba(172,196,222,0.15);
        border: 1px solid rgba(172,196,222,0.2);
        border-radius: 8px;
        color: #ACC4DE;
        padding: 8px 14px;
        font-size: 12px;
        cursor: pointer;
    }

    .search-btn:hover {
        background: rgba(172,196,222,0.25);
    }

    .create-post-btn {
        background: rgba(172,196,222,0.2);
        border: 1px solid rgba(172,196,222,0.3);
        border-radius: 8px;
        color: white;
        padding: 8px 16px;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        white-space: nowrap;
    }

    .create-post-btn:hover {
        background: rgba(172,196,222,0.3);
    }

    .tabs {
        display: flex;
        gap: 4px;
        background: rgba(255,255,255,0.04);
        border-radius: 10px;
        padding: 3px;
    }

    .tab {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        padding: 8px;
        border: none;
        background: transparent;
        color: rgba(255,255,255,0.5);
        font-size: 12px;
        cursor: pointer;
        border-radius: 8px;
        transition: all 0.15s;
    }

    .tab.active {
        background: rgba(172,196,222,0.15);
        color: white;
    }

    .tab:hover:not(.active) {
        background: rgba(255,255,255,0.05);
        color: rgba(255,255,255,0.7);
    }

    .tab-icon {
        font-size: 14px;
    }

    .hashtag-filters {
        display: flex;
        gap: 6px;
        flex-wrap: wrap;
    }

    .hashtag-chip {
        background: rgba(172,196,222,0.1);
        border: 1px solid rgba(172,196,222,0.15);
        border-radius: 14px;
        padding: 4px 10px;
        font-size: 11px;
        color: rgba(255,255,255,0.6);
        cursor: pointer;
    }

    .hashtag-chip:hover, .hashtag-chip.active {
        background: rgba(172,196,222,0.2);
        color: white;
    }

    .ht-count {
        opacity: 0.5;
        font-size: 10px;
        margin-left: 2px;
    }

    .tab-content {
        flex: 1;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .empty-state {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 40px;
        color: rgba(255,255,255,0.3);
        font-size: 13px;
    }

    .loading {
        padding: 12px 0;
    }

    .error-banner {
        background: rgba(255,80,80,0.15);
        border: 1px solid rgba(255,80,80,0.2);
        border-radius: 8px;
        padding: 8px 12px;
        font-size: 12px;
        color: #ff6b6b;
    }

    .offline-banner {
        background: rgba(255,180,50,0.12);
        border: 1px solid rgba(255,180,50,0.2);
        border-radius: 8px;
        padding: 8px 12px;
        font-size: 12px;
        color: #ffb833;
    }

    .search-results-header {
        font-size: 12px;
        color: rgba(255,255,255,0.4);
        padding: 4px 0;
    }

    /* PostCard styles moved to SocialPostCard.svelte */

    /* Profile */
    .profile-section {
        display: flex;
        flex-direction: column;
    }

    .profile-card {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 16px;
        background: rgba(255,255,255,0.04);
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 10px;
    }

    .profile-avatar {
        width: 72px;
        height: 72px;
        border-radius: 10px;
        background: rgba(255,255,255,0.05);
    }

    .profile-info h2 {
        margin: 0;
        font-size: 18px;
        color: white;
    }

    .profile-badge {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-top: 4px;
    }

    .badge-note {
        font-size: 11px;
        color: rgba(255,255,255,0.3);
    }

    .badge-chip {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        background: rgba(172,196,222,0.15);
        border: 1px solid rgba(172,196,222,0.3);
        border-radius: 6px;
        padding: 2px 8px;
        font-size: 12px;
        font-weight: 700;
        color: #ACC4DE;
        font-family: monospace;
        text-transform: uppercase;
    }

    .badge-chip.small {
        font-size: 10px;
        padding: 1px 5px;
    }

    .profile-stats {
        display: flex;
        gap: 12px;
        margin-top: 12px;
    }

    .stat {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 12px;
        background: rgba(255,255,255,0.03);
        border: 1px solid rgba(255,255,255,0.05);
        border-radius: 8px;
    }

    .stat-value {
        font-size: 18px;
        font-weight: 700;
        color: white;
    }

    .stat-label {
        font-size: 10px;
        color: rgba(255,255,255,0.3);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    /* Composer */
    .composer-identity {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 4px;
    }

    .composer-avatar {
        width: 28px;
        height: 28px;
        border-radius: 50%;
    }

    .composer-name {
        font-weight: 600;
        font-size: 13px;
        color: white;
    }

    .composer-input {
        width: 100%;
        background: rgba(0,0,0,0.3);
        border: 1px solid rgba(255,255,255,0.08);
        border-radius: 8px;
        padding: 10px;
        color: white;
        font-size: 13px;
        resize: vertical;
        outline: none;
        font-family: inherit;
        box-sizing: border-box;
    }

    .composer-input:focus {
        border-color: rgba(172,196,222,0.3);
    }

    .attachment-preview {
        margin: 0;
    }

    .attach-card {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 10px;
        background: rgba(172,196,222,0.06);
        border: 1px solid rgba(172,196,222,0.12);
        border-radius: 8px;
        font-size: 12px;
        color: rgba(255,255,255,0.7);
    }

    .attach-icon {
        font-size: 16px;
    }

    .attach-addr {
        font-family: monospace;
        color: #7ec8e3;
        font-size: 11px;
        margin-left: auto;
    }

    .attach-remove {
        background: none;
        border: none;
        color: rgba(255,255,255,0.3);
        cursor: pointer;
        font-size: 14px;
        padding: 2px 6px;
    }

    .attach-remove:hover {
        color: #ff6b6b;
    }

    .attach-tools {
        display: flex;
        gap: 6px;
    }

    .tool-btn {
        background: rgba(255,255,255,0.05);
        border: 1px solid rgba(255,255,255,0.08);
        border-radius: 6px;
        padding: 6px 10px;
        font-size: 11px;
        color: rgba(255,255,255,0.5);
        cursor: pointer;
    }

    .tool-btn:hover:not(:disabled) {
        background: rgba(255,255,255,0.1);
        color: white;
    }

    .tool-btn:disabled {
        opacity: 0.4;
        cursor: default;
    }

    .publish-btn {
        width: 100%;
        background: rgba(172,196,222,0.2);
        border: 1px solid rgba(172,196,222,0.3);
        border-radius: 8px;
        padding: 10px;
        font-size: 13px;
        font-weight: 600;
        color: white;
        cursor: pointer;
    }

    .publish-btn:hover:not(:disabled) {
        background: rgba(172,196,222,0.3);
    }

    .publish-btn:disabled {
        opacity: 0.4;
        cursor: default;
    }

    /* Server Picker */
    .share-mode-tabs {
        display: flex;
        gap: 4px;
        background: rgba(255,255,255,0.04);
        border-radius: 8px;
        padding: 3px;
    }

    .mode-tab {
        flex: 1;
        padding: 7px;
        border: none;
        background: transparent;
        color: rgba(255,255,255,0.4);
        font-size: 12px;
        cursor: pointer;
        border-radius: 6px;
    }

    .mode-tab.active {
        background: rgba(172,196,222,0.15);
        color: white;
    }

    .server-option {
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding: 10px 12px;
        background: rgba(255,255,255,0.03);
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 8px;
        color: white;
        text-align: left;
        cursor: pointer;
        width: 100%;
        font-size: 13px;
    }

    .server-option:hover {
        background: rgba(172,196,222,0.08);
        border-color: rgba(172,196,222,0.2);
    }

    .srv-meta {
        font-size: 11px;
        color: rgba(255,255,255,0.4);
    }

    .custom-input {
        width: 100%;
        background: rgba(0,0,0,0.3);
        border: 1px solid rgba(255,255,255,0.08);
        border-radius: 8px;
        padding: 9px 10px;
        color: white;
        font-size: 13px;
        outline: none;
        box-sizing: border-box;
    }

    .custom-input:focus {
        border-color: rgba(172,196,222,0.3);
    }

    /* YouTube */
    .profile-links {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-top: 8px;
    }

    .yt-link {
        font-size: 12px;
        color: #ff4444;
        text-decoration: none;
        font-weight: 600;
    }

    .yt-link:hover {
        text-decoration: underline;
    }

    .yt-link.missing {
        color: rgba(255,255,255,0.3);
        font-weight: 400;
    }

    .edit-yt-btn {
        background: none;
        border: 1px solid rgba(255,255,255,0.1);
        border-radius: 6px;
        color: rgba(255,255,255,0.4);
        font-size: 11px;
        padding: 3px 8px;
        cursor: pointer;
    }

    .edit-yt-btn:hover {
        border-color: rgba(172,196,222,0.3);
        color: white;
    }

    .yt-editor {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-top: 8px;
        padding: 8px 12px;
        background: rgba(0,0,0,0.2);
        border-radius: 8px;
    }

    .yt-input {
        flex: 1;
        background: rgba(255,255,255,0.06);
        border: 1px solid rgba(255,255,255,0.08);
        border-radius: 6px;
        padding: 6px 10px;
        color: white;
        font-size: 12px;
        outline: none;
    }

    .yt-input:focus {
        border-color: rgba(172,196,222,0.3);
    }

    .yt-actions {
        display: flex;
        gap: 4px;
    }

    .yt-save {
        background: rgba(172,196,222,0.15);
        border: 1px solid rgba(172,196,222,0.2);
        border-radius: 6px;
        color: white;
        padding: 5px 10px;
        font-size: 11px;
        cursor: pointer;
    }

    .yt-save:hover {
        background: rgba(172,196,222,0.25);
    }

    .yt-cancel {
        background: rgba(255,255,255,0.05);
        border: 1px solid rgba(255,255,255,0.08);
        border-radius: 6px;
        color: rgba(255,255,255,0.4);
        padding: 5px 10px;
        font-size: 11px;
        cursor: pointer;
    }

    .yt-cancel:hover {
        color: white;
    }
</style>
