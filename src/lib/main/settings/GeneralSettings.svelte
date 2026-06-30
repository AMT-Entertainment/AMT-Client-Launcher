<script>
    import SelectSetting from "../../settings/SelectSetting.svelte";
    import FileSelectorSetting from "../../settings/FileSelectorSetting.svelte";
    import DirectorySelectorSetting from "../../settings/DirectorySelectorSetting.svelte";
    import RangeSetting from "../../settings/RangeSetting.svelte";
    import ToggleSetting from "../../settings/ToggleSetting.svelte";
    import ButtonSetting from "../../settings/ButtonSetting.svelte";
    import TextSetting from "../../settings/TextSetting.svelte";
    import LauncherVersion from "../../settings/LauncherVersion.svelte";
    import {onMount} from "svelte";
    import {invoke} from "@tauri-apps/api/core";
    import {locale, t} from "../../i18n/index.js";

    export let options;

    let launcherVersion = "";
    let defaultDataFolder = "";
    let systemMemory = options.start.memory;

    async function clearData() {
        try {
            await invoke("clear_data", { options });
            alert("Data cleared.");
        } catch (error) {
            console.error("Failed to clear data:", error);
            alert(`Failed to clear data: ${error}`);
        }
    }

    async function logout() {
        try {
            await invoke("logout", { accountData: options.start.account });
            options.start.account = null;
            options.start.accounts = [];
            options.start.activeAccountIndex = 0;
            await options.store();
        } catch (error) {
            console.error("Logout failed:", error);
            alert("Failed to logout properly. Please try again.");
        }
    }

    async function clearAllData() {
        if (!confirm("Delete all account data?")) return;
        try {
            const fresh = await invoke("clear_all_data");
            Object.assign(options, fresh);
            options.store = async function() {
                await invoke("store_options", {options});
            };
            alert("All data cleared. The launcher will restart.");
            window.location.reload();
        } catch (error) {
            console.error("Failed to clear all data:", error);
            alert(`Failed to clear all data: ${error}`);
        }
    }

    function resetJavaDistribution() {
        if (options.start.javaDistribution.type === "custom") {
            options.start.javaDistribution.value = "";
        } else if (options.start.javaDistribution.type === "manual") {
            options.start.javaDistribution.value = "temurin";
        }
    }

    $: if (options.amt_options?.language) {
        locale.set(options.amt_options.language);
    }

    onMount(async () => {
        const [version, folder, memory] = await Promise.all([
            invoke("get_launcher_version"),
            invoke("default_data_folder_path"),
            invoke("sys_memory"),
        ]);
        systemMemory = Math.min(memory, Math.floor(memory * 0.75));
        launcherVersion = version;
        defaultDataFolder = folder;
    });
</script>

<div class="general-settings">
    <!-- Performance -->
    <div class="settings-section">
        <h4 class="section-label">Performance</h4>

        <SelectSetting
            title={$t("settings.jvmDistribution")}
            items={[
                { value: "automatic", text: "Automatic (Recommended)" },
                { value: "manual", text: "Manual" },
                { value: "custom", text: "Custom" },
            ]}
            on:change={resetJavaDistribution}
            bind:value={options.start.javaDistribution.type}
        />

        {#if options.start.javaDistribution.type === "manual"}
            <SelectSetting
                title="Distribution"
                items={[
                    { value: "temurin", text: "Eclipse Temurin" },
                    { value: "graalvm", text: "GraalVM" },
                    { value: "zulu", text: "Azul Zulu" },
                ]}
                bind:value={options.start.javaDistribution.value}
            />
        {/if}

        {#if options.start.javaDistribution.type === "custom"}
            <FileSelectorSetting
                title="Custom JVM Path"
                placeholder="Select Java wrapper location"
                bind:value={options.start.javaDistribution.value}
                filters={[{ name: "java", extensions: [] }]}
                windowTitle="Select custom Java wrapper"
            />
        {/if}

        <RangeSetting
            title={$t("settings.memory")}
            min={2048}
            max={systemMemory}
            bind:value={options.start.memory}
            valueSuffix=" MB"
            step={128}
        />

        <RangeSetting
            title={$t("settings.concurrentDownloads")}
            min={1}
            max={50}
            bind:value={options.launcher.concurrentDownloads}
            valueSuffix=" connections"
            step={1}
        />
    </div>

    <div class="amt-divider"></div>

    <!-- Game -->
    <div class="settings-section">
        <h4 class="section-label">Game</h4>

        <ToggleSetting
            title={$t("settings.keepLauncherOpen")}
            disabled={false}
            bind:value={options.launcher.keepLauncherOpen}
        />

        <DirectorySelectorSetting
            title={$t("settings.dataLocation")}
            placeholder={defaultDataFolder}
            bind:value={options.start.customDataPath}
            windowTitle="Select custom data directory"
        />
    </div>

    <div class="amt-divider"></div>

    <!-- Account Actions -->
    <div class="settings-section">
        <h4 class="section-label">Account</h4>
        <ButtonSetting
            text={$t("settings.signout.minecraft")}
            on:click={logout}
            color="#ACC4DE"
        />
    </div>

    <div class="amt-divider"></div>

    <!-- Data Management -->
    <div class="settings-section">
        <h4 class="section-label">Data Management</h4>
        <ButtonSetting text={$t("settings.clearData")} on:click={clearData} color="#E5534B" />
        <div class="danger-zone">
            <h4 class="danger-title">{$t("settings.clearAllData")}</h4>
            <p class="danger-desc">{$t("settings.clearAllData.desc")}</p>
            <ButtonSetting text={$t("settings.clearAllData")} on:click={clearAllData} color="#E5534B" />
        </div>
    </div>

    <LauncherVersion version={launcherVersion} />
</div>

<style>
    .general-settings {
        display: flex;
        flex-direction: column;
        gap: 8px;
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

    .section-desc {
        color: var(--amt-text-muted);
        font-size: 12px;
        margin: 0;
    }

    .danger-zone {
        border: 1px solid rgba(229, 83, 75, 0.2);
        border-radius: 8px;
        padding: 12px;
        background: rgba(229, 83, 75, 0.04);
    }

    .danger-title {
        color: var(--amt-danger);
        font-size: 12px;
        font-weight: 600;
        margin: 0 0 4px;
    }

    .danger-desc {
        color: var(--amt-text-muted);
        font-size: 11px;
        margin: 0 0 8px;
    }
</style>
