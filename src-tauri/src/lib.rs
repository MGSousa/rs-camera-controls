use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use nokhwa::{
    pixel_format::RgbFormat,
    query,
    utils::{ApiBackend, CameraIndex, ControlValueSetter, RequestedFormat, RequestedFormatType},
    Camera,
};
use serde::{de::Error as DeError, Serialize};
use serde_json::{from_value, to_value, Map, Value};

use tauri::{AppHandle, Manager, State, Wry};
use tauri_plugin_log::log;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "settings.json";

fn fetch_cam(index: CameraIndex) -> Result<Camera, nokhwa::NokhwaError> {
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
    return Camera::new(index, requested);
}

struct AppState {
    index: CameraIndex,
}
type CamState = Mutex<AppState>;

#[derive(Debug, Serialize)]
struct Controls {
    zoom: Options,
    tilt: Options,
    pan: Options,
}
#[derive(Debug, Serialize)]
struct Options {
    val: i64,
    max: i64,
    min: i64,
}

fn to_map<T>(value: &T) -> serde_json::Result<BTreeMap<String, serde_json::Value>>
where
    T: Serialize,
{
    match to_value(value)? {
        Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err(serde_json::Error::custom(
            "expected struct-like object from default",
        )),
    }
}

#[tauri::command]
fn default() -> BTreeMap<String, serde_json::Value> {
    let c = Controls {
        zoom: Options {
            min: 100,
            val: 100,
            max: 500,
        },
        tilt: Options {
            min: -10,
            val: 0,
            max: 10,
        },
        pan: Options {
            min: -10,
            val: 0,
            max: 10,
        },
    };
    to_map(&c).unwrap()
}

#[tauri::command]
fn get_store() -> String {
    STORE_FILE.to_string()
}

#[tauri::command]
fn list_cameras() -> Vec<String> {
    let mut vc: Vec<String> = Vec::new();
    let cameras = match query(ApiBackend::Auto) {
        Ok(cameras) => cameras,
        Err(e) => {
            log::error!("Failed to query cameras: {:?}", e);
            panic!()
        }
    };
    let mut i = 0;
    for camera in cameras {
        vc.insert(
            i.clone(),
            format!(
                "{}: {}",
                camera.human_name(),
                camera.description().to_string()
            ),
        );
        i += 1;
    }
    vc
}

#[tauri::command]
fn set_camera(state: State<'_, CamState>, device: i64) -> String {
    let mut state = state.lock().unwrap();
    state.index = CameraIndex::Index(device as u32);

    "OK".to_string()
}

fn title(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[tauri::command]
fn control(state: State<'_, CamState>, method: String, value: i64) -> String {
    let state = state.lock().unwrap();
    match fetch_cam(state.index.clone()) {
        Ok(mut camera) => {
            match camera.camera_controls_string() {
                Ok(v) => match v.get(&title(&method)) {
                    Some(m) => {
                        match camera
                            .set_camera_control(m.control(), ControlValueSetter::Integer(value))
                        {
                            Ok(_) => (),
                            Err(e) => log::error!(
                                "Failed to set action <{}> on device: {:?}",
                                method.clone(),
                                e
                            ),
                        }
                    }
                    None => log::error!(
                        "Method {} not found for device {:?}",
                        method.clone(),
                        state.index.clone()
                    ),
                },
                Err(e) => log::error!("Cannot list device controls: {}", e.to_string()),
            };
            format!("")
        }
        Err(e) => format!("Cannot fetch devices: {}", e.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Error)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let store = app.store(STORE_FILE)?;

            // set default profile
            let mut res = Map::new();
            let default = default();
            for (k, d) in default {
                res.insert(k, Value::from(d.get("val").unwrap().clone()));
            }

            store.set("profile_0", Value::Object(res));

            // set default device
            let mut index = CameraIndex::Index(1);
            let value = store.get("index");
            match value {
                Some(v) => {
                    let c: HashMap<String, u32> = from_value(v).unwrap();
                    index = CameraIndex::Index(*c.get("value").unwrap());
                }
                None => {}
            }

            let state = AppState { index };
            app.manage(Mutex::new(state));

            #[cfg(desktop)]
            {
                use tauri::{
                    image::Image,
                    menu::{MenuBuilder, MenuItem},
                    tray::TrayIconBuilder,
                };

                // Create menu items
                let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
                let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>).unwrap();
                let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>).unwrap();

                // Build menu
                let menu = MenuBuilder::new(app)
                    .item(&open_i)
                    .separator()
                    .item(&hide_i)
                    .separator()
                    .item(&quit_i)
                    .build()
                    .unwrap();

                // Create tray icon
                let _tray = TrayIconBuilder::with_id("tray")
                    .icon(
                        Image::from_bytes(include_bytes!("../icons/32x32.png"))
                            .expect("Failed to load icon"),
                    )
                    .menu(&menu)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "open" => restore(app),
                        "hide" => hide(app),
                        "quit" => app.exit(0),
                        _ => println!("Unhandled menu item: {:?}", event.id),
                    })
                    .build(app)
                    .unwrap();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            control,
            default,
            list_cameras,
            set_camera,
            get_store
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn restore(app: &AppHandle<Wry>) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(err) = window.show() {
            log::error!("Failed to restore the window: {:?}", err);
        }
    } else {
        println!("Main window not found.");
    }
}

fn hide(app: &AppHandle<Wry>) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(err) = window.hide() {
            log::error!("Failed to hide the window: {:?}", err);
        }
    } else {
        println!("Main window not found.");
    }
}
