<script>
    import {onMount} from "svelte";
    import {createEventDispatcher} from "svelte";
    import {invoke} from "@tauri-apps/api/core";

    export let backendUrl = "https://amt-entertainment.github.io/AMT-Client-Backend";
    export let options = null;

    const dispatch = createEventDispatcher();

    let capes = [];
    let filter = "newest";
    let loading = true;
    let selectedCape = null;

    $: GALLERY_URL = `${backendUrl}/data/gallery.json`;

    async function loadGallery() {
        loading = true;
        try {
            const res = await fetch(GALLERY_URL);
            if (res.ok) {
                const data = await res.json();
                capes = data.capes || [];
            }
        } catch (e) {
            console.warn("Gallery not available:", e);
            capes = [];
        }
        loading = false;
    }

    $: sortedCapes = (() => {
        let c = [...capes];
        switch (filter) {
            case "newest":
                c.sort((a, b) => new Date(b.uploadDate || 0) - new Date(a.uploadDate || 0));
                break;
            case "top":
                c.sort((a, b) => (b.votes || 0) - (a.votes || 0));
                break;
            case "trending":
                c.sort((a, b) => (b.trending || 0) - (a.trending || 0));
                break;
        }
        return c;
    })();

    function getCapeImage(cape) {
        return cape.thumbnail || `${backendUrl}/assets/capes/${cape.id}.png`;
    }

    function equipCape(cape) {
        if (!options) return;
        options.amt_options.equippedCape = cape.id;
        options.store();
        dispatch("capeEquipped", { id: cape.id, title: cape.title });
    }

    function unequipCape() {
        if (!options) return;
        options.amt_options.equippedCape = "";
        options.store();
        dispatch("capeEquipped", { id: "", title: "" });
    }

    $: isEquipped = options?.amt_options?.equippedCape === selectedCape?.id;

    onMount(loadGallery);
</script>

