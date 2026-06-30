<script>
    import { createEventDispatcher } from "svelte";

    export let post;

    const dispatch = createEventDispatcher();

    function handleHashtagClick(e) {
        const el = e.target.closest('.hashtag');
        if (el && el.dataset.tag) dispatch('tag', el.dataset.tag);
    }

    function renderContent(text) {
        if (!text) return "";
        return text
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/@(\w+)/g, '<span class="mention">@$1</span>')
            .replace(/#(\w+)/g, '<span class="hashtag" data-tag="$1">#$1</span>');
    }

    function formatTime(iso) {
        if (!iso) return "";
        const d = new Date(iso);
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

<div class="post-card">
    <div class="post-author">
        <img
            class="post-avatar"
            src="https://mc-heads.net/avatar/{post.author_uuid}/40"
            alt=""
            on:error={(e) => e.target.src = "https://mc-heads.net/avatar/MHF_Steve/40"}
        />
        <div class="post-author-info">
            <span class="post-author-name">{post.author_username || "Unknown"}</span>
            {#if post.author_badge}
                <span class="badge-chip-pill">{post.author_badge}</span>
            {/if}
        </div>
        <span class="post-time">{formatTime(post.created_at)}</span>
    </div>

    <div class="post-body" on:click={handleHashtagClick}>
        <p class="post-text">{@html renderContent(post.content)}</p>
    </div>

    {#if post.attachment_data}
        <div class="post-attachment">
            {#if post.post_type === "server_invite"}
                <div class="server-invite-card">
                    <div class="si-header">⛁ Server</div>
                    <div class="si-name">{post.attachment_data.server_name}</div>
                    <div class="si-meta">{post.attachment_data.server_type} · {post.attachment_data.mc_version}</div>
                    <div class="si-address">{post.attachment_data.address}</div>
                </div>
            {:else if post.post_type === "badge_showcase"}
                <div class="badge-showcase">
                    <span class="badge-chip large">{post.attachment_data.badge}</span>
                    <span class="badge-owner">{post.attachment_data.minecraft_username}</span>
                </div>
            {:else if post.post_type === "cosmetics_share"}
                <div class="cosmetics-share-card">
                    <span>◇</span> Cape showcase
                    {#if post.attachment_data.badge_text}
                        <span class="badge-chip small">{post.attachment_data.badge_text}</span>
                    {/if}
                </div>
            {/if}
        </div>
    {/if}

    <div class="post-actions">
        <button class="action-btn" class:liked={post.liked_by_me} on:click={() => dispatch('like')}>
            {post.liked_by_me ? "❤" : "♡"} {post.likes}
        </button>
    </div>

    {#if post.hashtags?.length > 0}
        <div class="post-hashtags">
            {#each post.hashtags as tag}
                <button class="post-hashtag" on:click={() => dispatch('tag', tag)}>#{tag}</button>
            {/each}
        </div>
    {/if}
</div>

<style>
    .post-card {
        background: rgba(255,255,255,0.04);
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 10px;
        padding: 14px;
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .post-author {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .post-avatar {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        background: rgba(255,255,255,0.05);
    }

    .post-author-info {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .post-author-name {
        font-weight: 600;
        font-size: 13px;
        color: white;
    }

    .badge-chip-pill {
        background: rgba(172,196,222,0.2);
        border: 1px solid rgba(172,196,222,0.25);
        border-radius: 8px;
        padding: 1px 7px;
        font-size: 10px;
        color: #ACC4DE;
        font-weight: 600;
    }

    .post-time {
        font-size: 11px;
        color: rgba(255,255,255,0.3);
        white-space: nowrap;
    }

    .post-body {
        font-size: 13px;
        line-height: 1.5;
        color: rgba(255,255,255,0.85);
    }

    .post-text {
        margin: 0;
    }

    .post-text :global(.mention) {
        color: #7ec8e3;
        font-weight: 500;
    }

    .post-text :global(.hashtag) {
        color: #ACC4DE;
        font-weight: 500;
        cursor: pointer;
    }

    .post-text :global(.hashtag:hover) {
        text-decoration: underline;
    }

    .server-invite-card {
        background: rgba(255,255,255,0.04);
        border: 1px solid rgba(172,196,222,0.15);
        border-radius: 8px;
        padding: 12px;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .si-header {
        font-size: 10px;
        text-transform: uppercase;
        color: rgba(172,196,222,0.6);
        letter-spacing: 1px;
    }

    .si-name {
        font-size: 14px;
        font-weight: 600;
        color: white;
    }

    .si-meta {
        font-size: 11px;
        color: rgba(255,255,255,0.4);
    }

    .si-address {
        font-size: 12px;
        color: #7ec8e3;
        font-family: monospace;
        margin-top: 4px;
    }

    .badge-showcase {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 12px;
        background: rgba(255,255,255,0.03);
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 8px;
    }

    .badge-owner {
        font-size: 12px;
        color: rgba(255,255,255,0.4);
    }

    .cosmetics-share-card {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        background: rgba(255,255,255,0.03);
        border: 1px solid rgba(172,196,222,0.1);
        border-radius: 8px;
        font-size: 13px;
        color: rgba(255,255,255,0.6);
    }

    .post-actions {
        display: flex;
        gap: 8px;
    }

    .action-btn {
        background: none;
        border: 1px solid rgba(255,255,255,0.08);
        border-radius: 6px;
        padding: 5px 12px;
        font-size: 12px;
        color: rgba(255,255,255,0.5);
        cursor: pointer;
    }

    .action-btn.liked {
        color: #ff6b6b;
        border-color: rgba(255,107,107,0.3);
    }

    .action-btn:hover {
        background: rgba(255,255,255,0.05);
    }

    .post-hashtags {
        display: flex;
        gap: 6px;
        flex-wrap: wrap;
    }

    .post-hashtag {
        background: rgba(172,196,222,0.08);
        border: none;
        border-radius: 6px;
        padding: 3px 8px;
        font-size: 11px;
        color: rgba(172,196,222,0.7);
        cursor: pointer;
    }

    .post-hashtag:hover {
        background: rgba(172,196,222,0.15);
        color: #ACC4DE;
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

    .badge-chip.large {
        font-size: 18px;
        padding: 4px 14px;
        border-radius: 8px;
    }
</style>
