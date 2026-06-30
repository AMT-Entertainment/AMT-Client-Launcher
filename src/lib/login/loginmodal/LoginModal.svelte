<script>
    import ModalButton from "./ModalButton.svelte";
    import {invoke} from "@tauri-apps/api/core";
    import {listen} from "@tauri-apps/api/event";
    import {openUrl} from "@tauri-apps/plugin-opener";
    import {onMount, onDestroy} from "svelte";
    import {t} from "../../i18n/index.js";

    export let options;

    let microsoftCode = null;
    let accounts = options.start.accounts || [];
    let activeIndex = options.start.activeAccountIndex ?? 0;

    $: if (options.start.accounts) {
        accounts = options.start.accounts;
        activeIndex = options.start.activeAccountIndex ?? 0;
    }

    $: activeAccount = accounts[activeIndex];

    function upsertAccount(existing, newAcc) {
        const uuid = newAcc.profile?.id || newAcc.uuid;
        if (!uuid) { existing.push(newAcc); return existing; }
        const idx = existing.findIndex(a => (a.profile?.id || a.uuid) === uuid);
        if (idx >= 0) { existing[idx] = newAcc; return existing; }
        existing.push(newAcc);
        return existing;
    }

    async function handleMicrosoftLoginClick(e) {
        try {
            const account = await invoke("login_microsoft");
            upsertAccount(accounts, account);
            activeIndex = accounts.length - 1;
            options.start.accounts = accounts;
            options.start.activeAccountIndex = activeIndex;
            options.start.account = account;
            options.store();
        } catch (err) {
            alert(
                "Microsoft authentication failed.\n\n" +
                 err + "\n\n" +
                "Please try again or contact support."
            );
            cancelMicrosoft();
        }
    }

    async function addAnotherAccount() {
        await handleMicrosoftLoginClick();
    }

    async function switchAccount(index) {
        activeIndex = index;
        options.start.activeAccountIndex = index;
        options.start.account = accounts[index];
        options.store();
    }

    async function removeAccount(index) {
        accounts.splice(index, 1);
        if (activeIndex >= accounts.length) {
            activeIndex = Math.max(0, accounts.length - 1);
        }
        options.start.accounts = accounts;
        options.start.activeAccountIndex = accounts.length > 0 ? activeIndex : 0;
        options.start.account = accounts[activeIndex] || null;
        options.store();
    }

    function cancelMicrosoft() {
        microsoftCode = null;
    }

    let unlistenCode;
    onMount(async () => {
        unlistenCode = await listen("microsoft_code", (e) => {
            microsoftCode = e.payload;
        });
    });

    onDestroy(() => {
        if (unlistenCode) unlistenCode();
    });
</script>

