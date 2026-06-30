<script>
    import BadgeEditor from "../amt/BadgeEditor.svelte";
    import SettingWrapper from "../settings/SettingWrapper.svelte";
    import ToggleSetting from "../settings/ToggleSetting.svelte";
    import TextSetting from "../settings/TextSetting.svelte";

    export let options;

    $: amt = options.amt_options;
</script>

<div class="identity-settings">
    <SettingWrapper title="Display Name">
        <TextSetting
            value={amt.display_name}
            placeholder="AMT Alias (optional)"
            on:change={(e) => {
                amt.display_name = e.detail;
                options.store();
            }}
        />
    </SettingWrapper>

    <SettingWrapper title="Badge">
        <BadgeEditor bind:badge={amt.badge} bind:accent={amt.accent_color} />
        <button class="amt-btn amt-btn-primary save-badge-btn" on:click={() => options.store()}>
            Save Badge
        </button>
    </SettingWrapper>

    <SettingWrapper title="Badge Visibility">
        <label class="amt-toggle-row">
            <span class="toggle-label">Show in Launcher</span>
            <label class="amt-toggle">
                <input type="checkbox" checked={amt.badge_visibility.launcher}
                    on:change={() => { amt.badge_visibility.launcher = !amt.badge_visibility.launcher; options.store(); }} />
                <span class="track"></span>
            </label>
        </label>
        <label class="amt-toggle-row">
            <span class="toggle-label">Show In-Game</span>
            <label class="amt-toggle">
                <input type="checkbox" checked={amt.badge_visibility.in_game}
                    on:change={() => { amt.badge_visibility.in_game = !amt.badge_visibility.in_game; options.store(); }} />
                <span class="track"></span>
            </label>
        </label>
    </SettingWrapper>

    <SettingWrapper title="Profile Visibility">
        <div class="select-row">
            <select class="amt-input" bind:value={amt.profileVisibility} on:change={() => options.store()}>
                <option value="public">Public</option>
                <option value="friends">Friends Only</option>
                <option value="private">Private</option>
            </select>
        </div>
    </SettingWrapper>
</div>

<style>
    .identity-settings {
        display: flex;
        flex-direction: column;
        gap: 24px;
    }

    .select-row {
        display: flex;
        gap: 8px;
    }

    .select-row select {
        flex: 1;
    }

    .amt-toggle-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 8px 0;
    }

    .toggle-label {
        color: var(--amt-text);
        font-size: 13px;
    }

    .save-badge-btn {
        margin-top: 8px;
    }
</style>
