<script>
    import {invoke} from "@tauri-apps/api/core";
    import CapeDesigner from "../cape/CapeDesigner.svelte";
    import BadgeEditor from "../amt/BadgeEditor.svelte";
    import CosmeticsGallery from "./CosmeticsGallery.svelte";
    import UploadCape from "./UploadCape.svelte";

    export let options;
    export let playerUuid = "steve";

    let activeTab = "designer";
    const tabs = [
        { id: "designer", label: "Designer", icon: "✎" },
        { id: "upload", label: "Upload", icon: "⬆" },
        { id: "gallery", label: "Gallery", icon: "☐" },
        { id: "badge", label: "Badge", icon: "◆" },
    ];
</script>

<div class="cosmetics-hub">
    <header class="hub-header">
        <h1 class="hub-title">Cosmetics</h1>
        <nav class="hub-tabs" role="tablist">
            {#each tabs as tab}
                <button
                    role="tab"
                    class="hub-tab"
                    class:active={activeTab === tab.id}
                    aria-selected={activeTab === tab.id}
                    on:click={() => activeTab = tab.id}
                >
                    <span class="tab-icon">{tab.icon}</span>
                    <span class="tab-label">{tab.label}</span>
                </button>
            {/each}
        </nav>
    </header>

    <main class="hub-content">
        {#if activeTab === "designer"}
            <CapeDesigner
                accent={options?.amt_options?.accent_color}
                githubToken={options?.amt_options?.githubToken}
                githubUser={options?.amt_options?.githubRepoOwner || "AMT-Entertainment"}
                githubRepo={options?.amt_options?.githubRepoName || "AMT-Client-Backend"}
                {playerUuid}
            />
        {:else if activeTab === "upload"}
            <UploadCape
                backendUrl={options?.amt_options?.backendUrl}
                githubToken={options?.amt_options?.githubToken}
                githubUser={options?.amt_options?.githubRepoOwner || "AMT-Entertainment"}
                githubRepo={options?.amt_options?.githubRepoName || "AMT-Client-Backend"}
                {playerUuid}
                {options}
                on:capeEquipped={() => options.store()}
            />
        {:else if activeTab === "gallery"}
            <CosmeticsGallery
                backendUrl={options?.amt_options?.backendUrl || "https://amt-entertainment.github.io/AMT-Client-Backend"}
                {options}
                on:capeEquipped={() => options.store()}
            />
        {:else if activeTab === "badge"}
            <div class="badge-section">
                <BadgeEditor
                    bind:badge={options.amt_options.badge}
                    bind:accent={options.amt_options.accentColor}
                />
                <div class="badge-actions">
                    <button class="amt-btn amt-btn-primary" on:click={() => options.store()}>
                        Save Badge
                    </button>
                </div>
            </div>
        {/if}
    </main>
</div>

<style>
    .cosmetics-hub {
        display: flex;
        flex-direction: column;
        height: 100%;
        background: var(--amt-bg-primary);
    }

    .hub-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 16px 24px;
        border-bottom: 1px solid var(--amt-border);
        background: var(--amt-bg-secondary);
    }

    .hub-title {
        margin: 0;
        font-size: 20px;
        font-weight: 700;
        color: var(--amt-text-primary);
    }

    .hub-tabs {
        display: flex;
        gap: 4px;
        background: var(--amt-bg-tertiary);
        padding: 4px;
        border-radius: 10px;
    }

    .hub-tab {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 16px;
        border: none;
        background: transparent;
        border-radius: 8px;
        color: var(--amt-text-secondary);
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .hub-tab:hover {
        color: var(--amt-text-primary);
        background: var(--amt-bg-hover);
    }

    .hub-tab.active {
        color: var(--amt-accent);
        background: var(--amt-accent-bg);
    }

    .tab-icon {
        font-size: 14px;
    }

    .hub-content {
        flex: 1;
        overflow-y: auto;
        padding: 24px;
    }

    .badge-section {
        display: flex;
        flex-direction: column;
        gap: 20px;
        max-width: 500px;
    }

    .badge-actions {
        display: flex;
        justify-content: flex-start;
    }
</style>