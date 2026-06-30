<script>
    import {createEventDispatcher, onDestroy} from "svelte";
    import {invoke} from "@tauri-apps/api/core";

    export let githubToken = "";
    export let githubUser = "AMT-Entertainment";
    export let githubRepo = "AMT-Client-Backend";
    export let playerUuid = "steve";
    export let options = null;

    const dispatch = createEventDispatcher();

    let dragActive = false;
    let file = null;
    let previewUrl = null;
    let uploading = false;
    let uploadProgress = 0;
    let error = "";
    let success = false;
    let capeTitle = "";
    let lastUploadedCapeId = "";
    let lastUploadedTitle = "";

    onDestroy(() => {
        if (previewUrl) URL.revokeObjectURL(previewUrl);
    });

    const MAX_SIZE = 2 * 1024 * 1024; // 2MB
    const ALLOWED_TYPES = ["image/png"];
    const REQUIRED_WIDTH = 64;
    const REQUIRED_HEIGHT = 32;

    function handleDragOver(e) {
        e.preventDefault();
        e.stopPropagation();
        dragActive = true;
    }

    function handleDragLeave(e) {
        e.preventDefault();
        e.stopPropagation();
        dragActive = false;
    }

    async function handleDrop(e) {
        e.preventDefault();
        e.stopPropagation();
        dragActive = false;

        const droppedFile = e.dataTransfer.files[0];
        if (droppedFile) await processFile(droppedFile);
    }

    async function handleFileSelect(e) {
        const selectedFile = e.target.files[0];
        if (selectedFile) await processFile(selectedFile);
        e.target.value = "";
    }

    async function processFile(f) {
        error = "";
        success = false;
        capeTitle = "";

        if (!ALLOWED_TYPES.includes(f.type)) {
            error = "Only PNG files are allowed.";
            return;
        }

        if (f.size > MAX_SIZE) {
            error = "File must be under 2MB.";
            return;
        }

        // Validate dimensions
        const img = new Image();
        const objectUrl = URL.createObjectURL(f);
        img.src = objectUrl;
        try {
            await new Promise((resolve, reject) => {
                img.onload = () => {
                    URL.revokeObjectURL(img.src);
                    if (img.width !== REQUIRED_WIDTH || img.height !== REQUIRED_HEIGHT) {
                        reject(new Error(`Image must be exactly ${REQUIRED_WIDTH}×${REQUIRED_HEIGHT} pixels. Got ${img.width}×${img.height}.`));
                    } else {
                        resolve();
                    }
                };
                img.onerror = () => {
                    reject(new Error("Invalid image file."));
                };
            });
        } catch (e) {
            error = e.message;
            return;
        }

        file = f;
        previewUrl = URL.createObjectURL(f);
        capeTitle = f.name.replace(/\.png$/i, "");
    }

    function removeFile() {
        if (previewUrl) URL.revokeObjectURL(previewUrl);
        file = null;
        previewUrl = null;
        capeTitle = "";
        error = "";
        success = false;
    }

    async function uploadCape() {
        if (!file) return;
        if (!capeTitle.trim()) {
            error = "Please enter a cape title.";
            return;
        }
        if (!githubToken.trim()) {
            error = "GitHub token required in AMT Settings.";
            return;
        }

        uploading = true;
        uploadProgress = 0;
        error = "";
        lastUploadedCapeId = crypto.randomUUID();
        lastUploadedTitle = capeTitle.trim();

        try {
            const reader = new FileReader();
            reader.readAsDataURL(file);
            const base64 = await new Promise((resolve) => {
                reader.onload = () => resolve(reader.result.split(",")[1]);
            });

            uploadProgress = 50;

            await invoke("github_dispatch", {
                repoOwner: githubUser,
                repoName: githubRepo,
                eventType: "upload-cape",
                payload: {
                    cape_id: lastUploadedCapeId,
                    cape_title: lastUploadedTitle,
                    cape_data: base64,
                    author: playerUuid,
                    user_uuid: playerUuid,
                    animated: false,
                    frame_count: 1,
                    frame_delay: 0
                },
                token: githubToken
            });

            uploadProgress = 100;
            success = true;
            dispatch("capeUploaded", { title: lastUploadedTitle, id: lastUploadedCapeId });
        } catch (err) {
            uploading = false;
            uploadProgress = 0;
            error = String(err);
            lastUploadedCapeId = "";
            lastUploadedTitle = "";
        }
    }

    function equipLastUpload() {
        if (!options || !lastUploadedCapeId) return;
        options.amt_options.equippedCape = lastUploadedCapeId;
        options.store();
        dispatch("capeEquipped", { id: lastUploadedCapeId, title: lastUploadedTitle });
        setTimeout(() => {
            removeFile();
            uploading = false;
            uploadProgress = 0;
            success = false;
        }, 1000);
    }
</script>

