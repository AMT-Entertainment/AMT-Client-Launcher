<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";

    export let serverId;
    export let running = false;

    let onlinePlayers = [];
    let whitelistedPlayers = [];
    let ops = [];
    let bannedPlayers = [];
    let playerInput = "";
    let inputMode = "whitelist";
    let rconConnected = false;
    let statusMsg = "";
    let playersInterval;

    const modes = ["whitelist", "op", "ban"];

    onMount(async () => {
        await loadPlayers();
        if (running) playersInterval = setInterval(loadOnlinePlayers, 5000);
    });

    onDestroy(() => {
        if (playersInterval) clearInterval(playersInterval);
    });

    async function connectRcon() {
        try {
            await invoke("rcon_connect", { serverId });
            rconConnected = true;
            statusMsg = "RCON connected";
        } catch (e) {
            statusMsg = `RCON failed: ${e}`;
        }
    }

    async function loadOnlinePlayers() {
        if (!rconConnected && running) {
            try {
                const result = await invoke("server_player_history", { serverId });
                onlinePlayers = result?.players || [];
            } catch {}
            return;
        }
        if (!rconConnected) return;
        try {
            const result = await invoke("rcon_player_list", { serverId });
            onlinePlayers = result || [];
        } catch {}
    }

    async function loadPlayers() {
        try {
            const info = await invoke("server_status", { serverId });
            whitelistedPlayers = info?.whitelist || [];
            ops = info?.ops || [];
            bannedPlayers = info?.banned || [];
        } catch {}
    }

    async function handleAction(name, action) {
        if (!name.trim()) return;
        try {
            if (rconConnected) {
                const cmd = action === "whitelist" ? `whitelist add ${name}`
                    : action === "op" ? `op ${name}`
                    : action === "ban" ? `ban ${name}`
                    : action === "kick" ? `kick ${name}`
                    : action === "pardon" ? `pardon ${name}` : "";
                if (cmd) await invoke("rcon_send_command", { serverId, command: cmd });
            }
            if (action === "whitelist" && !whitelistedPlayers.includes(name)) {
                whitelistedPlayers = [...whitelistedPlayers, name];
            } else if (action === "op" && !ops.includes(name)) {
                ops = [...ops, name];
            } else if (action === "ban" && !bannedPlayers.includes(name)) {
                bannedPlayers = [...bannedPlayers, name];
            }
            playerInput = "";
        } catch (e) {
            statusMsg = `Failed: ${e}`;
        }
    }

    async function removeAction(name, list) {
        try {
            const cmd = list === "whitelist" ? `whitelist remove ${name}`
                : list === "ops" ? `deop ${name}`
                : `pardon ${name}`;
            if (rconConnected) await invoke("rcon_send_command", { serverId, command: cmd });
            if (list === "whitelist") whitelistedPlayers = whitelistedPlayers.filter(p => p !== name);
            else if (list === "ops") ops = ops.filter(p => p !== name);
            else bannedPlayers = bannedPlayers.filter(p => p !== name);
        } catch (e) {
            statusMsg = `Failed: ${e}`;
        }
    }
</script>

