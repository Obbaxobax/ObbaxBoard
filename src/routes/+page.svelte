<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
    import { resourceDir } from '@tauri-apps/api/path';
    import { readDir } from '@tauri-apps/plugin-fs';
    import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';
    import { slide } from 'svelte/transition';

    let sounds: any[] = $state([]);

    let resourceDirPath: string;
    let hotkeysDict = $state(new SvelteMap<string, Array<string>>());

    let assinging: string = $state("");

    let menuOpen = $state(false);

    let disableKey = $state("Grave");

    type Hotkey = {
        name: string;
        keys: Array<string>;
    }

    onMount(async () => {   
        resourceDirPath = await resourceDir();
        sounds = await readDir(resourceDirPath + "\\Sounds");

        invoke("get_hotkeys_loaded_from_file");
    })

    function playSound(name: string)
    {
        invoke("play_sound", { name: name });
    }

    function assignHotkey(name: string) 
    {
        invoke("assign_hotkey", { name: name })
        assinging = assinging == name ? "" : name;
    }

    listen<Hotkey>('hotkey-assigned', (event) => {
        if (event.payload.name == "toggle") {
            disableKey = event.payload.keys.toString();
            return;
        }

        hotkeysDict.set(event.payload.name, event.payload.keys);
        console.log(hotkeysDict);
    });

    listen<Hotkey>('hotkey-removed', (event) => {
        hotkeysDict.delete(event.payload.name);
        console.log(hotkeysDict);
    });
</script>

<style>
    :global(body) {
        background-color: rgb(0, 10, 3);
    }

    .blurBackground::before {
        content: "";
        background-color: #00000088;
        filter: blur(5px);
        width: 100%;
        height: 100%;
        position: fixed;
        display: block;
        top: 0;
        z-index: -1;
    }

    .transition {
        transition: translate 0.7s ease-in-out;
    }
</style>

<div class="w-full h-screen fixed top-0 pointer-events-none {menuOpen ? "blurBackground" : ""}">
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button class="bg-emerald-900 w-10 h-10 fixed top-0 transition  {menuOpen ? "translate-x-72" : ""}" onclick={() => {menuOpen = !menuOpen}}>
        <!-- svelte-ignore a11y_missing_attribute -->
        <img src="https://upload.wikimedia.org/wikipedia/commons/thumb/b/b2/Hamburger_icon.svg/640px-Hamburger_icon.svg.png"/>
    </button>
    <div class="bg-emerald-900 w-72 h-screen fixed top-0 transition flex flex-col items-center {menuOpen ? "" : "-translate-x-72"}">
        <div class="w-full flex flex-col items-center">
            <p class="text-white text-center text-lg w-full my-2">Soundboard Disable Key: {disableKey}</p>
            <button onclick={() => assignHotkey("toggle")} class="text-white text-center text-lg w-[90%] h-10 bg-emerald-950 rounded-md">Set</button>
        </div>
    </div>
</div>

<div class="mt-10 min-h-full w-full">
    <div class="p-5 w-full">
        <ul class="w-full grid grid-cols-3 self-center">
            {#each sounds as sound}
                <li class="justify-items-center">
                    <button onclick={() => playSound(sound.name)} class="my-2 p-4 w-[70%] h-20 bg-[#002010] text-white align-top">
                        {sound.name.split(".")[0]}
                        {#if hotkeysDict.has(sound.name)}
                            :
                            {hotkeysDict.get(sound.name)}
                        {/if}
                    </button>
                    <button onclick={() => assignHotkey(sound.name)} class="my-2 text-lg h-20 w-[25%] bg-[#003015] text-white align-top">            
                        {#if assinging == sound.name}
                            Assigning
                        {:else}
                            Assign
                        {/if}
                    </button>
                </li>
            {/each}
        </ul>
    </div>
</div>
