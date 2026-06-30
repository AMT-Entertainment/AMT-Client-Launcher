<script>
    import SettingWrapper from "./SettingWrapper.svelte";

    export let options;

    // amt_options uses serde rename = camelCase in JSON, so JS properties are camelCase
    $: amt = options.amt_options || {};
    $: if (amt.language) {
        import("../i18n/index.js").then(({locale}) => locale.set(amt.language));
    }

    const THEMES = [
        { value: "default", label: "Classic AMT", accent: "#ACC4DE", bg: "#0D1117" },
        { value: "midnight", label: "Midnight Ocean", accent: "#1E90FF", bg: "#0A0E1A" },
        { value: "cyberpunk", label: "Cyberpunk", accent: "#FF00FF", bg: "#0A0015" },
        { value: "forest", label: "Forest Glade", accent: "#3FB950", bg: "#0A120A" },
        { value: "sakura", label: "Sakura Dawn", accent: "#FF7B9C", bg: "#1A0A0F" },
        { value: "sunset", label: "Sunset Ember", accent: "#FF6B35", bg: "#1A0D05" },
        { value: "ocean", label: "Deep Ocean", accent: "#00BCD4", bg: "#000D12" },
        { value: "dracula", label: "Dracula", accent: "#BD93F9", bg: "#12121A" },
        { value: "nord", label: "Nord Frost", accent: "#88C0D0", bg: "#10141A" },
        { value: "monokai", label: "Monokai", accent: "#A6E22E", bg: "#121212" },
    ];

    const BACKGROUNDS = [
        { value: "", label: "Default Dark" },
        { value: "space", label: "Space" },
        { value: "forest", label: "Forest" },
        { value: "abstract", label: "Abstract" },
        { value: "mountains", label: "Mountains" },
        { value: "ocean", label: "Ocean" },
        { value: "city", label: "City Night" },
        { value: "custom", label: "Custom Image..." },
    ];

    const LANGUAGES = [
        { value: "en", text: "English" },
        { value: "de", text: "Deutsch" },
        { value: "pl", text: "Polski" },
    ];

    let customBgPath = "";

    function hexToRgb(hex) {
        const r = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return r ? `${parseInt(r[1], 16)}, ${parseInt(r[2], 16)}, ${parseInt(r[3], 16)}` : "172, 196, 222";
    }

    function applyTheme(themeVal) {
        const theme = THEMES.find(t => t.value === themeVal);
        if (!theme) return;
        amt.themePreset = themeVal;
        amt.accentColor = theme.accent;
        document.documentElement.style.setProperty('--amt-accent', theme.accent);
        document.documentElement.style.setProperty('--amt-accent-rgb', hexToRgb(theme.accent));
        options.store();
    }

    function updateAccent(e) {
        const val = e.target.value;
        if (/^#[0-9A-Fa-f]{6}$/.test(val)) {
            amt.accentColor = val;
            document.documentElement.style.setProperty('--amt-accent', val);
            document.documentElement.style.setProperty('--amt-accent-rgb', hexToRgb(val));
            options.store();
        }
    }

    function applyBgPreset() {
        if (amt.backgroundPreset === "custom") {
            customBgPath = "";
            return;
        }
        const bgUrl = amt.backgroundPreset
            ? `url(/backgrounds/${amt.backgroundPreset}.jpg)`
            : 'none';
        document.documentElement.style.setProperty('--amt-bg-image', bgUrl);
        options.store();
    }

    function handleCustomBg() {
        if (!customBgPath) return;
        amt.backgroundPreset = "custom";
        document.documentElement.style.setProperty('--amt-bg-image', `url(${customBgPath})`);
        options.store();
    }

    async function changeLanguage(e) {
        const val = e.target.value;
        amt.language = val;
        const {locale} = await import("../i18n/index.js");
        locale.set(val);
        options.store();
    }

    function updateBlur(e) {
        amt.glassBlur = parseFloat(e.target.value);
        document.documentElement.style.setProperty('--amt-glass-blur', `${amt.glassBlur}px`);
        document.documentElement.style.setProperty('--amt-glass-blur-heavy', `${Math.min(amt.glassBlur * 1.33, 48)}px`);
        options.store();
    }

    function toggleAnimations() {
        amt.animations = !amt.animations;
        document.documentElement.style.setProperty('--amt-transition-fast', amt.animations !== false ? '0.15s ease' : '0s');
        document.documentElement.style.setProperty('--amt-transition-normal', amt.animations !== false ? '0.2s ease' : '0s');
        document.documentElement.style.setProperty('--amt-transition-slow', amt.animations !== false ? '0.3s ease' : '0s');
        options.store();
    }

    function toggleBgBlur() {
        amt.bgBlur = !amt.bgBlur;
        document.documentElement.style.setProperty('--amt-bg-blur', amt.bgBlur ? 'blur(12px)' : 'none');
        options.store();
    }
