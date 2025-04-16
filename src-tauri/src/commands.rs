use std::{collections::HashMap, sync::Mutex};
use mki::Keyboard;
use soloud::*;
use tauri::{async_runtime::block_on, path::BaseDirectory, Emitter, Manager};

use crate::{clear_last_key, get_last_key, ActiveInner, HotkeyAssignment, HotkeyRemoval};

#[tauri::command]
pub async fn play_sound(name: String, handle: tauri::AppHandle) {
    let active = handle.state::<Mutex<ActiveInner>>();
    if !active.lock().unwrap().active { return }

    let path_buf = handle.path().resolve("Sounds\\".to_owned() + &name, BaseDirectory::Resource).expect("Uh oh");
    let path = path_buf.to_str().unwrap();

    println!("{}", &path);

    let sl = handle.state::<Soloud>();

    sl.stop_all();

    let mut wav = audio::Wav::default();

    let _ = wav.load(&path);
    sl.play(&wav);
    while sl.active_voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100))
    }
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
    else 
    {
        mki::register_hotkey(prev_key.as_slice(), move || {
            block_on(play_sound(name.clone(), handle.clone()));
        });
    }
}