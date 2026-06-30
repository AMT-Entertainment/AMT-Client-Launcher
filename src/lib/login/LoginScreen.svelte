<script>
    import ButtonClose from "../common/ButtonClose.svelte";
    import Logo from "../common/Logo.svelte";
    import TitleBar from "../common/TitleBar.svelte";
    import VerticalFlexWrapper from "../common/VerticalFlexWrapper.svelte";
    import Facts from "./Facts.svelte";
    import LoginModal from "./loginmodal/LoginModal.svelte";
    import {locale, t} from "../i18n/index.js";

    export let options;

    let step = options.start.accounts?.length > 0 ? "login" : "language";

    function selectLanguage(lang) {
        if (!options.amt_options) { options.amt_options = {}; }
        options.amt_options.language = lang;
        locale.set(lang);
        options.store();
        step = "login";
    }

    $: if (options.amt_options?.language) {
        locale.set(options.amt_options.language);
    }

    import { onMount } from "svelte";
    onMount(() => {
        if (options.amt_options?.language) {
            locale.set(options.amt_options.language);
        }
    });
</script>

<VerticalFlexWrapper blur={false}>
    <TitleBar>
        <Logo />
        <div class="title-bar-spacer"></div>
        <ButtonClose />
    </TitleBar>

    <div class="login-layout">
        <div class="login-main">
            {#if step === "language"}
                <div class="welcome-panel glass-panel glass-panel-lg">
                    <div class="welcome-brand">
                        <span class="welcome-icon">◈</span>
                        <h1 class="welcome-title">AMT Client</h1>
                        <p class="welcome-subtitle">Performance-focused Minecraft Launcher</p>
                    </div>

                    <div class="language-picker">
                        <h3 class="picker-label">{$t("setupwizard.title")}</h3>
                        <p class="picker-desc">{$t("setupwizard.language")}</p>
                        <div class="lang-grid">
                            <button class="lang-btn" on:click={() => selectLanguage("de")}>
                                <span class="lang-flag">🇩🇪</span>
                                <span class="lang-name">Deutsch</span>
                                <span class="lang-badge">Hauptsprache</span>
                            </button>
                            <button class="lang-btn active" on:click={() => selectLanguage("en")}>
                                <span class="lang-flag">🇬🇧</span>
                                <span class="lang-name">English</span>
                            </button>
                            <button class="lang-btn" on:click={() => selectLanguage("pl")}>
                                <span class="lang-flag">🇵🇱</span>
                                <span class="lang-name">Polski</span>
                            </button>
                        </div>
                    </div>
                </div>
            {:else}
                <LoginModal bind:options />
            {/if}
        </div>

        <div class="login-sidebar">
            <Facts />
        </div>
    </div>
</VerticalFlexWrapper>

<style>
    .title-bar-spacer {
        flex: 1;
    }

    .login-layout {
        display: flex;
        gap: 32px;
        flex: 1;
        padding: 0 50px 20px;
        align-items: center;
        justify-content: center;
    }

    .login-main {
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .login-sidebar {
        width: 280px;
        flex-shrink: 0;
    }

    .welcome-panel {
        padding: 40px 36px;
        width: 380px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 32px;
    }

    .welcome-brand {
        text-align: center;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
    }

    .welcome-icon {
        font-size: 40px;
        color: var(--amt-accent);
        margin-bottom: 8px;
    }

    .welcome-title {
        font-size: 28px;
        font-weight: 800;
        color: var(--amt-text);
        margin: 0;
        letter-spacing: -0.5px;
    }

    .welcome-subtitle {
        font-size: 13px;
        color: var(--amt-text-muted);
        margin: 0;
    }

    .language-picker {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .picker-label {
        color: var(--amt-text-secondary);
        font-size: 13px;
        font-weight: 600;
        margin: 0;
        text-align: center;
    }

    .picker-desc {
        color: var(--amt-text-muted);
        font-size: 12px;
        margin: 0;
        text-align: center;
    }

    .lang-grid {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .lang-btn {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 14px 16px;
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.08);
        background: rgba(255, 255, 255, 0.04);
        color: white;
        cursor: pointer;
        transition: all var(--amt-transition-normal);
        font-size: 14px;
        text-align: left;
        font-family: var(--amt-font-family);
    }

    .lang-btn:hover {
        background: rgba(var(--amt-accent-rgb), 0.1);
        border-color: rgba(var(--amt-accent-rgb), 0.25);
    }

    .lang-btn.active {
        background: rgba(var(--amt-accent-rgb), 0.12);
        border-color: var(--amt-accent);
    }

    .lang-flag {
        font-size: 22px;
    }

    .lang-name {
        flex: 1;
        font-weight: 600;
    }

    .lang-badge {
        font-size: 10px;
        padding: 2px 8px;
        border-radius: 4px;
        background: rgba(var(--amt-accent-rgb), 0.2);
        color: var(--amt-accent);
        font-weight: 600;
    }
</style>
