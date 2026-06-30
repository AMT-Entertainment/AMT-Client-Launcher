<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    export let serverId;

    let currentPath = ".";
    let entries = [];
    let loading = false;
    let breadcrumbs = [];
    let selectedFile = null;
    let fileContent = "";
    let newFileName = "";
    let newDirName = "";
    let showNewFile = false;
    let showNewDir = false;
    let statusMsg = "";

    onMount(() => navigate("."));

    async function navigate(relPath) {
        loading = true;
        try {
            const result = await invoke("server_files_list", {
                serverId, path: relPath,
            });
            currentPath = result.current_path || relPath;
            entries = result.entries || [];
            breadcrumbs = currentPath.split("/").filter(Boolean);
        } catch (e) {
            statusMsg = `Error: ${e}`;
        }
        loading = false;
    }

    async function openFile(entry) {
        if (entry.is_dir) {
            await navigate(entry.path);
        } else {
            try {
                selectedFile = entry;
                fileContent = await invoke("server_files_read", {
                    serverId, path: entry.path,
                });
            } catch (e) {
                statusMsg = `Error reading: ${e}`;
            }
        }
    }

    async function saveFile() {
        if (!selectedFile) return;
        try {
            await invoke("server_files_write", {
                serverId, path: selectedFile.path, content: fileContent,
            });
            statusMsg = "File saved";
        } catch (e) {
            statusMsg = `Error saving: ${e}`;
        }
    }

    async function deleteEntry(entry) {
        if (!confirm(`Delete ${entry.name}?`)) return;
        try {
            await invoke("server_files_delete", {
                serverId, path: entry.path,
            });
            await navigate(currentPath);
        } catch (e) {
            statusMsg = `Error deleting: ${e}`;
        }
    }

    async function createFile() {
        if (!newFileName.trim()) return;
        const fullPath = (currentPath === "." ? "" : currentPath) + "/" + newFileName.trim();
        try {
            await invoke("server_files_write", {
                serverId, path: fullPath, content: "",
            });
            newFileName = "";
            showNewFile = false;
            await navigate(currentPath);
        } catch (e) {
            statusMsg = `Error: ${e}`;
        }
    }

    async function createDir() {
        if (!newDirName.trim()) return;
        const fullPath = (currentPath === "." ? "" : currentPath) + "/" + newDirName.trim();
        try {
            await invoke("server_files_mkdir", {
                serverId, path: fullPath,
            });
            newDirName = "";
            showNewDir = false;
            await navigate(currentPath);
        } catch (e) {
            statusMsg = `Error: ${e}`;
        }
    }

    async function renameEntry(entry) {
        const name = prompt("New name:", entry.name);
        if (!name || name === entry.name) return;
        const newPath = (currentPath === "." ? "" : currentPath) + "/" + name;
        try {
            await invoke("server_files_rename", {
                serverId, oldPath: entry.path, newPath,
            });
            await navigate(currentPath);
        } catch (e) {
            statusMsg = `Error: ${e}`;
        }
    }

    function iconFor(entry) {
        if (entry.is_dir) return "folder";
        const ext = entry.name.split(".").pop()?.toLowerCase();
        if (["json", "yaml", "yml", "toml"].includes(ext)) return "settings";
        if (["txt", "md", "log"].includes(ext)) return "description";
        if (["jar"].includes(ext)) return "inventory_2";
        if (["png", "jpg", "gif", "svg"].includes(ext)) return "image";
        return "insert_drive_file";
    }
</script>

