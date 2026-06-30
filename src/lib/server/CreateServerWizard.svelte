<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";

    export let onCreate;

    let step = 1;
    let creating = false;

    let name = "My AMT Server";
    let serverType = "Paper";
    let mcVersion = "";
    let ramMb = 2048;
    let port = 25565;
    let motd = "Hosted on AMT Launcher";
    let maxPlayers = 20;
    let difficulty = "easy";
    let gamemode = "survival";
    let onlineMode = true;
    let pvp = true;
    let whitelist = [];
    let ops = [];
    let modIds = [];

    let modQuery = "";
    let modResults = [];
    let modLoading = false;
    let searchDebounce;
    let selectedMods = [];
    let modDetail = null;
    let modDetailVersions = [];
    let dependencyTree = null;

    const serverTypes = [
        { id: "Paper", desc: "High-performance patch server", icon: "P", cls: "paper" },
        { id: "Fabric", desc: "Lightweight mod loader", icon: "F", cls: "fabric" },
        { id: "Forge", desc: "Heavyweight mod loader", icon: "Fo", cls: "forge" },
        { id: "NeoForge", desc: "Next-gen Forge fork", icon: "Nf", cls: "forge" },
        { id: "Spigot", desc: "Bukkit-based server", icon: "Sp", cls: "vanilla" },
        { id: "Purpur", desc: "High-fork of Paper", icon: "Pu", cls: "paper" },
        { id: "Folia", desc: "Region-threaded Paper fork", icon: "Fo", cls: "paper" },
        { id: "Vanilla", desc: "Pure Minecraft", icon: "V", cls: "vanilla" },
    ];

    const mcVersions = [
        "26.1.2", "1.21.11", "1.21.4", "1.21.3", "1.21.1", "1.20.6",
        "1.20.4", "1.20.1", "1.19.4", "1.18.2", "1.17.1", "1.16.5"
    ];

    const ramOptions = [1024, 2048, 4096, 8192, 16384];

    let mcVersionList = mcVersions;
    onMount(async () => {
        try {
            const versions = await invoke("modrinth_versions_list");
            if (versions && versions.length) {
                mcVersionList = versions;
            }
        } catch {}
        if (!mcVersion && mcVersionList.length) mcVersion = mcVersionList[0];
    });

    onDestroy(() => clearTimeout(searchDebounce));

    async function searchMods() {
        if (!modQuery.trim()) return;
        modLoading = true;
        try {
            const result = await invoke("modrinth_search", { query: modQuery, projectType: "mod", limit: 20, offset: 0 });
            modResults = result.hits || [];
        } catch (e) { console.error("Mod search:", e); }
        modLoading = false;
    }

    async function showModDetail(mod) {
        try {
            modDetail = await invoke("modrinth_get_project", { projectId: mod.project_id });
            modDetailVersions = await invoke("modrinth_get_versions", { projectId: mod.project_id });
            const validLoaders = ["fabric", "quilt", "forge", "neoforge", "paper", "bukkit", "spigot", "purpur"];
            modDetailVersions = modDetailVersions.filter(v =>
                v.loaders?.some(l => validLoaders.includes(l.toLowerCase()))
            );
        } catch (e) { console.error("Mod detail:", e); }
    }

    async function toggleMod(mod) {
        if (selectedMods.find(m => m.project_id === mod.project_id)) {
            selectedMods = selectedMods.filter(m => m.project_id !== mod.project_id);
        } else {
            selectedMods = [...selectedMods, mod];
            try {
                dependencyTree = await invoke("modrinth_resolve_dependencies", {
                    projectId: mod.project_id,
                    gameVersion: mcVersion,
                    loader: serverType.toLowerCase(),
                });
            } catch (e) { console.error("Deps:", e); }
        }
    }

    async function handleCreate() {
        creating = true;
        try {
            await onCreate({
                name, server_type: serverType, mc_version: mcVersion,
                ram_mb: ramMb, port, motd, max_players: maxPlayers,
                online_mode: onlineMode, pvp, difficulty, gamemode,
                whitelist, ops, mod_ids: selectedMods.map(m => m.project_id),
                jvm_args: [], jvm_extra_args: "", auto_sharing: false,
            });
        } finally { creating = false; }
    }

    function nextStep() { if (step < 4) step++; }
    function prevStep() { if (step > 1) step--; }
