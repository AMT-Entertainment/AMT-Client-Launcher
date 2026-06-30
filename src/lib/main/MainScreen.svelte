<script>
import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import {ask} from "@tauri-apps/plugin-dialog";
import {onMount, onDestroy} from "svelte";
import { scale } from "svelte/transition";
import { t } from "../i18n/index.js";
import MainHeader from "./MainHeader.svelte";
import LaunchArea from "./LaunchArea.svelte";
import NewsArea from "./news/NewsArea.svelte";
import ClientLog from "./log/ClientLog.svelte";
import VersionSelect from "./VersionSelect.svelte";
import VersionWarning from "./VersionWarning.svelte";
import FirstRunWarning from "./FirstRunWarning.svelte";
import CosmeticsHub from "../cosmetics/CosmeticsHub.svelte";
import ModrinthHub from "../modrinth/ModrinthHub.svelte";
import ServerHub from "../server/ServerHub.svelte";
import SocialHub from "../social/SocialHub.svelte";
import AMTSettings from "../settings/AMTSettings.svelte";

export let client;
export let options;
export let error;

const NAV_ITEMS = [
  { id: "home", label: "Home", icon: "⌂" },
  { id: "versions", label: "Versions", icon: "◎" },
  { id: "mods", label: "Mods", icon: "◇" },
  { id: "cosmetics", label: "Cosmetics", icon: "◈" },
  { id: "servers", label: "Servers", icon: "⛁" },
  { id: "social", label: "Social", icon: "☰" },
];

let activeNav = "home";
let running = false;
let logShown = false;
let settingsShown = false;
let versionSelectShown = false;
let launchVersionWarningShown = false;
let firstRunWarningShown = false;
let launchVersionWarningCountdown = 0;
let newsOpen = false;
let log = [];

let versionState = {
  branches: [],
  builds: [],
  currentBuild: null,
  recommendedMods: [],
  customMods: []
};

let progressState = {
  max: 0,
  value: 0,
  text: ""
};

let countdownInterval;
$: if (launchVersionWarningShown && launchVersionWarningCountdown > 0) {
  clearInterval(countdownInterval);
  countdownInterval = setInterval(() => {
    launchVersionWarningCountdown--;
    if (launchVersionWarningCountdown <= 0) clearInterval(countdownInterval);
  }, 1000);
} else if (!launchVersionWarningShown) {
  clearInterval(countdownInterval);
}

$: activeAccount = options.start?.accounts?.[options.start?.activeAccountIndex ?? 0] || null;
$: avatarUuid = activeAccount?.profile?.id || activeAccount?.uuid || 'steve';
$: amt = options?.amt_options || { accent_color: "#ACC4DE", badge: "AMT" };

function updateModStates() {
  const branchName = options.version.branchName;
  if (!options.version.options[branchName]) {
    options.version.options[branchName] = { modStates: {}, customModStates: {} };
  }
  const branchOptions = options.version.options[branchName];
  versionState.recommendedMods.forEach(mod => {
    branchOptions.modStates[mod.name] = mod.enabled;
  });
  versionState.customMods.forEach(mod => {
    branchOptions.customModStates[mod.name] = mod.enabled;
  });
  options.store();
}

async function updateData() {
  let newBuilds;
  try {
    newBuilds = await invoke("request_builds", {
      client, branch: options.version.branchName, release: !options.launcher.showNightlyBuilds
    });
  } catch (e) {
    console.error("Failed to request builds:", e);
    error = { message: "Failed to establish connection with API", error: e };
    return;
  }
  newBuilds.forEach(build => {
    const date = new Date(build.date);
    build.date = date.toLocaleString();
    build.dateDay = date.toLocaleDateString();
  });
  newBuilds.sort((a, b) => b.buildId - a.buildId);
  versionState.builds = newBuilds;
  const buildId = options.version.buildId;
  if (buildId !== -1 && !versionState.builds.find(b => b.buildId === buildId)) {
    options.version.buildId = -1;
    await options.store();
  }
  const activeBuild = buildId === -1 ? versionState.builds[0] : versionState.builds.find(b => b.buildId === buildId);
  if (!activeBuild) return;
  let changelog = null;
  try {
    changelog = await invoke("fetch_changelog", { client, buildId: activeBuild.buildId });
  } catch (e) { console.error("Failed to fetch changelog:", e); }
  versionState.currentBuild = { ...activeBuild, changelog: changelog?.changelog || "" };
  try { await updateMods(); } catch (e) { console.error("Failed to load mods:", e); }
}

