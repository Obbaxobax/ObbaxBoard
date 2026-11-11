use std::{collections::HashMap, fs::{File, OpenOptions}, sync::Mutex};
use commands::play_sound;
use serde::{Deserialize, Serialize};
use soloud::Soloud;
use tauri::{Emitter, Manager, async_runtime::{block_on}, path::BaseDirectory};
use mki::{Action, Keyboard};

mod commands;
mod audio_filter;

static LAST_KEY: Mutex<Vec<Keyboard>> = Mutex::new(Vec::new());

pub fn get_last_key() -> Vec<Keyboard> {
    let l_key = LAST_KEY.lock().unwrap();

    //println!("Last key pressed pressed: {:?}", l_key);
    return (&l_key).to_vec();
}

pub fn clear_last_key() {
    let mut l_key = LAST_KEY.lock().unwrap();
    l_key.clear();
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyAssignment<'a> {
  name: &'a str,
  keys: Vec<Keyboard>
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyRemoval<'a> {
    name: &'a str,
}

struct AssignmentActive {
    active: bool,
}

struct ActiveInner {
  active: bool,
}

struct SoundPitchInner {
  pitch: f32,
}

#[derive(Deserialize)]
struct VoiceChangerValues {
    freq: f32,
    freqEnab: bool,
    flang: f32, 
    flangEnab: bool,
    bandpass: f32,
    bandEnab: bool,
}

#[derive(Serialize)]
#[derive(Deserialize)]
#[derive(Clone)]
struct AudioDevices {
    outDevice: String,
    inDevice: String,
    enabled: bool
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let voiceChangerValues: VoiceChangerValues = VoiceChangerValues { 
                freq: 1.0, 
                freqEnab: false, 
                flang: 0.5, 
                flangEnab: false, 
                bandpass: 200.0, 
                bandEnab: false 
            };
            app.manage(Mutex::new(voiceChangerValues));
            
            let sl = Soloud::default().unwrap();
            app.manage(sl);

            // Bool for when a hotkey is being assigned
            let assignmentActive: AssignmentActive = AssignmentActive { active: false };
            app.manage(Mutex::new(assignmentActive));

            // Bool for if the soundboard is active
            let active: ActiveInner = ActiveInner { active: true };
            app.manage(Mutex::new(active));

            // Value of sound effect pitch
            let soundPitch: SoundPitchInner = SoundPitchInner { pitch: 1.0 };
            app.manage(Mutex::new(soundPitch));
            
            // Generate hotkeys hashmap
            // Loaded from file if the file exists, otherwise a default is created
            let mut active_hotkeys: HashMap<String, Vec<Keyboard>> = HashMap::new();

            let path_buf = app.path().resolve("hotkeys.json".to_owned(), BaseDirectory::Resource).expect("Should never error as we are in the base directory");
            
            if path_buf.exists() {
                let path = path_buf.to_str().unwrap();
                let file = File::open(path)?;

                let file_size = std::fs::metadata(path).expect("File exists so metadata should too").len();

                if file_size > 0 {
                    println!("Loading from file");
                    active_hotkeys = serde_json::from_reader(file).expect("Should be json as written by the app");
                    println!("{:?}", active_hotkeys);

                    let hotkeys_clone = active_hotkeys.clone();

                    for (name, keys) in hotkeys_clone {
                        let handle = app.handle().clone();

                        if name == "toggle" {
                            mki::register_hotkey(keys.as_slice(), move || {
                                let active = handle.state::<Mutex<ActiveInner>>();
                                let mut active = active.lock().unwrap();
                
                                active.active = !active.active;
                            });
                        }
                        else if name == "pitchup" {
                            mki::register_hotkey(keys.as_slice(), move || {
                                let pitch = handle.state::<Mutex<SoundPitchInner>>();
                                let mut pitch = pitch.lock().unwrap();
                                pitch.pitch += 0.1;

                                handle.emit("pitch-changed", pitch.pitch).expect("good");
                            });
                        }
                        else if name == "pitchdown" {
                            mki::register_hotkey(keys.as_slice(), move || {
                                let pitch = handle.state::<Mutex<SoundPitchInner>>();
                                let mut pitch = pitch.lock().unwrap();
                                pitch.pitch -= 0.1;

                                handle.emit("pitch-changed", pitch.pitch).expect("good");
                            });
                        }
                        else if &name == "pitchreset" {
                            mki::register_hotkey(keys.as_slice(), move || {
                                let pitch = handle.state::<Mutex<SoundPitchInner>>();
                                let mut pitch = pitch.lock().unwrap();
                                pitch.pitch = 1.0;

                                handle.emit("pitch-changed", pitch.pitch).expect("good");
                            });
                        }
                        else 
                        {
                            mki::register_hotkey(keys.as_slice(),  move || {
                                block_on(play_sound(name.clone(), handle.clone()));
                            });
                        }
                    }
                }
                else 
                {
                    let handle = app.handle().clone();

                    mki::register_hotkey(&[Keyboard::Grave], move || {
                        let active = handle.state::<Mutex<ActiveInner>>();
                        let mut active = active.lock().unwrap();
        
                        active.active = !active.active;
                    });

                    active_hotkeys.insert("toggle".to_owned(), vec![Keyboard::Grave]);
                }             
            }
            else 
            {
                let handle = app.handle().clone();

                mki::register_hotkey(&[Keyboard::Grave], move || {
                    let active = handle.state::<Mutex<ActiveInner>>();
                    let mut active = active.lock().unwrap();
    
                    active.active = !active.active;
                });

                active_hotkeys.insert("toggle".to_owned(), vec![Keyboard::Grave]);
            }                

            app.manage(Mutex::new(active_hotkeys));  

            mki::bind_any_key(Action::callback_kb(|key|
            {
                use Keyboard::*;
                let mut state = LAST_KEY.lock().unwrap();

                if matches!(key, LeftShift | LeftControl | LeftAlt) {
                    let mut key_vec: Vec<Keyboard> = Vec::new();
                    key_vec.push(key);
                    *state = key_vec;
                } else {
                    //println!("Some key pressed pressed: {:?}. Is shift down: {:?}", key,  mki::are_pressed(&[LeftShift]));
                    if !state.is_empty() {
                        if !matches!(state.last().unwrap(), LeftShift | LeftControl | LeftAlt) {
                            state.clear();
                        }
                    }
                    
                    state.push(key);
                }
            }));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::play_sound, commands::assign_hotkey, commands::get_hotkeys_loaded_from_file, commands::update_voice_changer, commands::get_audio_devices, commands::change_devices, commands::update_sound_pitch])
        .device_event_filter(tauri::DeviceEventFilter::Always)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|handle, events| {
            if matches!(events, tauri::RunEvent::ExitRequested { .. }) {
                let path_buf = handle.path().resolve("hotkeys.json".to_owned(), BaseDirectory::Resource).expect("Should never error as we are in the base directory");
                let path = path_buf.to_str().unwrap();
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path).expect("The file should exist");

                let hotkeys_list = handle.state::<Mutex<HashMap<String, Vec<Keyboard>>>>();
                let hotkeys_list = hotkeys_list.lock().unwrap().clone();
                
                serde_json::to_writer(file, &hotkeys_list).expect("File exists and hotkeys_list exists so should not fail");
            }
        })
}
