<script>
    import {invoke} from "@tauri-apps/api/core";
    import {check} from "@tauri-apps/plugin-updater";
    import {exit, relaunch} from "@tauri-apps/plugin-process";
    import {ask} from "@tauri-apps/plugin-dialog";
    import {onMount} from "svelte";
    import MainScreen from "./main/MainScreen.svelte";
    import LoginScreen from "./login/LoginScreen.svelte";
    import LoadingScreen from "./main/LoadingScreen.svelte";
    import ErrorScreen from "./main/ErrorScreen.svelte";
    import NonSecureConnectionScreen from "./main/NonSecureConnectionScreen.svelte";

    let loading = true;
    let error = null;
    let options = null;
    let client = null;

    let allowNonSecure = false;

    let bgUrl = "";
    let bgCredit = "";
    let bgCreditUrl = "";

    const FALLBACK_BG = "/img/fallback-bg.png";

    async function fetchBackground() {
        try {
            const backendUrl = options?.amt_options?.backendUrl || "https://amt-entertainment.github.io/AMT-Client-Backend";
            const creditUrls = [
                "https://amt-entertainment.github.io/AMT-Client-Backend/data/backgrounds.json",
                `${backendUrl}/data/backgrounds.json`
            ];
            for (const url of creditUrls) {
                try {
                    const res = await fetch(url);
                    if (res.ok) {
                        const data = await res.json();
                        const wallpapers = data.wallpapers || [];
                        if (wallpapers.length > 0) {
                            const pick = wallpapers[Math.floor(Math.random() * wallpapers.length)];
                            bgUrl = pick.url;
                            bgCredit = pick.credit || "";
                            bgCreditUrl = pick.creditUrl || "";
                            break;
                        } else if (data.default) {
                            bgUrl = data.default;
                            break;
                        }
                    }
                } catch { /* try next URL */ }
            }
        } catch { /* use fallback */ }
        if (!bgUrl) bgUrl = FALLBACK_BG;
        document.documentElement.style.setProperty('--amt-bg-image', `url(${bgUrl})`);
    }

    async function handleUpdate() {
        try {
            const result = await check();

            if (result?.available) {
                const shouldUpdate = await ask(
                    "A launcher update is available. Would you like to install it now?",
                    "AMT Client"
                );

                if (shouldUpdate) {
                    await result.downloadAndInstall();
                    await relaunch();
                }
            }
        } catch (error) {
            console.error("Update process failed:", error);
        }
    }

    function hexToRgb(hex) {
        const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return result ? `${parseInt(result[1], 16)}, ${parseInt(result[2], 16)}, ${parseInt(result[3], 16)}` : "172, 196, 222";
    }

    function applyAccentColor(color) {
        if (!color) return;
        document.documentElement.style.setProperty('--amt-accent', color);
        document.documentElement.style.setProperty('--amt-accent-rgb', hexToRgb(color));
    }

    async function setupOptions() {
        try {
            options = {
                store: async function () {
                    console.debug("Storing options...", options);
                    try {
                        await invoke("store_options", {options});
                    } catch (error) {
                        console.error("Failed to store options:", error);
                        throw error;
                    }
                },
                ...await invoke("get_options")
            };
            console.debug("Options loaded:", options);
        } catch (e) {
            console.error("Failed to load options:", e);

            error = {
                message: "Failed to load launcher options",
                error: e
            };
        }
    }

    async function setupClient() {
        try {
            client = await invoke("setup_client", {
                sessionToken: options.launcher.sessionToken,
                backendUrl: options.amt_options?.backendUrl || null
            });
            console.info("API Client has been set up", client);
        } catch (e) {
            console.error("Failed to set up API client:", e);
            error = {
                message: "Failed to establish connection with API",
                error: e
            };
        }
    }

    async function checkSystem() {
        try {
            await invoke("check_system");
        } catch (e) {
            alert("Looks like there is a configuration issue with your system.\n\n" + e);
        }
    }

    onMount(async () => {
        await setupOptions();
        if (options?.amt_options?.language) {
            const {initLocale} = await import("./i18n/index.js");
            initLocale(options.amt_options.language);
        }
        if (options?.amt_options?.accentColor) {
            applyAccentColor(options.amt_options.accentColor);
        }
        if (options?.amt_options?.glassBlur != null) {
            document.documentElement.style.setProperty('--amt-glass-blur', `${options.amt_options.glassBlur}px`);
            document.documentElement.style.setProperty('--amt-glass-blur-heavy', `${Math.min(options.amt_options.glassBlur * 1.33, 48)}px`);
        }
        if (options?.amt_options?.animations === false) {
            document.documentElement.style.setProperty('--amt-transition-fast', '0s');
            document.documentElement.style.setProperty('--amt-transition-normal', '0s');
            document.documentElement.style.setProperty('--amt-transition-slow', '0s');
        }
        if (options?.amt_options?.bgBlur) {
            document.documentElement.style.setProperty('--amt-bg-blur', 'blur(12px)');
        }
        await fetchBackground();
        await Promise.all([handleUpdate(), setupClient(), checkSystem()]);
        loading = false;
    });
</script>

<div class="window">
    <div class="bg-layer"></div>
    <div class="bg-overlay"></div>

    <div class="drag-area" data-tauri-drag-region></div>

    {#if error}
        <ErrorScreen {error} />
    {:else if loading || !client}
        <LoadingScreen />
    {:else if client && !client.is_secure && !allowNonSecure}
        <NonSecureConnectionScreen
            on:allowNonSecure={() => {
                allowNonSecure = true;
            }}
            on:cancel={async () => {
                await exit(0);
            }}
        />
    {:else if options}
        {#if options.start.accounts?.length > 0}
            <MainScreen {client} bind:options bind:error />
        {:else}
            <LoginScreen bind:options />
        {/if}
    {/if}

    {#if bgCredit}
        <a class="bg-credit" href={bgCreditUrl} target="_blank" rel="noopener">{bgCredit}</a>
    {/if}
</div>

<style>
    .window {
        width: 100vw;
        height: 100vh;
        padding: 32px;
        overflow-y: auto;
        position: relative;
    }

    .bg-layer {
        position: fixed;
        inset: 0;
        z-index: -2;
        background: var(--amt-bg-image, none) center/cover no-repeat;
        filter: var(--amt-bg-blur, blur(2px)) brightness(0.5);
        transform: scale(1.05);
    }

    .bg-overlay {
        position: fixed;
        inset: 0;
        z-index: -1;
        background: linear-gradient(135deg, rgba(13, 17, 23, 0.7) 0%, rgba(13, 17, 23, 0.3) 50%, rgba(13, 17, 23, 0.7) 100%);
    }

    .drag-area {
        position: fixed;
        top: 0;
        left: 0;
        width: calc(100vw - 150px);
        height: 100px;
        z-index: 100;
    }

    .bg-credit {
        position: fixed;
        bottom: 10px;
        right: 14px;
        z-index: 200;
        color: rgba(255, 255, 255, 0.3);
        font-size: 11px;
        text-decoration: none;
        transition: color 0.2s;
        font-family: "Inter", sans-serif;
    }

    .bg-credit:hover {
        color: rgba(255, 255, 255, 0.6);
    }
</style>
