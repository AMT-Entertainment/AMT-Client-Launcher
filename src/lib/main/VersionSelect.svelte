<script>
  import {createEventDispatcher} from "svelte";
  import {invoke} from "@tauri-apps/api/core";
  import {open as dialogOpen} from "@tauri-apps/plugin-dialog";

  export let options;
  export let versionState = {
    builds: [], branches: [], recommendedMods: [], customMods: [], currentBuild: null
  };

  const dispatch = createEventDispatcher();

  async function deleteMod(event) {
    try {
      await invoke("delete_custom_mod", {
        branch: versionState.currentBuild.branch, mcVersion: versionState.currentBuild.mcVersion,
        modName: `${event.detail.name}.jar`
      });
      dispatch('updateMods');
    } catch (error) {
      console.error("Failed to delete mod:", error);
      alert(`Failed to delete mod: ${error}`);
    }
  }

  async function installMod() {
    try {
      const selected = await dialogOpen({
        directory: false, multiple: true,
        filters: [{ name: "", extensions: ["jar"] }],
        title: "Select a custom mod to install"
      });
      if (selected) {
        for (const file of selected) {
          await invoke("install_custom_mod", {
            branch: versionState.currentBuild.branch, mcVersion: versionState.currentBuild.mcVersion, path: file
          });
        }
        dispatch('updateMods');
      }
    } catch (error) {
      console.error("Failed to install mod:", error);
      alert(`Failed to install mod: ${error}`);
    }
  }

  const BRANCH_LABELS = {
    stable: { label: "Stable", className: "stable" },
    legacy: { label: "Legacy", className: "legacy" },
    nextgen: { label: "Latest", className: "latest" }
  };
</script>

