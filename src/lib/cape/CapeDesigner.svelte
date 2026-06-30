<script>
    import {onMount} from "svelte";
    import {invoke} from "@tauri-apps/api/core";

    export let accent = "#ACC4DE";
    export let githubToken = "";
    export let githubUser = "AMT-Entertainment";
    export let githubRepo = "AMT-Client-Backend";
    export let playerUuid = "steve";

    const CANVAS_W = 64;
    const CANVAS_H = 32;
    const SCALE = 8;

    let canvas;
    let previewCanvas;
    let ctx;
    let previewCtx;
    let currentColor = "#ACC4DE";
    let currentTool = "pencil";
    let symmetry = false;
    let undoStack = [];
    let redoStack = [];
    let layers = [
        { name: "Base", visible: true, data: null },
        { name: "Overlay", visible: true, data: null },
        { name: "Frame 2", visible: true, data: null },
    ];
    let activeLayer = 0;
    let isDrawing = false;
    let hexInput = "#ACC4DE";

    const PALETTE = [
        "#FFFFFF", "#D4D4D4", "#AAAAAA", "#808080", "#555555", "#2D2D2D", "#000000",
        "#FF6B6B", "#FF4444", "#CC0000", "#990000",
        "#FFB347", "#FF8800", "#CC6600",
        "#FFE066", "#FFCC00", "#CCAA00",
        "#6BFF6B", "#44CC44", "#228B22", "#006600",
        "#6BB5FF", "#4488FF", "#0055CC", "#003399",
        "#B36BFF", "#8844FF", "#5500CC", "#330099",
        "#FF6BB5", "#FF44AA", "#CC0088",
        "#B5B5FF", "#8888FF", "#5555CC",
        "#ACC4DE", "#88AACC", "#6688AA",
    ];

    function initCanvas() {
        ctx = canvas.getContext('2d');
        previewCtx = previewCanvas.getContext('2d');

        layers.forEach(l => {
            if (!l.data) {
                l.data = new Uint8ClampedArray(CANVAS_W * CANVAS_H * 4);
                fillLayer(l.data, [0, 0, 0, 0]);
            }
        });

        drawCanvas();
        updatePreview();
        saveState();
    }

    function fillLayer(data, color) {
        for (let i = 0; i < data.length; i += 4) {
            data[i] = color[0];
            data[i+1] = color[1];
            data[i+2] = color[2];
            data[i+3] = color[3];
        }
    }

    function rgbaToUint8(hex, alpha = 255) {
        const r = parseInt(hex.slice(1,3), 16);
        const g = parseInt(hex.slice(3,5), 16);
        const b = parseInt(hex.slice(5,7), 16);
        return [r, g, b, alpha];
    }

    function drawCanvas() {
        ctx.clearRect(0, 0, CANVAS_W * SCALE, CANVAS_H * SCALE);

        // Draw checkerboard background
        for (let y = 0; y < CANVAS_H; y++) {
            for (let x = 0; x < CANVAS_W; x++) {
                const check = (x + y) % 2 === 0;
                ctx.fillStyle = check ? '#1a1a2e' : '#16213e';
                ctx.fillRect(x * SCALE, y * SCALE, SCALE, SCALE);
            }
        }

        // Dim non-cape areas (outside the cape texture bounds: x 20-41, y 0-16)
        for (let y = 0; y < CANVAS_H; y++) {
            for (let x = 0; x < CANVAS_W; x++) {
                const inCapeArea = x >= 20 && x <= 41 && y >= 0 && y <= 16;
                if (!inCapeArea) {
                    ctx.fillStyle = 'rgba(0,0,0,0.35)';
                    ctx.fillRect(x * SCALE, y * SCALE, SCALE, SCALE);
                }
            }
        }

        // Draw cape area border
        ctx.strokeStyle = 'rgba(255,255,255,0.15)';
        ctx.lineWidth = 1;
        ctx.strokeRect(20 * SCALE, 0, 22 * SCALE, 17 * SCALE);

        // Draw front/back divider line in the cape area
        ctx.strokeStyle = 'rgba(255,200,100,0.25)';
        ctx.lineWidth = 2;
        ctx.setLineDash([4, 4]);
        ctx.beginPath();
        ctx.moveTo(31 * SCALE, 0);
        ctx.lineTo(31 * SCALE, 17 * SCALE);
        ctx.stroke();
        ctx.setLineDash([]);

        // Draw labels on the cape area
        ctx.font = 'bold 11px Inter, sans-serif';
        ctx.textAlign = 'center';
        ctx.fillStyle = 'rgba(255,200,100,0.5)';
        ctx.fillText('BACK', 25.5 * SCALE, 9 * SCALE + 4);
        ctx.fillText('FRONT', 36 * SCALE, 9 * SCALE + 4);

        // "Not visible" labels on sides
        ctx.font = '9px Inter, sans-serif';
        ctx.fillStyle = 'rgba(255,255,255,0.2)';
        ctx.textAlign = 'center';
        ctx.fillText('(not visible)', 10 * SCALE, 16 * SCALE);
        ctx.fillText('(not visible)', 53 * SCALE, 16 * SCALE);

        // Draw layers
        for (let li = 0; li < layers.length; li++) {
            if (!layers[li].visible) continue;
            const data = layers[li].data;
            for (let y = 0; y < CANVAS_H; y++) {
                for (let x = 0; x < CANVAS_W; x++) {
                    const idx = (y * CANVAS_W + x) * 4;
                    const a = data[idx + 3];
                    if (a === 0) continue;
                    ctx.fillStyle = `rgba(${data[idx]},${data[idx+1]},${data[idx+2]},${a/255})`;
                    ctx.fillRect(x * SCALE, y * SCALE, SCALE, SCALE);
                }
            }
        }

        // Draw grid
        ctx.strokeStyle = 'rgba(255,255,255,0.05)';
        ctx.lineWidth = 0.5;
        for (let x = 0; x <= CANVAS_W; x++) {
            ctx.beginPath();
            ctx.moveTo(x * SCALE, 0);
            ctx.lineTo(x * SCALE, CANVAS_H * SCALE);
            ctx.stroke();
        }
        for (let y = 0; y <= CANVAS_H; y++) {
            ctx.beginPath();
            ctx.moveTo(0, y * SCALE);
            ctx.lineTo(CANVAS_W * SCALE, y * SCALE);
            ctx.stroke();
        }

        // Active layer indicator
        ctx.strokeStyle = accent;
        ctx.lineWidth = 2;
        ctx.strokeRect(0, 0, CANVAS_W * SCALE, CANVAS_H * SCALE);
    }

    function updatePreview() {
        const W = 22 * 4, H = 17 * 4;
        previewCtx.clearRect(0, 0, W, H);

        const merged = mergeLayers();
        // Render the cape area (back of torso) - pixels 0-22, 0-17 roughly
        for (let y = 0; y < 17; y++) {
            for (let x = 0; x < 22; x++) {
                const sx = x + 20;
                const sy = y;
                if (sx >= CANVAS_W || sy >= CANVAS_H) continue;
                const idx = (sy * CANVAS_W + sx) * 4;
                previewCtx.fillStyle = `rgba(${merged[idx]},${merged[idx+1]},${merged[idx+2]},${merged[idx+3]/255})`;
                previewCtx.fillRect(x * 4, y * 4, 4, 4);
            }
        }
        // Dividing line between back (left) and front (right)
        previewCtx.strokeStyle = "rgba(255,200,100,0.3)";
        previewCtx.lineWidth = 1;
        previewCtx.setLineDash([3, 3]);
        previewCtx.beginPath();
        previewCtx.moveTo(11 * 4 + 2, 0);
        previewCtx.lineTo(11 * 4 + 2, H);
        previewCtx.stroke();
        previewCtx.setLineDash([]);
        // Side labels
        previewCtx.fillStyle = "rgba(255,200,100,0.6)";
        previewCtx.font = "bold 9px Inter, sans-serif";
        previewCtx.textAlign = "center";
        previewCtx.fillText("← BACK", 11 * 2, 14);
        previewCtx.fillText("FRONT →", 11 * 2 + 11 * 2, 14);
    }

    function mergeLayers() {
        const merged = new Uint8ClampedArray(CANVAS_W * CANVAS_H * 4);
        fillLayer(merged, [0, 0, 0, 0]);

        for (let li = 0; li < layers.length; li++) {
            if (!layers[li].visible) continue;
            const data = layers[li].data;
            for (let i = 0; i < data.length; i += 4) {
                const srcAlpha = data[i+3] / 255;
                if (srcAlpha === 0) continue;
                const dstAlpha = merged[i+3] / 255;
                const outAlpha = srcAlpha + dstAlpha * (1 - srcAlpha);
                if (outAlpha === 0) continue;
                merged[i] = (data[i] * srcAlpha + merged[i] * dstAlpha * (1 - srcAlpha)) / outAlpha;
                merged[i+1] = (data[i+1] * srcAlpha + merged[i+1] * dstAlpha * (1 - srcAlpha)) / outAlpha;
                merged[i+2] = (data[i+2] * srcAlpha + merged[i+2] * dstAlpha * (1 - srcAlpha)) / outAlpha;
                merged[i+3] = outAlpha * 255;
            }
        }
        return merged;
    }

    function getPixel(e) {
        const rect = canvas.getBoundingClientRect();
        const x = Math.floor((e.clientX - rect.left) / SCALE);
        const y = Math.floor((e.clientY - rect.top) / SCALE);
        return { x: Math.max(0, Math.min(CANVAS_W - 1, x)), y: Math.max(0, Math.min(CANVAS_H - 1, y)) };
    }

    function paint(e) {
        if (!isDrawing) return;
        const { x, y } = getPixel(e);
        const data = layers[activeLayer].data;
        const [r, g, b, a] = rgbaToUint8(currentColor);

        const idx = (y * CANVAS_W + x) * 4;
        if (currentTool === "pencil") {
            data[idx] = r; data[idx+1] = g; data[idx+2] = b; data[idx+3] = a;
            if (symmetry) {
                const symX = CANVAS_W - 1 - x;
                const sidx = (y * CANVAS_W + symX) * 4;
                data[sidx] = r; data[sidx+1] = g; data[sidx+2] = b; data[sidx+3] = a;
            }
        } else if (currentTool === "eraser") {
            data[idx] = 0; data[idx+1] = 0; data[idx+2] = 0; data[idx+3] = 0;
            if (symmetry) {
                const symX = CANVAS_W - 1 - x;
                const sidx = (y * CANVAS_W + symX) * 4;
                data[sidx] = 0; data[sidx+1] = 0; data[sidx+2] = 0; data[sidx+3] = 0;
            }
        }

        drawCanvas();
        updatePreview();
    }

    function startDraw(e) {
        isDrawing = true;
        saveState();
        paint(e);
    }

    function stopDraw() {
        isDrawing = false;
    }

    function saveState() {
        const state = layers.map(l => new Uint8ClampedArray(l.data));
        undoStack.push(state);
        if (undoStack.length > 50) undoStack.shift();
        redoStack = [];
    }

    function undo() {
        if (undoStack.length < 2) return;
        const current = layers.map(l => new Uint8ClampedArray(l.data));
        redoStack.push(current);
        const state = undoStack.pop();
        layers.forEach((l, i) => l.data.set(state[i]));
        drawCanvas();
        updatePreview();
    }

    function redo() {
        if (redoStack.length === 0) return;
        const current = layers.map(l => new Uint8ClampedArray(l.data));
        undoStack.push(current);
        const state = redoStack.pop();
        layers.forEach((l, i) => l.data.set(state[i]));
        drawCanvas();
        updatePreview();
    }

    function fillArea(x, y) {
        const data = layers[activeLayer].data;
        const [r, g, b, a] = rgbaToUint8(currentColor);
        const targetIdx = (y * CANVAS_W + x) * 4;
        const targetR = data[targetIdx], targetG = data[targetIdx+1], targetB = data[targetIdx+2], targetA = data[targetIdx+3];
        if (targetR === r && targetG === g && targetB === b && targetA === a) return;

        const visited = new Set();
        const stack = [[x, y]];

        while (stack.length > 0) {
            const [cx, cy] = stack.pop();
            const key = `${cx},${cy}`;
            if (visited.has(key)) continue;
            visited.add(key);

            const idx = (cy * CANVAS_W + cx) * 4;
            if (data[idx] !== targetR || data[idx+1] !== targetG || data[idx+2] !== targetB || data[idx+3] !== targetA) continue;

            data[idx] = r; data[idx+1] = g; data[idx+2] = b; data[idx+3] = a;

            if (cx > 0) stack.push([cx-1, cy]);
            if (cx < CANVAS_W-1) stack.push([cx+1, cy]);
            if (cy > 0) stack.push([cx, cy-1]);
            if (cy < CANVAS_H-1) stack.push([cx, cy+1]);
        }
    }

    function handleCanvasClick(e) {
        const { x, y } = getPixel(e);
        if (currentTool === "fill") {
            saveState();
            fillArea(x, y);
            drawCanvas();
            updatePreview();
        }
    }

    function clearLayer() {
        saveState();
        fillLayer(layers[activeLayer].data, [0, 0, 0, 0]);
        drawCanvas();
        updatePreview();
    }

    function exportPNG() {
        const merged = mergeLayers();
        const exportCanvas = document.createElement('canvas');
        exportCanvas.width = CANVAS_W;
        exportCanvas.height = CANVAS_H;
        const ectx = exportCanvas.getContext('2d');
        const imageData = ectx.createImageData(CANVAS_W, CANVAS_H);
        imageData.data.set(merged);
        ectx.putImageData(imageData, 0, 0);

        const link = document.createElement('a');
        link.download = 'amt-cape.png';
        link.href = exportCanvas.toDataURL('image/png');
        link.click();
    }

    let show3DPreview = false;
    let previewScale = 1;

    const TEMPLATES = [
        { name: "Classic AMT", colors: ["#ACC4DE", "#88AACC", "#6688AA", "#FFFFFF"] },
        { name: "Flame", colors: ["#FF6B35", "#FF8C42", "#FFD166", "#FFFFFF"] },
        { name: "Ocean", colors: ["#0077B6", "#00B4D8", "#90E0EF", "#FFFFFF"] },
        { name: "Night Sky", colors: ["#0F0C29", "#302B63", "#24243E", "#FFFFFF"] },
        { name: "Forest", colors: ["#2D6A4F", "#40916C", "#52B788", "#95D5B2"] },
    ];

    function loadTemplate(template) {
        saveState();
        const data = layers[activeLayer].data;
        fillLayer(data, [0, 0, 0, 0]);

        const [c1, c2, c3, c4] = template.colors.map(c => rgbaToUint8(c));
        // Fill top-left quadrant with c1
        for (let y = 0; y < 16; y++) {
            for (let x = 0; x < 32; x++) {
                const idx = (y * CANVAS_W + x) * 4;
                if (x < 22) {
                    data[idx] = c1[0]; data[idx+1] = c1[1]; data[idx+2] = c1[2]; data[idx+3] = 200;
                } else {
                    data[idx] = c2[0]; data[idx+1] = c2[1]; data[idx+2] = c2[2]; data[idx+3] = 200;
                }
            }
        }
        // Fill bottom section with gradient
        for (let y = 16; y < 32; y++) {
            for (let x = 0; x < CANVAS_W; x++) {
                const idx = (y * CANVAS_W + x) * 4;
                const t = (y - 16) / 16;
                const r = Math.round(c3[0] * t + c4[0] * (1 - t));
                const g = Math.round(c3[1] * t + c4[1] * (1 - t));
                const b = Math.round(c3[2] * t + c4[2] * (1 - t));
                data[idx] = r; data[idx+1] = g; data[idx+2] = b; data[idx+3] = 200;
            }
        }
        // Draw a simple emblem in the center (top section)
        const cx = 20, cy = 8;
        for (let dy = -3; dy <= 3; dy++) {
            for (let dx = -3; dx <= 3; dx++) {
                if (Math.abs(dx) + Math.abs(dy) > 3) continue;
                const idx = ((cy + dy) * CANVAS_W + (cx + dx)) * 4;
                data[idx] = 255; data[idx+1] = 255; data[idx+2] = 255; data[idx+3] = 230;
            }
        }

        drawCanvas();
        updatePreview();
    }

    let publishModal = false;
    let publishTitle = "";
    let publishStatus = "";
    let publishing = false;

    function getCapeBase64() {
        const merged = mergeLayers();
        const exportCanvas = document.createElement('canvas');
        exportCanvas.width = CANVAS_W;
        exportCanvas.height = CANVAS_H;
        const ectx = exportCanvas.getContext('2d');
        const imageData = ectx.createImageData(CANVAS_W, CANVAS_H);
        imageData.data.set(merged);
        ectx.putImageData(imageData, 0, 0);
        return exportCanvas.toDataURL('image/png').split(',')[1];
    }

    async function publishToGallery() {
        if (!publishTitle.trim() || !githubToken) return;
        publishing = true;
        publishStatus = "Publishing...";

        try {
            const capeData = getCapeBase64();
            const capeId = "cape_" + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);

            await invoke("github_dispatch", {
                token: githubToken,
                eventType: "upload-cape",
                payload: {
                    cape_id: capeId,
                    cape_title: publishTitle.trim(),
                    cape_data: capeData,
                    author: githubUser,
                    user_uuid: capeId,
                    animated: false,
                    frame_count: 1,
                    frame_delay: 0,
                },
                repoOwner: githubUser,
                repoName: githubRepo,
            });

            publishStatus = "✓ Published! Gallery will update in ~1 min.";
            setTimeout(() => { publishModal = false; publishStatus = ""; }, 2000);
        } catch (e) {
            publishStatus = "✗ Failed: " + e;
        }
        publishing = false;
    }

    function importPNG(e) {
        const file = e.target.files[0];
        if (!file) return;
        const reader = new FileReader();
        reader.onload = (ev) => {
            const img = new Image();
            img.onload = () => {
                const tempCanvas = document.createElement('canvas');
                tempCanvas.width = CANVAS_W;
                tempCanvas.height = CANVAS_H;
                const tctx = tempCanvas.getContext('2d');
                tctx.drawImage(img, 0, 0, CANVAS_W, CANVAS_H);
                const imageData = tctx.getImageData(0, 0, CANVAS_W, CANVAS_H);
                saveState();
                layers[activeLayer].data.set(imageData.data);
                drawCanvas();
                updatePreview();
            };
            img.src = ev.target.result;
        };
        reader.readAsDataURL(file);
    }

    function updateHexColor(e) {
        const val = e.target.value;
        if (/^#[0-9A-Fa-f]{6}$/.test(val)) {
            currentColor = val;
            hexInput = val;
        }
    }

    let preview3DCanvas;
    let preview3DCtx;
    let rotationAngle = 0;
    let animFrame = null;
    let skinLoaded = false;
    let skinImage = null;
    let isDragging3D = false;
    let dragStartX = 0;
    let skinLoadAttempted = false;

    function init3DPreview() {
        if (!preview3DCanvas) return;
        preview3DCtx = preview3DCanvas.getContext('2d');
        if (!skinLoadAttempted) loadSkin();
        animate3D();
    }

    function loadSkin() {
        skinLoadAttempted = true;
        skinImage = new Image();
        skinImage.crossOrigin = "anonymous";
        skinImage.onload = () => { skinLoaded = true; };
        skinImage.onerror = () => {
            // Try minotar fallback
            const fallback = new Image();
            fallback.crossOrigin = "anonymous";
            fallback.onload = () => { skinImage = fallback; skinLoaded = true; };
            fallback.onerror = () => { skinLoaded = false; };
            fallback.src = `https://minotar.net/skin/${playerUuid}`;
        };
        skinImage.src = `https://mc-heads.net/skin/${playerUuid}`;
    }

    $: if (playerUuid && skinLoadAttempted) {
        skinLoadAttempted = false;
        skinLoaded = false;
        if (show3DPreview) loadSkin();
    }

    function animate3D() {
        if (!show3DPreview) { animFrame = null; return; }
        if (!isDragging3D) rotationAngle += 0.006;
        draw3DPreview();
        animFrame = requestAnimationFrame(animate3D);
    }

    function draw3DPreview() {
        const ctx = preview3DCtx;
        const W = preview3DCanvas.width;
        const H = preview3DCanvas.height;
        ctx.clearRect(0, 0, W, H);

        const s = previewScale * 0.7;
        const cx = W / 2;
        const cy = H / 2 + 15 * s;

        // Background glow
        const grad = ctx.createRadialGradient(cx, cy - 20, 5, cx, cy - 20, 120 * s);
        grad.addColorStop(0, `rgba(${hexToRgb(accent)}, 0.08)`);
        grad.addColorStop(0.5, `rgba(${hexToRgb(accent)}, 0.03)`);
        grad.addColorStop(1, 'rgba(0,0,0,0)');
        ctx.fillStyle = grad;
        ctx.fillRect(0, 0, W, H);

        // Ground shadow
        ctx.save();
        const shadowEl = ctx.createRadialGradient(cx, cy + 38 * s, 2, cx, cy + 38 * s, 50 * s);
        shadowEl.addColorStop(0, 'rgba(0,0,0,0.35)');
        shadowEl.addColorStop(1, 'rgba(0,0,0,0)');
        ctx.fillStyle = shadowEl;
        ctx.beginPath();
        ctx.ellipse(cx, cy + 38 * s, 50 * s, 12 * s, 0, 0, Math.PI * 2);
        ctx.fill();
        ctx.restore();

        const merged = mergeLayers();
        const capeCanvas = document.createElement('canvas');
        capeCanvas.width = CANVAS_W;
        capeCanvas.height = CANVAS_H;
        const cctx = capeCanvas.getContext('2d');
        const imgData = cctx.createImageData(CANVAS_W, CANVAS_H);
        imgData.data.set(merged);
        cctx.putImageData(imgData, 0, 0);

        const angle = rotationAngle;
        const walkCycle = Math.sin(angle * 1.2);
        const bodyBob = Math.abs(Math.sin(angle * 0.6)) * 0.5;

        ctx.save();
        ctx.translate(cx, cy + bodyBob);

        // ── BODY ──
        const bw = 8 * s;
        const bh = 12 * s;
        ctx.save();
        ctx.translate(0, -bh * 0.5);

        // Cape (behind body, rendered first)
        ctx.shadowColor = 'rgba(0,0,0,0.2)';
        ctx.shadowBlur = 10 * s;
        ctx.shadowOffsetY = 3 * s;
        ctx.globalAlpha = 0.9;
        ctx.drawImage(capeCanvas, -bw * 0.9, -bh * 0.5 + 2 * s, bw * 1.8, bh * 1.1);
        ctx.globalAlpha = 1;
        ctx.shadowBlur = 0;

        // Body box
        ctx.shadowColor = 'rgba(0,0,0,0.15)';
        ctx.shadowBlur = 6 * s;
        ctx.shadowOffsetY = 2 * s;
        const bodyGrad = ctx.createLinearGradient(-bw, -bh * 0.5, bw, bh * 0.5);
        bodyGrad.addColorStop(0, skinLoaded ? '#6B8BA8' : '#4A6B8A');
        bodyGrad.addColorStop(0.5, skinLoaded ? '#5A7A97' : '#3D5A7A');
        bodyGrad.addColorStop(1, skinLoaded ? '#4A6A87' : '#2D4A6A');
        roundRect(ctx, -bw, -bh * 0.5, bw * 2, bh, 2 * s);
        ctx.fillStyle = bodyGrad;
        ctx.fill();

        // Body accent line
        ctx.strokeStyle = `rgba(${hexToRgb(accent)}, 0.15)`;
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(0, -bh * 0.5 + 2 * s);
        ctx.lineTo(0, bh * 0.5 - 2 * s);
        ctx.stroke();

        // ── ARMS ──
        const aw = 4 * s;
        const ah = 11 * s;
        const armSwing = walkCycle * 0.2;

        ctx.save();
        ctx.translate(-bw - aw * 0.3, -bh * 0.5 + 1 * s);
        ctx.rotate(-armSwing);
        const armL = ctx.createLinearGradient(0, 0, aw, 0);
        armL.addColorStop(0, skinLoaded ? '#5A7A97' : '#3D5A7A');
        armL.addColorStop(1, skinLoaded ? '#4A6A87' : '#2D4A6A');
        roundRect(ctx, 0, 0, aw, ah, 1.5 * s);
        ctx.fillStyle = armL;
        ctx.fill();
        ctx.restore();

        ctx.save();
        ctx.translate(bw - aw * 0.7, -bh * 0.5 + 1 * s);
        ctx.rotate(armSwing);
        const armR = ctx.createLinearGradient(0, 0, aw, 0);
        armR.addColorStop(0, skinLoaded ? '#6B8BA8' : '#4A6B8A');
        armR.addColorStop(1, skinLoaded ? '#5A7A97' : '#3D5A7A');
        roundRect(ctx, 0, 0, aw, ah, 1.5 * s);
        ctx.fillStyle = armR;
        ctx.fill();
        ctx.restore();

        ctx.restore();

        // ── LEGS ──
        const lw = 4 * s;
        const lh = 10 * s;
        const legSwing = walkCycle * 0.2;

        ctx.save();
        ctx.translate(-bw * 0.5 - lw * 0.5, bh * 0.5);
        ctx.rotate(legSwing);
        const legL = ctx.createLinearGradient(0, 0, lw, 0);
        legL.addColorStop(0, skinLoaded ? '#4A5A6A' : '#2D4A6A');
        legL.addColorStop(1, skinLoaded ? '#3A4A5A' : '#1D3A5A');
        roundRect(ctx, 0, 0, lw, lh, 1.5 * s);
        ctx.fillStyle = legL;
        ctx.fill();
        ctx.restore();

        ctx.save();
        ctx.translate(bw * 0.5 - lw * 0.5, bh * 0.5);
        ctx.rotate(-legSwing);
        const legR = ctx.createLinearGradient(0, 0, lw, 0);
        legR.addColorStop(0, skinLoaded ? '#5A6A7A' : '#3D5A7A');
        legR.addColorStop(1, skinLoaded ? '#4A5A6A' : '#2D4A6A');
        roundRect(ctx, 0, 0, lw, lh, 1.5 * s);
        ctx.fillStyle = legR;
        ctx.fill();
        ctx.restore();

        // ── HEAD ──
        ctx.save();
        const hw = 8 * s;
        const hh = 8 * s;
        ctx.translate(0, -bh * 0.5 - hh * 0.5 + 1 * s);
        const headTilt = Math.sin(angle * 0.3) * 0.08;

        ctx.shadowColor = 'rgba(0,0,0,0.2)';
        ctx.shadowBlur = 8 * s;
        ctx.shadowOffsetY = 3 * s;

        ctx.save();
        ctx.rotate(headTilt);

        if (skinLoaded && skinImage) {
            try {
                const faceSize = Math.min(hw * 2, hh * 2);
                ctx.drawImage(skinImage, 8, 8, 8, 8, -hw, -hh, faceSize, faceSize);
                ctx.drawImage(skinImage, 40, 8, 8, 8, -hw, -hh, faceSize, faceSize);
            } catch (e) {
                drawFallbackHead(ctx, hw, hh, s);
            }
        } else {
            drawFallbackHead(ctx, hw, hh, s);
        }

        ctx.restore();

        // Head outline glow
        ctx.shadowBlur = 0;
        ctx.strokeStyle = `rgba(${hexToRgb(accent)}, 0.08)`;
        ctx.lineWidth = 1;
        roundRect(ctx, -hw - 1, -hh - 1, hw * 2 + 2, hh * 2 + 2, 2 * s);
        ctx.stroke();

        ctx.restore();
        ctx.restore();
    }

    function drawFallbackHead(ctx, hw, hh, s) {
        const headGrad = ctx.createLinearGradient(-hw, -hh, hw, hh);
        headGrad.addColorStop(0, '#8B9BA8');
        headGrad.addColorStop(1, '#6B7B88');
        roundRect(ctx, -hw, -hh, hw * 2, hh * 2, 2 * s);
        ctx.fillStyle = headGrad;
        ctx.fill();

        // Face features
        ctx.fillStyle = 'rgba(255,255,255,0.15)';
        roundRect(ctx, -hw * 0.4, -hh * 0.1, hw * 0.3, hh * 0.15, 1);
        ctx.fill();
        roundRect(ctx, hw * 0.1, -hh * 0.1, hw * 0.3, hh * 0.15, 1);
        ctx.fill();
        ctx.fillStyle = 'rgba(255,255,255,0.1)';
        roundRect(ctx, -hw * 0.15, hh * 0.2, hw * 0.3, hh * 0.1, 1);
        ctx.fill();
    }

    function roundRect(ctx, x, y, w, h, r) {
        ctx.beginPath();
        ctx.moveTo(x + r, y);
        ctx.lineTo(x + w - r, y);
        ctx.quadraticCurveTo(x + w, y, x + w, y + r);
        ctx.lineTo(x + w, y + h - r);
        ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
        ctx.lineTo(x + r, y + h);
        ctx.quadraticCurveTo(x, y + h, x, y + h - r);
        ctx.lineTo(x, y + r);
        ctx.quadraticCurveTo(x, y, x + r, y);
        ctx.closePath();
    }

    function hexToRgb(hex) {
        const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex || '#ACC4DE');
        return result ? `${parseInt(result[1], 16)}, ${parseInt(result[2], 16)}, ${parseInt(result[3], 16)}` : '172, 196, 222';
    }

    $: if (show3DPreview) {
        if (preview3DCanvas) init3DPreview();
    } else {
        if (animFrame) { cancelAnimationFrame(animFrame); animFrame = null; }
    }

    onMount(() => {
        initCanvas();
    });
