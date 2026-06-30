<script>
    import VirtualList from "./VirtualList.svelte";
    import { createEventDispatcher } from "svelte";
    import ToggleSetting from "../../settings/ToggleSetting.svelte";
    import ButtonSetting from "../../settings/ButtonSetting.svelte";
    import { t } from "../../i18n/index.js";
    import LogMessage from "./LogMessage.svelte";

    export let messages;

    let autoScroll = true;

    const dispatch = createEventDispatcher();

    async function handleCopyLog() {
        const log = messages.join("");
        try {
            await navigator.clipboard.writeText(log);
            alert("Log copied to clipboard!");
        } catch {
            alert("Could not copy log to clipboard.");
        }
    }
</script>

<div class="amt-modal-overlay" role="dialog" aria-modal="true" on:click={() => dispatch("hideClientLog")} on:keydown={(e) => e.key === 'Escape' && dispatch("hideClientLog")}>
    <div class="amt-modal" style="max-width: 800px; max-height: 80vh;" on:click|stopPropagation>
        <div class="amt-modal-header">
            <h2>{$t("clientlog.title")}</h2>
            <button class="amt-modal-close" on:click={() => dispatch("hideClientLog")}>✕</button>
        </div>
        <div class="amt-modal-body" style="display: flex; flex-direction: column; gap: 16px;">
            <div class="output">
                <VirtualList items={messages} let:item {autoScroll}>
                    <LogMessage text={item} />
                </VirtualList>
            </div>
            <div class="settings">
                <ButtonSetting text="Copy log" color="#4677FF" on:click={handleCopyLog}></ButtonSetting>
                <ToggleSetting title="Auto scroll" disabled={false} bind:value={autoScroll} />
            </div>
        </div>
    </div>
</div>

<style>
    .output {
        flex: 1;
        min-height: 300px;
        overflow: hidden;
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 8px;
        background: rgba(0,0,0,0.2);
    }

    .settings {
        display: flex;
        gap: 16px;
        align-items: center;
    }
</style>