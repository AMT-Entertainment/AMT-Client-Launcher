<div class="wrapper">
    <slot />
</div>

<style>
    .wrapper {
        background-color: rgba(0, 0, 0, 0.26);
        border-radius: 12px;
        flex: 1;
        display: flex;
        flex-direction: column;
        padding: 25px;
        gap: 24px;
        overflow-y: auto;
    }
</style>
