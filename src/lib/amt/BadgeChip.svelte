<script>
    export let badge = "AMT";
    export let accent = "#ACC4DE";
    export let size = "small";
</script>

<span class="badge" style="--badge-accent: {accent}" class:badge-large={size === "large"}>
    {badge}
</span>

<style>
    .badge {
        display: inline-flex;
        align-items: center;
        padding: 1px 6px;
        border-radius: 3px;
        font-family: var(--amt-font-mono);
        font-size: 10px;
        font-weight: 700;
        letter-spacing: 0.5px;
        text-transform: uppercase;
        background: rgba(172, 196, 222, 0.15);
        color: var(--badge-accent, var(--amt-accent));
        border: 1px solid rgba(172, 196, 222, 0.25);
    }

    .badge-large {
        font-size: 13px;
        padding: 3px 10px;
        border-radius: 4px;
    }
</style>
