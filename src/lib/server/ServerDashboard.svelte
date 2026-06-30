<script>
    import { invoke } from "@tauri-apps/api/core";
    import { createEventDispatcher, onMount, onDestroy } from "svelte";
    import ServerTerminal from "./ServerTerminal.svelte";
    import ServerPlayers from "./ServerPlayers.svelte";
    import ServerFileManager from "./ServerFileManager.svelte";
    import ServerStats from "./ServerStats.svelte";
    import ServerBackups from "./ServerBackups.svelte";
    import ServerWorld from "./ServerWorld.svelte";

    export let server;
    export let status;

    const dispatch = createEventDispatcher();

    let activeTab = "overview";
    let logs = [];
    let logSince = 0;
    let logInterval;

    let configEdit = { ...server };
    let saved = false;

    let tunnelling = false;
    let publicAddress = "";
    let tunnelError = "";

    let modQuery = "";
    let modResults = [];
    let modLoading = false;
    let installedMods = [];
    let modDetail = null;
    let modDetailVersions = [];
    let modInstalling = null;
    let modInstallStatus = "";
    let modDebounce;
    let dependencyTree = null;

    const tabs = [
        { id: "overview", label: "Overview", icon: "dashboard" },
        { id: "terminal", label: "Terminal", icon: "terminal" },
        { id: "players", label: "Players", icon: "people" },
        { id: "files", label: "Files", icon: "folder" },
        { id: "mods", label: "Mods", icon: "inventory_2" },
        { id: "backups", label: "Backups", icon: "backup" },
        { id: "world", label: "World", icon: "public" },
        { id: "stats", label: "Stats", icon: "bar_chart" },
    ];

    onMount(() => {
        if (status?.running) startLogPolling();
        refreshMods();
    });

    onDestroy(() => { stopLogPolling(); });

    function startLogPolling() {
        logInterval = setInterval(pollLogs, 2000);
        pollLogs();
    }

    function stopLogPolling() {
        if (logInterval) clearInterval(logInterval);
    }

    async function pollLogs() {
        if (!status?.running) return;
        try {
            const newLogs = await invoke("server_logs", {
                serverId: server.id, since: logSince,
            });
            if (newLogs.length > 0) {
                logSince += newLogs.length;
                logs = [...logs, ...newLogs];
            }
        } catch {}
    }

    async function sendCommand(cmd) {
        try {
            await invoke("server_send_command", {
                serverId: server.id, command: cmd,
            });
        } catch (e) { console.error("Command failed:", e); }
    }

    async function saveConfig() {
        try {
            await invoke("server_update", { config: { ...server, ...configEdit } });
            saved = true;
            setTimeout(() => saved = false, 2000);
        } catch (e) { alert("Failed to save: " + e); }
    }

    async function copyShareLink() {
        try {
            const info = await invoke("server_share_info", { serverId: server.id });
            const addr = publicAddress || `${info.host}:${info.port}`;
            const text = `Join my AMT server!\n${info.name}\n${info.server_type} · ${info.version}\nAddress: ${addr}`;
            await navigator.clipboard.writeText(text);
        } catch (e) { alert("Failed to get share info: " + e); }
    }

    async function toggleTunnel() {
        if (tunnelling) {
            try {
                await invoke("tunnel_stop", { serverId: server.id });
                tunnelling = false; publicAddress = ""; tunnelError = "";
            } catch (e) { tunnelError = "Failed to stop tunnel: " + e; }
        } else {
            try {
                tunnelling = true; tunnelError = "";
                const addr = await invoke("tunnel_start", {
                    serverId: server.id, localPort: server.port,
                });
                publicAddress = addr;
            } catch (e) { tunnelling = false; tunnelError = "Tunnel failed: " + e; }
        }
    }

    async function refreshMods() {
        try {
            installedMods = await invoke("server_mods_list", { serverId: server.id });
        } catch {}
    }

    async function searchMods() {
        if (!modQuery.trim()) return;
        modLoading = true;
        try {
            const result = await invoke("server_mods_search", { query: modQuery, limit: 30, offset: 0 });
            modResults = result.hits || [];
        } catch (e) { console.error("Mod search:", e); }
        modLoading = false;
    }

    async function showModDetail(mod) {
        try {
            modDetail = await invoke("server_mods_get_project", { projectId: mod.project_id });
            modDetailVersions = await invoke("server_mods_get_versions", { projectId: mod.project_id });
            const validLoaders = ["fabric", "quilt", "forge", "neoforge", "paper", "bukkit", "spigot", "purpur"];
            modDetailVersions = modDetailVersions.filter(v =>
                v.loaders?.some(l => validLoaders.includes(l.toLowerCase()))
            );
            const typeOrder = { release: 0, beta: 1, alpha: 2 };
            modDetailVersions.sort((a, b) => (typeOrder[a.version_type] || 99) - (typeOrder[b.version_type] || 99));
        } catch (e) { console.error("Mod detail:", e); }
    }

    async function installServerMod(version) {
        const primaryFile = version.files.find(f => f.primary) || version.files[0];
        if (!primaryFile) return;
        modInstalling = version.id;
        modInstallStatus = `Downloading ${primaryFile.filename}...`;
        try {
            await invoke("server_mods_install", {
                serverId: server.id, fileUrl: primaryFile.url, filename: primaryFile.filename,
            });
            modInstallStatus = `✓ ${primaryFile.filename} installed!`;
            await refreshMods();
            setTimeout(() => { modInstallStatus = ""; modInstalling = null; }, 2000);
        } catch (e) {
            modInstallStatus = `✗ Failed: ${e}`;
            modInstalling = null;
        }
    }

    async function removeMod(filename) {
        if (!confirm(`Remove ${filename}?`)) return;
        try {
            await invoke("server_mods_delete", { serverId: server.id, filename });
            await refreshMods();
        } catch (e) { alert("Failed to remove mod: " + e); }
    }

    async function resolveDeps() {
        if (!modDetail?.project_id) return;
        try {
            dependencyTree = await invoke("modrinth_resolve_dependencies", {
                projectId: modDetail.project_id,
                gameVersion: server.mc_version,
                loader: server.server_type?.toLowerCase(),
            });
        } catch (e) { modInstallStatus = `Dependency resolve failed: ${e}`; }
    }

    function formatNumber(n) {
        if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
        if (n >= 1000) return (n / 1000).toFixed(1) + "K";
        return n.toString();
    }

    function formatUptime(secs) {
        const h = Math.floor(secs / 3600);
        const m = Math.floor((secs % 3600) / 60);
        const s = secs % 60;
        if (h > 0) return `${h}h ${m}m`;
        if (m > 0) return `${m}m ${s}s`;
        return `${s}s`;
    }