<div class="amt-modal-overlay" role="dialog" aria-modal="true" on:click={() => dispatch('hide')} on:keydown={(e) => e.key === 'Escape' && dispatch('hide')}>
  <div class="version-manager amt-modal" on:click|stopPropagation>
    <div class="amt-modal-header">
      <h2>Version Manager</h2>
      <button class="amt-modal-close" on:click={() => dispatch('hide')}>✕</button>
    </div>

    <div class="amt-modal-body">
      <div class="manager-layout">
        <!-- Left: Version Selection -->
        <div class="manager-sidebar glass-panel">
          <h4 class="manager-section-title">Version</h4>

          <div class="branch-list">
            {#each versionState.branches as branch}
              <button
                class="branch-item"
                class:active={options.version.branchName === branch}
                on:click={async () => {
                  options.version.branchName = branch;
                  options.version.buildId = -1;
                  await options.store();
                  dispatch('updateData');
                }}
              >
                <div class="branch-info">
                  <span class="branch-name">{BRANCH_LABELS[branch]?.label || branch}</span>
                  {#if BRANCH_LABELS[branch]}
                    <span class="branch-tag {BRANCH_LABELS[branch].className}">{BRANCH_LABELS[branch].label}</span>
                  {/if}
                </div>
                {#if options.version.branchName === branch}
                  <span class="branch-check">✓</span>
                {/if}
              </button>
            {/each}
          </div>

          <div class="manager-divider"></div>
          <h4 class="manager-section-title">Build</h4>

          <div class="build-list">
            <button
              class="build-item"
              class:active={options.version.buildId === -1}
              on:click={async () => {
                options.version.buildId = -1;
                await options.store();
                dispatch('updateData');
              }}
            >
              <div class="build-info">
                <span class="build-name">Latest Build</span>
                <span class="build-meta">Auto-selects newest</span>
              </div>
              {#if options.version.buildId === -1}
                <span class="build-check">✓</span>
              {/if}
            </button>
            {#each versionState.builds as build}
              <button
                class="build-item"
                class:active={options.version.buildId === build.buildId}
                on:click={async () => {
                  options.version.buildId = build.buildId;
                  await options.store();
                  dispatch('updateData');
                }}
              >
                <div class="build-info">
                  <span class="build-name">{build.lbVersion}</span>
                  <span class="build-meta">MC {build.mcVersion} · {build.dateDay}</span>
                </div>
                {#if options.version.buildId === build.buildId}
                  <span class="build-check">✓</span>
                {/if}
              </button>
            {/each}
          </div>

          <label class="nightly-toggle">
            <span class="nightly-label">Show nightly builds</span>
            <label class="amt-toggle" on:click={async () => {
              options.launcher.showNightlyBuilds = !options.launcher.showNightlyBuilds;
              await options.store();
              dispatch('updateData');
            }}>
              <input type="checkbox" checked={options.launcher.showNightlyBuilds} />
              <span class="track"></span>
            </label>
          </label>

          <label class="nightly-toggle">
            <span class="nightly-label">Vanilla Mode <span style="font-size:10px;opacity:0.5;">(no mods)</span></span>
            <label class="amt-toggle" on:click={async () => {
              options.launcher.vanillaMode = !options.launcher.vanillaMode;
              await options.store();
            }}>
              <input type="checkbox" checked={options.launcher.vanillaMode} />
              <span class="track"></span>
            </label>
          </label>
        </div>

        <!-- Right: Mod Configuration -->
        <div class="manager-mods">
          <h4 class="manager-section-title">Mod Configuration</h4>
          <div class="mods-scroll">
            {#if versionState.recommendedMods.length > 0}
              <div class="mod-group">
                <h5 class="mod-group-title">Bundled Mods</h5>
                {#each versionState.recommendedMods as mod}
                  <div class="mod-item glass-panel glass-panel-sm">
                    <div class="mod-info">
                      <span class="mod-name">{mod.name}</span>
                      {#if mod.required}
                        <span class="mod-required-badge">Required</span>
                      {/if}
                    </div>
                    <label class="amt-toggle">
                      <input type="checkbox" bind:checked={mod.enabled} disabled={mod.required}
                        on:change={() => dispatch('updateModStates')} />
                      <span class="track"></span>
                    </label>
                  </div>
                {/each}
              </div>
            {/if}

            <div class="mod-group">
              <div class="mod-group-header">
                <h5 class="mod-group-title">Custom Mods</h5>
                <button class="amt-btn amt-btn-secondary" on:click={installMod} style="padding: 6px 14px; font-size: 12px;">
                  + Add Mod
                </button>
              </div>
              {#if versionState.customMods.length === 0}
                <div class="amt-empty-state">
                  <p>No custom mods installed</p>
                  <p style="font-size: 12px; color: var(--amt-text-dim);">Drag & drop .jar files or click "Add Mod"</p>
                </div>
              {:else}
                {#each versionState.customMods as mod}
                  <div class="mod-item glass-panel glass-panel-sm">
                    <div class="mod-info">
                      <span class="mod-name">{mod.name}</span>
                      <span class="mod-local-badge">Local</span>
                    </div>
                    <div class="mod-actions">
                      <label class="amt-toggle">
                        <input type="checkbox" bind:checked={mod.enabled}
                          on:change={() => dispatch('updateModStates')} />
                        <span class="track"></span>
                      </label>
                      <button class="mod-remove-btn" on:click={() => deleteMod({detail: {name: mod.name}})}
                        title="Remove mod">✕</button>
                    </div>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .version-manager {
    max-width: 900px;
    height: 80vh;
  }

  .manager-layout {
    display: flex;
    gap: 20px;
    height: 100%;
  }

  .manager-sidebar {
    width: 280px;
    flex-shrink: 0;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .manager-section-title {
    color: var(--amt-text-secondary);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .manager-divider {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .branch-list, .build-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .branch-item, .build-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-radius: 8px;
    background: none;
    border: 1px solid transparent;
    color: var(--amt-text);
    cursor: pointer;
    transition: all var(--amt-transition-fast);
    text-align: left;
    width: 100%;
    font-family: var(--amt-font-family);
    font-size: 13px;
  }

  .branch-item:hover, .build-item:hover {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.08);
  }

  .branch-item.active, .build-item.active {
    background: rgba(var(--amt-accent-rgb), 0.08);
    border-color: rgba(var(--amt-accent-rgb), 0.2);
  }

  .branch-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .branch-name {
    font-weight: 600;
  }

  .branch-tag {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    font-weight: 600;
    font-family: var(--amt-font-mono);
  }

  .branch-tag.latest {
    background: rgba(var(--amt-accent-rgb), 0.15);
    color: var(--amt-accent);
  }

  .branch-tag.stable {
    background: var(--amt-success-bg);
    color: var(--amt-success);
  }

  .branch-tag.legacy {
    background: rgba(255, 255, 255, 0.06);
    color: var(--amt-text-muted);
  }

  .branch-check, .build-check {
    color: var(--amt-accent);
    font-weight: 700;
  }

  .build-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .build-name {
    font-weight: 500;
  }

  .build-meta {
    font-size: 11px;
    color: var(--amt-text-muted);
  }

  .nightly-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 0;
    cursor: pointer;
  }

  .nightly-label {
    font-size: 12px;
    color: var(--amt-text-secondary);
  }

  /* Mods section */
  .manager-mods {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .mods-scroll {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .mod-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .mod-group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .mod-group-title {
    color: var(--amt-text-secondary);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0;
  }

  .mod-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    gap: 10px;
  }

  .mod-info {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .mod-name {
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mod-required-badge, .mod-local-badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 4px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .mod-required-badge {
    background: var(--amt-warning-bg);
    color: var(--amt-warning);
  }

  .mod-local-badge {
    background: rgba(255, 255, 255, 0.06);
    color: var(--amt-text-muted);
  }

  .mod-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .mod-remove-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: none;
    border: none;
    color: var(--amt-text-muted);
    cursor: pointer;
    border-radius: 4px;
    font-size: 12px;
    transition: all var(--amt-transition-fast);
  }

  .mod-remove-btn:hover {
    color: var(--amt-danger);
    background: var(--amt-danger-bg);
  }
</style>
