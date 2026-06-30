<script>
    import TitleBar from "../common/TitleBar.svelte";
    import AMTLogo from "../amt/Logo.svelte";
    import StatusBar from "./statusbar/StatusBar.svelte";
    import TextStatus from "./statusbar/TextStatus.svelte";
    import ProgressStatus from "./statusbar/ProgressStatus.svelte";
    import ButtonClose from "../common/ButtonClose.svelte";
    import {locale, t} from "../i18n/index.js";

    export let options;
    export let account;
    export let running;
    export let progressState;

    let dropdownOpen = false;
    let accounts = [];

    $: accounts = options.start?.accounts || [];
    $: activeIndex = options.start?.activeAccountIndex ?? 0;

    async function switchAccount(index) {
        options.start.activeAccountIndex = index;
        options.start.account = accounts[index];
        await options.store();
        dropdownOpen = false;
    }
</script>

<TitleBar>
    <AMTLogo />
    <StatusBar>
        {#if !running}
            <TextStatus text={$t("welcome.back", { name: account?.profile?.name || account?.name || "Unknown" })} />
        {:else}
            <ProgressStatus {...progressState} />
        {/if}
    </StatusBar>
    <div class="account-area">
        <button class="header-avatar-btn" on:click={() => dropdownOpen = !dropdownOpen}>
            <slot name="accountSlot" />
        </button>
        {#if dropdownOpen && accounts.length > 1}
            <div class="account-dropdown" on:mouseleave={() => dropdownOpen = false}>
                <div class="dropdown-title">{$t("account.switch")}</div>
                {#each accounts as acc, i}
                    <button
                        class="dropdown-item"
                        class:active={i === activeIndex}
                        on:click={() => switchAccount(i)}
                    >
                        <object
                            data="https://minotar.net/avatar/{acc.profile?.id || acc.uuid}/20"
                            type="image/png"
                            class="drop-avatar"
                            aria-label="avatar"
                        >
                            <div class="drop-avatar-fallback">?</div>
                        </object>
                        <span>{acc.profile?.name || acc.name}</span>
                        {#if i === activeIndex}
                            <span class="drop-check">✓</span>
                        {/if}
                    </button>
                {/each}
            </div>
        {/if}
    </div>
    <ButtonClose />
</TitleBar>

<style>
    .account-area {
        position: relative;
    }

    .header-avatar-btn {
        background: none;
        border: none;
        cursor: pointer;
        padding: 2px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all var(--amt-transition-fast);
    }

    .header-avatar-btn:hover {
        background: rgba(255, 255, 255, 0.06);
    }

    .account-dropdown {
        position: absolute;
        top: 100%;
        right: 0;
        margin-top: 4px;
        min-width: 220px;
        background: rgba(13, 17, 23, 0.95);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 8px;
        z-index: 200;
        backdrop-filter: blur(16px);
    }

    .dropdown-title {
        color: rgba(255, 255, 255, 0.5);
        font-size: 11px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        padding: 4px 8px 8px;
    }

    .dropdown-item {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        padding: 8px;
        border-radius: 6px;
        border: none;
        background: transparent;
        color: white;
        cursor: pointer;
        font-size: 13px;
        text-align: left;
        transition: all 0.15s;
    }

    .dropdown-item:hover {
        background: rgba(172, 196, 222, 0.1);
    }

    .dropdown-item.active {
        background: rgba(172, 196, 222, 0.15);
    }

    .drop-avatar {
        width: 20px;
        height: 20px;
        border-radius: 50%;
        object-fit: cover;
    }

    .drop-avatar-fallback {
        width: 20px;
        height: 20px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.1);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 10px;
    }

    .drop-check {
        margin-left: auto;
        color: #2EA043;
        font-weight: 700;
    }
</style>