</script>

<div class="dashboard">
    <div class="dash-header">
        <button class="back-btn" on:click={() => dispatch("back")}>←</button>
        <div class="dash-info">
            <span class="dash-name">{server.name}</span>
            <span class="dash-meta">{server.server_type} · {server.mc_version}</span>
        </div>
        <div class="dash-status">
            <span class="status-indicator" class:running={status?.running} class:stopped={!status?.running}></span>
            <span class="status-label">{status?.running ? "Running" : "Stopped"}</span>
            {#if status?.running}
                <span class="uptime">{formatUptime(status.uptime_secs)}</span>
            {/if}
        </div>
        <div class="dash-actions">
            {#if status?.running}
                <button class="action-btn stop" on:click={() => dispatch("stop")}>Stop</button>
            {:else}
                <button class="action-btn start" on:click={() => dispatch("start")}>Start</button>
            {/if}
            <button class="action-btn connect" on:click={() => invoke("server_connect", { serverId: server.id }).catch(e => alert("Failed to connect: " + e))}>Connect</button>
            <button class="action-btn delete" on:click={() => dispatch("delete")}>Delete</button>
        </div>
    </div>

    <div class="tabs">
        {#each tabs as tab}
            <button class="tab" class:active={activeTab === tab.id}
                on:click={() => activeTab = tab.id}
                title={tab.label}>
                <span class="mat-icon">{tab.icon}</span>
                <span class="tab-label">{tab.label}</span>
            </button>
        {/each}
    </div>

    <div class="tab-content">
        {#if activeTab === "overview"}
            <div class="overview-panel">
                <div class="overview-cards">
                    <div class="ov-card">
                        <span class="ov-label">Players</span>
                        <span class="ov-value">{status?.player_count || 0}</span>
                    </div>
                    <div class="ov-card">
                        <span class="ov-label">TPS</span>
                        <span class="ov-value" class:tps-good={status?.tps >= 19} class:tps-warn={status?.tps < 19 && status?.tps >= 10} class:tps-bad={status?.tps < 10}>
                            {status?.tps?.toFixed(1) || "—"}
                        </span>
                    </div>
                    <div class="ov-card">
                        <span class="ov-label">Memory</span>
                        <span class="ov-value">{status?.memory_mb || "—"} MB</span>
                    </div>
                    <div class="ov-card">
                        <span class="ov-label">CPU</span>
                        <span class="ov-value">{status?.cpu_percent?.toFixed(1) || "—"}%</span>
                    </div>
                    <div class="ov-card">
                        <span class="ov-label">Uptime</span>
                        <span class="ov-value">{status?.uptime_secs ? formatUptime(status.uptime_secs) : "—"}</span>
                    </div>
                    <div class="ov-card">
                        <span class="ov-label">Mods</span>
                        <span class="ov-value">{installedMods.length}</span>
                    </div>
                </div>

                <div class="share-section">
                    <h4>Share</h4>
                    {#if publicAddress}
                        <div class="public-addr">
                            <span class="addr-label">Public Address:</span>
                            <span class="addr-value">{publicAddress}</span>
                        </div>
                    {/if}
                    {#if tunnelError}
                        <div class="tunnel-error">{tunnelError}</div>
                    {/if}
                    <div class="share-actions">
                        <button class="share-btn" on:click={copyShareLink}>Copy Share Info</button>
                        {#if status?.running}
                            <button class="tunnel-btn" class:active={tunnelling} on:click={toggleTunnel}>
                                {tunnelling ? "Disable Tunnel" : "Enable Tunnel"}
                            </button>
                        {/if}
                    </div>
                </div>

                <ServerStats serverId={server.id} {status} />
            </div>

        {:else if activeTab === "terminal"}
            <ServerTerminal {logs} running={status?.running}
                on:command={(e) => sendCommand(e.detail)}
                on:clear={() => { logs = []; logSince = 0; }} />

        {:else if activeTab === "players"}
            <ServerPlayers serverId={server.id} running={status?.running} />

        {:else if activeTab === "files"}
            <ServerFileManager serverId={server.id} />

        {:else if activeTab === "mods"}
            <div class="mods-tab">
                {#if modDetail}
                    <button class="back-link" on:click={() => { modDetail = null; modDetailVersions = []; dependencyTree = null; }}>← Back</button>
                    <div class="mod-detail-header">
                        {#if modDetail.icon_url}
                            <img class="mod-detail-icon" src={modDetail.icon_url} alt={modDetail.title} />
                        {/if}
                        <div>
                            <h3>{modDetail.title}</h3>
                            <p class="mod-detail-desc">{modDetail.description}</p>
                        </div>
                    </div>

                    <button class="resolve-btn" on:click={resolveDeps}>
                        {dependencyTree ? "Refresh Dependencies" : "Resolve Dependencies"}
                    </button>

                    {#if dependencyTree}
                        <div class="dep-tree">
                            <h4>Dependencies</h4>
                            <div class="tree-node root">{dependencyTree.project_title} v{dependencyTree.version_number}</div>
                            {#each dependencyTree.dependencies as dep}
                                <div class="tree-child">
                                    <div class="tree-node">{dep.project_title} v{dep.version_number}</div>
                                    {#each dep.dependencies as sub}
                                        <div class="tree-child sub">
                                            <div class="tree-node leaf">{sub.project_title} v{sub.version_number}</div>
                                        </div>
                                    {/each}
                                </div>
                            {/each}
                        </div>
                    {/if}

                    <div class="mod-versions">
                        {#each modDetailVersions.slice(0, 10) as version}
                            <div class="mod-version-row">
                                <div class="mod-version-info">
                                    <span class="mv-name">{version.version_number}</span>
                                    <span class="mv-meta">{version.game_versions?.join(", ") || "Any"} · {version.loaders?.join(", ")}</span>
                                </div>
                                <button class="install-mod-btn" on:click={() => installServerMod(version)}
                                    disabled={modInstalling === version.id}>
                                    {modInstalling === version.id ? "..." : "Install"}
                                </button>
                            </div>
                        {:else}
                            <p class="no-versions">No compatible versions</p>
                        {/each}
                    </div>
                {:else}
                    <div class="mods-header">
                        <h3>Installed Mods ({installedMods.length})</h3>
                        <div class="mod-search-row">
                            <input type="text" placeholder="Search Modrinth..." bind:value={modQuery}
                                on:input={() => { clearTimeout(modDebounce); modDebounce = setTimeout(searchMods, 300); }} />
                        </div>
                    </div>

                    {#if modLoading}
                        <div class="mods-loading">Searching...</div>
                    {:else if modQuery && modResults.length > 0}
                        <div class="mod-search-results">
                            {#each modResults as mod}
                                <button class="mod-result-card" on:click={() => showModDetail(mod)}>
                                    <div class="mod-result-icon">
                                        {#if mod.icon_url}
                                            <img src={mod.icon_url} alt={mod.title} />
                                        {:else}
                                            <span class="mat-icon">inventory_2</span>
                                        {/if}
                                    </div>
                                    <div class="mod-result-info">
                                        <span class="mod-result-title">{mod.title}</span>
                                        <span class="mod-result-author">{mod.author}</span>
                                        <span class="mod-result-desc">{mod.description}</span>
                                    </div>
                                    <span class="mod-result-dl">{formatNumber(mod.downloads)}</span>
                                </button>
                            {/each}
                        </div>
                    {:else if modQuery}
                        <div class="mods-loading">No mods found</div>
                    {:else}
                        <div class="installed-mods">
                            {#each installedMods as mod}
                                <div class="installed-mod-row">
                                    <span class="mat-icon">inventory_2</span>
                                    <span>{mod}</span>
                                    <button class="remove-mod-btn" on:click={() => removeMod(mod)}>✕</button>
                                </div>
                            {:else}
                                <p class="no-mods">No mods installed</p>
                            {/each}
                        </div>
                    {/if}
                {/if}
                {#if modInstallStatus}
                    <div class="mod-install-status">{modInstallStatus}</div>
                {/if}
            </div>

        {:else if activeTab === "backups"}
            <ServerBackups serverId={server.id} running={status?.running} />

        {:else if activeTab === "world"}
            <ServerWorld serverId={server.id} />

        {:else if activeTab === "stats"}
            <ServerStats serverId={server.id} {status} />

        {/if}
    </div>
</div>

<style>
    .dashboard { display: flex; flex-direction: column; height: 100%; gap: 0; }
    .dash-header { display: flex; align-items: center; gap: 10px; padding-bottom: 10px; border-bottom: 1px solid rgba(255,255,255,0.06); margin-bottom: 6px; flex-wrap: wrap; }
    .back-btn { background: none; border: none; color: var(--amt-accent, #ACC4DE); font-size: 18px; cursor: pointer; padding: 0 4px; }
    .dash-info { flex: 1; min-width: 120px; }
    .dash-name { font-weight: 600; font-size: 14px; display: block; }
    .dash-meta { font-size: 10px; color: rgba(255,255,255,0.4); }
    .dash-status { display: flex; align-items: center; gap: 6px; font-size: 12px; }
    .status-indicator { width: 8px; height: 8px; border-radius: 50%; }
    .status-indicator.running { background: #60B675; box-shadow: 0 0 6px rgba(96,182,117,0.4); }
    .status-indicator.stopped { background: rgba(255,255,255,0.2); }
    .status-label { color: rgba(255,255,255,0.5); }
    .uptime { font-size: 10px; color: rgba(255,255,255,0.3); }
    .dash-actions { display: flex; gap: 4px; flex-wrap: wrap; }
    .action-btn { padding: 5px 12px; border: none; border-radius: 5px; font-size: 11px; font-weight: 600; cursor: pointer; font-family: "Inter", sans-serif; }
    .action-btn.start { background: #60B675; color: white; }
    .action-btn.stop { background: #FE4C2E; color: white; }
    .action-btn.delete { background: rgba(254,76,46,0.12); color: #FE4C2E; }
    .action-btn.connect { background: rgba(96,182,117,0.12); color: #60B675; }

    .tabs { display: flex; gap: 2px; background: rgba(0,0,0,0.2); padding: 3px; border-radius: 8px; margin-bottom: 10px; overflow-x: auto; }
    .tab { display: flex; align-items: center; gap: 4px; padding: 6px 10px; border: none; background: transparent; color: rgba(255,255,255,0.45); border-radius: 5px; cursor: pointer; font-family: "Inter", sans-serif; font-size: 11px; font-weight: 500; white-space: nowrap; transition: all 0.12s; }
    .tab.active { background: rgba(255,255,255,0.08); color: white; }
    .tab .mat-icon { font-size: 16px; }
    .tab-label { display: none; }
    @media (min-width: 700px) { .tab-label { display: inline; } }

    .tab-content { flex: 1; overflow: hidden; display: flex; flex-direction: column; }

    .overview-panel { display: flex; flex-direction: column; gap: 12px; overflow-y: auto; }
    .overview-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: 6px; }
    .ov-card { background: rgba(0,0,0,0.12); border-radius: 8px; padding: 12px; display: flex; flex-direction: column; gap: 2px; }
    .ov-label { font-size: 10px; color: rgba(255,255,255,0.4); text-transform: uppercase; letter-spacing: 0.3px; }
    .ov-value { font-size: 18px; font-weight: 700; }
    .ov-value.tps-good { color: #60B675; }
    .ov-value.tps-warn { color: #FFCC00; }
    .ov-value.tps-bad { color: #FE4C2E; }

    .share-section { background: rgba(0,0,0,0.12); border-radius: 8px; padding: 12px; }
    .share-section h4 { margin: 0 0 8px; font-size: 12px; }
    .public-addr { background: rgba(96,182,117,0.1); border: 1px solid rgba(96,182,117,0.2); border-radius: 6px; padding: 8px 12px; margin-bottom: 8px; }
    .addr-label { font-size: 10px; color: rgba(255,255,255,0.5); text-transform: uppercase; }
    .addr-value { font-size: 13px; font-weight: 600; color: #60B675; font-family: "JetBrains Mono", monospace; }
    .tunnel-error { background: rgba(254,76,46,0.1); border: 1px solid rgba(254,76,46,0.2); border-radius: 6px; padding: 6px 10px; font-size: 11px; color: #FE4C2E; margin-bottom: 8px; }
    .share-actions { display: flex; gap: 6px; }
    .share-btn, .tunnel-btn { padding: 7px 14px; border: none; border-radius: 6px; font-size: 11px; font-weight: 600; cursor: pointer; font-family: "Inter", sans-serif; }
    .share-btn { background: var(--amt-accent, #ACC4DE); color: #0D1117; }
    .tunnel-btn { background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1); color: white; }
    .tunnel-btn.active { background: rgba(254,76,46,0.12); border-color: #FE4C2E; color: #FE4C2E; }

    .back-link { background: none; border: none; color: var(--amt-accent, #ACC4DE); cursor: pointer; font-size: 12px; text-align: left; padding: 0 0 8px; font-family: "Inter", sans-serif; }

    .mods-tab { display: flex; flex-direction: column; gap: 8px; height: 100%; overflow: hidden; }
    .mods-header { display: flex; flex-direction: column; gap: 6px; }
    .mods-header h3 { margin: 0; font-size: 13px; font-weight: 600; }
    .mod-search-row input { width: 100%; padding: 7px 10px; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.1); border-radius: 6px; color: white; font-family: "Inter", sans-serif; font-size: 12px; outline: none; box-sizing: border-box; }
    .mod-search-results { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 4px; }
    .mod-result-card { display: flex; gap: 8px; padding: 8px 10px; background: rgba(0,0,0,0.12); border: 1px solid transparent; border-radius: 6px; cursor: pointer; transition: all 0.12s; text-align: left; font-family: "Inter", sans-serif; color: white; width: 100%; align-items: center; }
    .mod-result-card:hover { border-color: rgba(255,255,255,0.08); }
    .mod-result-icon { width: 32px; height: 32px; border-radius: 4px; overflow: hidden; flex-shrink: 0; display: flex; align-items: center; justify-content: center; background: rgba(255,255,255,0.04); }
    .mod-result-icon img { width: 100%; height: 100%; object-fit: cover; }
    .mod-result-info { flex: 1; min-width: 0; }
    .mod-result-title { font-weight: 600; font-size: 12px; display: block; }
    .mod-result-author { font-size: 9px; color: rgba(255,255,255,0.35); }
    .mod-result-desc { font-size: 10px; color: rgba(255,255,255,0.45); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .mod-result-dl { font-size: 10px; color: rgba(255,255,255,0.3); flex-shrink: 0; }
    .installed-mods { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }
    .installed-mod-row { display: flex; align-items: center; gap: 6px; padding: 6px 10px; background: rgba(0,0,0,0.12); border-radius: 4px; font-size: 12px; }
    .installed-mod-row .mat-icon { font-size: 14px; color: rgba(255,255,255,0.3); }
    .remove-mod-btn { margin-left: auto; background: none; border: none; color: rgba(255,255,255,0.3); cursor: pointer; padding: 0 4px; }
    .remove-mod-btn:hover { color: #FE4C2E; }
    .no-mods, .mods-loading { text-align: center; color: rgba(255,255,255,0.3); padding: 16px; font-size: 12px; }
    .mod-detail-header { display: flex; gap: 10px; align-items: flex-start; }
    .mod-detail-icon { width: 44px; height: 44px; border-radius: 6px; }
    .mod-detail-header h3 { margin: 0; font-size: 14px; }
    .mod-detail-desc { margin: 3px 0 0; font-size: 11px; color: rgba(255,255,255,0.5); }
    .resolve-btn { padding: 5px 12px; background: rgba(172,196,222,0.1); border: 1px solid rgba(172,196,222,0.2); border-radius: 5px; color: var(--amt-accent, #ACC4DE); cursor: pointer; font-size: 11px; font-weight: 500; }
    .dep-tree { background: rgba(0,0,0,0.15); border-radius: 6px; padding: 8px 10px; }
    .dep-tree h4 { margin: 0 0 4px; font-size: 11px; color: rgba(255,255,255,0.5); }
    .tree-node { padding: 2px 6px; margin: 1px 0; border-radius: 3px; font-size: 11px; background: rgba(255,255,255,0.04); }
    .tree-node.root { background: rgba(172,196,222,0.1); color: var(--amt-accent, #ACC4DE); font-weight: 600; }
    .tree-node.leaf { color: rgba(255,255,255,0.45); }
    .tree-child { margin-left: 14px; }
    .tree-child.sub { margin-left: 28px; }
    .mod-versions { display: flex; flex-direction: column; gap: 3px; flex: 1; overflow-y: auto; }
    .mod-version-row { display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: rgba(0,0,0,0.12); border-radius: 4px; }
    .mod-version-info { display: flex; flex-direction: column; gap: 1px; }
    .mv-name { font-weight: 500; font-size: 12px; }
    .mv-meta { font-size: 10px; color: rgba(255,255,255,0.45); }
    .install-mod-btn { padding: 4px 12px; background: var(--amt-accent, #ACC4DE); color: #0D1117; border: none; border-radius: 4px; font-weight: 600; font-size: 10px; cursor: pointer; }
    .install-mod-btn:disabled { opacity: 0.4; }
    .no-versions { color: rgba(255,255,255,0.3); font-size: 12px; text-align: center; padding: 12px; }
    .mod-install-status { padding: 6px 10px; background: rgba(96,182,117,0.08); border: 1px solid rgba(96,182,117,0.15); border-radius: 4px; color: #60B675; font-size: 11px; text-align: center; }
</style>