<div class="file-manager">
    <div class="fm-toolbar">
        <div class="breadcrumbs">
            <button class="bc-item" on:click={() => navigate(".")}>root</button>
            {#each breadcrumbs as crumb, i}
                <span class="bc-sep">/</span>
                <button class="bc-item" on:click={() => navigate(breadcrumbs.slice(0, i + 1).join("/"))}>
                    {crumb}
                </button>
            {/each}
        </div>
        <div class="fm-actions">
            <button class="fm-btn" on:click={() => showNewFile = !showNewFile}>+ File</button>
            <button class="fm-btn" on:click={() => showNewDir = !showNewDir}>+ Dir</button>
        </div>
    </div>

    {#if showNewFile}
        <div class="inline-form">
            <input type="text" bind:value={newFileName} placeholder="filename.ext"
                on:keydown={(e) => e.key === "Enter" && createFile()} />
            <button class="tiny-btn" on:click={createFile}>Create</button>
            <button class="tiny-btn muted" on:click={() => { showNewFile = false; newFileName = ""; }}>Cancel</button>
        </div>
    {/if}
    {#if showNewDir}
        <div class="inline-form">
            <input type="text" bind:value={newDirName} placeholder="directory name"
                on:keydown={(e) => e.key === "Enter" && createDir()} />
            <button class="tiny-btn" on:click={createDir}>Create</button>
            <button class="tiny-btn muted" on:click={() => { showNewDir = false; newDirName = ""; }}>Cancel</button>
        </div>
    {/if}

    {#if statusMsg}
        <div class="status-msg">{statusMsg}</div>
    {/if}

    <div class="fm-body">
        <div class="file-list">
            {#if loading}
                <div class="loading-text">Loading...</div>
            {:else}
                {#each entries as entry}
                    <button class="file-row" on:click={() => openFile(entry)}
                        on:contextmenu|preventDefault={() => renameEntry(entry)}>
                        <span class="file-icon mat-icon">{iconFor(entry)}</span>
                        <span class="file-name">{entry.name}</span>
                        <span class="file-size">
                            {entry.is_dir ? "—" : entry.size >= 1048576
                                ? `${(entry.size / 1048576).toFixed(1)} MB`
                                : entry.size >= 1024
                                    ? `${(entry.size / 1024).toFixed(1)} KB`
                                    : `${entry.size} B`}
                        </span>
                        <button class="file-del" on:click|stopPropagation={() => deleteEntry(entry)}>✕</button>
                    </button>
                {:else}
                    <div class="empty-text">Empty directory</div>
                {/each}
            {/if}
        </div>

        {#if selectedFile}
            <div class="file-editor">
                <div class="editor-header">
                    <span>{selectedFile.name}</span>
                    <div class="editor-actions">
                        <button class="fm-btn" on:click={saveFile}>Save</button>
                        <button class="fm-btn" on:click={() => selectedFile = null}>Close</button>
                    </div>
                </div>
                <textarea class="editor-content" bind:value={fileContent} spellcheck="false"></textarea>
            </div>
        {/if}
    </div>
</div>

<style>
    .file-manager { display: flex; flex-direction: column; gap: 8px; height: 100%; }
    .fm-toolbar { display: flex; justify-content: space-between; align-items: center; gap: 8px; flex-wrap: wrap; }
    .breadcrumbs { display: flex; align-items: center; gap: 2px; flex-wrap: wrap; font-size: 12px; }
    .bc-item { background: none; border: none; color: var(--amt-accent, #ACC4DE); cursor: pointer; padding: 2px 4px; font-family: "Inter", sans-serif; font-size: 12px; border-radius: 3px; }
    .bc-item:hover { background: rgba(255,255,255,0.06); }
    .bc-sep { color: rgba(255,255,255,0.2); }
    .fm-actions { display: flex; gap: 4px; }
    .fm-btn { padding: 4px 10px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1); border-radius: 4px; color: rgba(255,255,255,0.7); cursor: pointer; font-size: 11px; font-family: "Inter", sans-serif; }
    .fm-btn:hover { background: rgba(255,255,255,0.1); }
    .inline-form { display: flex; gap: 4px; align-items: center; }
    .inline-form input { flex: 1; padding: 5px 8px; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.1); border-radius: 4px; color: white; font-family: "Inter", sans-serif; font-size: 11px; outline: none; }
    .tiny-btn { padding: 4px 8px; background: #4677FF; color: white; border: none; border-radius: 3px; font-size: 10px; cursor: pointer; font-weight: 600; }
    .tiny-btn.muted { background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.5); }
    .status-msg { padding: 4px 8px; background: rgba(96,182,117,0.1); border-radius: 4px; font-size: 10px; color: #60B675; text-align: center; }
    .fm-body { display: flex; gap: 8px; flex: 1; overflow: hidden; }
    .file-list { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 2px; min-width: 200px; }
    .file-row { display: flex; align-items: center; gap: 6px; padding: 5px 8px; background: rgba(0,0,0,0.12); border: none; border-radius: 4px; cursor: pointer; text-align: left; font-family: "Inter", sans-serif; color: white; width: 100%; }
    .file-row:hover { background: rgba(255,255,255,0.04); }
    .file-icon { font-size: 14px; color: rgba(255,255,255,0.4); width: 20px; text-align: center; }
    .file-name { flex: 1; font-size: 12px; }
    .file-size { font-size: 10px; color: rgba(255,255,255,0.3); }
    .file-del { background: none; border: none; color: rgba(255,255,255,0.2); cursor: pointer; padding: 0 4px; font-size: 12px; }
    .file-del:hover { color: #FE4C2E; }
    .loading-text, .empty-text { color: rgba(255,255,255,0.3); font-size: 12px; padding: 16px; text-align: center; }
    .file-editor { flex: 1; display: flex; flex-direction: column; gap: 6px; min-width: 200px; }
    .editor-header { display: flex; justify-content: space-between; align-items: center; font-size: 12px; font-weight: 600; }
    .editor-actions { display: flex; gap: 4px; }
    .editor-content { flex: 1; background: rgba(0,0,0,0.4); border: 1px solid rgba(255,255,255,0.08); border-radius: 6px; color: rgba(255,255,255,0.85); font-family: "JetBrains Mono", monospace; font-size: 11px; padding: 10px; resize: none; outline: none; }
    .editor-content:focus { border-color: var(--amt-accent, #ACC4DE); }
</style>
