<script>
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    const sections = [
        { id: "mods", icon: "extension", label: "Mods" },
        { id: "shaders", icon: "palette", label: "Shaders" },
        { id: "resourcepacks", icon: "grid_view", label: "Packs" },
        { id: "server", icon: "dns", label: "Server" },
        { id: "social", icon: "diversity_3", label: "Social" },
    ];

    function handleSectionClick(sectionId) {
        dispatch("navigate", sectionId);
    }
</script>

<div class="nav-bar">
    {#each sections as section}
        <button
            class="nav-button"
            on:click={() => handleSectionClick(section.id)}
            title={section.label}
        >
            <span class="material-icons-round icon">{section.icon}</span>
            <span class="label">{section.label}</span>
        </button>
    {/each}
</div>

<style>
    .nav-bar {
        background-color: rgba(0, 0, 0, .26);
        border-radius: 6px;
        display: flex;
        width: max-content;
        padding: 8px;
        gap: 8px;
        align-items: center;
    }

    .nav-button {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 4px;
        padding: 8px 16px;
        border-radius: 6px;
        border: none;
        background: transparent;
        color: rgba(255, 255, 255, 0.6);
        cursor: pointer;
        transition: all 0.2s ease;
        font-family: "Inter", sans-serif;
    }

    .nav-button:hover {
        background-color: rgba(255, 255, 255, 0.1);
        color: white;
    }

    .icon {
        font-size: 22px;
    }

    .label {
        font-size: 11px;
        font-weight: 500;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
</style>
