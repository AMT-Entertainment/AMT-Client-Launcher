<script>
    import { fly } from "svelte/transition";
    import { onMount, onDestroy } from "svelte";

    export let text;

    let element;
    let shown = false;
    let parentEl;

    onMount(() => {
        parentEl = element.parentNode;
        parentEl.addEventListener("mouseenter", onEnter);
        parentEl.addEventListener("mouseleave", onLeave);
    });

    onDestroy(() => {
        if (parentEl) {
            parentEl.removeEventListener("mouseenter", onEnter);
            parentEl.removeEventListener("mouseleave", onLeave);
        }
    });

    function onEnter() { shown = true; }
    function onLeave() { shown = false; }
</script>

<div bind:this={element}>
    {#if shown}
        <div transition:fly={{ y: -10, duration: 200 }} class="tooltip">
            {text}
        </div>
    {/if}
</div>

<style>
    .tooltip {
        background-color: #4677FF;
        color: white;
        padding: 7px 10px;
        border-radius: 15px;
        font-size: 14px;
        font-weight: 600;
        position: absolute;
        white-space: nowrap;
        left: 50%;
        top: 0;
        transform: translate(-50%, -45px);
        z-index: 1000;
    }

    .tooltip::after {
        content: "";
        display: block;
        height: 10px;
        width: 10px;
        background-color: #4677FF;
        position: absolute;
        left: 50%;
        transform: translateX(-50%) rotate(45deg);
    }
</style>
