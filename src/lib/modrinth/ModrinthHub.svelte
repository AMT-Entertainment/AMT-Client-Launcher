<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onDestroy } from "svelte";

    export let initialTab = "mods";

    let activeTab = initialTab;
    let query = "";
    let results = [];
    let loading = false;
    let selectedProject = null;
    let versions = [];
    let installing = false;
    let installStatus = "";

    const tabs = [
        { id: "mods", label: "Mods" },
        { id: "shader", label: "Shaders" },
        { id: "resourcepack", label: "Resource Packs" },
    ];

    const installDirs = {
        mods: "mods",
        shader: "shaderpacks",
        resourcepack: "resourcepacks",
    };

    let debounceTimer;
    let installTimer;

    onDestroy(() => {
        clearTimeout(debounceTimer);
        clearTimeout(installTimer);
    });

    async function search() {
        loading = true;
        try {
            const result = await invoke("modrinth_search", {
                query: query || "",
                projectType: activeTab,
                limit: 30,
                offset: 0,
            });
            results = result.hits || [];
        } catch (e) {
            console.error("Search failed:", e);
            results = [];
        }
        loading = false;
    }

    function onSearchInput() {
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(search, 300);
    }

    async function selectProject(project) {
        selectedProject = project;
        try {
            const projectDetail = await invoke("modrinth_get_project", {
                projectId: project.project_id,
            });
            selectedProject = projectDetail;
            versions = await invoke("modrinth_get_versions", {
                projectId: project.project_id,
            });
            // Sort: releases first, then betas, then alphas
            const typeOrder = { release: 0, beta: 1, alpha: 2 };
            versions.sort((a, b) => (typeOrder[a.version_type] || 99) - (typeOrder[b.version_type] || 99));
        } catch (e) {
            console.error("Failed to load project details:", e);
        }
    }

    function goBack() {
        selectedProject = null;
        versions = [];
    }

    async function installVersion(version) {
        const primaryFile = version.files.find(f => f.primary) || version.files[0];
        if (!primaryFile) return;

        installing = true;
        installStatus = `Downloading ${primaryFile.filename}...`;
        try {
            await invoke("modrinth_install", {
                projectId: version.project_id,
                versionId: version.id,
                fileUrl: primaryFile.url,
                filename: primaryFile.filename,
                installDir: installDirs[activeTab] || "mods",
            });
            installStatus = "Installed successfully!";
            installTimer = setTimeout(() => { installStatus = ""; installing = false; }, 2000);
        } catch (e) {
            installStatus = `Install failed: ${e}`;
            installing = false;
        }
    }

    function formatNumber(n) {
        if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
        if (n >= 1000) return (n / 1000).toFixed(1) + "K";
        return n.toString();
    }

    function loaderIcon(loaders) {
        if (!loaders) return "?";
        if (loaders.includes("fabric")) return "F";
        if (loaders.includes("forge")) return "Fo";
        if (loaders.includes("quilt")) return "Q";
        if (loaders.includes("neoforge")) return "Neo";
        return "?";
    }
</script>

