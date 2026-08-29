use std::{collections::HashMap, fs::File, io::BufReader, sync::Mutex};
use cpal::traits::{DeviceTrait, HostTrait};
use crossbeam_channel::{Sender, bounded};
use mki::Keyboard;
use rodio::{Decoder, OutputStreamBuilder, Sink};
use tauri::{Emitter, Manager, async_runtime::{block_on, spawn}};

use crate::{ActiveInner, HotkeyAssignment, HotkeyRemoval, SoundPitchInner, StackingInner, audio_filter, clear_last_key, get_last_key};

#[tauri::command]
pub async fn play_sound(name: String, handle: tauri::AppHandle) {
    let active = handle.state::<Mutex<ActiveInner>>();
    if !active.lock().unwrap().active { return }

    let stream = OutputStreamBuilder::open_default_stream()
        .expect("Failed to open default audio stream");

    // 2. Create a Sink which manages playback control (play, pause, volume, etc.).
    let sink = Sink::connect_new(stream.mixer());

    // 3. Load the audio file from disk.
    let file = File::open("Sounds\\".to_owned() + &name)
        .expect("Failed to open audio file");
    
    // Wrap it in a BufReader for more efficient streaming performance.
    let reader = BufReader::new(file);

    // 4. Decode the sound file into a source.
    let source = Decoder::new(reader)
        .expect("Failed to decode audio file");

    // 5. Append the sound to the sink to begin playing it immediately.
    sink.append(source);

    let soundPitch = handle.state::<Mutex<SoundPitchInner>>();
    let mut soundPitch = soundPitch.lock().unwrap();

    sink.set_speed(soundPitch.pitch);

    drop(soundPitch);

    // 6. Block the current thread until the audio finishes playing.
    // Without this, the program will instantly exit and close the audio stream.
    while !sink.empty() {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if !active.lock().unwrap().active {
            sink.stop();
            break;
        }
    }


    /* let active = handle.state::<Mutex<ActiveInner>>();
    if !active.lock().unwrap().active { return }

    let path_buf = handle.path().resolve("Sounds\\".to_owned() + &name, BaseDirectory::Resource).expect("Uh oh");
    let path = path_buf.to_str().unwrap();

    println!("{}", &path);

    let sl = handle.state::<Soloud>();

    let stackingEnabled = handle.state::<Mutex<StackingInner>>();

    match stackingEnabled.try_lock() {
        Ok(guard) => {
            if !guard.active {
                sl.stop_all();
            }
        }
        Err(_) => return
    }

    let stackingEnabled = stackingEnabled.lock().unwrap();

    if !stackingEnabled.active {
        sl.stop_all();
    }

    std::thread::sleep(std::time::Duration::from_millis(200));
    drop(stackingEnabled);

    let mut wav = audio::Wav::default();

    let _ = wav.load(&path);
    sl.play(&wav);
    while sl.active_voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if !active.lock().unwrap().active {
            sl.stop_all();
        }
    } */
}

#[tauri::command]
pub async fn get_hotkeys_loaded_from_file(handle: tauri::AppHandle) {
    let hotkeys = handle.state::<Mutex<HashMap<String, Vec<Keyboard>>>>();
    let hotkeys = hotkeys.lock().unwrap();

    for (name, keys) in hotkeys.iter() {
        handle.emit("hotkey-assigned", HotkeyAssignment { name: &name, keys: keys.clone() }).unwrap();
    }
}

