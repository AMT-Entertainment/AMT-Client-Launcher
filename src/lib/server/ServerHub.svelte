<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";
    import CreateServerWizard from "./CreateServerWizard.svelte";
    import ServerDashboard from "./ServerDashboard.svelte";

    let servers = [];
    let loading = true;
    let showCreate = false;
    let selectedServer = null;
    let statusMap = {};
    let statusInterval;

    onMount(async () => {
        await refreshServers();
        statusInterval = setInterval(refreshStatuses, 5000);
    });

    onDestroy(() => {
        if (statusInterval) clearInterval(statusInterval);
    });

    async function refreshServers() {
        try {
            servers = await invoke("server_list");
        } catch (e) {
            console.error("Failed to load servers:", e);
        }
        loading = false;
        await refreshStatuses();
    }

    async function refreshStatuses() {
        for (const s of servers) {
            try {
                const status = await invoke("server_status", { serverId: s.id });
                statusMap[s.id] = status;
            } catch { /* ignore */ }
        }
        statusMap = { ...statusMap };
    }

    async function handleCreate(config) {
        try {
            await invoke("server_create", { config });
            showCreate = false;
            await refreshServers();
        } catch (e) {
            alert("Failed to create server: " + e);
        }
    }

    async function handleStart(serverId) {
        try {
            await invoke("server_start", { serverId });
            await refreshStatuses();
        } catch (e) {
            alert("Failed to start: " + e);
        }
    }

    async function handleStop(serverId) {
        try {
            await invoke("server_stop", { serverId });
            await refreshStatuses();
        } catch (e) {
            alert("Failed to stop: " + e);
        }
    }

    async function handleDelete(serverId) {
        if (!confirm("Delete this server? All data will be lost.")) return;
        try {
            await invoke("server_delete", { serverId });
        selectedServer = null;
        await refreshServers();
        } catch (e) {
            alert("Failed to delete: " + e);
        }
    }

    function statusBadge(status) {
        if (!status) return { text: "Unknown", cls: "unknown" };
        if (status.running) return { text: "Running", cls: "running" };
        return { text: "Stopped", cls: "stopped" };
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

<div class="server-hub">
    {#if selectedServer}
        <ServerDashboard
            server={selectedServer}
            status={statusMap[selectedServer.id]}
            on:back={() => selectedServer = null}
            on:start={() => handleStart(selectedServer.id)}
            on:stop={() => handleStop(selectedServer.id)}
            on:delete={() => { const id = selectedServer.id; selectedServer = null; handleDelete(id); }}
        />
    {:else if showCreate}
        <div class="create-header">
            <button class="back-btn" on:click={() => showCreate = false}>← Back</button>
            <h2>Create New Server</h2>
        </div>
        <CreateServerWizard onCreate={handleCreate} />
    {:else}
        <div class="hub-header">
            <h2>My Servers</h2>
            <button class="create-btn" on:click={() => showCreate = true}>+ New Server</button>
        </div>

        {#if loading}
            <div class="loading">Loading servers...</div>
        {:else if servers.length === 0}
            <div class="empty">
                <p>No servers yet. Create your first server to get started!</p>
            </div>
        {:else}
            <div class="server-list">
                {#each servers as server}
                    {@const status = statusMap[server.id]}
                    {@const badge = statusBadge(status)}
                    <button class="server-card" on:click={() => selectedServer = server}>
                        <div class="server-icon type-{server.server_type?.toLowerCase()}">
                            <span class="icon-text">{server.server_type === "Paper" ? "P" : server.server_type === "Fabric" ? "F" : server.server_type === "Forge" ? "Fo" : server.server_type === "NeoForge" ? "Nf" : server.server_type === "Spigot" ? "Sp" : server.server_type === "Purpur" ? "Pu" : server.server_type === "Folia" ? "Fl" : "V"}</span>
                        </div>
                        <div class="server-info">
                            <div class="server-name">{server.name}</div>
                            <div class="server-meta">
                                {server.server_type} · {server.mc_version}
                            </div>
                        </div>
                        <div class="status-section">
                            <span class="status-dot {badge.cls}"></span>
                            <span class="status-text">{badge.text}</span>
                            {#if status?.running}
                                <span class="uptime">{formatUptime(status.uptime_secs)}</span>
                            {/if}
                        </div>
                        <div class="server-actions">
                            {#if status?.running}
                                <button class="action-btn stop" on:click|stopPropagation={() => handleStop(server.id)}>Stop</button>
                            {:else}
                                <button class="action-btn start" on:click|stopPropagation={() => handleStart(server.id)}>Start</button>
                            {/if}
                            <button class="action-btn delete" on:click|stopPropagation={() => handleDelete(server.id)}>Del</button>
                        </div>
                    </button>
                {/each}
            </div>
        {/if}
    {/if}
</div>

<style>
    .server-hub {
        display: flex;
        flex-direction: column;
        height: 100%;
        color: white;
    }

    .hub-header, .create-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 16px;
    }

    .hub-header h2, .create-header h2 {
        margin: 0;
        font-size: 18px;
    }

    .back-btn {
        background: none;
        border: none;
        color: var(--amt-accent, #ACC4DE);
        cursor: pointer;
        font-family: "Inter", sans-serif;
        font-size: 13px;
        padding: 0;
        margin-right: 12px;
    }

    .create-btn {
        padding: 8px 16px;
        background: var(--amt-accent, #ACC4DE);
        color: #0D1117;
        border: none;
        border-radius: 8px;
        font-weight: 600;
        font-size: 13px;
        cursor: pointer;
        font-family: "Inter", sans-serif;
    }

    .server-list {
        display: flex;
        flex-direction: column;
        gap: 8px;
        flex: 1;
        overflow-y: auto;
    }

    .server-card {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 14px 16px;
        background: rgba(0,0,0,0.2);
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 10px;
        cursor: pointer;
        transition: all 0.15s;
        width: 100%;
        text-align: left;
        font-family: "Inter", sans-serif;
    }

    .server-card:hover {
        background: rgba(255,255,255,0.04);
        border-color: rgba(255,255,255,0.12);
    }

    .server-icon {
        width: 44px;
        height: 44px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 10px;
        font-weight: 700;
        font-size: 16px;
    }

    .server-icon.type-paper { background: rgba(96,182,117,0.2); color: #60B675; }
    .server-icon.type-fabric { background: rgba(172,196,222,0.2); color: #ACC4DE; }
    .server-icon.type-forge { background: rgba(254,76,46,0.2); color: #FE4C2E; }
    .server-icon.type-vanilla { background: rgba(255,204,0,0.2); color: #FFCC00; }

    .server-info {
        flex: 1;
    }

    .server-name {
        font-weight: 600;
        font-size: 14px;
        margin-bottom: 2px;
    }

    .server-meta {
        font-size: 12px;
        color: rgba(255,255,255,0.4);
    }

    .status-section {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .status-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
    }

    .status-dot.running { background: #60B675; }
    .status-dot.stopped { background: rgba(255,255,255,0.2); }
    .status-dot.unknown { background: rgba(255,255,255,0.1); }

    .status-text {
        font-size: 12px;
        color: rgba(255,255,255,0.5);
    }

    .uptime {
        font-size: 11px;
        color: rgba(255,255,255,0.3);
        margin-left: 4px;
    }

    .server-actions {
        display: flex;
        gap: 6px;
    }

    .action-btn {
        padding: 5px 12px;
        border: none;
        border-radius: 6px;
        font-size: 11px;
        font-weight: 600;
        cursor: pointer;
        font-family: "Inter", sans-serif;
    }

    .action-btn.start { background: #60B675; color: white; }
    .action-btn.stop { background: #FE4C2E; color: white; }
    .action-btn.delete { background: rgba(254,76,46,0.15); color: #FE4C2E; }

    .loading, .empty {
        text-align: center;
        padding: 40px;
        color: rgba(255,255,255,0.4);
    }
</style>
