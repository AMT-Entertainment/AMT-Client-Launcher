<script>
    import { createEventDispatcher } from "svelte";
    import BadgeChip from "../amt/BadgeChip.svelte";
    import { t } from "../i18n/index.js";

    export let username;
    export let uuid;
    export let accountType;
    export let badge = "AMT";
    export let accent = "#ACC4DE";

    const dispatch = createEventDispatcher();
</script>

<div class="account glass-panel">
    <object data="https://mc-heads.net/avatar/{uuid}/35" type="image/png" class="avatar" aria-label="avatar">
        <div class="avatar-fallback">?</div>
    </object>
    <div class="details">
        <div class="name-row">
            <span class="username">{username}</span>
            <BadgeChip {badge} {accent} />
        </div>
        <div class="type">{$t("account.type")}</div>
    </div>
    <button class="button-settings" type="button" on:click={() => dispatch("showSettings")}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M8 10a2 2 0 100-4 2 2 0 000 4z" stroke="currentColor" stroke-width="1.5"/>
            <path d="M12.5 8c0-.3.2-.6.4-.8l.3-.3c.3-.3.4-.7.3-1.1l-.3-.6c-.2-.4-.5-.6-.9-.5l-.4.1c-.3.1-.6 0-.8-.2-.2-.2-.3-.5-.2-.8l.1-.4c.1-.4-.1-.7-.5-.9l-.6-.3c-.4-.2-.8-.1-1.1.2l-.3.3c-.2.2-.5.4-.8.4s-.6-.2-.8-.4l-.3-.3c-.3-.3-.7-.4-1.1-.2l-.6.3c-.4.2-.6.5-.5.9l.1.4c.1.3 0 .6-.2.8-.2.2-.5.3-.8.2l-.4-.1c-.4-.1-.7.1-.9.5l-.3.6c-.1.4 0 .8.3 1.1l.3.3c.2.2.4.5.4.8s-.2.6-.4.8l-.3.3c-.3.3-.4.7-.3 1.1l.3.6c.2.4.5.6.9.5l.4-.1c.3-.1.6 0 .8.2.2.2.3.5.2.8l-.1.4c-.1.4.1.7.5.9l.6.3c.4.2.8.1 1.1-.2l.3-.3c.2-.2.5-.4.8-.4s.6.2.8.4l.3.3c.3.3.7.4 1.1.2l.6-.3c.4-.2.6-.5.5-.9l-.1-.4c-.1-.3 0-.6.2-.8.2-.2.5-.3.8-.2l.4.1c.4.1.7-.1.9-.5l.3-.6c.1-.4 0-.8-.3-1.1l-.3-.3c-.2-.2-.4-.5-.4-.8z" stroke="currentColor" stroke-width="1.5"/>
        </svg>
    </button>
</div>

<style>
    .account {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 12px;
        min-width: 220px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
    }

    .avatar {
        width: 35px;
        height: 35px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.1);
        object-fit: cover;
    }

    .avatar-fallback {
        width: 35px;
        height: 35px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.1);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 14px;
        color: rgba(255, 255, 255, 0.5);
    }

    .details {
        flex: 1;
        min-width: 0;
    }

    .name-row {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .username {
        color: white;
        font-size: 13px;
        font-weight: 600;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .type {
        color: rgba(255, 255, 255, 0.4);
        font-size: 10px;
    }

    .button-settings {
        background: none;
        border: none;
        color: rgba(255, 255, 255, 0.4);
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        transition: all 0.2s;
        display: flex;
        align-items: center;
    }

    .button-settings:hover {
        color: white;
        background: rgba(255, 255, 255, 0.1);
    }
</style>
