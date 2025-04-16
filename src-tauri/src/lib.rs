use std::{collections::HashMap, fs::{File, OpenOptions}, sync::Mutex};
use commands::play_sound;
use serde::Serialize;
use soloud::Soloud;
use tauri::{async_runtime::block_on, path::BaseDirectory, Manager};
use mki::{Action, Keyboard};
use webview2_com_sys::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3;
use windows_core::Interface;

mod commands;

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

#[derive(Default)]
struct ActiveInner {
  active: bool,
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

                        
            let mut active_hotkeys: HashMap<String, Vec<Keyboard>> = HashMap::new();

            let path_buf = app.path().resolve("hotkeys.json".to_owned(), BaseDirectory::Resource).expect("Uh oh");
            
            if path_buf.exists() {
                let path = path_buf.to_str().unwrap();
                let file = File::open(path)?;

                let file_size = std::fs::metadata(path).expect("file metadata not found").len();
                if file_size > 0 {
                    println!("Loading from file");
                    active_hotkeys = serde_json::from_reader(file).expect("uh oh");
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

            

            app.manage(Mutex::new(active_hotkeys));  

            // Get the window
            let Some(window) = app.get_webview_window("main") else {
                return Ok(());
            };

            let _ = window.with_webview(|webview| unsafe {
              webview.controller().SetZoomFactor(1.).unwrap();
      
              let icore_webview2 = webview.controller().CoreWebView2().unwrap();
              let icore_webview2_settings = icore_webview2.Settings().unwrap();

              let icore_webview2_settings3 = icore_webview2_settings.cast::<ICoreWebView2Settings3>().unwrap();
              let _ = icore_webview2_settings3.SetAreBrowserAcceleratorKeysEnabled(false);
            }).unwrap();

            let sl = Soloud::default().unwrap();
            app.manage(sl);

            let hotkey_mode = false;
            app.manage(Mutex::new(hotkey_mode));

            let active: ActiveInner = ActiveInner { active: true };
            app.manage(Mutex::new(active));

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
        .invoke_handler(tauri::generate_handler![commands::play_sound, commands::assign_hotkey, commands::get_hotkeys_loaded_from_file])
        .device_event_filter(tauri::DeviceEventFilter::Always)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|handle, events| {
            if matches!(events, tauri::RunEvent::ExitRequested { .. }) {
                let path_buf = handle.path().resolve("hotkeys.json".to_owned(), BaseDirectory::Resource).expect("Uh oh");
                let path = path_buf.to_str().unwrap();
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path).expect("Failed to open/create file.");

                let hotkeys_list = handle.state::<Mutex<HashMap<String, Vec<Keyboard>>>>();
                let hotkeys_list = hotkeys_list.lock().unwrap().clone();
                
                serde_json::to_writer(file, &hotkeys_list).expect("Write to file failed");
            }
        })
}