#[tauri::command]
pub async fn assign_hotkey(name: String, handle: tauri::AppHandle) {
    let prev_key = get_last_key();
    
    print!("{name}");
    {
        let mode = handle.state::<Mutex<bool>>();
        let mut mode = mode.lock().unwrap();

        let hotkeys_list = handle.state::<Mutex<HashMap<String, Vec<Keyboard>>>>();
        let mut hotkeys_list = hotkeys_list.lock().unwrap();

        if *mode == false {
            clear_last_key();

            *mode = true;
            return;
        }

        if hotkeys_list.contains_key(&name) {
            let sequence = hotkeys_list[&name].as_slice();
            mki::unregister_hotkey(sequence);
            hotkeys_list.remove(&name);

            handle.emit("hotkey-removed", HotkeyRemoval { name: &name }).unwrap();
        }
            
        *mode = false;

        if prev_key.is_empty() { return }

        let mut temp_keys = prev_key.clone();
        
        if prev_key.contains(&Keyboard::LeftAlt) {
            temp_keys.remove(temp_keys.iter().position(|n| n == &Keyboard::LeftAlt).unwrap());
        }

        hotkeys_list.insert(name.clone(), temp_keys.clone());

        handle.emit("hotkey-assigned", HotkeyAssignment { name: &name, keys: temp_keys.clone() }).unwrap();
    }

    if &name == "toggle" 
    {
        mki::register_hotkey(prev_key.as_slice(), move || {
            let active = handle.state::<Mutex<ActiveInner>>();
            let mut active = active.lock().unwrap();

            active.active = !active.active;
        });
    }
    else if &name == "pitchup" {
        mki::register_hotkey(prev_key.as_slice(), move || {
            let pitch = handle.state::<Mutex<SoundPitchInner>>();
            let mut pitch = pitch.lock().unwrap();
            pitch.pitch += 0.1;

            handle.emit("pitch-changed", pitch.pitch).expect("good");
        });
    }
    else if &name == "pitchdown" {
        mki::register_hotkey(prev_key.as_slice(), move || {
            let pitch = handle.state::<Mutex<SoundPitchInner>>();
            let mut pitch = pitch.lock().unwrap();
            pitch.pitch -= 0.1;

            handle.emit("pitch-changed", pitch.pitch).expect("good");
        });
    }
    else if &name == "pitchreset" {
        mki::register_hotkey(prev_key.as_slice(), move || {
            let pitch = handle.state::<Mutex<SoundPitchInner>>();
            let mut pitch = pitch.lock().unwrap();
            pitch.pitch = 1.0;

            handle.emit("pitch-changed", pitch.pitch).expect("good");
        });
    }
    else 
    {
        mki::register_hotkey(prev_key.as_slice(), move || {
            block_on(play_sound(name.clone(), handle.clone()));
        });
    }
}

#[tauri::command]
pub fn update_voice_changer(value: [f32; 3], enabled: [bool; 3], handle: tauri::AppHandle) {
    let voiceChangerValues = handle.state::<Mutex<[f32; 3]>>();
    let mut voiceChangerValues = voiceChangerValues.lock().unwrap();

    *voiceChangerValues = value;

    let enabledEffects = handle.state::<Mutex<[bool; 3]>>();
    let mut enabledEffects = enabledEffects.lock().unwrap();

    *enabledEffects = enabled;

    let sender = handle.state::<Sender<i16>>();
    sender.send(1).expect("send failledlldlwkflJEKGFHKLUWJHGFE");
}

#[tauri::command]
pub fn update_sound_settings(enabled: bool, handle: tauri::AppHandle) {
    let stackingEnabled = handle.state::<Mutex<StackingInner>>();
    let mut stackingEnabled = stackingEnabled.lock().unwrap();

    stackingEnabled.active = enabled;
}

#[tauri::command]
pub fn update_sound_pitch(pitch: f32, handle: tauri::AppHandle) {
    let soundPitch = handle.state::<Mutex<SoundPitchInner>>();
    let mut soundPitch = soundPitch.lock().unwrap();

    soundPitch.pitch = pitch;
}

#[tauri::command]
pub async fn get_audio_devices(handle: tauri::AppHandle) {
    let host = cpal::default_host();

    let in_devices = host.input_devices();
    let mut input_device_names = Vec::new();

    in_devices.expect("").for_each(|device| {
        input_device_names.push(device.name().expect(""))
    });

    handle.emit("input-devices", input_device_names).unwrap();

    let out_devices = host.output_devices();
    let mut output_device_name = Vec::new();

    out_devices.expect("").for_each(|device| {
        output_device_name.push(device.name().expect(""))
    });

    handle.emit("output-devices", output_device_name).unwrap();

    let out_device = host.default_output_device().unwrap();
    let in_device = host.default_input_device().unwrap();
    let defaults = [in_device.name().expect(""), out_device.name().expect(""), "Disabled".to_owned()];
    handle.emit("default-devices", defaults.clone()).unwrap();


    handle.manage(Mutex::new(defaults));
    println!("defaults set");

    let (s, r) = bounded(1);
    handle.manage(s);

    spawn(audio_filter::start_audio(handle, r));
}

#[tauri::command]
pub async fn change_devices(values: [String; 3], handle: tauri::AppHandle) {
    let defaults = handle.state::<Mutex<[String; 3]>>();
    let mut defaults = defaults.lock().unwrap();

    *defaults = values;

    let sender = handle.state::<Sender<i16>>();
    sender.send(1).expect("send failledlldlwkflJEKGFHKLUWJHGFE");
}