async function updateMods() {
  if (!versionState.currentBuild) return;
  const [newRecommendedMods, newCustomMods] = await Promise.all([
    invoke("request_mods", { client, mcVersion: versionState.currentBuild.mcVersion, subsystem: versionState.currentBuild.subsystem }),
    invoke("get_custom_mods", { branch: versionState.currentBuild.branch, mcVersion: versionState.currentBuild.mcVersion })
  ]);
  const branchOptions = options.version.options[versionState.currentBuild.branch];
  if (branchOptions) {
    newRecommendedMods.forEach(mod => { mod.enabled = branchOptions.modStates[mod.name] ?? mod.enabled; });
    newCustomMods.forEach(mod => { mod.enabled = branchOptions.customModStates[mod.name] ?? mod.enabled; });
  }
  versionState.recommendedMods = newRecommendedMods;
  versionState.customMods = newCustomMods;
}

async function runClientWithWarning() {
  if (!versionState.currentBuild) return;
  const isWarning = options.version.branchName === "legacy" ||
    (options.version.branchName === "nextgen" && options.version.buildId !== -1);
  if (isWarning) {
    launchVersionWarningShown = true;
    launchVersionWarningCountdown = 3;
  } else {
    await runClient();
  }
}

async function runClient() {
  if (options.launcher.firstRun) { firstRunWarningShown = true; return; }
  if (running) return;
  try {
    running = true;
    progressState = { max: 0, value: 0, text: "Starting client..." };
    await authenticate();
    await checkMemory();
    await launchClient();
  } catch (error) {
    console.error("Failed to start client:", error);
    log = [...log, `Failed to start client: ${error}`];
    running = false;
    logShown = true;
  }
}

async function authenticate() {
  if (options.premium.account) {
    try {
      progressState.text = "Authenticating account...";
      options.premium.account = await invoke("client_account_update", { client, account: options.premium.account });
    } catch (e) {
      console.error("Failed to authenticate account:", e);
      log = [...log, `Failed to authenticate: ${e}`];
      options.premium.account = null;
    }
  }
  progressState.text = "Refreshing session...";
  try {
    options.start.account = await invoke("refresh", { client, accountData: options.start.account });
  } catch (e) {
    options.start.account = null;
    throw e;
  }
}

async function checkMemory() {
  if (options.start.memory < 4096) {
    const confirmed = await ask(`You are about to launch with less than 4096 MB of memory. Continue?`, { title: "Low Memory Warning", kind: "warning" });
    if (!confirmed) { running = false; throw new Error("Memory warning declined"); }
  }
}

async function launchClient() {
  await options.store();
  await invoke("run_client", {
    client, buildId: versionState.currentBuild.buildId, options,
    mods: [...versionState.recommendedMods, ...versionState.customMods]
  });
}

async function terminateClient() {
  await invoke("terminate");
}

async function continueAfterFirstRun() {
  await hideFirstRunWarning();
  await runClient();
}

async function hideFirstRunWarning() {
  firstRunWarningShown = false;
  if (options.launcher.firstRun) {
    options.launcher.firstRun = false;
    await options.store();
  }
}

async function switchToNextgen() {
  launchVersionWarningShown = false;
  options.version.branchName = "nextgen";
  options.version.buildId = -1;
  await options.store();
  await updateData();
  await runClient();
}

function handleNav(id) {
  if (id === "settings") { settingsShown = true; return; }
  if (id === "versions") { versionSelectShown = true; return; }
  activeNav = id;
}