</script>

<div class="wizard">
    <div class="steps">
        {#each ["Basic", "Mods", "Settings", "Review"] as s, i}
            <div class="step-indicator" class:active={step === i + 1} class:done={step > i + 1}>
                <span class="step-num">{step > i + 1 ? "✓" : i + 1}</span>
                <span class="step-label">{s}</span>
            </div>
        {/each}
    </div>

    <div class="step-content">
        {#if step === 1}
            <div class="step-pane">
                <div class="field">
                    <label>Server Name</label>
                    <input type="text" bind:value={name} placeholder="My AMT Server" />
                </div>
                <div class="field">
                    <label>Server Type</label>
                    <div class="type-grid">
                        {#each serverTypes as st}
                            <button class="type-card" class:selected={serverType === st.id} on:click={() => serverType = st.id}>
                                <span class="type-icon type-{st.cls}">{st.icon}</span>
                                <span class="type-name">{st.id}</span>
                                <span class="type-desc">{st.desc}</span>
                            </button>
                        {/each}
                    </div>
                </div>
                <div class="field">
                    <label>Minecraft Version</label>
                    <select bind:value={mcVersion}>
                        {#each mcVersionList as v}
                            <option value={v}>{v}</option>
                        {/each}
                    </select>
                </div>
            </div>

        {:else if step === 2}
            <div class="step-pane mods-step">
                <div class="field">
                    <label>Search Modrinth Mods</label>
                    <input type="text" bind:value={modQuery} placeholder="Search mods..."
                        on:input={() => { clearTimeout(searchDebounce); searchDebounce = setTimeout(searchMods, 300); }}
                    />
                </div>

                {#if modDetail}
                    <button class="back-link" on:click={() => { modDetail = null; modDetailVersions = []; }}>← Back</button>
                    <div class="mod-detail">
                        {#if modDetail.icon_url}
                            <img src={modDetail.icon_url} alt={modDetail.title} class="mod-icon" />
                        {/if}
                        <div>
                            <strong>{modDetail.title}</strong>
                            <p class="mod-desc">{modDetail.description}</p>
                        </div>
                    </div>
                    <div class="version-list">
                        {#each modDetailVersions.slice(0, 8) as version}
                            <div class="version-row">
                                <span>{version.version_number}</span>
                                <span class="ver-meta">{version.game_versions?.join(", ")}</span>
                                <button class="add-mod-btn" on:click={() => {
                                    toggleMod({ project_id: modDetail.project_id, title: modDetail.title, version_id: version.id });
                                    modDetail = null;
                                }}>Select</button>
                            </div>
                        {/each}
                    </div>
                {:else if modLoading}
                    <div class="loading-text">Searching...</div>
                {:else if modResults.length > 0}
                    <div class="mod-results">
                        {#each modResults as mod}
                            <button class="mod-card" class:selected={selectedMods.find(m => m.project_id === mod.project_id)}
                                on:click={() => showModDetail(mod)}>
                                <div class="mod-card-icon">
                                    {#if mod.icon_url}
                                        <img src={mod.icon_url} alt={mod.title} />
                                    {:else}
                                        <span class="mat-icon">inventory_2</span>
                                    {/if}
                                </div>
                                <div class="mod-card-info">
                                    <span class="mod-title">{mod.title}</span>
                                    <span class="mod-author">{mod.author}</span>
                                </div>
                                {#if selectedMods.find(m => m.project_id === mod.project_id)}
                                    <span class="selected-badge">Selected</span>
                                {/if}
                            </button>
                        {/each}
                    </div>
                {/if}

                {#if dependencyTree}
                    <div class="dep-tree">
                        <h4>Dependencies</h4>
                        <div class="tree-root">
                            <span class="tree-node root">{dependencyTree.project_title}</span>
                            {#each dependencyTree.dependencies as dep}
                                <div class="tree-child">
                                    <span class="tree-node">{dep.project_title}</span>
                                    {#each dep.dependencies as sub}
                                        <div class="tree-child sub">
                                            <span class="tree-node leaf">{sub.project_title}</span>
                                        </div>
                                    {/each}
                                </div>
                            {/each}
                        </div>
                    </div>
                {/if}

                <div class="selected-mods">
                    {#each selectedMods as mod}
                        <span class="selected-mod-chip">
                            {mod.title}
                            <button class="chip-remove" on:click={() => selectedMods = selectedMods.filter(m => m !== mod)}>×</button>
                        </span>
                    {/each}
                </div>
            </div>

        {:else if step === 3}
            <div class="step-pane">
                <div class="field-row">
                    <div class="field">
                        <label>RAM</label>
                        <select bind:value={ramMb}>
                            {#each ramOptions as r}
                                <option value={r}>{r / 1024 >= 1 ? `${r / 1024} GB` : `${r} MB`}</option>
                            {/each}
                        </select>
                    </div>
                    <div class="field">
                        <label>Port</label>
                        <input type="number" bind:value={port} min="1024" max="65535" />
                    </div>
                </div>
                <div class="field">
                    <label>MOTD</label>
                    <input type="text" bind:value={motd} />
                </div>
                <div class="field-row">
                    <div class="field">
                        <label>Max Players</label>
                        <input type="number" bind:value={maxPlayers} min="1" max="100" />
                    </div>
                    <div class="field">
                        <label>Difficulty</label>
                        <select bind:value={difficulty}>
                            <option value="peaceful">Peaceful</option>
                            <option value="easy">Easy</option>
                            <option value="normal">Normal</option>
                            <option value="hard">Hard</option>
                        </select>
                    </div>
                </div>
                <div class="field-row">
                    <div class="field">
                        <label>Game Mode</label>
                        <select bind:value={gamemode}>
                            <option value="survival">Survival</option>
                            <option value="creative">Creative</option>
                            <option value="adventure">Adventure</option>
                            <option value="spectator">Spectator</option>
                        </select>
                    </div>
                    <div class="field checkboxes">
                        <label class="checkbox-label"><input type="checkbox" bind:checked={onlineMode} /> Online Mode</label>
                        <label class="checkbox-label"><input type="checkbox" bind:checked={pvp} /> PvP</label>
                    </div>
                </div>
            </div>

        {:else if step === 4}
            <div class="step-pane review">
                <h3>Review Configuration</h3>
                <div class="review-grid">
                    <div class="review-item"><span class="rl">Name</span><span>{name}</span></div>
                    <div class="review-item"><span class="rl">Type</span><span>{serverType}</span></div>
                    <div class="review-item"><span class="rl">Version</span><span>{mcVersion}</span></div>
                    <div class="review-item"><span class="rl">RAM</span><span>{ramMb} MB</span></div>
                    <div class="review-item"><span class="rl">Port</span><span>{port}</span></div>
                    <div class="review-item"><span class="rl">Players</span><span>{maxPlayers}</span></div>
                    <div class="review-item"><span class="rl">Difficulty</span><span class="cap">{difficulty}</span></div>
                    <div class="review-item"><span class="rl">Gamemode</span><span class="cap">{gamemode}</span></div>
                    <div class="review-item"><span class="rl">MOTD</span><span>{motd}</span></div>
                </div>
                {#if selectedMods.length > 0}
                    <h4>Selected Mods ({selectedMods.length})</h4>
                    <div class="review-mods">
                        {#each selectedMods as mod}
                            <span class="review-mod">{mod.title}</span>
                        {/each}
                    </div>
                {/if}
            </div>
        {/if}
    </div>

    <div class="step-actions">
        {#if step > 1}
            <button class="nav-btn" on:click={prevStep}>Back</button>
        {:else}
            <div></div>
        {/if}
        {#if step < 4}
            <button class="nav-btn primary" on:click={nextStep}>Next</button>
        {:else}
            <button class="nav-btn primary" on:click={handleCreate} disabled={creating}>
                {creating ? "Creating..." : "Create Server"}
            </button>
        {/if}
    </div>
</div>

<style>
    .wizard { display: flex; flex-direction: column; gap: 16px; height: 100%; }
    .steps { display: flex; gap: 4px; background: rgba(0,0,0,0.2); padding: 4px; border-radius: 8px; }
    .step-indicator { flex: 1; display: flex; align-items: center; gap: 6px; padding: 8px 12px; border-radius: 6px; font-size: 12px; color: rgba(255,255,255,0.4); }
    .step-indicator.active { background: rgba(172,196,222,0.12); color: var(--amt-accent, #ACC4DE); }
    .step-indicator.done { color: #60B675; }
    .step-num { width: 20px; height: 20px; display: flex; align-items: center; justify-content: center; border-radius: 50%; background: rgba(255,255,255,0.06); font-size: 11px; font-weight: 700; }
    .step-indicator.active .step-num { background: var(--amt-accent, #ACC4DE); color: #0D1117; }
    .step-content { flex: 1; overflow-y: auto; }
    .step-pane { display: flex; flex-direction: column; gap: 14px; }
    .field { display: flex; flex-direction: column; gap: 6px; }
    .field-row { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
    label { font-size: 12px; font-weight: 600; color: rgba(255,255,255,0.6); text-transform: uppercase; letter-spacing: 0.5px; }
    input, select { padding: 10px 12px; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.1); border-radius: 8px; color: white; font-family: "Inter", sans-serif; font-size: 13px; outline: none; }
    input:focus, select:focus { border-color: var(--amt-accent, #ACC4DE); }
    .type-grid { display: grid; grid-template-columns: 1fr 1fr 1fr 1fr; gap: 6px; }
    .type-card { display: flex; flex-direction: column; align-items: center; gap: 3px; padding: 10px 8px; background: rgba(0,0,0,0.2); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; cursor: pointer; font-family: "Inter", sans-serif; text-align: center; }
    .type-card.selected { border-color: var(--amt-accent, #ACC4DE); background: rgba(172,196,222,0.08); }
    .type-icon { width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; border-radius: 6px; font-weight: 700; font-size: 13px; }
    .type-icon.type-paper { background: rgba(96,182,117,0.2); color: #60B675; }
    .type-icon.type-fabric { background: rgba(172,196,222,0.2); color: #ACC4DE; }
    .type-icon.type-forge { background: rgba(254,76,46,0.2); color: #FE4C2E; }
    .type-icon.type-vanilla { background: rgba(255,204,0,0.2); color: #FFCC00; }
    .type-name { font-weight: 600; font-size: 12px; color: white; }
    .type-desc { font-size: 9px; color: rgba(255,255,255,0.4); line-height: 1.2; }
    .mods-step { gap: 10px; }
    .back-link { background: none; border: none; color: var(--amt-accent, #ACC4DE); cursor: pointer; font-size: 12px; text-align: left; padding: 0; font-family: "Inter", sans-serif; }
    .mod-detail { display: flex; gap: 10px; align-items: flex-start; }
    .mod-icon { width: 40px; height: 40px; border-radius: 6px; }
    .mod-detail strong { font-size: 14px; }
    .mod-desc { font-size: 11px; color: rgba(255,255,255,0.5); margin: 4px 0 0; }
    .version-list { display: flex; flex-direction: column; gap: 4px; }
    .version-row { display: flex; align-items: center; justify-content: space-between; padding: 6px 10px; background: rgba(0,0,0,0.15); border-radius: 6px; font-size: 12px; }
    .ver-meta { font-size: 10px; color: rgba(255,255,255,0.4); }
    .add-mod-btn { padding: 3px 10px; background: var(--amt-accent, #ACC4DE); color: #0D1117; border: none; border-radius: 4px; font-weight: 600; font-size: 10px; cursor: pointer; }
    .loading-text { text-align: center; color: rgba(255,255,255,0.4); padding: 16px; font-size: 13px; }
    .mod-results { display: flex; flex-direction: column; gap: 4px; max-height: 200px; overflow-y: auto; }
    .mod-card { display: flex; align-items: center; gap: 8px; padding: 8px 10px; background: rgba(0,0,0,0.15); border: 1px solid transparent; border-radius: 6px; cursor: pointer; text-align: left; font-family: "Inter", sans-serif; color: white; width: 100%; }
    .mod-card.selected { border-color: var(--amt-accent, #ACC4DE); background: rgba(172,196,222,0.06); }
    .mod-card-icon { width: 28px; height: 28px; border-radius: 4px; overflow: hidden; flex-shrink: 0; display: flex; align-items: center; justify-content: center; background: rgba(255,255,255,0.05); }
    .mod-card-icon img { width: 100%; height: 100%; object-fit: cover; }
    .mod-card-info { flex: 1; min-width: 0; }
    .mod-title { font-weight: 600; font-size: 12px; display: block; }
    .mod-author { font-size: 10px; color: rgba(255,255,255,0.4); }
    .selected-badge { font-size: 10px; color: #60B675; font-weight: 600; }
    .dep-tree { background: rgba(0,0,0,0.15); border-radius: 8px; padding: 10px; }
    .dep-tree h4 { margin: 0 0 6px; font-size: 12px; }
    .tree-root { font-size: 12px; }
    .tree-node { display: block; padding: 3px 8px; background: rgba(255,255,255,0.05); border-radius: 4px; margin: 2px 0; }
    .tree-node.root { background: rgba(172,196,222,0.12); color: var(--amt-accent, #ACC4DE); font-weight: 600; }
    .tree-node.leaf { color: rgba(255,255,255,0.5); }
    .tree-child { margin-left: 16px; }
    .tree-child.sub { margin-left: 32px; }
    .selected-mods { display: flex; flex-wrap: wrap; gap: 4px; }
    .selected-mod-chip { display: inline-flex; align-items: center; gap: 4px; padding: 4px 8px; background: rgba(172,196,222,0.12); border-radius: 4px; font-size: 11px; color: var(--amt-accent, #ACC4DE); }
    .chip-remove { background: none; border: none; color: rgba(255,255,255,0.4); cursor: pointer; padding: 0; font-size: 14px; line-height: 1; }
    .checkboxes { display: flex; flex-direction: column; gap: 8px; justify-content: center; }
    .checkbox-label { display: flex; align-items: center; gap: 8px; font-size: 13px; color: white; cursor: pointer; }
    .step-actions { display: flex; justify-content: space-between; gap: 8px; padding-top: 8px; border-top: 1px solid rgba(255,255,255,0.06); }
    .nav-btn { padding: 10px 24px; border: none; border-radius: 8px; font-weight: 600; font-size: 13px; cursor: pointer; font-family: "Inter", sans-serif; color: white; background: rgba(255,255,255,0.08); }
    .nav-btn.primary { background: var(--amt-accent, #ACC4DE); color: #0D1117; }
    .nav-btn:disabled { opacity: 0.5; }
    .review h3 { margin: 0 0 8px; font-size: 15px; }
    .review h4 { margin: 12px 0 6px; font-size: 13px; }
    .review-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
    .review-item { display: flex; justify-content: space-between; padding: 6px 10px; background: rgba(0,0,0,0.15); border-radius: 6px; font-size: 12px; }
    .review-item .rl { color: rgba(255,255,255,0.4); }
    .review-item .cap { text-transform: capitalize; }
    .review-mods { display: flex; flex-wrap: wrap; gap: 4px; }
    .review-mod { padding: 3px 8px; background: rgba(172,196,222,0.1); border-radius: 4px; font-size: 11px; color: var(--amt-accent, #ACC4DE); }
</style>
