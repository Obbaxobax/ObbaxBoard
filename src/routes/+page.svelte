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
    let setting: boolean = $state(false)

    let menuOpen = $state(true);

    let disableKey = $state("Grave");

    type Hotkey = {
        name: string;
        keys: Array<string>;
    }

    let voiceChangerValues = [1.0, 0.5]

    let inputDevices = $state([]);
    let outputDevices = $state([]);
    let currentInput: string = $state("");
    let currentOutput: string = $state("");
    let defaultsSet = false;

    onMount(async () => {   
        resourceDirPath = await resourceDir();
        sounds = await readDir(resourceDirPath + "\\Sounds");

        invoke("get_hotkeys_loaded_from_file");
        invoke("get_audio_devices");
    })

    function playSound(name: string)
    {
        
        if (menuOpen) return;
        console.log(name)
        invoke("play_sound", { name: name });
    }

    function assignHotkey(name: string) 
    {
        if (name != "toggle" && menuOpen) return
        invoke("assign_hotkey", { name: name })

        if (name != "toggle") {
            assinging = assinging == name ? "" : name;
        }
        else
        {
            setting = !setting
        }
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

    listen<[]>('input-devices', (event) => { 
        inputDevices = event.payload;
        console.log(inputDevices);
    });

    listen<[]>('output-devices', (event) => {    
        outputDevices = event.payload;
    });

    listen<[]>('default-devices', (event) => {
        if (event.payload.length < 2) return

        // @ts-ignore
        currentInput = event.payload[0];
        defaultsSet = true;

        // @ts-ignore
        currentOutput = event.payload[1];
        defaultsSet = true;
    });

    function updateVoiceChanger(e: Event){
        let freqHtml = document.getElementById("freq") as HTMLInputElement;
        let flangHtml = document.getElementById("flang") as HTMLInputElement;
        let bandHtml = document.getElementById("bandpass") as HTMLInputElement;

        voiceChangerValues[0] = parseFloat(freqHtml.value);
        voiceChangerValues[1] = parseFloat(flangHtml.value);
        voiceChangerValues[2] = parseFloat(bandHtml.value);

        let freqTogHtml = document.getElementById("freqtoggle") as HTMLInputElement;
        let flangTogHtml = document.getElementById("flangtoggle") as HTMLInputElement;
        let bandTogHtml = document.getElementById("bandtoggle") as HTMLInputElement;

        let enabledEffects = [
            freqTogHtml.checked, 
            flangTogHtml.checked, 
            bandTogHtml.checked
        ]

        console.log("Update voice changer")
        invoke("update_voice_changer", { value: voiceChangerValues, enabled: enabledEffects });
    }

    function updateDevices(e: Event) {
        if (defaultsSet) {
            defaultsSet = false;
        };

        let inputHtml = document.getElementById("input") as HTMLInputElement;
        let outputHtml = document.getElementById("output") as HTMLInputElement;

        currentInput = inputHtml.value;
        currentOutput = outputHtml.value;

        let devices = [currentInput, currentOutput];

        console.log("Update devices")
        invoke("change_devices", { values: devices });
    }

    function updateSoundSettings(e: Event) {
        let stackElement = document.getElementById("stackEnab") as HTMLInputElement
        let stackEnabled = stackElement.checked;

        console.log(stackEnabled);
        invoke("update_sound_settings", { enabled: stackEnabled })
    }
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

    input[type="checkbox"]:checked {
        accent-color: var(--color-emerald-950);
        border-color: black;
        border-width: 2px;
        justify-content: center;
    }

    input[type="checkbox"]:checked::before {
        content: '✓'; /* Or use an SVG/image for the checkmark */
        display: block;
        color: white; /* Color of the checkmark */
        font-size: 28px;
        line-height: 36px; /* Adjust to vertically center the checkmark */
        text-align: center;
    }

    input[type="checkbox"] {
        appearance: none;
        border-color: black;
        border-width: 2px;
        background-color: var(--color-emerald-950);
    }
</style>

<div class="w-full h-screen fixed top-0 pointer-events-none {menuOpen ? "blurBackground" : ""}">
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button class="bg-emerald-900 w-10 h-10 fixed top-0 transition pointer-events-auto {menuOpen ? "translate-x-72" : ""}" onclick={() => {menuOpen = !menuOpen}}>
        <!-- svelte-ignore a11y_missing_attribute -->
        <img src="https://www.svgrepo.com/show/506800/burger-menu.svg"/>
    </button>
    <div class="bg-emerald-900 w-72 h-screen fixed top-0 transition flex flex-col items-center pointer-events-auto {menuOpen ? "" : "-translate-x-72"}">
        <div class="w-full flex flex-col items-center">
            <p class="text-white text-center text-lg w-full my-2">Soundboard Disable Key: {disableKey}</p>
            <button onclick={() => assignHotkey("toggle")} class="text-white text-center text-lg w-[90%] h-10 bg-emerald-950 rounded-md">
                {#if setting}
                    Setting
                {:else}
                    Set Toggle Key
                {/if}
            </button>

            <p class="text-white text-center text-lg w-full my-2">Sound Stacking</p>
            <div class="flex flex-row space-x-2 ">
                <input id="stackEnab" class="h-10 w-10" type="checkbox" defaultChecked onchange={(e) => updateSoundSettings(e)}/>
            </div>

            <p class="text-white text-center text-lg w-full my-2">Input Device</p>
            <select value={currentInput} id="input" class="w-52 bg-emerald-900  text-white " onchange={(e) => updateDevices(e)}>
                {#each inputDevices as device}
                    <option class="bg-black text-white" value={device}>{device}</option>
                {/each}
            </select>

            <p class="text-white text-center text-lg w-full my-2">Output Device</p>
            <select value={currentOutput} id="output" class="w-52 bg-emerald-900  text-white " onchange={(e) => updateDevices(e)}>
                {#each outputDevices as device}
                    <option class="bg-black text-white" value={device}>{device}</option>
                {/each}
            </select>
            
            <p class="text-white text-center text-lg w-full my-2">Frequency</p>
            <div class="flex flex-row space-x-2 ">
                <input id="freq" name="freq" class="w-52 h-10 bg-black text-white text-center" readonly={false} defaultValue="1.0" type="number" onchange={(e) => updateVoiceChanger(e)}/>
                <input id="freqtoggle" class="h-10 w-10" type="checkbox" onchange={(e) => updateVoiceChanger(e)}/>
            </div>

            <p class="text-white text-center text-lg w-full my-2">Feedback</p>
            <div class="flex flex-row space-x-2 ">
                <input id="flang" name="flang" class="w-52 h-10 bg-black text-white text-center" readonly={false} defaultValue="0.5" type="number" onchange={(e) => updateVoiceChanger(e)}/>
                <input id="flangtoggle" class="h-10 w-10" type="checkbox" onchange={(e) => updateVoiceChanger(e)}/>
            </div>

            <p class="text-white text-center text-lg w-full my-2">Bandpass</p>
            <div class="flex flex-row space-x-2 ">
                <input id="bandpass" name="bandpass" class="w-52 h-10 bg-black text-white text-center" readonly={false} defaultValue="200.0" type="number" onchange={(e) => updateVoiceChanger(e)}/>
                <input id="bandtoggle" class="h-10 w-10" type="checkbox" onchange={(e) => updateVoiceChanger(e)}/>
            </div>
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
