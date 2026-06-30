<script>
    import { createEventDispatcher, onMount, onDestroy, tick } from "svelte";

    export let logs = [];
    export let running = false;

    const dispatch = createEventDispatcher();

    let command = "";
    let filter = "";
    let searchQuery = "";
    let searchResults = [];
    let searchIndex = -1;
    let showFilter = false;
    let terminalEl;
    let autoScroll = true;

    $: filteredLogs = filter
        ? logs.filter(l => l.toLowerCase().includes(filter.toLowerCase()))
        : logs;

    function renderLine(log) {
        let html = log.replace(/</g, '&lt;').replace(/>/g, '&gt;');
        html = html.replace(/\x1b\[(\d+)m/g, (_, code) => {
            if (code === '0') return '</span>';
            const style = {
                '1': 'font-weight: bold',
                '30': 'color: #000', '31': 'color: #FE4C2E', '32': 'color: #60B675',
                '33': 'color: #FFCC00', '34': 'color: #4677FF', '35': 'color: #C084FC',
                '36': 'color: #22D3EE', '37': 'color: #E2E8F0',
                '90': 'color: #64748B', '91': 'color: #FF6B6B', '92': 'color: #7DDFA0',
                '93': 'color: #FFD666', '94': 'color: #7AA2F7', '95': 'color: #C792EA',
                '96': 'color: #4FD6E2', '97': 'color: #F8FAFC',
            }[code];
            return style ? `<span style="${style}">` : '';
        });
        const openTags = (html.match(/<span /g) || []).length;
        const closeTags = (html.match(/<\/span>/g) || []).length;
        for (let i = 0; i < openTags - closeTags; i++) html += '</span>';

        if (searchQuery) {
            const idx = html.toLowerCase().indexOf(searchQuery.toLowerCase());
            if (idx !== -1) {
                html = html.slice(0, idx) +
                    '<mark style="background: rgba(172,196,222,0.3); color: #ACC4DE; border-radius: 2px; padding: 0 2px;">' +
                    html.slice(idx, idx + searchQuery.length) + '</mark>' +
                    html.slice(idx + searchQuery.length);
            }
        }
        return html;
    }

    $: {
        if (searchQuery) {
            searchResults = [];
            filteredLogs.forEach((l, i) => {
                if (l.toLowerCase().includes(searchQuery.toLowerCase())) {
                    searchResults.push(i);
                }
            });
            searchIndex = searchResults.length > 0 ? 0 : -1;
        } else {
            searchResults = [];
            searchIndex = -1;
        }
    }

    function sendCommand() {
        if (!command.trim()) return;
        dispatch("command", command.trim());
        command = "";
    }

    function handleKey(e) {
        if (e.key === "Enter") sendCommand();
    }

    function scrollToSearch() {
        if (searchIndex >= 0 && searchResults.length > 0) {
            const lineIdx = searchResults[searchIndex];
            const el = terminalEl?.querySelector(`[data-line="${lineIdx}"]`);
            if (el) el.scrollIntoView({ behavior: "smooth", block: "center" });
        }
    }

    function nextSearch() {
        if (searchResults.length > 0) {
            searchIndex = (searchIndex + 1) % searchResults.length;
            scrollToSearch();
        }
    }

    function prevSearch() {
        if (searchResults.length > 0) {
            searchIndex = (searchIndex - 1 + searchResults.length) % searchResults.length;
            scrollToSearch();
        }
    }

    function handleScroll() {
        if (!terminalEl) return;
        const el = terminalEl;
        autoScroll = el.scrollTop + el.clientHeight >= el.scrollHeight - 40;
    }

    $: if (autoScroll && logs.length > 0) {
        tick().then(() => {
            if (terminalEl) terminalEl.scrollTop = terminalEl.scrollHeight;
        });
    }
</script>

<div class="terminal-wrapper">
    <div class="terminal-toolbar">
        <div class="toolbar-left">
            <button class="tool-btn" class:active={showFilter} on:click={() => showFilter = !showFilter}>
                <span class="mat-icon">filter_list</span>
            </button>
            {#if searchQuery}
                <span class="search-meta">{searchIndex + 1}/{searchResults.length}</span>
            {/if}
            <button class="tool-btn" on:click={() => dispatch("clear")}>Clear</button>
        </div>
        <div class="toolbar-right">
            {#if showFilter}
                <input class="filter-input" type="text" bind:value={filter} placeholder="Filter..." />
            {/if}
            <div class="search-group">
                <input class="search-input" type="text" bind:value={searchQuery} placeholder="Search..." />
                {#if searchResults.length > 0}
                    <button class="tool-btn sm" on:click={prevSearch} title="Previous">▲</button>
                    <button class="tool-btn sm" on:click={nextSearch} title="Next">▼</button>
                {/if}
            </div>
        </div>
    </div>

    <div class="terminal-output" bind:this={terminalEl} on:scroll={handleScroll}>
        {#if filteredLogs.length === 0}
            <span class="placeholder">Server logs will appear here...</span>
        {:else}
            {#each filteredLogs as log, i}
                <div class="log-line" class:highlight={searchResults.includes(i) && searchResults[searchIndex] === i}
                    data-line={i}>
                    <span class="line-text">{@html renderLine(log)}</span>
                </div>
            {/each}
        {/if}
    </div>

    {#if running}
        <div class="terminal-input-row">
            <input class="terminal-input" type="text" bind:value={command}
                on:keydown={handleKey} placeholder="Type a server command..." />
            <button class="send-btn" on:click={sendCommand}>Send</button>
        </div>
    {/if}
</div>

<style>
    .terminal-wrapper { display: flex; flex-direction: column; height: 100%; gap: 6px; }
    .terminal-toolbar { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
    .toolbar-left, .toolbar-right { display: flex; align-items: center; gap: 4px; }
    .tool-btn { padding: 4px 8px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); border-radius: 4px; color: rgba(255,255,255,0.6); cursor: pointer; font-size: 11px; font-family: "Inter", sans-serif; }
    .tool-btn.active { background: rgba(172,196,222,0.12); color: var(--amt-accent, #ACC4DE); }
    .tool-btn.sm { padding: 2px 6px; font-size: 10px; }
    .mat-icon { font-size: 14px; }
    .search-meta { font-size: 10px; color: rgba(255,255,255,0.4); }
    .filter-input, .search-input { padding: 4px 8px; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.1); border-radius: 4px; color: white; font-family: "Inter", sans-serif; font-size: 11px; width: 120px; outline: none; }
    .filter-input:focus, .search-input:focus { border-color: var(--amt-accent, #ACC4DE); }
    .search-group { display: flex; align-items: center; gap: 2px; }
    .terminal-output { flex: 1; background: rgba(0,0,0,0.4); border-radius: 8px; padding: 10px; font-family: "JetBrains Mono", "Fira Code", monospace; font-size: 11px; line-height: 1.5; overflow-y: auto; white-space: pre-wrap; word-break: break-all; color: rgba(255,255,255,0.8); }
    .placeholder { color: rgba(255,255,255,0.2); }
    .log-line { padding: 1px 0; }
    .log-line.highlight { background: rgba(172,196,222,0.08); border-radius: 2px; }
    .line-text { display: inline; }
    .terminal-input-row { display: flex; gap: 6px; }
    .terminal-input { flex: 1; padding: 8px 12px; background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.1); border-radius: 6px; color: white; font-family: "JetBrains Mono", monospace; font-size: 12px; outline: none; }
    .terminal-input:focus { border-color: var(--amt-accent, #ACC4DE); }
    .send-btn { padding: 8px 16px; background: var(--amt-accent, #ACC4DE); color: #0D1117; border: none; border-radius: 6px; font-weight: 600; font-size: 12px; cursor: pointer; font-family: "Inter", sans-serif; }
</style>
