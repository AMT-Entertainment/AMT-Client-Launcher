<script>
    import { createEventDispatcher } from "svelte";

    export let title;

    const dispatch = createEventDispatcher();

    function handleHideClick(e) {
        dispatch("hideSettings");
    }
</script>

<div class="amt-modal-overlay" role="dialog" aria-modal="true" on:click={handleHideClick} on:keydown={(e) => e.key === 'Escape' && handleHideClick()}>
    <div class="amt-modal" style="max-width: 540px; max-height: 85vh;" on:click|stopPropagation>
        <div class="amt-modal-header">
            <h2>{title}</h2>
            <button class="amt-modal-close" on:click={handleHideClick}>✕</button>
        </div>
        <div class="amt-modal-body">
            <slot name="tabs" />
            <div class="settings-content">
                <slot />
            </div>
        </div>
    </div>
</div>

<style>
    .settings-content {
        display: flex;
        flex-direction: column;
        gap: 20px;
        margin-top: 16px;
    }
</style>