</script>

<div class="appearance-settings">
    <div class="settings-section">
        <h4 class="section-label">Theme Preset</h4>
        <p class="section-desc">Choose a curated color scheme</p>
        <div class="theme-grid">
            {#each THEMES as theme}
                <button
                    class="theme-card"
                    class:active={amt.themePreset === theme.value}
                    style="--card-accent: {theme.accent}; --card-bg: {theme.bg};"
                    on:click={() => applyTheme(theme.value)}
                    title={theme.label}
                >
                    <span class="theme-swatch" style="background: {theme.accent};"></span>
                    <span class="theme-label">{theme.label}</span>
                </button>
            {/each}
        </div>
    </div>

    <div class="amt-divider"></div>

    <SettingWrapper title="Accent Color">
        <div class="color-picker-row">
            <input
                type="color"
                value={amt.accentColor}
                on:input={updateAccent}
                class="color-input"
            />
            <input
                class="amt-input hex-input"
                type="text"
                value={amt.accentColor}
                on:input={updateAccent}
                placeholder="#ACC4DE"
            />
        </div>
        <div class="preset-colors">
            {#each ["#ACC4DE", "#1E90FF", "#FF00FF", "#3FB950", "#FF7B9C", "#FF6B35", "#00BCD4", "#BD93F9", "#FF4444", "#FFAA00"] as color}
                <button
                    class="color-swatch"
                    style="background: {color};"
                    class:active={amt.accentColor === color}
                    on:click={() => {
                        amt.accentColor = color;
                        document.documentElement.style.setProperty('--amt-accent', color);
                        document.documentElement.style.setProperty('--amt-accent-rgb', hexToRgb(color));
                        options.store();
                    }}
                ></button>
            {/each}
        </div>
    </SettingWrapper>

    <SettingWrapper title="Background">
        <select class="amt-select" bind:value={amt.backgroundPreset} on:change={applyBgPreset}>
            {#each BACKGROUNDS as bg}
                <option value={bg.value}>{bg.label}</option>
            {/each}
        </select>
        {#if amt.backgroundPreset === "custom"}
            <div class="custom-bg-row">
                <input
                    class="amt-input"
                    type="text"
                    placeholder="Paste image URL or path..."
                    bind:value={customBgPath}
                />
                <button class="amt-btn amt-btn-sm" on:click={handleCustomBg}>Apply</button>
            </div>
        {/if}
        <div class="bg-toggle-row">
            <label class="toggle-label">
                <input type="checkbox" checked={amt.bgBlur} on:change={toggleBgBlur} />
                <span class="toggle-text">Blur Background</span>
            </label>
        </div>
    </SettingWrapper>

    <SettingWrapper title="Glass Effect">
        <div class="slider-row">
            <span class="slider-label">Opacity</span>
            <input
                type="range"
                class="amt-slider"
                min="0"
                max="0.2"
                step="0.01"
                bind:value={amt.glassOpacity}
                on:change={() => options.store()}
            />
            <span class="slider-value">{Math.round((amt.glassOpacity || 0.05) * 100)}%</span>
        </div>
        <div class="slider-row">
            <span class="slider-label">Blur</span>
            <input
                type="range"
                class="amt-slider"
                min="4"
                max="48"
                step="2"
                value={amt.glassBlur || 24}
                on:input={updateBlur}
            />
            <span class="slider-value">{amt.glassBlur || 24}px</span>
        </div>
    </SettingWrapper>

    <SettingWrapper title="Font Size">
        <div class="font-options">
            {#each ["S", "M", "L"] as size}
                <button
                    class="font-btn"
                    class:active={amt.fontSize === size}
                    on:click={() => {
                        amt.fontSize = size;
                        document.documentElement.className = `font-size-${size.toLowerCase()}`;
                        options.store();
                    }}
                >
                    {size}
                </button>
            {/each}
        </div>
    </SettingWrapper>

    <SettingWrapper title="Animations">
        <label class="toggle-label">
            <input type="checkbox" checked={amt.animations !== false} on:change={toggleAnimations} />
            <span class="toggle-text">Enable UI Animations</span>
        </label>
    </SettingWrapper>

    <SettingWrapper title="Language">
        <select class="amt-select" on:change={changeLanguage}>
            {#each LANGUAGES as lang}
                <option value={lang.value} selected={amt.language === lang.value}>{lang.text}</option>
            {/each}
        </select>
    </SettingWrapper>
</div>

<style>
    .appearance-settings {
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

    .theme-grid {
        display: grid;
        grid-template-columns: repeat(5, 1fr);
        gap: 6px;
    }

    .theme-card {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 4px;
        padding: 8px 4px;
        background: rgba(255,255,255,0.03);
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 8px;
        cursor: pointer;
        transition: all 0.15s;
        color: white;
    }

    .theme-card:hover {
        background: rgba(255,255,255,0.06);
        border-color: var(--card-accent);
    }

    .theme-card.active {
        background: rgba(var(--amt-accent-rgb), 0.1);
        border-color: var(--card-accent);
        box-shadow: 0 0 12px rgba(var(--amt-accent-rgb), 0.15);
    }

    .theme-swatch {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        border: 2px solid rgba(255,255,255,0.1);
    }

    .theme-label {
        font-size: 10px;
        color: var(--amt-text-secondary);
        text-align: center;
        line-height: 1.2;
    }

    .color-picker-row {
        display: flex;
        gap: 10px;
        align-items: center;
        margin-bottom: 8px;
    }

    .color-input {
        width: 40px;
        height: 40px;
        padding: 0;
        border: 1px solid rgba(255,255,255,0.1);
        border-radius: 8px;
        cursor: pointer;
        background: none;
    }

    .color-input::-webkit-color-swatch-wrapper {
        padding: 2px;
    }

    .color-input::-webkit-color-swatch {
        border-radius: 6px;
        border: none;
    }

    .hex-input {
        flex: 1;
        font-family: var(--amt-font-mono);
    }

    .preset-colors {
        display: flex;
        gap: 6px;
        flex-wrap: wrap;
    }

    .color-swatch {
        width: 28px;
        height: 28px;
        border-radius: 50%;
        border: 2px solid rgba(255,255,255,0.1);
        cursor: pointer;
        transition: all 0.15s;
        padding: 0;
    }

    .color-swatch:hover {
        transform: scale(1.15);
        border-color: rgba(255,255,255,0.3);
    }

    .color-swatch.active {
        border-color: white;
        box-shadow: 0 0 8px rgba(255,255,255,0.3);
    }

    .slider-row {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 4px;
    }

    .slider-row input {
        flex: 1;
    }

    .slider-label {
        color: var(--amt-text-secondary);
        font-size: 12px;
        min-width: 50px;
    }

    .slider-value {
        color: rgba(255,255,255,0.6);
        font-size: 13px;
        min-width: 40px;
        text-align: right;
        font-family: var(--amt-font-mono);
    }

    .font-options {
        display: flex;
        gap: 4px;
    }

    .font-btn {
        padding: 8px 18px;
        background: rgba(255,255,255,0.08);
        border: 1px solid rgba(255,255,255,0.1);
        border-radius: 6px;
        color: white;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.15s;
    }

    .font-btn:hover {
        background: rgba(255,255,255,0.12);
    }

    .font-btn.active {
        background: rgba(var(--amt-accent-rgb), 0.15);
        border-color: var(--amt-accent);
        color: var(--amt-accent);
    }

    .custom-bg-row {
        display: flex;
        gap: 8px;
        margin-top: 8px;
    }

    .custom-bg-row input {
        flex: 1;
    }

    .bg-toggle-row {
        margin-top: 8px;
    }

    .toggle-label {
        display: flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
        color: var(--amt-text-secondary);
        font-size: 13px;
    }

    .toggle-label input[type="checkbox"] {
        accent-color: var(--amt-accent);
        width: 16px;
        height: 16px;
    }

    .toggle-text {
        user-select: none;
    }

    .amt-divider {
        height: 1px;
        background: rgba(255,255,255,0.06);
        margin: 4px 0;
    }
</style>
