<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
    import { resourceDir } from '@tauri-apps/api/path';
    import { readDir } from '@tauri-apps/plugin-fs';
    import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';

    function clamp(value: number, min: number, max: number) {
        return Math.min(Math.max(value, min), max);
    };

    let sounds: any[] = $state([]);

    let resourceDirPath: string;
    let hotkeysDict = $state(new SvelteMap<string, Array<string>>());

    let assinging: string = $state("");
    let setting: string = $state("");

    let disableKey = $state("Grave");
    let pitchUpKey = $state("-");
    let pitchDownKey = $state("+");
    let pitchResetKey = $state("Backspace");

    interface Hotkey {
        name: string;
        keys: Array<string>;
    }

    interface VoiceChangerValues {
        freq: number,
        freqEnab: boolean,
        flang: number,
        flangEnab: boolean,
        bandpass: number,
        bandEnab: boolean
    }

    interface AudioDevices {
        outDevice: string,
        inDevice: string,
        enabled: boolean
    }

    let inputDevices = $state([]);
    let outputDevices = $state([]);
    let currentInput: string = $state("");
    let currentOutput: string = $state("");
    let enabled: boolean = $state(false);
    let defaultsSet = false;

    onMount(async () => {   
        resourceDirPath = await resourceDir();
        sounds = await readDir(resourceDirPath + "\\Sounds");

        invoke("get_hotkeys_loaded_from_file");
        invoke("get_audio_devices");
    })

    function playSound(name: string)
    {
        console.log(name)
        invoke("play_sound", { name: name });
    }

    function assignHotkey(name: string) 
    {
        invoke("assign_hotkey", { name: name })

        if (name != "toggle" && name != "pitchup" && name != "pitchdown" && name != "pitchreset") {
            assinging = assinging == name ? "" : name;
        }
        else
        {
            console.log(name)
            if (name == "toggle") {
                setting = setting == "toggle" ? "" : "toggle";
            }
            else if (name == "pitchup") {
                setting = setting == "pitchup" ? "" : "pitchup";
            }
            else if (name == "pitchdown") {
                setting = setting == "pitchdown" ? "" : "pitchdown";
            }
            else if (name == "pitchreset") {
                setting = setting == "pitchreset" ? "" : "pitchreset";
            }
        }
    }

    // Incoming Events
    listen<Hotkey>('hotkey-assigned', (event) => {
        if (event.payload.name == "toggle") {
            disableKey = event.payload.keys.toString();
            return;
        }
        else if (event.payload.name == "pitchup") {
            pitchUpKey = event.payload.keys.toString();
            return;
        }
        else if (event.payload.name == "pitchdown") {
            pitchDownKey = event.payload.keys.toString();
            return;
        }
        else if (event.payload.name == "pitchreset") {
            pitchResetKey = event.payload.keys.toString();
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

    listen<[]>('pitch-changed', (event) => {
        let pitchInput = document.getElementById("soundPitch") as HTMLInputElement;
        pitchInput.value = event.payload.toString();
    });

    // Outgoing events
    function updateVoiceChanger(e: Event){
        let freqHtml = document.getElementById("freq") as HTMLInputElement;
        let flangHtml = document.getElementById("flang") as HTMLInputElement;
        let bandHtml = document.getElementById("bandpass") as HTMLInputElement;
        let freqTogHtml = document.getElementById("freqtoggle") as HTMLInputElement;
        let flangTogHtml = document.getElementById("flangtoggle") as HTMLInputElement;
        let bandTogHtml = document.getElementById("bandtoggle") as HTMLInputElement;

        const voiceChangerValues: VoiceChangerValues = {
            freq: parseFloat(freqHtml.value),
            freqEnab: freqTogHtml.checked, 
            flang: clamp(parseFloat(flangHtml.value), -0.9, 0.9),
            flangEnab: flangTogHtml.checked, 
            bandpass: parseFloat(bandHtml.value),
            bandEnab: bandTogHtml.checked
        }

        console.log("Update voice changer")
        invoke("update_voice_changer", { value: voiceChangerValues });
    }

    function updateDevices(e: Event) {
        if (defaultsSet) {
            defaultsSet = false;
        };

        let inputHtml = document.getElementById("input") as HTMLInputElement;
        let outputHtml = document.getElementById("output") as HTMLInputElement;
        let voiceEnab = document.getElementById("voiceEnab") as HTMLInputElement

        currentInput = inputHtml.value;
        currentOutput = outputHtml.value;
        enabled = voiceEnab.checked;

        let audioDevices: AudioDevices = {
            outDevice: currentOutput,
            inDevice: currentInput,
            enabled: enabled
        }

        invoke("change_devices", { values: audioDevices });
    }

    function updateSoundPitch(e: Event) {
        let soundPitchHtml = document.getElementById("soundPitch") as HTMLInputElement;
        let pitch = parseFloat(soundPitchHtml.value);

        invoke("update_sound_pitch", { pitch: pitch })
    }
</script>

<style>
    :global(body) {
        background-color: rgb(0, 10, 3);
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

<div class="w-full flex flex-row">
    <div class="w-fit pr-2 flex">
        <div class="bg-emerald-900 w-72 pb-10 flex flex-col items-center pointer-events-auto">
            <div class="w-full flex flex-col items-center">
                <p class="text-white text-center text-lg w-full my-2">Soundboard Disable Key: {disableKey}</p>
                <button onclick={() => assignHotkey("toggle")} class="text-white text-center text-lg w-[90%] h-10 bg-emerald-950 rounded-md">
                    {#if setting == "toggle"}
                        Setting
                    {:else}
                        Set Toggle Key
                    {/if}
                </button>

                <p class="text-white text-center text-lg w-full my-2">Pitch Up Key: {pitchUpKey}</p>
                <button onclick={() => assignHotkey("pitchup")} class="text-white text-center text-lg w-[90%] h-10 bg-emerald-950 rounded-md">
                    {#if setting == "pitchup"}
                        Setting
                    {:else}
                        Set Pitch Up Key
                    {/if}
                </button>

                <p class="text-white text-center text-lg w-full my-2">Pitch Down Key: {pitchDownKey}</p>
                <button onclick={() => assignHotkey("pitchdown")} class="text-white text-center text-lg w-[90%] h-10 bg-emerald-950 rounded-md">
                    {#if setting == "pitchdown"}
                        Setting
                    {:else}
                        Set Pitch Down Key
                    {/if}
                </button>

                <p class="text-white text-center text-lg w-full my-2">Pitch Reset Key: {pitchResetKey}</p>
                <button onclick={() => assignHotkey("pitchreset")} class="text-white text-center text-lg w-[90%] h-10 bg-emerald-950 rounded-md">
                    {#if setting == "pitchreset"}
                        Setting
                    {:else}
                        Set Pitch Reset Key
                    {/if}
                </button>

                <p class="text-white text-center text-lg w-full my-2">Sound Pitch</p>
                <div class="flex flex-row space-x-2 ">
                    <input id="soundPitch" name="soundPitch" class="w-52 h-10 bg-black text-white text-center" readonly={false} defaultValue="1.0" type="number" onchange={(e) => updateSoundPitch(e)}/>
                </div>

                <p class="text-white text-center text-lg w-full my-2">Voice Changer</p>
                <div class="flex flex-row space-x-2 ">
                    <input id="voiceEnab" class="h-10 w-10" type="checkbox" defaultChecked={false} onchange={(e) => updateDevices(e)}/>
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

    <div class="min-h-full w-full flex">
        <div class="p-5 w-full">
            <ul class="w-full grid grid-cols-4 self-center">
                {#each sounds as sound}
                    <li class="justify-items-center">
                        <button onclick={() => playSound(sound.name)} class="my-2 p-4 w-[70%] h-28 bg-[#002010] text-white align-top">
                            {sound.name.split(".")[0]}
                            {#if hotkeysDict.has(sound.name)}
                                :
                                {hotkeysDict.get(sound.name)}
                            {/if}
                        </button>
                        <button onclick={() => assignHotkey(sound.name)} class="my-2 text-lg h-28 w-[25%] bg-[#003015] text-white align-top">            
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
</div>