<div class="players-panel">
    {#if !rconConnected && running}
        <div class="rcon-banner">
            <span>RCON not connected — player actions use log parsing (limited)</span>
            <button class="rcon-btn" on:click={connectRcon}>Connect RCON</button>
        </div>
    {/if}

    {#if statusMsg}
        <div class="status-msg">{statusMsg}</div>
    {/if}

    {#if onlinePlayers.length > 0}
        <div class="section">
            <h3>Online Players ({onlinePlayers.length})</h3>
            <div class="player-grid">
                {#each onlinePlayers as p}
                    <div class="player-chip">
                        <span class="online-dot"></span>
                        <span>{p.name || p}</span>
                        <div class="player-actions">
                            <button class="paction" on:click={() => handleAction(p.name || p, "kick")}
                                title="Kick">✕</button>
                            <button class="paction" on:click={() => handleAction(p.name || p, "op")}
                                title="OP">★</button>
                            <button class="paction warn" on:click={() => handleAction(p.name || p, "ban")}
                                title="Ban">⛔</button>
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    {/if}

    <div class="action-bar">
        <select class="mode-select" bind:value={inputMode}>
            <option value="whitelist">Whitelist</option>
            <option value="op">Operator</option>
            <option value="ban">Ban</option>
        </select>
        <input class="player-input" type="text" bind:value={playerInput}
            placeholder="Player name..."
            on:keydown={(e) => e.key === "Enter" && handleAction(playerInput, inputMode)} />
        <button class="action-btn" on:click={() => handleAction(playerInput, inputMode)}>Add</button>
    </div>

    <div class="sections">
        <div class="section">
            <h4>Whitelist ({whitelistedPlayers.length})</h4>
            {#each whitelistedPlayers as p}
                <div class="list-row">
                    <span class="mat-icon sm">person</span>
                    <span>{p}</span>
                    <button class="remove-btn" on:click={() => removeAction(p, "whitelist")}>✕</button>
                </div>
            {:else}
                <p class="empty-text">No whitelisted players</p>
            {/each}
        </div>

        <div class="section">
            <h4>Operators ({ops.length})</h4>
            {#each ops as p}
                <div class="list-row">
                    <span class="mat-icon sm">star</span>
                    <span>{p}</span>
                    <button class="remove-btn" on:click={() => removeAction(p, "ops")}>✕</button>
                </div>
            {:else}
                <p class="empty-text">No operators</p>
            {/each}
        </div>

        <div class="section">
            <h4>Banned ({bannedPlayers.length})</h4>
            {#each bannedPlayers as p}
                <div class="list-row">
                    <span class="mat-icon sm">block</span>
                    <span>{p}</span>
                    <button class="remove-btn" on:click={() => removeAction(p, "banned")}>Pardon</button>
                </div>
            {:else}
                <p class="empty-text">No banned players</p>
            {/each}
        </div>
    </div>
</div>

<style>
    .players-panel { display: flex; flex-direction: column; gap: 10px; height: 100%; overflow-y: auto; }
    .rcon-banner { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; background: rgba(255,204,0,0.08); border: 1px solid rgba(255,204,0,0.15); border-radius: 6px; font-size: 11px; color: #FFCC00; }
    .rcon-btn { padding: 4px 12px; background: var(--amt-accent, #ACC4DE); color: #0D1117; border: none; border-radius: 4px; font-weight: 600; font-size: 10px; cursor: pointer; }
    .status-msg { padding: 6px 10px; background: rgba(96,182,117,0.1); border-radius: 4px; font-size: 11px; color: #60B675; text-align: center; }
    .section { display: flex; flex-direction: column; gap: 4px; }
    .section h3, .section h4 { margin: 0; font-size: 13px; font-weight: 600; color: rgba(255,255,255,0.7); }
    .section h4 { font-size: 12px; margin-top: 6px; }
    .player-grid { display: flex; flex-wrap: wrap; gap: 4px; }
    .player-chip { display: inline-flex; align-items: center; gap: 4px; padding: 4px 8px; background: rgba(0,0,0,0.2); border-radius: 6px; font-size: 12px; }
    .online-dot { width: 6px; height: 6px; border-radius: 50%; background: #60B675; }
    .player-actions { display: flex; gap: 2px; margin-left: 4px; }
    .paction { background: none; border: none; color: rgba(255,255,255,0.4); cursor: pointer; padding: 0 2px; font-size: 11px; }
    .paction:hover { color: white; }
    .paction.warn:hover { color: #FE4C2E; }
    .action-bar { display: flex; gap: 6px; }
    .mode-select { padding: 6px 8px; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.1); border-radius: 6px; color: white; font-family: "Inter", sans-serif; font-size: 12px; outline: none; }
    .player-input { flex: 1; padding: 6px 10px; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.1); border-radius: 6px; color: white; font-family: "Inter", sans-serif; font-size: 12px; outline: none; }
    .action-btn { padding: 6px 14px; background: #4677FF; color: white; border: none; border-radius: 6px; font-weight: 600; font-size: 11px; cursor: pointer; }
    .sections { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px; }
    .list-row { display: flex; align-items: center; gap: 6px; padding: 5px 8px; background: rgba(0,0,0,0.12); border-radius: 4px; font-size: 12px; }
    .mat-icon.sm { font-size: 12px; color: rgba(255,255,255,0.4); }
    .remove-btn { margin-left: auto; background: none; border: none; color: rgba(255,255,255,0.3); cursor: pointer; font-size: 11px; padding: 0 4px; }
    .remove-btn:hover { color: #FE4C2E; }
    .empty-text { color: rgba(255,255,255,0.25); font-size: 11px; text-align: center; padding: 8px; }
</style>
