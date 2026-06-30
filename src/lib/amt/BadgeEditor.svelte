<script>
    import BadgeChip from "./BadgeChip.svelte";

    export let badge = "AMT";
    export let accent = "#ACC4DE";

    const SYMBOLS = ["⚔", "★", "◆", "⚡", "♥", "♦", "●", "■", "▲", "▼", "✚", "✦", "✧", "※", "⁂", "†", "‡", "❖", "♠", "♣", "►", "◄"];

    function selectSymbol(symbol) {
        if (badge.length >= 5) return;
        badge = badge + symbol;
    }

    function updateBadge(e) {
        const val = e.target.value.toUpperCase();
        if (val.length <= 5) {
            badge = val;
        }
    }

    function clearBadge() {
        badge = "";
    }
</script>

<div class="badge-editor">
    <h3 class="section-title">Badge Editor</h3>

    <div class="preview-row">
        <span class="preview-label">Preview:</span>
        <div class="chat-preview">
            <BadgeChip {badge} {accent} size="large" />
            <span class="player-name">PlayerName</span>
        </div>
    </div>

    <div class="input-row">
        <input
            class="amt-input badge-input"
            type="text"
            value={badge}
            on:input={updateBadge}
            placeholder="Enter badge (1-5 chars)"
            maxlength="5"
        />
        <button class="amt-btn amt-btn-secondary small-btn" on:click={clearBadge}>Clear</button>
    </div>

    <div class="symbol-grid">
        {#each SYMBOLS as symbol}
            <button
                class="symbol-btn"
                on:click={() => selectSymbol(symbol)}
                title="Insert {symbol}"
            >
                {symbol}
            </button>
        {/each}
    </div>

    <p class="hint">Mix text and symbols. Max 5 characters total.</p>
</div>

<style>
    .badge-editor {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .section-title {
        font-size: 16px;
        font-weight: 600;
        color: white;
        margin: 0;
    }

    .preview-row {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .preview-label {
        color: rgba(255, 255, 255, 0.5);
        font-size: 13px;
        min-width: 60px;
    }

    .chat-preview {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        background: rgba(0, 0, 0, 0.3);
        border-radius: 6px;
        border: 1px solid rgba(255, 255, 255, 0.08);
    }

    .player-name {
        color: rgba(255, 255, 255, 0.8);
        font-size: 13px;
    }

    .input-row {
        display: flex;
        gap: 8px;
    }

    .badge-input {
        flex: 1;
    }

    .small-btn {
        padding: 8px 14px;
        font-size: 12px;
    }

    .symbol-grid {
        display: flex;
        flex-wrap: wrap;
        gap: 4px;
    }

    .symbol-btn {
        width: 36px;
        height: 36px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 6px;
        cursor: pointer;
        font-size: 16px;
        color: white;
        transition: all 0.15s;
    }

    .symbol-btn:hover {
        background: rgba(255, 255, 255, 0.15);
        border-color: var(--amt-accent);
    }

    .hint {
        color: rgba(255, 255, 255, 0.35);
        font-size: 11px;
        margin: 0;
    }
</style>