<div class="upload-cape">
    <div class="section-header">
        <h2>Upload Cape</h2>
        <p class="section-desc">Publish a custom cape to the community gallery. Requires a GitHub token with repo access.</p>
    </div>

    {#if success}
        <div class="success-banner">
            <span class="success-icon">✓</span>
            <span>Cape "{lastUploadedTitle}" published!</span>
            <button class="equip-btn" on:click={equipLastUpload}>Equip Now</button>
        </div>
    {/if}

    {#if error}
        <div class="error-banner">{error}</div>
    {/if}

    <div class="upload-area" class:drag-active={dragActive} on:dragover={handleDragOver} on:dragleave={handleDragLeave} on:drop={handleDrop}>
        <input type="file" id="file-input" accept=".png,image/png" on:change={handleFileSelect} style="display:none" />

        {#if file && previewUrl}
            <div class="preview-section">
                <div class="cape-preview">
                    <img src={previewUrl} alt={capeTitle} />
                    <div class="cape-dimensions">64 × 32 px</div>
                </div>
                <div class="preview-details">
                    <label>
                        <span>Cape Title</span>
                        <input type="text" bind:value={capeTitle} placeholder="My Awesome Cape" maxlength={64} />
                    </label>
                    <div class="file-info">
                        <span>{file.name}</span>
                        <span>{(file.size / 1024).toFixed(1)} KB</span>
                    </div>
                    <div class="preview-actions">
                        <button class="btn btn-secondary" on:click={removeFile}>Remove</button>
                        <button class="btn btn-primary" on:click={uploadCape} disabled={uploading || !capeTitle.trim() || !githubToken.trim()}>
                            {uploading ? `Publishing... ${uploadProgress}%` : "Publish to Gallery"}
                        </button>
                    </div>
                    {#if uploading}
                        <div class="progress-bar">
                            <div class="progress-fill" style="width: {uploadProgress}%"></div>
                        </div>
                    {/if}
                </div>
            </div>
        {:else}
            <div class="drop-zone">
                <div class="drop-icon">◇</div>
                <p class="drop-text">Drag & drop a <strong>64×32 PNG</strong> here</p>
                <p class="drop-hint">or click to browse</p>
                <button class="btn btn-outline" on:click={() => document.getElementById('file-input').click()}>
                    Choose File
                </button>
                <ul class="requirements">
                    <li>✓ PNG format only</li>
                    <li>✓ Exactly 64×32 pixels</li>
                    <li>✓ Under 2MB</li>
                    <li>✓ Transparency supported</li>
                </ul>
            </div>
        {/if}
    </div>

    <div class="guidelines">
        <h3>Design Guidelines</h3>
        <div class="guideline-grid">
            <div class="guideline">
                <span class="guideline-icon">🎨</span>
                <div>
                    <strong>Use Transparency</strong>
                    <p>Transparent pixels show the player's skin underneath.</p>
                </div>
            </div>
            <div class="guideline">
                <span class="guideline-icon">📐</span>
                <div>
                    <strong>Safe Area</strong>
                    <p>Keep important details in the center 46×32 area to avoid clipping.</p>
                </div>
            </div>
            <div class="guideline">
                <span class="guideline-icon">🔄</span>
                <div>
                    <strong>Animation Ready</strong>
                    <p>Static capes work great. For animated, use the Cape Designer.</p>
                </div>
            </div>
            <div class="guideline">
                <span class="guideline-icon">👁️</span>
                <div>
                    <strong>Preview First</strong>
                    <p>Test in the 3D preview before publishing to the gallery.</p>
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    .upload-cape {
        display: flex;
        flex-direction: column;
        gap: 24px;
        padding: 24px;
        max-width: 720px;
    }

    .section-header h2 {
        margin: 0 0 4px;
        font-size: 20px;
        font-weight: 700;
        color: white;
    }

    .section-desc {
        margin: 0;
        font-size: 13px;
        color: rgba(255,255,255,0.5);
    }

    .success-banner {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 12px 16px;
        background: rgba(76, 175, 80, 0.15);
        border: 1px solid rgba(76, 175, 80, 0.3);
        border-radius: 10px;
        color: #81c784;
        font-size: 13px;
        font-weight: 500;
    }

    .equip-btn {
        margin-left: auto;
        padding: 6px 14px;
        border: 1px solid rgba(172,196,222,0.3);
        border-radius: 6px;
        background: rgba(172,196,222,0.15);
        color: #ACC4DE;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.15s;
    }

    .equip-btn:hover {
        background: rgba(172,196,222,0.25);
    }

    .success-icon {
        width: 20px;
        height: 20px;
        background: #4caf50;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 12px;
        color: white;
    }

    .error-banner {
        padding: 10px 14px;
        background: rgba(255, 80, 80, 0.15);
        border: 1px solid rgba(255, 80, 80, 0.3);
        border-radius: 10px;
        color: #ff6b6b;
        font-size: 13px;
    }

    .upload-area {
        border-radius: 12px;
        overflow: hidden;
        transition: all 0.2s;
    }

    .drop-zone {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        padding: 48px 24px;
        background: rgba(255,255,255,0.02);
        border: 2px dashed rgba(255,255,255,0.08);
        border-radius: 12px;
        text-align: center;
        cursor: pointer;
        transition: all 0.2s;
    }

    .upload-area.drag-active .drop-zone {
        border-color: rgba(172,196,222,0.5);
        background: rgba(172,196,222,0.05);
    }

    .drop-zone:hover {
        border-color: rgba(172,196,222,0.3);
        background: rgba(172,196,222,0.03);
    }

    .drop-icon {
        font-size: 48px;
        opacity: 0.4;
    }

    .drop-text {
        margin: 0;
        font-size: 15px;
        color: rgba(255,255,255,0.7);
    }

    .drop-text strong {
        color: white;
    }

    .drop-hint {
        margin: 0;
        font-size: 12px;
        color: rgba(255,255,255,0.3);
    }

    .requirements {
        list-style: none;
        padding: 0;
        margin: 16px 0 0;
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
        gap: 12px;
        font-size: 11px;
        color: rgba(255,255,255,0.4);
    }

    .requirements li {
        display: flex;
        align-items: center;
        gap: 4px;
    }

    .preview-section {
        display: grid;
        grid-template-columns: 160px 1fr;
        gap: 20px;
        padding: 20px;
        background: rgba(255,255,255,0.03);
        border: 1px solid rgba(255,255,255,0.06);
        border-radius: 12px;
    }

    .cape-preview {
        position: relative;
        width: 160px;
        height: 80px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(0,0,0,0.2);
        border-radius: 8px;
        overflow: hidden;
    }

    .cape-preview img {
        max-width: 100%;
        max-height: 100%;
        image-rendering: pixelated;
    }

    .cape-dimensions {
        position: absolute;
        bottom: 4px;
        right: 4px;
        font-size: 9px;
        color: rgba(255,255,255,0.4);
        background: rgba(0,0,0,0.5);
        padding: 2px 6px;
        border-radius: 4px;
    }

    .preview-details {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .preview-details label {
        display: flex;
        flex-direction: column;
        gap: 6px;
        font-size: 12px;
        color: rgba(255,255,255,0.5);
    }

    .preview-details input {
        background: rgba(0,0,0,0.3);
        border: 1px solid rgba(255,255,255,0.08);
        border-radius: 8px;
        padding: 8px 12px;
        color: white;
        font-size: 13px;
        outline: none;
    }

    .preview-details input:focus {
        border-color: rgba(172,196,222,0.4);
    }

    .file-info {
        display: flex;
        justify-content: space-between;
        font-size: 12px;
        color: rgba(255,255,255,0.3);
    }

    .preview-actions {
        display: flex;
        gap: 8px;
        padding-top: 4px;
    }

    .btn {
        flex: 1;
        padding: 10px 16px;
        border: none;
        border-radius: 8px;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.15s;
    }

    .btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .btn-primary {
        background: linear-gradient(135deg, rgba(172,196,222,0.2), rgba(172,196,222,0.1));
        border: 1px solid rgba(172,196,222,0.3);
        color: #ACC4DE;
    }

    .btn-primary:hover:not(:disabled) {
        background: linear-gradient(135deg, rgba(172,196,222,0.3), rgba(172,196,222,0.2));
    }

    .btn-secondary {
        background: rgba(255,255,255,0.05);
        border: 1px solid rgba(255,255,255,0.1);
        color: rgba(255,255,255,0.7);
    }

    .btn-secondary:hover:not(:disabled) {
        background: rgba(255,255,255,0.1);
        color: white;
    }

    .btn-outline {
        background: transparent;
        border: 1px solid rgba(255,255,255,0.15);
        color: rgba(255,255,255,0.6);
    }

    .btn-outline:hover {
        border-color: rgba(172,196,222,0.4);
        color: #ACC4DE;
    }

    .progress-bar {
        height: 4px;
        background: rgba(255,255,255,0.1);
        border-radius: 2px;
        overflow: hidden;
    }

    .progress-fill {
        height: 100%;
        background: linear-gradient(90deg, #ACC4DE, #7ec8e3);
        border-radius: 2px;
        transition: width 0.3s ease;
    }

    .guidelines {
        padding: 20px;
        background: rgba(255,255,255,0.02);
        border: 1px solid rgba(255,255,255,0.04);
        border-radius: 12px;
    }

    .guidelines h3 {
        margin: 0 0 16px;
        font-size: 14px;
        font-weight: 600;
        color: rgba(255,255,255,0.7);
    }

    .guideline-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 12px;
    }

    .guideline {
        display: flex;
        gap: 12px;
        padding: 12px;
        background: rgba(255,255,255,0.02);
        border-radius: 8px;
        border: 1px solid rgba(255,255,255,0.03);
    }

    .guideline-icon {
        font-size: 20px;
        flex-shrink: 0;
    }

    .guideline strong {
        display: block;
        font-size: 12px;
        color: white;
        margin-bottom: 2px;
    }

    .guideline p {
        margin: 0;
        font-size: 11px;
        color: rgba(255,255,255,0.4);
        line-height: 1.4;
    }

    @media (max-width: 600px) {
        .preview-section {
            grid-template-columns: 1fr;
        }

        .cape-preview {
            width: 100%;
            height: 100px;
        }

        .guideline-grid {
            grid-template-columns: 1fr;
        }
    }
</style>