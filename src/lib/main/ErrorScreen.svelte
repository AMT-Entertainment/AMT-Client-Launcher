<script>
    import VerticalFlexWrapper from "../common/VerticalFlexWrapper.svelte";
    import TitleBar from "../common/TitleBar.svelte";
    import Logo from "../common/Logo.svelte";
    import ButtonClose from "../common/ButtonClose.svelte";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import ButtonCopyClipboard from "../common/ButtonCopyClipboard.svelte";
    import { t } from "../i18n/index.js";
    import ButtonSetting from "../settings/ButtonSetting.svelte";
    
    export let error = {
        message: "Unknown error",
        error: null
    };

    let userMessage = "";

    async function reportIssue() {
        const subject = encodeURIComponent("AMT Client Error Report");
        const body = encodeURIComponent(
            `Error: ${error.message}\n\nDetails:\n${error.error || "N/A"}\n\nYour message:\n${userMessage || "(none)"}`
        );
        await openUrl(`mailto:amt.entertainment@outlook.de?subject=${subject}&body=${body}`);
    }
</script>

<VerticalFlexWrapper blur={false}>
    <TitleBar>
        <Logo />
        <ButtonClose />
    </TitleBar>

    <div class="error-container">
        <h1>{$t("error.title")}</h1>
        <p class="error-message">{error.message}</p>
        
        {#if error.error}
            <div class="error-details">
                <div class="error-header">
                    <h3>{$t("error.details")}</h3>
                    <div class="copy-button">
                        <ButtonCopyClipboard textToCopy={JSON.stringify(error)} iconOnly={true} />
                    </div>
                </div>
                <pre>{error.error}</pre>
            </div>
        {/if}

        <div class="user-message-area">
            <label for="error-report-message">Add a message (optional):</label>
            <textarea id="error-report-message" bind:value={userMessage} placeholder="Describe what you were doing when the error occurred..." rows="3"></textarea>
        </div>

        <div class="help-buttons">
            <ButtonSetting text={$t("error.report")} color="#4677ff" on:click={reportIssue}></ButtonSetting>
        </div>
    </div>
</VerticalFlexWrapper>

<style>
    .error-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 2rem;
        max-width: 800px;
        margin: 0 auto;
        color: white;
        flex: 1;
    }

    h1 {
        color: #ff4444;
        font-size: 2.5rem;
        margin-bottom: 1rem;
    }

    .error-message {
        font-size: 1.2rem;
        margin-bottom: 2rem;
        text-align: center;
    }

    .error-details {
        background-color: rgba(0, 0, 0, 0.3);
        padding: 1rem;
        border-radius: 6px;
        width: 100%;
        margin-bottom: 2rem;
        max-height: 300px;
        overflow: auto;
    }

    .error-details pre {
        user-select: all;
    }

    .error-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 0.5rem;
    }
    
    .copy-button {
        flex-shrink: 0;
    }

    .error-details pre {
        white-space: pre-wrap;
        word-break: break-all;
    }

    .user-message-area {
        width: 100%;
        margin-bottom: 1rem;
    }

    .user-message-area label {
        display: block;
        margin-bottom: 0.4rem;
        font-size: 0.9rem;
        color: #ccc;
    }

    .user-message-area textarea {
        width: 100%;
        padding: 0.6rem;
        border-radius: 6px;
        border: 1px solid rgba(255,255,255,0.2);
        background: rgba(0,0,0,0.3);
        color: white;
        font-family: inherit;
        font-size: 0.9rem;
        resize: vertical;
        box-sizing: border-box;
    }

    .user-message-area textarea:focus {
        outline: none;
        border-color: #4677ff;
    }

    .help-buttons {
        display: flex;
        gap: 1rem;
        margin-top: 1rem;
    }
</style>