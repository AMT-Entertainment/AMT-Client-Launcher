<script>
    import {createEventDispatcher, onMount} from "svelte";
    import GeneralSettings from "../main/settings/GeneralSettings.svelte";
    import IdentitySettings from "../identity/IdentitySettings.svelte";
    import AppearanceSettings from "./AppearanceSettings.svelte";
    import SettingsContainer from "./SettingsContainer.svelte";
    import Tabs from "./tab/Tabs.svelte";
    import BadgeEditor from "../amt/BadgeEditor.svelte";
    import AccountSettings from "../main/settings/AccountSettings.svelte";
    import {locale, t} from "../i18n/index.js";

    export let client;
    export let options;
    let activeTab = "General";

    const dispatch = createEventDispatcher();

    $: if (options.amt_options?.language) {
        locale.set(options.amt_options.language);
    }

    onMount(() => {
        if (options.amt_options.accent_color) {
            document.documentElement.style.setProperty('--amt-accent', options.amt_options.accent_color);
        }
    });
</script>

<SettingsContainer
    title="Settings"
    on:hideSettings={() => dispatch('hide')}
>
    <Tabs
        tabs={[$t("settings.general"), $t("settings.appearance"), $t("settings.identity"), $t("settings.accounts")]}
        bind:activeTab={activeTab}
        slot="tabs"
    />

    {#if activeTab === $t("settings.general") || activeTab === "General"}
        <GeneralSettings bind:options />
    {:else if activeTab === $t("settings.appearance") || activeTab === "Appearance"}
        <AppearanceSettings bind:options />
    {:else if activeTab === $t("settings.identity") || activeTab === "Identity"}
        <IdentitySettings bind:options />
    {:else if activeTab === $t("settings.accounts") || activeTab === "Accounts"}
        <AccountSettings {client} bind:options />
    {/if}
</SettingsContainer>
