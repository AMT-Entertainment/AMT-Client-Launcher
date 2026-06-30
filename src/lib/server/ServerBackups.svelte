<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    export let serverId;
    export let running = false;

    let backups = [];
    let loading = false;
    let creating = false;
    let statusMsg = "";

    onMount(() => refreshBackups());

    async function refreshBackups() {
        loading = true;
        try {
            backups = await invoke("server_backup_list", { serverId });
        } catch (e) { statusMsg = `Error: ${e}`; }
        loading = false;
    }

    async function createBackup() {
        creating = true;
        statusMsg = "Creating backup...";
        try {
            await invoke("server_backup_create", { serverId });
            statusMsg = "Backup created!";
            await refreshBackups();
        } catch (e) { statusMsg = `Failed: ${e}`; }
        creating = false;
    }

    async function restoreBackup(filename) {
        if (!confirm(`Restore backup ${filename}? This will overwrite current data.`)) return;
        statusMsg = "Restoring...";
        try {
            await invoke("server_backup_restore", { serverId, filename });
            statusMsg = "Backup restored!";
        } catch (e) { statusMsg = `Failed: ${e}`; }
    }

    async function deleteBackup(filename) {
        if (!confirm(`Delete backup ${filename}?`)) return;
        try {
            await invoke("server_backup_delete", { serverId, filename });
            await refreshBackups();
        } catch (e) { statusMsg = `Error: ${e}`; }
    }

    function formatSize(bytes) {
        if (bytes >= 1073741824) return `${(bytes / 1073741824).toFixed(1)} GB`;
        if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(1)} MB`;
        if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        return `${bytes} B`;
    }

    function formatDate(ts) {
        const d = new Date(ts * 1000);
        return d.toLocaleString();
    }
</script>

<div class="backups-panel">
    {#if statusMsg}
        <div class="status-msg">{statusMsg}</div>
    {/if}

    <button class="create-backup-btn" on:click={createBackup} disabled={creating}>
        {creating ? "Creating..." : "Create Backup"}
    </button>

    {#if loading}
        <div class="loading">Loading...</div>
    {:else if backups.length === 0}
        <div class="empty">No backups yet</div>
    {:else}
        <div class="backup-list">
            {#each backups as b}
                <div class="backup-row">
                    <div class="backup-info">
                        <span class="backup-name">{b.filename}</span>
                        <span class="backup-meta">
                            {formatSize(b.size_bytes)} · {formatDate(b.created_at)}
                        </span>
                    </div>
                    <div class="backup-actions">
                        <button class="b-btn restore" on:click={() => restoreBackup(b.filename)}
                            disabled={running}>Restore</button>
                        <button class="b-btn delete" on:click={() => deleteBackup(b.filename)}>Delete</button>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .backups-panel { display: flex; flex-direction: column; gap: 10px; }
    .status-msg { padding: 6px 10px; background: rgba(96,182,117,0.1); border-radius: 4px; font-size: 11px; color: #60B675; text-align: center; }
    .create-backup-btn { padding: 10px; background: var(--amt-accent, #ACC4DE); color: #0D1117; border: none; border-radius: 8px; font-weight: 600; font-size: 13px; cursor: pointer; }
    .create-backup-btn:disabled { opacity: 0.5; }
    .loading, .empty { text-align: center; color: rgba(255,255,255,0.3); padding: 20px; font-size: 13px; }
    .backup-list { display: flex; flex-direction: column; gap: 6px; max-height: 300px; overflow-y: auto; }
    .backup-row { display: flex; justify-content: space-between; align-items: center; padding: 10px 12px; background: rgba(0,0,0,0.15); border-radius: 8px; gap: 12px; }
    .backup-info { flex: 1; min-width: 0; }
    .backup-name { display: block; font-size: 12px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .backup-meta { font-size: 10px; color: rgba(255,255,255,0.4); }
    .backup-actions { display: flex; gap: 4px; flex-shrink: 0; }
    .b-btn { padding: 4px 10px; border: none; border-radius: 4px; font-size: 10px; font-weight: 600; cursor: pointer; }
    .b-btn.restore { background: rgba(96,182,117,0.12); color: #60B675; }
    .b-btn.restore:disabled { opacity: 0.4; }
    .b-btn.delete { background: rgba(254,76,46,0.1); color: #FE4C2E; }
</style>
