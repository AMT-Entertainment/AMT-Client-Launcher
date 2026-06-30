<script>
    import { createEventDispatcher } from "svelte";

    export let versionInfo;
    export let mcVersion;
    export let lbVersion;
    export let running;
    export let canLaunch = true;
    export let accountName = "Guest";
    export let accountAvatar = "steve";
    export let badge = "AMT";

    const dispatch = createEventDispatcher();

    function parseChangelog(text) {
        text = text.replaceAll("[a]", "•");
        text = text.replaceAll("[i]", "•");
        text = text.replaceAll("[b]", "•");
        text = text.replaceAll("[r]", "•");
        return text;
    }
</script>

<div class="launch-area glass-panel glass-panel-lg">
    <!-- Account Bar -->
    <div class="account-bar">
        <div class="account-bar-left">
            <img
                src="https://crafatar.com/avatars/{accountAvatar}?size=32&overlay"
                on:error={(e) => { e.target.src = `https://minotar.net/avatar/${accountAvatar}/32`; }}
                class="account-bar-avatar"
                alt="avatar"
            />
            <div class="account-bar-info">
                <span class="account-bar-name">{accountName}</span>
                <span class="amt-badge amt-badge-accent">{badge || "AMT"}</span>
            </div>
        </div>
    </div>

    <!-- Version Banner -->
    <div class="launch-banner">
        <div class="banner-bg" style="background-image: linear-gradient(135deg, rgba(13,17,23,0.92) 0%, rgba(13,17,23,0.2) 100%), url({versionInfo.bannerUrl});"></div>
        <div class="banner-content">
            <div class="banner-left">
                <h1 class="banner-title">{versionInfo.title}</h1>
                <div class="banner-meta">
                    <span class="amt-version-pill latest">MC {mcVersion}</span>
                    <span class="banner-date">{versionInfo.date}</span>
                </div>
            </div>
            <div class="banner-actions">
                {#if running}
                    <button class="amt-launch-btn running" on:click={() => dispatch("terminate")}>
                        <span>■</span> Terminate
                    </button>
                {:else}
                    <button class="amt-launch-btn" disabled={!canLaunch} on:click={() => dispatch("launch")}>
                        <span>▶</span> Launch
                    </button>
                {/if}
            </div>
        </div>
    </div>

    <!-- Changelog / Description -->
    {#if versionInfo.description}
        <div class="changelog-section">
            <h4 class="section-label">Changelog</h4>
            <div class="changelog-content">
                {@html parseChangelog(versionInfo.description)}
            </div>
        </div>
    {/if}

    <!-- Version Selector & Controls -->
    <div class="launch-controls">
        <div class="version-pills">
            <button class="version-pill glass-panel glass-panel-sm" on:click={() => dispatch("showVersionSelect")}>
                <span class="pill-icon">⚙</span>
                <div class="pill-info">
                    <span class="pill-label">AMT Client</span>
                    <span class="pill-value">{lbVersion}</span>
                </div>
                <span class="pill-arrow">⌵</span>
            </button>
            <button class="version-pill glass-panel glass-panel-sm" on:click={() => dispatch("showVersionSelect")}>
                <span class="pill-icon" style="background: rgba(63,185,80,0.15); color: #3FB950;">⛁</span>
                <div class="pill-info">
                    <span class="pill-label">Minecraft</span>
                    <span class="pill-value">{mcVersion}</span>
                </div>
                <span class="pill-arrow">⌵</span>
            </button>
        </div>
        <div class="control-buttons">
            {#if running}
                <button class="amt-btn amt-btn-ghost" on:click={() => dispatch("showClientLog")}>
                    <span>⎚</span> Log
                </button>
            {:else}
                <button class="amt-btn amt-btn-ghost" on:click={() => dispatch("showVersionSelect")}>
                    <span>⚙</span> Configure
                </button>
            {/if}
        </div>
    </div>
</div>

<style>
    .launch-area {
        padding: 0;
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    /* ── Account Bar ── */
    .account-bar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 12px 20px;
        border-bottom: 1px solid rgba(255,255,255,0.04);
    }

    .account-bar-left {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .account-bar-avatar {
        width: 28px;
        height: 28px;
        border-radius: 50%;
        object-fit: cover;
        border: 1.5px solid rgba(var(--amt-accent-rgb), 0.2);
    }

    .account-bar-info {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .account-bar-name {
        font-size: 13px;
        font-weight: 600;
        color: var(--amt-text);
    }

    /* ── Banner ── */
    .launch-banner {
        position: relative;
        height: 140px;
        overflow: hidden;
    }

    .banner-bg {
        position: absolute;
        inset: 0;
        background-size: cover;
        background-position: center;
    }

    .banner-content {
        position: relative;
        height: 100%;
        display: flex;
        align-items: flex-end;
        justify-content: space-between;
        padding: 20px 24px;
    }

    .banner-left {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .banner-title {
        font-size: 22px;
        font-weight: 800;
        color: white;
        margin: 0;
        line-height: 1.2;
    }

    .banner-meta {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .banner-date {
        font-size: 12px;
        color: var(--amt-text-muted);
    }

    .banner-actions {
        display: flex;
        gap: 10px;
        align-items: center;
    }

    /* ── Changelog ── */
    .changelog-section {
        padding: 16px 24px 8px;
    }

    .section-label {
        font-size: 11px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: var(--amt-text-muted);
        margin: 0 0 8px;
    }

    .changelog-content {
        font-size: 12px;
        color: var(--amt-text-secondary);
        line-height: 1.6;
        display: -webkit-box;
        -webkit-line-clamp: 3;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }

    /* ── Controls ── */
    .launch-controls {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 12px 24px 16px;
        gap: 16px;
    }

    .version-pills {
        display: flex;
        gap: 8px;
        flex: 1;
    }

    .version-pill {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 14px;
        cursor: pointer;
        transition: all var(--amt-transition-normal);
        background: rgba(255,255,255,0.03);
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 8px;
        font-family: var(--amt-font-family);
        color: var(--amt-text);
        text-align: left;
        flex: 1;
        max-width: 200px;
    }

    .version-pill:hover {
        background: rgba(var(--amt-accent-rgb), 0.06);
        border-color: rgba(var(--amt-accent-rgb), 0.15);
    }

    .pill-icon {
        width: 28px;
        height: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(var(--amt-accent-rgb), 0.1);
        border-radius: 6px;
        color: var(--amt-accent);
        font-size: 14px;
        flex-shrink: 0;
    }

    .pill-info {
        display: flex;
        flex-direction: column;
        gap: 1px;
        flex: 1;
        min-width: 0;
    }

    .pill-label {
        font-size: 10px;
        color: var(--amt-text-muted);
        text-transform: uppercase;
        letter-spacing: 0.3px;
    }

    .pill-value {
        font-size: 13px;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .pill-arrow {
        color: var(--amt-text-dim);
        font-size: 14px;
    }

    .control-buttons {
        display: flex;
        gap: 6px;
        flex-shrink: 0;
    }
</style>