const unsubs = [];
onMount(async () => {
  const MAX_LOG = 2000;
  const unsub1 = await listen("process-output", (event) => {
    log = [...log.slice(-(MAX_LOG - 1)), event.payload];
  });
  const unsub2 = await listen("progress-update", (event) => {
    const { type, value } = event.payload;
    switch (type) {
      case "max": progressState.max = value; break;
      case "progress": progressState.value = value; break;
      case "label": progressState.text = value; break;
    }
  });
  const unsub3 = await listen("client-exited", () => {
    running = false;
    progressState = { max: 0, value: 0, text: "Done" };
  });
  const unsub4 = await listen("client-error", () => { logShown = true; });
  unsubs.push(unsub1, unsub2, unsub3, unsub4);

  let branchesData;
  try {
    branchesData = await invoke("request_branches", { client });
  } catch (e) {
    console.error("Failed to request branches:", e);
    error = { message: "Failed to establish connection with API", error: e };
    return;
  }
  versionState.branches = branchesData.branches.sort((a, b) =>
    (a === branchesData.defaultBranch ? -1 : b === branchesData.defaultBranch ? 1 : 0));
  if (!options.version.branchName || !versionState.branches.includes(options.version.branchName)) {
    options.version.branchName = branchesData.defaultBranch;
    await options.store();
  }
  await updateData();
  if (options.amt_options?.accent_color) {
    document.documentElement.style.setProperty('--amt-accent', options.amt_options.accent_color);
  }
});

onDestroy(() => {
  unsubs.forEach(fn => { if (typeof fn === 'function') fn(); });
});
</script>