<div class="login-panel glass-panel glass-panel-lg">
    {#if accounts.length === 0}
        {#if !microsoftCode}
            <div class="login-brand">
                <span class="brand-icon">◈</span>
                <h1 class="brand-title">{$t("app.title")}</h1>
                <p class="brand-desc">{$t("login.subtitle")}</p>
            </div>
            <ModalButton text={$t("login.microsoft")} primary={true} on:click={handleMicrosoftLoginClick} />
        {:else}
            <div class="code-flow">
                <h2 class="code-title">{$t("login.code.title")}</h2>
                <p class="code-desc">{$t("login.code.subtitle")}</p>
                <div class="code-display">
                    <span class="code-label">Your code:</span>
                    <span class="code-value">{microsoftCode}</span>
                </div>
                <p class="code-instruction">Open the link below and enter this code to sign in.</p>
                <div class="code-buttons">
                    <ModalButton text={$t("login.code.link")} primary={true} on:click={() => openUrl("https://microsoft.com/link")} />
                    <ModalButton text={$t("login.code.cancel")} primary={false} on:click={cancelMicrosoft} />
                </div>
            </div>
        {/if}
    {:else}
        <div class="account-select">
            <div class="login-brand">
                <span class="brand-icon">◈</span>
                <h1 class="brand-title">{$t("app.title")}</h1>
                <p class="brand-desc">Select your account to continue</p>
            </div>
            <div class="account-list">
                {#each accounts as acc, i}
                    <button
                        class="account-item"
                        class:active={i === activeIndex}
                        on:click={() => switchAccount(i)}
                    >
                        <object
                            data="https://mc-heads.net/avatar/{acc.profile?.id || acc.uuid}/24"
                            type="image/png"
                            class="avatar"
                            aria-label="avatar"
                        >
                            <div class="avatar-fallback">?</div>
                        </object>
                        <span class="acc-name">{acc.profile?.name || acc.name}</span>
                        {#if i === activeIndex}
                            <span class="acc-check">✓</span>
                        {/if}
                        <button
                            class="acc-remove"
                            on:click|stopPropagation={() => removeAccount(i)}
                            title={$t("account.remove")}
                        >×</button>
                    </button>
                {/each}
            </div>
            <ModalButton text={$t("account.add")} primary={true} on:click={addAnotherAccount} />
        </div>
    {/if}
</div>

<style>
    .login-panel {
        padding: 36px;
        width: 380px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 20px;
    }

    .login-brand {
        text-align: center;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        margin-bottom: 8px;
    }

    .brand-icon {
        font-size: 36px;
        color: var(--amt-accent);
    }

    .brand-title {
        font-size: 24px;
        font-weight: 800;
        color: var(--amt-text);
        margin: 0;
    }

    .brand-desc {
        font-size: 13px;
        color: var(--amt-text-muted);
        margin: 0;
    }

    /* Code flow */
    .code-flow {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 16px;
        text-align: center;
    }

    .code-title {
        font-size: 18px;
        font-weight: 700;
        color: var(--amt-text);
        margin: 0;
    }

    .code-desc {
        font-size: 13px;
        color: var(--amt-text-muted);
        margin: 0;
    }

    .code-display {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        padding: 16px;
        background: rgba(0, 0, 0, 0.3);
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .code-label {
        font-size: 12px;
        color: var(--amt-text-muted);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .code-value {
        font-size: 28px;
        font-weight: 800;
        font-family: var(--amt-font-mono);
        color: var(--amt-accent);
        letter-spacing: 4px;
        user-select: all;
    }

    .code-instruction {
        font-size: 12px;
        color: var(--amt-text-muted);
        margin: 0;
    }

    .code-buttons {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    /* Account select */
    .account-select {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .account-list {
        display: flex;
        flex-direction: column;
        gap: 6px;
        max-height: 200px;
        overflow-y: auto;
    }

    .account-item {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 12px;
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        background: rgba(255, 255, 255, 0.04);
        color: white;
        cursor: pointer;
        transition: all var(--amt-transition-fast);
        font-size: 13px;
        text-align: left;
        width: 100%;
        font-family: var(--amt-font-family);
    }

    .account-item:hover {
        background: rgba(var(--amt-accent-rgb), 0.1);
    }

    .account-item.active {
        background: rgba(var(--amt-accent-rgb), 0.12);
        border-color: var(--amt-accent);
    }

    .avatar {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        object-fit: cover;
    }

    .avatar-fallback {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.1);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 11px;
        color: rgba(255, 255, 255, 0.5);
    }

    .acc-name {
        flex: 1;
        font-weight: 500;
    }

    .acc-check {
        color: var(--amt-success);
        font-weight: 700;
        font-size: 14px;
    }

    .acc-remove {
        background: none;
        border: none;
        color: var(--amt-text-muted);
        cursor: pointer;
        font-size: 16px;
        padding: 2px 4px;
        border-radius: 4px;
        transition: all var(--amt-transition-fast);
    }

    .acc-remove:hover {
        color: var(--amt-danger);
        background: var(--amt-danger-bg);
    }
</style>
