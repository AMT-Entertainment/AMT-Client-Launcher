<script>
    import {fly} from "svelte/transition";
    import { onDestroy } from "svelte";

    const facts = [
        {
            title: "All-in-One Platform",
            description: "Browse, install, and manage mods, shaders, and resource packs from Modrinth—all from one place.",
        },
        {
            title: "Built-in Server Hosting",
            description: "Create Fabric, Paper, or Forge servers with a single click. Full GUI control—no terminal needed.",
        },
        {
            title: "Zero Port Forwarding",
            description: "Share your server with friends instantly. No router config, no port forwarding—just play.",
        },
        {
            title: "Cape & Badge Cosmetics",
            description: "Design your own cape with the pixel-art editor and equip unique badges to stand out in-game.",
        },
        {
            title: "Social Hub",
            description: "Share servers, cosmetics, and posts. DM friends and discover new communities—all inside the launcher.",
        },
        {
            title: "Free & Open Source",
            description: "AMT Client is completely free and open source under GPL-3.0.",
        },
        {
            title: "Smart Mod Packs",
            description: "When joining a friend's server, the correct mods and version install automatically—zero config.",
        },
        {
            title: "Built with Tauri",
            description: "Lightning-fast native performance with a modern Svelte UI. No Electron bloat.",
        },
    ];

    let currentFact = 0;
    let autoAdvance;

    function handlePrevClick(e) {
        if (currentFact <= 0) {
            currentFact = facts.length - 1;
        } else {
            currentFact--;
        }
    }

    function handleNextClick(e) {
        if (currentFact >= facts.length - 1) {
            currentFact = 0;
        } else {
            currentFact++;
        }
    }

    autoAdvance = setInterval(() => {
        handleNextClick();
    }, 5000);

    onDestroy(() => {
        clearInterval(autoAdvance);
    });
</script>

<div class="wrapper">
    {#key currentFact}
        <div class="fact" in:fly={{duration: 200, x: 50}} out:fly={{duration: 200, x: -50}}>
            <div class="title">{facts[currentFact].title}</div>
            <div class="description">{facts[currentFact].description}</div>
            <div class="buttons-wrapper">
                <button
                    type="button"
                    class="button-switch-fact"
                    on:click={handlePrevClick}
                >
                    <img src="img/icon/icon-prev.svg" alt="prev" />
                </button>
                <button
                    type="button"
                    class="button-switch-fact"
                    on:click={handleNextClick}
                >
                    <img src="img/icon/icon-next.svg" alt="next" />
                </button>
            </div>
        </div>
    {/key}
</div>

<style>
    .wrapper {
        position: relative;
        padding: 20px;
    }

    .fact {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .title {
        font-size: 24px;
        color: white;
        font-weight: 700;
    }

    .description {
        font-size: 14px;
        color: rgba(255, 255, 255, 0.55);
        line-height: 1.5;
    }

    .buttons-wrapper {
        display: flex;
        column-gap: 10px;
    }

    .button-switch-fact {
        height: 44px;
        width: 44px;
        border-radius: 50%;
        border: solid 2px rgba(255, 255, 255, 0.5);
        background-color: transparent;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: ease background-color 0.2s;
        cursor: pointer;
    }

    .button-switch-fact:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }
</style>
