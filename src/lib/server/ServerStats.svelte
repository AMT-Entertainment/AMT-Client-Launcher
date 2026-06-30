<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";

    export let serverId;
    export let status;

    let metrics = [];
    let playerHistory = [];

    let metricsInterval;

    onMount(() => {
        loadMetrics();
        loadPlayerHistory();
        metricsInterval = setInterval(loadMetrics, 4000);
    });

    onDestroy(() => {
        if (metricsInterval) clearInterval(metricsInterval);
    });

    async function loadMetrics() {
        try {
            const data = await invoke("server_metrics", { serverId });
            metrics = data || [];
        } catch {}
    }

    async function loadPlayerHistory() {
        try {
            const data = await invoke("server_player_history", { serverId });
            playerHistory = data || [];
        } catch {}
    }

    $: memData = metrics.slice(-30).map(m => m.memory_mb || 0);
    $: tpsData = metrics.slice(-30).map(m => m.tps || 20);
    $: playerData = playerHistory.slice(-30);
    $: avgTps = tpsData.length > 0 ? (tpsData.reduce((a, b) => a + b, 0) / tpsData.length).toFixed(1) : "—";
    $: maxMem = Math.max(...memData, 1);
    $: currentMem = memData.length > 0 ? memData[memData.length - 1] : 0;

    function sparkline(data, max, color) {
        if (data.length < 2) return "";
        const h = 32; const w = 80;
        const points = data.map((v, i) => `${i * (w / (data.length - 1))},${h - (v / max) * h}`).join(" ");
        return `<svg width="${w}" height="${h}" viewBox="0 0 ${w} ${h}"><polyline points="${points}" fill="none" stroke="${color}" stroke-width="1.5"/></svg>`;
    }

    function formatTime(secs) {
        const h = Math.floor(secs / 3600);
        const m = Math.floor((secs % 3600) / 60);
        const s = secs % 60;
        if (h > 0) return `${h}h ${m}m`;
        if (m > 0) return `${m}m ${s}s`;
        return `${s}s`;
    }
</script>

<div class="stats-panel">
    <div class="stat-cards">
        <div class="stat-card">
            <span class="stat-label">Memory</span>
            <span class="stat-value">{currentMem} MB <span class="stat-sub">/ {status?.ram_mb || "?"} MB</span></span>
            <div class="sparkline">{@html sparkline(memData, maxMem, "#60B675")}</div>
        </div>
        <div class="stat-card">
            <span class="stat-label">TPS</span>
            <span class="stat-value" class:tps-good={avgTps >= 20} class:tps-ok={avgTps < 20 && avgTps >= 15} class:tps-bad={avgTps < 15}>
                {avgTps}
            </span>
            <div class="sparkline">{@html sparkline(tpsData, 20, "#ACC4DE")}</div>
        </div>
        <div class="stat-card">
            <span class="stat-label">CPU</span>
            <span class="stat-value">{status?.cpu_percent?.toFixed(1) || "—"}%</span>
        </div>
        <div class="stat-card">
            <span class="stat-label">Uptime</span>
            <span class="stat-value">{status?.uptime_secs ? formatTime(status.uptime_secs) : "—"}</span>
        </div>
        <div class="stat-card">
            <span class="stat-label">Players (peak)</span>
            <span class="stat-value">{status?.player_count || 0}</span>
        </div>
    </div>

    {#if playerHistory.length > 0}
        <div class="section">
            <h3>Player History</h3>
            <div class="hist-list">
                {#each playerHistory.slice(-20).reverse() as entry}
                    <div class="hist-row">
                        <span class="hist-name">{entry.player}</span>
                        <span class="hist-action" class:join={entry.action === "join"} class:leave={entry.action === "leave"}>
                            {entry.action === "join" ? "joined" : "left"}
                        </span>
                        <span class="hist-time">{entry.time || ""}</span>
                    </div>
                {/each}
            </div>
        </div>
    {/if}
</div>

<style>
    .stats-panel { display: flex; flex-direction: column; gap: 12px; }
    .stat-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 8px; }
    .stat-card { background: rgba(0,0,0,0.15); border-radius: 8px; padding: 12px; display: flex; flex-direction: column; gap: 4px; }
    .stat-label { font-size: 10px; color: rgba(255,255,255,0.4); text-transform: uppercase; letter-spacing: 0.5px; }
    .stat-value { font-size: 20px; font-weight: 700; font-variant-numeric: tabular-nums; }
    .stat-sub { font-size: 12px; font-weight: 400; color: rgba(255,255,255,0.3); }
    .stat-value.tps-good { color: #60B675; }
    .stat-value.tps-ok { color: #FFCC00; }
    .stat-value.tps-bad { color: #FE4C2E; }
    .sparkline { margin-top: 4px; opacity: 0.6; }
    .sparkline :global(svg) { display: block; }
    .section h3 { margin: 8px 0 6px; font-size: 13px; font-weight: 600; }
    .hist-list { display: flex; flex-direction: column; gap: 3px; max-height: 200px; overflow-y: auto; }
    .hist-row { display: flex; align-items: center; gap: 8px; padding: 4px 8px; background: rgba(0,0,0,0.1); border-radius: 4px; font-size: 11px; }
    .hist-name { font-weight: 500; flex: 1; }
    .hist-action { font-size: 10px; padding: 1px 6px; border-radius: 3px; }
    .hist-action.join { background: rgba(96,182,117,0.15); color: #60B675; }
    .hist-action.leave { background: rgba(254,76,46,0.12); color: #FE4C2E; }
    .hist-time { color: rgba(255,255,255,0.3); font-size: 10px; }
</style>
