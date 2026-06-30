<script>
    import { onMount } from "svelte";
    import News from "./News.svelte";
    import {invoke} from "@tauri-apps/api/core";

    export let client;

    let news = [];
    let loading = true;

    onMount(() => {
        invoke("fetch_blog_posts", {
            client,
            page: 1
        }).then((onlineNews) => {
            news = onlineNews.items || [];
            loading = false;
        }).catch((e) => {
            console.error(e);
            loading = false;
        });
    });
</script>

<div class="news-area">
    {#if loading}
        <div class="news-loading">
            <div class="amt-skeleton" style="height: 80px; width: 100%;"></div>
        </div>
    {:else if news.length === 0}
        <div class="news-empty">
            <p>No news yet</p>
        </div>
    {:else}
        <div class="news-wrapper">
            {#each news as n}
                <News {...n} />
            {/each}
        </div>
    {/if}
</div>

<style>
    .news-area {
        display: flex;
        flex-direction: column;
        flex: 1;
    }

    .news-wrapper {
        display: flex;
        flex-direction: column;
        gap: 12px;
        overflow-y: auto;
    }

    .news-loading {
        padding: 8px 0;
    }

    .news-empty {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 32px 16px;
        text-align: center;
        color: var(--amt-text-muted);
        font-size: 13px;
    }

    .news-empty p {
        margin: 4px 0;
    }
</style>
