<script>
    import { createEventDispatcher } from "svelte";
    import SettingsContainer from "../settings/SettingsContainer.svelte";
    import Description from "../settings/Description.svelte";
    import ButtonSetting from "../settings/ButtonSetting.svelte";
    import { t } from "../i18n/index.js";

    export let launchVersionWarningCountdown;

    const dispatch = createEventDispatcher();
</script>

<SettingsContainer
        title={$t("warning.unsupported.title")}
        on:hideSettings={() => dispatch('hide')}
>
    <Description
            description={$t("warning.unsupported.desc")}
    />
    <ButtonSetting
            disabled={launchVersionWarningCountdown > 0}
            text="Launch anyway{launchVersionWarningCountdown > 0 ? ` (${launchVersionWarningCountdown})` : ''}"
            on:click={() => dispatch('runClientAnyway')}
            color="#B83529"
    />
</SettingsContainer>
