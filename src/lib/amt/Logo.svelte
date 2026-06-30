<script>
    export let size = "small";
    $: logoSize = size === "large" ? "64px" : "40px";
    $: fontSize = size === "large" ? "22px" : "16px";
</script>

<div class="amt-logo" style="--size: {logoSize}; --font-size: {fontSize}">
    <span class="bracket">[</span>
    <span class="text">AMT</span>
    <span class="bracket">]</span>
</div>

<style>
    .amt-logo {
        display: flex;
        align-items: center;
        gap: 2px;
        height: var(--size);
        font-family: var(--amt-font-mono);
    }

    .bracket {
        color: var(--amt-accent);
        font-size: var(--font-size);
        font-weight: 700;
        opacity: 0.8;
    }

    .text {
        color: white;
        font-size: var(--font-size);
        font-weight: 700;
        letter-spacing: 1px;
    }
</style>
