<script>
    import {onMount, onDestroy} from "svelte";
    import Description from "../../settings/Description.svelte";
    import ButtonSetting from "../../settings/ButtonSetting.svelte";
    import TextSetting from "../../settings/TextSetting.svelte";
    import {openUrl} from "@tauri-apps/plugin-opener";
    import {invoke} from "@tauri-apps/api/core";
    import {listen} from "@tauri-apps/api/event";
    import {t} from "../../i18n/index.js";

    export let client = null;
    export let options;

    let accounts = [];
    let activeIndex = 0;
    let microsoftCode = null;

    $: accounts = options.start?.accounts || [];
    $: activeIndex = options.start?.activeAccountIndex ?? 0;
    $: activeAccount = accounts[activeIndex];

    async function logout() {
        options.start.accounts = [];
        options.start.activeAccountIndex = 0;
        options.start.account = null;
        await options.store();
    }

    function upsertAccount(existing, newAcc) {
        const uuid = newAcc.profile?.id || newAcc.uuid;
        if (!uuid) { return [...existing, newAcc]; }
        const idx = existing.findIndex(a => (a.profile?.id || a.uuid) === uuid);
        if (idx >= 0) {
            const updated = [...existing];
            updated[idx] = newAcc;
            return updated;
        }
        return [...existing, newAcc];
    }

    async function addAccount() {
        try {
            const account = await invoke("login_microsoft");
            upsertAccount(accounts, account);
            options.start.accounts = accounts;
            options.start.activeAccountIndex = accounts.length - 1;
            options.start.account = account;
            await options.store();
        } catch (err) {
            alert("Microsoft authentication failed.\n\n" + err);
        }
    }

    async function switchAccount(index) {
        options.start.activeAccountIndex = index;
        options.start.account = accounts[index];
        await options.store();
    }

    async function removeAccount(index) {
        const updated = options.start.accounts.filter((_, i) => i !== index);
        const newIdx = activeIndex >= updated.length ? Math.max(0, updated.length - 1) : activeIndex;
        options.start.accounts = updated;
        options.start.activeAccountIndex = newIdx;
        options.start.account = updated[newIdx] || null;
        await options.store();
    }

    async function linkGithub() {
        try {
            const url = await invoke("github_auth_url", { amtOptions: options.amt_options });
            await openUrl(url);
        } catch (err) {
            alert("GitHub OAuth error:\n\n" + err + "\n\nPlease configure your GitHub OAuth App credentials first.");
        }
    }

    let unlistenMS;
    onMount(() => {
        const unsub = listen("microsoft_code", (e) => {
            microsoftCode = e.payload;
        });
        unsub.then(fn => unlistenMS = fn);
    });
    onDestroy(() => {
        if (unlistenMS) unlistenMS();
    });
</script>

<div class="account-settings">
    <!-- Microsoft Accounts -->
    <div class="settings-section">
        <h4 class="section-label">Microsoft Accounts</h4>
        {#if accounts.length === 0}
            <Description description={$t("account.noAccounts")} />
            <ButtonSetting text={$t("account.add")} on:click={addAccount} color="#ACC4DE" />
        {:else}
            <Description description={$t("account.signedInAs", {name: activeAccount?.profile?.name || activeAccount?.name})} />
            <div class="account-list">
                {#each accounts as acc, i}
                    <div class="account-row" class:active={i === activeIndex}>
                        <div class="acc-info">
                            <object
                                data="https://mc-heads.net/avatar/{acc.profile?.id || acc.uuid}/24"
                                type="image/png"
                                class="acc-avatar"
                                aria-label="avatar"
                            >
                                <div class="acc-avatar-fallback">?</div>
                            </object>
                            <span>{acc.profile?.name || acc.name}</span>
                        </div>
                        <div class="acc-actions">
                            {#if i !== activeIndex}
                                <button class="acc-btn switch" on:click={() => switchAccount(i)}>{$t("account.switch")}</button>
                            {:else}
                                <span class="acc-active">✓ Active</span>
                            {/if}
                            <button class="acc-btn remove" on:click={() => removeAccount(i)}>Remove</button>
                        </div>
                    </div>
                {/each}
            </div>
            <div class="account-buttons">
                <ButtonSetting text={$t("account.add")} on:click={addAccount} color="#ACC4DE" />
                <ButtonSetting text={$t("settings.signout")} on:click={logout} color="#E5534B" />
            </div>
        {/if}
    </div>

    <div class="amt-divider"></div>

    <!-- GitHub Integration -->
    <div class="settings-section">
        <h4 class="section-label">GitHub Integration</h4>
        <Description description="Configure GitHub OAuth to enable cosmetics publishing and social features." />

        <TextSetting
            title="Client ID"
            placeholder="Your GitHub OAuth App client ID"
            bind:value={options.amt_options.github_client_id}
        />

        <TextSetting
            title="Client Secret"
            placeholder="Your GitHub OAuth App client secret"
            bind:value={options.amt_options.github_client_secret}
        />

        <TextSetting
            title="GitHub Token (Personal Access Token)"
            placeholder="ghp_xxxxxxxxxxxxxxxxxxxx"
            bind:value={options.amt_options.githubToken}
        />

        {#if options.amt_options.github_client_id}
            <ButtonSetting
                text="Link GitHub Account"
                on:click={linkGithub}
                color="#ACC4DE"
            />
        {:else}
            <p class="hint">Enter your GitHub OAuth App credentials above to enable linking.</p>
            <p class="hint hint-small">
                Create an OAuth App at 
                <a href="https://github.com/settings/developers" target="_blank" rel="noopener" style="color: #ACC4DE;">GitHub Settings → Developers → OAuth Apps</a>
                <br/>Set the callback URL to: <code style="color: var(--amt-accent);">http://127.0.0.1:42069/callback</code>
            </p>
        {/if}
    </div>
</div>

<style>
    .account-settings {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .settings-section {
        display: flex;
        flex-direction: column;
        gap: 10px;
    }

    .section-label {
        color: var(--amt-text-secondary);
        font-size: 11px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin: 0;
    }

    .account-list {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .account-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 10px 12px;
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        background: rgba(255, 255, 255, 0.03);
    }

    .account-row.active {
        border-color: rgba(172, 196, 222, 0.3);
        background: rgba(172, 196, 222, 0.08);
    }

    .acc-info {
        display: flex;
        align-items: center;
        gap: 10px;
        color: white;
        font-size: 13px;
        font-weight: 500;
    }

    .acc-avatar {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        object-fit: cover;
    }

    .acc-avatar-fallback {
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

    .acc-actions {
        display: flex;
        gap: 6px;
        align-items: center;
    }

    .acc-btn {
        background: none;
        border: none;
        cursor: pointer;
        font-size: 11px;
        padding: 4px 8px;
        border-radius: 4px;
        transition: all 0.15s;
        font-family: inherit;
    }

    .acc-btn.switch {
        color: #ACC4DE;
    }

    .acc-btn.switch:hover {
        background: rgba(172, 196, 222, 0.15);
    }

    .acc-btn.remove {
        color: #E5534B;
    }

    .acc-btn.remove:hover {
        background: rgba(229, 83, 75, 0.15);
    }

    .acc-active {
        font-size: 11px;
        color: #3FB950;
        font-weight: 500;
    }

    .account-buttons {
        display: flex;
        gap: 8px;
    }

    .hint {
        color: var(--amt-text-muted);
        font-size: 12px;
        margin: 0;
    }

    .hint-small {
        font-size: 11px;
        line-height: 1.6;
    }

    .hint code {
        font-family: var(--amt-font-mono);
        font-size: 11px;
    }
</style>