<div class="gallery">
    <div class="gallery-header">
        <h3>Community Gallery</h3>
        <div class="filter-row">
            {#each ["newest", "top", "trending"] as f}
                <button class="filter-btn" class:active={filter === f} on:click={() => filter = f}>
                    {f === "newest" ? "Newest" : f === "top" ? "Top Voted" : "Trending"}
                </button>
            {/each}
        </div>
    </div>

    {#if loading}
        <div class="loading"><p>Loading gallery...</p></div>
    {:else if sortedCapes.length === 0}
        <div class="empty">
            <p>No capes in the gallery yet.</p>
            <p class="sub">Design one in the Cape Designer or upload a PNG!</p>
        </div>
    {:else}
        <div class="cape-count">{sortedCapes.length} cape{sortedCapes.length !== 1 ? 's' : ''}</div>
        <div class="cape-grid">
            {#each sortedCapes as cape}
                <div class="cape-card" on:click={() => selectedCape = selectedCape?.id === cape.id ? null : cape}>
                    <div class="cape-thumb">
                        <img
                            src={getCapeImage(cape)}
                            alt={cape.title}
                            on:error={(e) => e.target.style.display = 'none'}
                        />
                    </div>
                    <div class="cape-info">
                        <span class="cape-title">{cape.title || "Untitled"}</span>
                        <span class="cape-author">by {cape.author || "Unknown"}</span>
                    </div>
                    <div class="cape-footer">
                        <span class="cape-votes">
                            <span class="vote-icon">♥</span> {cape.votes || 0}
                        </span>
                        <span class="cape-date">{cape.uploadDate ? new Date(cape.uploadDate).toLocaleDateString() : ''}</span>
                    </div>

                    {#if selectedCape?.id === cape.id}
                        <div class="cape-detail">
                            <div class="cape-preview-full">
                                <img src={getCapeImage(cape)} alt={cape.title} />
                            </div>
                            <div class="cape-full-info">
                                <span><b>Title:</b> {cape.title || "Untitled"}</span>
                                <span><b>Author:</b> {cape.author || "Unknown"}</span>
                                <span><b>Votes:</b> ♥ {cape.votes || 0}</span>
                                <span><b>Uploaded:</b> {cape.uploadDate ? new Date(cape.uploadDate).toLocaleDateString() : 'Unknown'}</span>
                                <div class="equip-actions">
                                    {#if isEquipped}
                                        <button class="equip-btn unequip" on:click|stopPropagation={unequipCape}>
                                            Unequip
                                        </button>
                                    {:else}
                                        <button class="equip-btn" on:click|stopPropagation={() => equipCape(cape)}>
                                            Equip Cape
                                        </button>
                                    {/if}
                                </div>
                            </div>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .gallery {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .gallery-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .gallery-header h3 {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        color: white;
    }

    .filter-row {
        display: flex;
        gap: 4px;
        background: rgba(0,0,0,0.3);
        padding: 3px;
        border-radius: 8px;
    }

    .filter-btn {
        padding: 6px 14px;
        border: none;
        background: none;
        border-radius: 6px;
        color: rgba(255,255,255,0.5);
        font-size: 12px;
        cursor: pointer;
    }

    .filter-btn:hover {
        color: white;
        background: rgba(255,255,255,0.08);
    }

    .filter-btn.active {
        color: #ACC4DE;
        background: rgba(172,196,222,0.15);
    }

    .loading, .empty {
        text-align: center;
        padding: 60px 20px;
        color: rgba(255,255,255,0.4);
    }

    .sub {
        font-size: 12px;
        color: rgba(255,255,255,0.25);
    }

    .cape-count {
        font-size: 11px;
        color: rgba(255,255,255,0.3);
    }

    .cape-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 12px;
    }

    .cape-card {
        background: rgba(255,255,255,0.03);
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 10px;
        overflow: hidden;
        cursor: pointer;
        transition: all 0.15s;
    }

    .cape-card:hover {
        border-color: rgba(172,196,222,0.2);
        transform: translateY(-2px);
    }

    .cape-thumb {
        height: 90px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(0,0,0,0.2);
        border-bottom: 1px solid rgba(255,255,255,0.04);
    }

    .cape-thumb img {
        max-width: 90%;
        max-height: 90%;
        image-rendering: pixelated;
    }

    .cape-info {
        padding: 10px 12px 4px;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .cape-title {
        font-size: 13px;
        font-weight: 600;
        color: white;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .cape-author {
        font-size: 11px;
        color: rgba(255,255,255,0.35);
    }

    .cape-footer {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 4px 12px 10px;
        font-size: 11px;
        color: rgba(255,255,255,0.3);
    }

    .cape-votes {
        color: #ff6b6b;
    }

    .vote-icon {
        font-size: 10px;
    }

    .cape-date {
        font-size: 10px;
    }

    .cape-detail {
        border-top: 1px solid rgba(255,255,255,0.06);
        padding: 12px;
        display: flex;
        gap: 12px;
        background: rgba(0,0,0,0.15);
    }

    .cape-preview-full {
        width: 80px;
        height: 40px;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(0,0,0,0.2);
        border-radius: 4px;
    }

    .cape-preview-full img {
        max-width: 100%;
        max-height: 100%;
        image-rendering: pixelated;
    }

    .cape-full-info {
        display: flex;
        flex-direction: column;
        gap: 3px;
        font-size: 11px;
        color: rgba(255,255,255,0.5);
    }

    .cape-full-info b {
        color: rgba(255,255,255,0.7);
    }

    .equip-actions {
        margin-top: 8px;
    }

    .equip-btn {
        padding: 6px 16px;
        border: 1px solid rgba(172,196,222,0.3);
        border-radius: 6px;
        background: rgba(172,196,222,0.1);
        color: #ACC4DE;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.15s;
    }

    .equip-btn:hover {
        background: rgba(172,196,222,0.2);
    }

    .equip-btn.unequip {
        border-color: rgba(255,100,100,0.3);
        background: rgba(255,100,100,0.1);
        color: #ff6b6b;
    }

    .equip-btn.unequip:hover {
        background: rgba(255,100,100,0.2);
    }
</style>