<div class="main-app">
  <!-- Modals -->
  {#if firstRunWarningShown}
    <FirstRunWarning on:hide={hideFirstRunWarning} on:continue={continueAfterFirstRun} />
  {/if}
  {#if launchVersionWarningShown}
    <VersionWarning {launchVersionWarningCountdown}
      on:switchToNextgen={switchToNextgen}
      on:runClientAnyway={async () => { launchVersionWarningShown = false; await runClient(); }}
      on:hide={() => launchVersionWarningShown = false} />
  {/if}
  {#if logShown}
    <ClientLog messages={log} on:hideClientLog={() => logShown = false} />
  {/if}
  {#if settingsShown}
    <AMTSettings {client} bind:options on:hide={async () => { settingsShown = false; await options.store(); }} />
  {/if}
  {#if versionSelectShown}
    <VersionSelect bind:options {versionState}
      on:updateData={updateData} on:updateModStates={updateModStates} on:updateMods={updateMods}
      on:hide={async () => { versionSelectShown = false; await options.store(); }} />
  {/if}

  <!-- Modals for nav features that open as overlays -->
  {#if activeNav === "mods"}
    <div class="amt-modal-overlay" role="dialog" aria-modal="true" on:click={() => activeNav = "home"} on:keydown={(e) => e.key === 'Escape' && (activeNav = "home")}>
      <div class="amt-modal wide" in:scale={{ start: 0.97, duration: 200 }} out:scale={{ start: 0.97, duration: 150 }} on:click|stopPropagation>
        <div class="amt-modal-header">
          <h2>{$t("modrinth.title")}</h2>
          <button class="amt-modal-close" on:click={() => activeNav = "home"}>✕</button>
        </div>
        <div class="amt-modal-body">
          <ModrinthHub initialTab="mods" />
        </div>
      </div>
    </div>
  {/if}
    {#if activeNav === "cosmetics"}
        <div class="amt-modal-overlay" role="dialog" aria-modal="true" on:click={() => activeNav = "home"} on:keydown={(e) => e.key === 'Escape' && (activeNav = "home")}>
          <div class="amt-modal wide" in:scale={{ start: 0.97, duration: 200 }} out:scale={{ start: 0.97, duration: 150 }} on:click|stopPropagation>
            <div class="amt-modal-header">
              <h2>{$t("cosmetics.title")}</h2>
              <button class="amt-modal-close" on:click={() => activeNav = "home"}>✕</button>
            </div>
            <div class="amt-modal-body">
              <CosmeticsHub bind:options playerUuid={avatarUuid} />
            </div>
          </div>
        </div>
    {/if}
  {#if activeNav === "servers"}
    <div class="amt-modal-overlay" role="dialog" aria-modal="true" on:click={() => activeNav = "home"} on:keydown={(e) => e.key === 'Escape' && (activeNav = "home")}>
      <div class="amt-modal wide" in:scale={{ start: 0.97, duration: 200 }} out:scale={{ start: 0.97, duration: 150 }} on:click|stopPropagation>
        <div class="amt-modal-header">
          <h2>{$t("server.title")}</h2>
          <button class="amt-modal-close" on:click={() => activeNav = "home"}>✕</button>
        </div>
        <div class="amt-modal-body">
          <ServerHub />
        </div>
      </div>
    </div>
  {/if}
  {#if activeNav === "social"}
    <div class="amt-modal-overlay" role="dialog" aria-modal="true" on:click={() => activeNav = "home"} on:keydown={(e) => e.key === 'Escape' && (activeNav = "home")}>
      <div class="amt-modal wide" in:scale={{ start: 0.97, duration: 200 }} out:scale={{ start: 0.97, duration: 150 }} on:click|stopPropagation>
        <div class="amt-modal-header">
          <h2>{$t("social.title")}</h2>
          <button class="amt-modal-close" on:click={() => activeNav = "home"}>✕</button>
        </div>
        <div class="amt-modal-body">
          <SocialHub {options} />
        </div>
      </div>
    </div>
  {/if}

  <div class="layout" class:blur={settingsShown || versionSelectShown || logShown || launchVersionWarningShown || firstRunWarningShown || activeNav !== "home"}>
    <!-- Sidebar Navigation -->
    <nav class="sidebar glass-panel">
      <div class="sidebar-header">
        <div class="sidebar-logo">
          <span class="logo-icon">◈</span>
          <span class="logo-text">AMT Client</span>
        </div>
        <span class="sidebar-version">v0.1.0</span>
      </div>

      <div class="sidebar-nav">
        {#each NAV_ITEMS as item}
          <button
            class="amt-nav-item"
            class:active={item.id === activeNav}
            on:click={() => handleNav(item.id)}
          >
            <span class="nav-icon">{item.icon}</span>
            <span>{item.label}</span>
          </button>
        {/each}
      </div>

      <div class="sidebar-footer">
        <button class="amt-nav-item" on:click={() => settingsShown = true}>
          <span class="nav-icon">⚙</span>
          <span>Settings</span>
        </button>
      </div>
    </nav>

    <!-- Main Content -->
    <div class="main-content">
      <MainHeader
        {options}
        account={activeAccount}
        {running}
        {progressState}
        badge={amt.badge}
        accent={amt.accent_color}
        on:showSettings={() => settingsShown = true}
      >
        <img slot="accountSlot"
          src="https://minotar.net/avatar/{avatarUuid}/22"
          class="header-mini-avatar"
          alt="avatar"
        />
      </MainHeader>

      <div class="content-area">
        {#if activeNav === "home"}
          <div class="home-layout">

            <!-- Integrated Account + Launch Area -->
            <LaunchArea
              accountName={activeAccount?.profile?.name || activeAccount?.name || "Guest"}
              accountAvatar={avatarUuid}
              badge={amt.badge}
              versionInfo={{
                bannerUrl: "img/banner.png",
                title: versionState.currentBuild
                  ? `AMT Client ${versionState.currentBuild.lbVersion}`
                  : "Loading...",
                date: versionState.currentBuild?.dateDay || "Loading...",
                description: versionState.currentBuild?.changelog || "Loading..."
              }}
              mcVersion={versionState.currentBuild?.mcVersion || "Loading..."}
              lbVersion={versionState.currentBuild?.lbVersion || "Loading..."}
              canLaunch={!!versionState.currentBuild}
              {running}
              on:showVersionSelect={() => versionSelectShown = true}
              on:showClientLog={() => logShown = true}
              on:launch={runClientWithWarning}
              on:terminate={terminateClient}
            />

            <!-- Quick Actions Row -->
            <div class="quick-actions">
              <button class="qa-item glass-panel glass-panel-sm" on:click={() => activeNav = "mods"}>
                <span class="qa-icon material-icons-round">extension</span>
                <span class="qa-label">Mods</span>
              </button>
              <button class="qa-item glass-panel glass-panel-sm" on:click={() => activeNav = "cosmetics"}>
                <span class="qa-icon material-icons-round">palette</span>
                <span class="qa-label">Cosmetics</span>
              </button>
              <button class="qa-item glass-panel glass-panel-sm" on:click={() => activeNav = "servers"}>
                <span class="qa-icon material-icons-round">dns</span>
                <span class="qa-label">Servers</span>
              </button>
              <button class="qa-item glass-panel glass-panel-sm" on:click={() => activeNav = "social"}>
                <span class="qa-icon material-icons-round">forum</span>
                <span class="qa-label">Social</span>
              </button>
            </div>

            <!-- Collapsible News -->
            <div class="glass-card news-section" class:collapsed={!newsOpen}>
              <button class="news-header" on:click={() => newsOpen = !newsOpen}>
                <span class="news-title">News & Updates</span>
                <span class="news-toggle-icon" class:open={newsOpen}>⌵</span>
              </button>
              {#if newsOpen}
                <div class="news-body">
                  <NewsArea {client}
                    on:navigate={(e) => {
                      const tab = e.detail;
                      if (tab === "mods" || tab === "shaders" || tab === "resourcepacks") {
                        activeNav = "mods";
                      } else if (tab === "server") {
                        activeNav = "servers";
                      } else if (tab === "social") {
                        activeNav = "social";
                      }
                    }} />
                </div>
              {/if}
            </div>
          </div>
        {:else if activeNav === "versions"}
          {#if versionState.builds.length > 0}
            <div class="quick-version-list">
              <h3 class="section-label">Available Versions</h3>
              {#each versionState.builds.slice(0, 10) as build}
                <button class="version-glass-item glass-panel glass-panel-sm" on:click={async () => {
                  options.version.buildId = build.buildId;
                  await options.store();
                  await updateData();
                  activeNav = "home";
                }}>
                  <div class="version-item-info">
                    <span class="version-item-name">{build.lbVersion}</span>
                    <span class="version-item-meta">MC {build.mcVersion} · {build.dateDay}</span>
                  </div>
                  <span class="version-item-arrow">→</span>
                </button>
              {/each}
              <button class="amt-btn amt-btn-ghost" style="margin-top: 8px;" on:click={() => versionSelectShown = true}>
                Open Version Manager →
              </button>
            </div>
          {:else}
            <div class="amt-empty-state">
              <span class="material-icons-round empty-icon" style="font-size: 40px;">update</span>
              <p>No versions available</p>
            </div>
          {/if}
        {:else}
          <div class="amt-empty-state" style="height: 100%;">
            <span class="material-icons-round" style="font-size: 48px; opacity: 0.15;">explore</span>
            <p style="color: var(--amt-text-dim);">Select a feature from the sidebar</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .main-app {
    width: 100%;
    height: 100vh;
    position: relative;
  }

  .layout {
    display: flex;
    height: 100vh;
    transition: filter 0.25s ease;
  }

  .layout.blur {
    filter: blur(4px);
  }

  /* ── Sidebar ── */
  .sidebar {
    width: var(--amt-sidebar-width);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    padding: 16px 10px;
    border-radius: 0;
    border-right: 1px solid rgba(255, 255, 255, 0.06);
    z-index: 10;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    background: rgba(13, 17, 23, 0.7);
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 4px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    margin-bottom: 16px;
  }

  .sidebar-logo {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .logo-icon {
    font-size: 20px;
    color: var(--amt-accent);
  }

  .logo-text {
    font-size: 16px;
    font-weight: 700;
    color: var(--amt-text);
    letter-spacing: 0.5px;
  }

  .sidebar-version {
    font-family: var(--amt-font-mono);
    font-size: 10px;
    color: var(--amt-text-muted);
  }

  .header-mini-avatar {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    object-fit: cover;
    background: rgba(255, 255, 255, 0.08);
  }

  .sidebar-nav {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .sidebar-footer {
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    padding-top: 8px;
    margin-top: 8px;
  }

  /* ── Main Content ── */
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

  .content-area {
    flex: 1;
    overflow-y: auto;
    padding: 24px 32px;
  }

  /* ── Home Layout ── */
  .home-layout {
    display: flex;
    flex-direction: column;
    gap: 14px;
    max-width: 720px;
    margin: 0 auto;
    width: 100%;
  }

  /* Glass Card base */
  .glass-card {
    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 14px;
    padding: 0;
    overflow: hidden;
    transition: all var(--amt-transition-normal);
  }

  .glass-card:hover {
    border-color: rgba(255, 255, 255, 0.12);
  }

  /* ── Quick Actions ── */
  .quick-actions {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 10px;
  }

  .qa-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 18px 8px;
    background: rgba(255,255,255,0.02);
    border: 1px solid rgba(255,255,255,0.06);
    border-radius: 12px;
    color: var(--amt-text);
    cursor: pointer;
    transition: all var(--amt-transition-fast);
    font-family: var(--amt-font-family);
  }

  .qa-item:hover {
    background: rgba(var(--amt-accent-rgb), 0.06);
    border-color: rgba(var(--amt-accent-rgb), 0.15);
    transform: translateY(-1px);
  }

  .qa-icon {
    font-size: 20px;
    width: 42px;
    height: 42px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(var(--amt-accent-rgb), 0.08);
    border-radius: 12px;
    color: var(--amt-accent);
    transition: all var(--amt-transition-fast);
  }

  .qa-item:hover .qa-icon {
    background: rgba(var(--amt-accent-rgb), 0.15);
    transform: scale(1.05);
  }

  .qa-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--amt-text-muted);
  }

  /* ── News Section ── */
  .news-section {
    overflow: hidden;
  }

  .news-section.collapsed {
    background: rgba(255,255,255,0.02);
  }

  .news-section :global(.amt-section-title) {
    display: none;
  }

  .news-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 14px 20px;
    cursor: pointer;
    background: none;
    border: none;
    color: var(--amt-text);
    font-family: var(--amt-font-family);
    transition: background var(--amt-transition-fast);
  }

  .news-header:hover {
    background: rgba(255,255,255,0.03);
  }

  .news-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--amt-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .news-toggle-icon {
    font-size: 14px;
    color: var(--amt-text-dim);
    transition: transform var(--amt-transition-normal);
  }

  .news-toggle-icon.open {
    transform: rotate(180deg);
    color: var(--amt-accent);
  }

  .news-body {
    border-top: 1px solid rgba(255,255,255,0.06);
    padding: 12px 20px 16px;
  }

  .news-body :global(.news-area) {
    display: flex;
    flex-direction: column;
  }

  /* ── Version List ── */
  .section-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--amt-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 12px;
  }

  .quick-version-list {
    max-width: 480px;
    margin: 0 auto;
    width: 100%;
  }

  .version-glass-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 14px 18px;
    margin-bottom: 6px;
    background: rgba(255,255,255,0.02);
    border: 1px solid rgba(255,255,255,0.06);
    border-radius: 10px;
    cursor: pointer;
    color: var(--amt-text);
    font-family: var(--amt-font-family);
    text-align: left;
    transition: all var(--amt-transition-fast);
  }

  .version-glass-item:hover {
    background: rgba(var(--amt-accent-rgb), 0.06);
    border-color: rgba(var(--amt-accent-rgb), 0.15);
  }

  .version-item-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .version-item-name {
    font-size: 14px;
    font-weight: 600;
  }

  .version-item-meta {
    font-size: 11px;
    color: var(--amt-text-muted);
  }

  .version-item-arrow {
    font-size: 16px;
    color: var(--amt-text-dim);
  }
</style>