</script>

<div class="cape-designer">
    <div class="editor-header">
        <h3 class="section-title">Cape Designer</h3>
        <div class="toolbar">
            <button
                class="tool-btn"
                class:active={currentTool === "pencil"}
                on:click={() => currentTool = "pencil"}
                title="Pencil"
            ><span class="material-icons-round">brush</span></button>
            <button
                class="tool-btn"
                class:active={currentTool === "eraser"}
                on:click={() => currentTool = "eraser"}
                title="Eraser"
            ><span class="material-icons-round">auto_fix_high</span></button>
            <button
                class="tool-btn"
                class:active={currentTool === "fill"}
                on:click={() => currentTool = "fill"}
                title="Fill"
            ><span class="material-icons-round">format_paint</span></button>
            <div class="separator"></div>
            <button class="tool-btn" on:click={undo} title="Undo"><span class="material-icons-round">undo</span></button>
            <button class="tool-btn" on:click={redo} title="Redo"><span class="material-icons-round">redo</span></button>
            <div class="separator"></div>
            <button class="tool-btn" class:active={symmetry} on:click={() => symmetry = !symmetry} title="Symmetry"><span class="material-icons-round">swap_horiz</span></button>
        </div>
    </div>

    <div class="editor-body">
        <div class="canvas-area">
            <canvas
                bind:this={canvas}
                width={CANVAS_W * SCALE}
                height={CANVAS_H * SCALE}
                on:mousedown={startDraw}
                on:mousemove={paint}
                on:mouseup={stopDraw}
                on:mouseleave={stopDraw}
                on:click={handleCanvasClick}
                class="pixel-canvas"
            ></canvas>
        </div>

        <div class="side-panel">
            <div class="color-section">
                <h4>Color</h4>
                <div class="color-input-row">
                    <input
                        type="color"
                        value={currentColor}
                        on:input={(e) => { currentColor = e.target.value; hexInput = e.target.value; }}
                        class="color-picker"
                    />
                    <input
                        class="amt-input hex-input"
                        type="text"
                        bind:value={hexInput}
                        on:change={updateHexColor}
                        placeholder="#ACC4DE"
                    />
                </div>
                <div class="palette">
                    {#each PALETTE as color}
                        <button
                            class="palette-swatch"
                            style="background: {color}"
                            class:active={currentColor === color}
                            on:click={() => { currentColor = color; hexInput = color; }}
                        ></button>
                    {/each}
                </div>
            </div>

            <div class="layers-section">
                <h4>Layers</h4>
                {#each layers as layer, i}
                    <div class="layer-row" class:active={activeLayer === i}>
                        <button
                            class="layer-vis"
                            on:click={() => { layer.visible = !layer.visible; drawCanvas(); updatePreview(); }}
                            ><span class="material-icons-round" style="font-size:14px">{layer.visible ? "visibility" : "visibility_off"}</span></button>
                        <button class="layer-name" on:click={() => activeLayer = i}>
                            {layer.name}
                        </button>
                        {#if i === activeLayer}
                            <button class="layer-clear" on:click={clearLayer}>✕</button>
                        {/if}
                    </div>
                {/each}
            </div>

            <div class="preview-section">
                <h4>Preview</h4>
                <canvas
                    bind:this={previewCanvas}
                    width={22 * 4}
                    height={17 * 4}
                    class="preview-canvas"
                ></canvas>
            </div>

            <div class="guide-section">
                <h4>Cape Layout Guide</h4>
                <div class="guide-diagram">
                    <div class="guide-label guide-back">BACK</div>
                    <div class="guide-label guide-front">FRONT</div>
                </div>
                <p class="guide-text">
                    The cape texture is 64×32 pixels. Only the <strong>22×17</strong> highlighted area is visible in-game.
                    The left half shows on the <strong>back</strong> of your character, the right half on the <strong>front</strong>.
                </p>
                <p class="guide-text">
                    Pixels outside the cape area are not visible — they are dimmed in the editor.
                </p>
            </div>
        </div>
    </div>

    {#if show3DPreview}
        <div class="preview-3d-container">
            <canvas
                bind:this={preview3DCanvas}
                width={220 * previewScale}
                height={280 * previewScale}
                class="preview-3d-canvas"
                on:mousedown={(e) => { isDragging3D = true; dragStartX = e.clientX; }}
                on:mousemove={(e) => { if (isDragging3D) { rotationAngle += (e.clientX - dragStartX) * 0.01; dragStartX = e.clientX; } }}
                on:mouseup={() => { isDragging3D = false; }}
                on:mouseleave={() => { isDragging3D = false; }}
            ></canvas>
            <div class="preview-3d-controls">
                <button class="preview-3d-btn" on:click={() => { previewScale = Math.min(previewScale * 1.5, 3); if (show3DPreview) draw3DPreview(); }} title="Zoom In">+</button>
                <button class="preview-3d-btn" on:click={() => { previewScale = Math.max(previewScale / 1.5, 0.5); if (show3DPreview) draw3DPreview(); }} title="Zoom Out">−</button>
                <span class="preview-3d-hint">Drag to rotate</span>
            </div>
        </div>
    {/if}

    <div class="editor-footer">
        <div class="footer-left">
            <label class="amt-btn amt-btn-secondary">
                Import PNG
                <input type="file" accept="image/png" hidden on:change={importPNG} />
            </label>
            <div class="template-picker">
                <span class="template-label">Template:</span>
                {#each TEMPLATES as tpl}
                    <button class="template-btn" style="background: {tpl.colors[0]}" title={tpl.name}
                        on:click={() => loadTemplate(tpl)}></button>
                {/each}
            </div>
        </div>
        <div class="footer-right">
            <button class="amt-btn amt-btn-primary" on:click={exportPNG}>Export Cape</button>
            <button class="amt-btn amt-btn-accent" on:click={() => show3DPreview = !show3DPreview}>
                {show3DPreview ? "Hide 3D" : "3D Preview"}
            </button>
            {#if githubToken}
                <button class="amt-btn amt-btn-secondary" on:click={() => { publishTitle = ""; publishStatus = ""; publishModal = true; }}>
                    Publish
                </button>
            {/if}
        </div>
    </div>
</div>

{#if publishModal}
    <div class="amt-modal-overlay" role="dialog" aria-modal="true" on:click={() => { if (!publishing) publishModal = false; }} on:keydown={(e) => e.key === 'Escape' && (publishModal = false)}>
        <div class="amt-modal" style="max-width: 420px; max-height: auto;" on:click|stopPropagation>
            <div class="amt-modal-header">
                <h2>Publish Cape to Gallery</h2>
                <button class="amt-modal-close" on:click={() => publishModal = false}>✕</button>
            </div>
            <div class="amt-modal-body">
                <div class="publish-form">
                    <input class="amt-input" type="text" bind:value={publishTitle} placeholder="Cape title..." disabled={publishing} />
                    {#if publishStatus}
                        <div class="publish-status">{publishStatus}</div>
                    {/if}
                    <div class="modal-actions">
                        <button class="amt-btn amt-btn-secondary" on:click={() => publishModal = false} disabled={publishing}>Cancel</button>
                        <button class="amt-btn amt-btn-primary" on:click={publishToGallery} disabled={publishing || !publishTitle.trim()}>
                            {publishing ? "Publishing..." : "Publish"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    .cape-designer {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .section-title {
        font-size: 16px;
        font-weight: 600;
        color: white;
        margin: 0;
    }

    .editor-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .toolbar {
        display: flex;
        align-items: center;
        gap: 4px;
        background: rgba(0,0,0,0.3);
        padding: 4px 8px;
        border-radius: 8px;
    }

    .tool-btn {
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: none;
        border: 1px solid transparent;
        border-radius: 6px;
        cursor: pointer;
        font-size: 15px;
        color: white;
        transition: all 0.15s;
    }

    .tool-btn:hover {
        background: rgba(255,255,255,0.1);
    }

    .tool-btn.active {
        background: rgba(172,196,222,0.2);
        border-color: var(--amt-accent);
    }

    .separator {
        width: 1px;
        height: 20px;
        background: rgba(255,255,255,0.1);
        margin: 0 4px;
    }

    .editor-body {
        display: flex;
        gap: 20px;
    }

    .canvas-area {
        flex-shrink: 0;
    }

    .pixel-canvas {
        border-radius: 8px;
        cursor: crosshair;
        image-rendering: pixelated;
        border: 1px solid rgba(255,255,255,0.1);
    }

    .side-panel {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 16px;
        min-width: 200px;
    }

    .color-section h4,
    .layers-section h4,
    .preview-section h4 {
        color: rgba(255,255,255,0.6);
        font-size: 12px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin: 0 0 8px 0;
    }

    .color-input-row {
        display: flex;
        gap: 8px;
        align-items: center;
        margin-bottom: 8px;
    }

    .color-picker {
        width: 36px;
        height: 36px;
        padding: 0;
        border: 1px solid rgba(255,255,255,0.1);
        border-radius: 6px;
        cursor: pointer;
        background: none;
    }

    .color-picker::-webkit-color-swatch-wrapper { padding: 2px; }
    .color-picker::-webkit-color-swatch { border-radius: 4px; border: none; }

    .hex-input {
        flex: 1;
        font-family: var(--amt-font-mono);
        font-size: 12px;
    }

    .palette {
        display: flex;
        flex-wrap: wrap;
        gap: 3px;
    }

    .palette-swatch {
        width: 20px;
        height: 20px;
        border-radius: 4px;
        border: 1px solid rgba(255,255,255,0.1);
        cursor: pointer;
        transition: transform 0.15s;
    }

    .palette-swatch:hover {
        transform: scale(1.2);
    }

    .palette-swatch.active {
        border-color: white;
        box-shadow: 0 0 0 2px var(--amt-accent);
    }

    .layer-row {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 4px 8px;
        border-radius: 6px;
        background: rgba(255,255,255,0.03);
        cursor: pointer;
    }

    .layer-row.active {
        background: rgba(172,196,222,0.1);
        border: 1px solid rgba(172,196,222,0.2);
    }

    .layer-vis {
        background: none;
        border: none;
        cursor: pointer;
        font-size: 12px;
        padding: 2px;
    }

    .layer-name {
        flex: 1;
        background: none;
        border: none;
        color: white;
        font-size: 12px;
        text-align: left;
        cursor: pointer;
        padding: 2px;
    }

    .layer-clear {
        background: none;
        border: none;
        color: rgba(255,255,255,0.4);
        cursor: pointer;
        font-size: 11px;
        padding: 2px;
    }

    .preview-canvas {
        border-radius: 4px;
        background: #1a1a2e;
        image-rendering: pixelated;
        border: 1px solid rgba(255,255,255,0.1);
    }

    .guide-section {
        background: rgba(255,200,100,0.05);
        border: 1px solid rgba(255,200,100,0.15);
        border-radius: 8px;
        padding: 12px;
    }

    .guide-section h4 {
        color: rgba(255,200,100,0.8);
        font-size: 12px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin: 0 0 8px 0;
    }

    .guide-diagram {
        display: flex;
        gap: 0;
        margin-bottom: 8px;
        border-radius: 6px;
        overflow: hidden;
    }

    .guide-label {
        flex: 1;
        padding: 6px 0;
        text-align: center;
        font-size: 11px;
        font-weight: 700;
        letter-spacing: 0.5px;
    }

    .guide-label.guide-back {
        background: rgba(255,200,100,0.1);
        color: rgba(255,200,100,0.7);
    }

    .guide-label.guide-front {
        background: rgba(172,196,222,0.1);
        color: rgba(172,196,222,0.7);
    }

    .guide-text {
        margin: 0 0 4px 0;
        font-size: 11px;
        color: rgba(255,255,255,0.5);
        line-height: 1.4;
    }

    .guide-text strong {
        color: rgba(255,255,255,0.7);
    }

    .preview-3d-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 10px;
        padding: 24px;
        background: rgba(0,0,0,0.2);
        border-radius: 14px;
        border: 1px solid rgba(255,255,255,0.06);
        position: relative;
        overflow: hidden;
    }

    .preview-3d-container::before {
        content: '';
        position: absolute;
        inset: 0;
        background: radial-gradient(ellipse at 50% 30%, rgba(var(--amt-accent-rgb), 0.03) 0%, transparent 70%);
        pointer-events: none;
    }

    .preview-3d-canvas {
        border-radius: 8px;
        image-rendering: auto;
        max-width: 100%;
        cursor: grab;
        position: relative;
        z-index: 1;
    }

    .preview-3d-controls {
        display: flex;
        align-items: center;
        gap: 10px;
        position: relative;
        z-index: 1;
    }

    .preview-3d-btn {
        width: 30px;
        height: 30px;
        border-radius: 50%;
        background: rgba(255,255,255,0.06);
        border: 1px solid rgba(255,255,255,0.1);
        color: white;
        font-size: 16px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.15s;
    }

    .preview-3d-btn:hover {
        background: rgba(var(--amt-accent-rgb), 0.15);
        border-color: rgba(var(--amt-accent-rgb), 0.3);
        color: var(--amt-accent);
    }

    .preview-3d-hint {
        font-size: 10px;
        color: var(--amt-text-muted);
        font-style: italic;
    }

    .editor-footer {
        display: flex;
        justify-content: space-between;
        gap: 10px;
        flex-wrap: wrap;
    }

    .footer-left, .footer-right {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .template-picker {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .template-label {
        font-size: 11px;
        color: var(--amt-text-muted);
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .template-btn {
        width: 22px;
        height: 22px;
        border-radius: 50%;
        border: 2px solid rgba(255,255,255,0.1);
        cursor: pointer;
        transition: all 0.15s;
        padding: 0;
    }

    .template-btn:hover {
        transform: scale(1.3);
        border-color: white;
    }

    .amt-btn-accent {
        background: rgba(96, 182, 117, 0.15);
        color: #60B675;
        border: 1px solid rgba(96, 182, 117, 0.3);
    }

    .amt-btn-accent:hover {
        background: rgba(96, 182, 117, 0.25);
    }

    .publish-form {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .publish-status {
        font-size: 12px;
        padding: 8px 12px;
        border-radius: 8px;
        background: rgba(96,182,117,0.1);
        color: #60B675;
        border: 1px solid rgba(96,182,117,0.2);
    }

    .modal-actions {
        display: flex;
        justify-content: flex-end;
        gap: 8px;
    }
</style>