<div class="modrinth-hub">
    {#if selectedProject}
        <div class="detail-view">
            <button class="back-btn" on:click={goBack}>← Back to search</button>

            <div class="detail-header">
                {#if selectedProject.icon_url}
                    <img class="project-icon" src={selectedProject.icon_url} alt={selectedProject.title} />
                {/if}
                <div>
                    <h2>{selectedProject.title}</h2>
                    <p class="project-author">by {selectedProject.author || selectedProject.title}</p>
                </div>
                <div class="project-stats">
                            <span class="material-icons-round" style="font-size:14px">download</span>
                            <span>{formatNumber(selectedProject.downloads)}</span>
                    <span class="material-icons-round" style="font-size:14px">star</span>
                    <span>{selectedProject.followers || 0}</span>
                </div>
            </div>

            <p class="project-desc">{selectedProject.description}</p>

            {#if selectedProject.gallery?.length}
                <div class="gallery">
                    {#each selectedProject.gallery.filter(g => g.url) as image}
                        <img class="gallery-img" src={image.url} alt={image.title || ""} />
                    {/each}
                </div>
            {/if}

            <div class="detail-categories">
                {#each selectedProject.categories || [] as cat}
                    <span class="badge">{cat}</span>
                {/each}
                {#each selectedProject.loaders || [] as loader}
                    <span class="badge loader">{loader}</span>
                {/each}
            </div>

            <h3>Versions</h3>
            <div class="version-list">
                {#each versions.slice(0, 10) as version}
                    <div class="version-item">
                        <div class="version-info">
                            <span class="version-name">{version.name || version.version_number}</span>
                            <span class="version-meta">
                                {version.game_versions?.join(", ") || "Any"}
                                {#if version.version_type === "release"}
                                    <span class="badge release">Release</span>
                                {:else if version.version_type === "beta"}
                                    <span class="badge beta">Beta</span>
                                {:else if version.version_type === "alpha"}
                                    <span class="badge alpha">Alpha</span>
                                {/if}
                            </span>
                        </div>
                        <button
                            class="install-btn"
                            on:click={() => installVersion(version)}
                            disabled={installing}
                        >
                            Install
                        </button>
                    </div>
                {/each}
                {#if versions.length === 0}
                    <p class="no-versions">No versions available</p>
                {/if}
            </div>

            {#if installStatus}
                <div class="install-status">{installStatus}</div>
            {/if}
        </div>
    {:else}
        <div class="hub-header">
            <div class="tabs">
                {#each tabs as tab}
                    <button
                        class="tab-btn"
                        class:active={activeTab === tab.id}
                        on:click={() => { activeTab = tab.id; search(); }}
                    >
                        {tab.label}
                    </button>
                {/each}
            </div>
            <input
                class="search-input"
                type="text"
                placeholder="Search {activeTab}..."
                bind:value={query}
                on:input={onSearchInput}
            />
        </div>

        <div class="results">
            {#if loading}
                <div class="loading">Searching...</div>
            {:else if results.length === 0}
                <div class="empty">
                    {query ? "No results found" : "Type to search for " + activeTab}
                </div>
            {:else}
                {#each results as project}
                    <button class="project-card" on:click={() => selectProject(project)}>
                        <div class="card-icon">
                            {#if project.icon_url}
                                <img src={project.icon_url} alt={project.title} />
                            {:else}
                                {loaderIcon(project.loaders || project.categories)}
                            {/if}
                        </div>
                        <div class="card-body">
                            <div class="card-title">{project.title}</div>
                            <div class="card-author">{project.author}</div>
                            <div class="card-desc">{project.description}</div>
                            <div class="card-cats">
                                {#each project.categories?.slice(0, 3) || [] as cat}
                                    <span class="badge small">{cat}</span>
                                {/each}
                            </div>
                        </div>
                        <div class="card-meta">
                            <span class="downloads"><span class="material-icons-round" style="font-size:12px">download</span> {formatNumber(project.downloads)}</span>
                            <span class="game-ver">{project.versions?.slice(-1)[0] || ""}</span>
                        </div>
                    </button>
                {/each}
            {/if}
        </div>
    {/if}
</div>

<style>
    .modrinth-hub {
        display: flex;
        flex-direction: column;
        height: 100%;
        color: white;
    }

    .hub-header {
        display: flex;
        gap: 12px;
        align-items: center;
        padding-bottom: 16px;
        border-bottom: 1px solid rgba(255,255,255,0.08);
        margin-bottom: 16px;
    }

    .tabs {
        display: flex;
        gap: 4px;
        background: rgba(0,0,0,0.3);
        padding: 4px;
        border-radius: 8px;
    }

    .tab-btn {
        padding: 8px 16px;
        border: none;
        background: transparent;
        color: rgba(255,255,255,0.6);
        border-radius: 6px;
        cursor: pointer;
        font-family: "Inter", sans-serif;
        font-size: 13px;
        font-weight: 500;
        transition: all 0.2s;
    }

    .tab-btn.active {
        background: var(--amt-accent);
        color: #0D1117;
    }

    .tab-btn:hover:not(.active) {
        background: rgba(255,255,255,0.08);
        color: white;
    }

    .search-input {
        flex: 1;
        padding: 10px 14px;
        border: 1px solid rgba(255,255,255,0.1);
        border-radius: 8px;
        background: rgba(0,0,0,0.3);
        color: white;
        font-family: "Inter", sans-serif;
        font-size: 13px;
        outline: none;
    }

    .search-input:focus {
        border-color: var(--amt-accent, #ACC4DE);
    }

    .search-input::placeholder {
        color: rgba(255,255,255,0.3);
    }

    .results {
        flex: 1;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .project-card {
        display: flex;
        gap: 12px;
        padding: 12px;
        background: rgba(0,0,0,0.2);
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 10px;
        cursor: pointer;
        transition: all 0.15s;
        text-align: left;
        font-family: "Inter", sans-serif;
        width: 100%;
    }

    .project-card:hover {
        background: rgba(255,255,255,0.04);
        border-color: rgba(255,255,255,0.12);
    }

    .card-icon {
        width: 48px;
        height: 48px;
        border-radius: 8px;
        overflow: hidden;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(255,255,255,0.05);
        font-size: 24px;
    }

    .card-icon img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .card-body {
        flex: 1;
        min-width: 0;
    }

    .card-title {
        font-weight: 600;
        font-size: 14px;
        margin-bottom: 2px;
    }

    .card-author {
        font-size: 11px;
        color: rgba(255,255,255,0.4);
        margin-bottom: 4px;
    }

    .card-desc {
        font-size: 12px;
        color: rgba(255,255,255,0.6);
        display: -webkit-box;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
        margin-bottom: 6px;
    }

    .card-cats {
        display: flex;
        gap: 4px;
        flex-wrap: wrap;
    }

    .card-meta {
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 4px;
        flex-shrink: 0;
    }

    .downloads {
        font-size: 12px;
        color: rgba(255,255,255,0.5);
    }

    .game-ver {
        font-size: 10px;
        color: rgba(255,255,255,0.3);
        background: rgba(255,255,255,0.06);
        padding: 2px 6px;
        border-radius: 4px;
    }

    .badge {
        display: inline-block;
        padding: 2px 8px;
        border-radius: 4px;
        font-size: 10px;
        font-weight: 500;
        background: rgba(172,196,222,0.15);
        color: var(--amt-accent, #ACC4DE);
    }

    .badge.small {
        font-size: 9px;
        padding: 1px 6px;
    }

    .badge.release { background: rgba(96,182,117,0.15); color: #60B675; }
    .badge.beta { background: rgba(172,196,222,0.15); color: var(--amt-accent, #ACC4DE); }
    .badge.alpha { background: rgba(254,76,46,0.15); color: #FE4C2E; }
    .badge.loader { background: rgba(70,119,255,0.15); color: #4677FF; }

    .loading, .empty {
        text-align: center;
        padding: 40px;
        color: rgba(255,255,255,0.4);
        font-size: 14px;
    }

    .detail-view {
        display: flex;
        flex-direction: column;
        gap: 12px;
        height: 100%;
        overflow-y: auto;
    }

    .back-btn {
        background: none;
        border: none;
        color: var(--amt-accent, #ACC4DE);
        cursor: pointer;
        font-family: "Inter", sans-serif;
        font-size: 13px;
        padding: 0;
        text-align: left;
    }

    .back-btn:hover {
        text-decoration: underline;
    }

    .detail-header {
        display: flex;
        gap: 16px;
        align-items: center;
    }

    .project-icon {
        width: 64px;
        height: 64px;
        border-radius: 12px;
    }

    .detail-header h2 {
        margin: 0;
        font-size: 20px;
    }

    .project-author {
        margin: 2px 0 0;
        color: rgba(255,255,255,0.4);
        font-size: 13px;
    }

    .project-stats {
        margin-left: auto;
        display: flex;
        gap: 12px;
        font-size: 13px;
        color: rgba(255,255,255,0.6);
    }

    .project-desc {
        color: rgba(255,255,255,0.7);
        font-size: 13px;
        line-height: 1.5;
    }

    .gallery {
        display: flex;
        gap: 8px;
        overflow-x: auto;
        padding: 4px 0;
    }

    .gallery-img {
        height: 120px;
        border-radius: 8px;
        border: 1px solid rgba(255,255,255,0.08);
    }

    .detail-categories {
        display: flex;
        gap: 6px;
        flex-wrap: wrap;
    }

    h3 {
        margin: 8px 0 0;
        font-size: 15px;
        font-weight: 600;
    }

    .version-list {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .version-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 10px 12px;
        background: rgba(0,0,0,0.2);
        border-radius: 8px;
    }

    .version-info {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .version-name {
        font-weight: 500;
        font-size: 13px;
    }

    .version-meta {
        display: flex;
        gap: 6px;
        align-items: center;
        font-size: 11px;
        color: rgba(255,255,255,0.5);
    }

    .install-btn {
        padding: 6px 16px;
        background: var(--amt-accent, #ACC4DE);
        color: #0D1117;
        border: none;
        border-radius: 6px;
        font-weight: 600;
        font-size: 12px;
        cursor: pointer;
        font-family: "Inter", sans-serif;
        transition: opacity 0.2s;
    }

    .install-btn:hover {
        opacity: 0.85;
    }

    .install-btn:disabled {
        opacity: 0.4;
        cursor: not-allowed;
    }

    .no-versions {
        color: rgba(255,255,255,0.4);
        font-size: 13px;
        text-align: center;
        padding: 16px;
    }

    .install-status {
        padding: 10px 14px;
        background: rgba(96,182,117,0.1);
        border: 1px solid rgba(96,182,117,0.2);
        border-radius: 8px;
        color: #60B675;
        font-size: 13px;
        text-align: center;
    }
</style>
