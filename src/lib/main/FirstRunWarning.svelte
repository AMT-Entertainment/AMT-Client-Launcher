<script>
    import {createEventDispatcher} from "svelte";
    import SettingsContainer from "../settings/SettingsContainer.svelte";
    import Description from "../settings/Description.svelte";
    import ButtonSetting from "../settings/ButtonSetting.svelte";
    import { t } from "../i18n/index.js";

    const dispatch = createEventDispatcher();
</script>

<SettingsContainer
        title={$t("firstrun.title")}
        on:hideSettings={() => dispatch('hide')}
>
    <Description
            description={$t("firstrun.desc")}
    />

    <ButtonSetting
            text="Get Started"
            on:click={() => dispatch('continue')}
            color="#4677FF"
    />
</SettingsContainer>
