<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    export let serverId;

    let worlds = [];
    let worldBackups = {};
    let loading = false;
    let selectedWorld = null;
    let statusMsg = "";

    onMount(() => loadWorlds());

    async function loadWorlds() {
        loading = true;
        try {
            worlds = await invoke("server_worlds_list", { serverId });
        } catch (e) { statusMsg = `Error: ${e}`; }
        loading = false;
    }

    async function loadWorldBackups(worldName) {
        try {
            worldBackups[worldName] = await invoke("server_world_backups_list", {
                serverId, worldName,
            });
            worldBackups = { ...worldBackups };
        } catch (e) { statusMsg = `Error: ${e}`; }
    }

    async function backupWorld(worldName) {
        statusMsg = `Backing up ${worldName}...`;
        try {
            await invoke("server_world_backup", { serverId, worldName });
            statusMsg = "World backup created!";
            await loadWorldBackups(worldName);
        } catch (e) { statusMsg = `Failed: ${e}`; }
    }

    async function restoreWorldBackup(worldName, filename) {
        if (!confirm(`Restore ${worldName} from ${filename}?`)) return;
        statusMsg = "Restoring...";
        try {
            await invoke("server_world_backup_restore", {
                serverId, worldName, filename,
            });
            statusMsg = "World restored!";
        } catch (e) { statusMsg = `Failed: ${e}`; }
    }

    async function deleteWorldBackup(worldName, filename) {
        if (!confirm(`Delete backup ${filename}?`)) return;
        try {
            await invoke("server_world_backup_delete", {
                serverId, worldName, filename,
            });
            await loadWorldBackups(worldName);
        } catch (e) { statusMsg = `Error: ${e}`; }
    }

    async function deleteWorld(worldName) {
        if (!confirm(`Delete world ${worldName}? This is permanent.`)) return;
        try {
            await invoke("server_world_delete", { serverId, worldName });
            worlds = worlds.filter(w => w.name !== worldName);
            statusMsg = "World deleted";
        } catch (e) { statusMsg = `Failed: ${e}`; }
    }

    function formatSize(bytes) {
        if (bytes >= 1073741824) return `${(bytes / 1073741824).toFixed(1)} GB`;
        if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(1)} MB`;
        if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        return `${bytes} B`;
    }
</script>

<div class="worlds-panel">
    {#if statusMsg}
        <div class="status-msg">{statusMsg}</div>
    {/if}

    {#if loading}
        <div class="loading">Loading worlds...</div>
    {:else if worlds.length === 0}
        <div class="empty">No worlds found</div>
    {:else}
        {#each worlds as world}
            <div class="world-card">
                <div class="world-header" on:click={() => {
                    selectedWorld = selectedWorld === world.name ? null : world.name;
                    if (selectedWorld === world.name) loadWorldBackups(world.name);
                }}>
                    <div class="world-info">
                        <span class="world-name">{world.name}</span>
                        <span class="world-meta">
                            {formatSize(world.size_bytes)} · {world.region_count || 0} regions
                            · {world.player_count || 0} player data
                        </span>
                    </div>
                    <div class="world-actions">
                        <button class="w-btn" on:click|stopPropagation={() => backupWorld(world.name)}
                            title="Backup this world">Backup</button>
                        <button class="w-btn danger" on:click|stopPropagation={() => deleteWorld(world.name)}
                            title="Delete this world">Delete</button>
                    </div>
                </div>

                {#if selectedWorld === world.name}
                    <div class="world-backups">
                        <h4>Backups</h4>
                        {#if worldBackups[world.name]?.length > 0}
                            {#each worldBackups[world.name] as b}
                                <div class="wb-row">
                                    <span class="wb-name">{b.filename}</span>
                                    <span class="wb-meta">{formatSize(b.size_bytes)}</span>
                                    <button class="w-btn sm" on:click={() => restoreWorldBackup(world.name, b.filename)}
                                        title="Restore this backup">Restore</button>
                                    <button class="w-btn sm danger" on:click={() => deleteWorldBackup(world.name, b.filename)}>✕</button>
                                </div>
                            {/each}
                        {:else}
                            <p class="no-backups">No backups for this world</p>
                        {/if}
                    </div>
                {/if}
            </div>
        {/each}
    {/if}
</div>

<style>
    .worlds-panel { display: flex; flex-direction: column; gap: 8px; }
    .status-msg { padding: 6px 10px; background: rgba(96,182,117,0.1); border-radius: 4px; font-size: 11px; color: #60B675; text-align: center; }
    .loading, .empty { text-align: center; color: rgba(255,255,255,0.3); padding: 20px; font-size: 13px; }
    .world-card { background: rgba(0,0,0,0.12); border-radius: 8px; overflow: hidden; }
    .world-header { display: flex; justify-content: space-between; align-items: center; padding: 10px 12px; cursor: pointer; transition: background 0.15s; }
    .world-header:hover { background: rgba(255,255,255,0.03); }
    .world-info { flex: 1; min-width: 0; }
    .world-name { display: block; font-weight: 600; font-size: 13px; }
    .world-meta { font-size: 10px; color: rgba(255,255,255,0.4); }
    .world-actions { display: flex; gap: 4px; flex-shrink: 0; }
    .w-btn { padding: 4px 10px; border: none; border-radius: 4px; font-size: 10px; font-weight: 600; cursor: pointer; background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.7); }
    .w-btn.danger { background: rgba(254,76,46,0.1); color: #FE4C2E; }
    .w-btn.sm { padding: 2px 8px; font-size: 9px; }
    .world-backups { padding: 8px 12px 12px; border-top: 1px solid rgba(255,255,255,0.05); }
    .world-backups h4 { margin: 0 0 6px; font-size: 11px; color: rgba(255,255,255,0.5); text-transform: uppercase; }
    .wb-row { display: flex; align-items: center; gap: 8px; padding: 4px 6px; font-size: 11px; }
    .wb-name { flex: 1; font-size: 10px; }
    .wb-meta { color: rgba(255,255,255,0.3); font-size: 10px; }
    .no-backups { font-size: 10px; color: rgba(255,255,255,0.25); padding: 4px 0; }
</style>
