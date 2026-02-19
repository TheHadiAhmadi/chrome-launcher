#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, process::Command};

#[derive(Serialize, Deserialize)]
struct Profile {
    folder: String,
    name: String,
    email: Option<String>,
}

#[tauri::command]
fn get_profiles() -> Result<Vec<Profile>, String> {
    let mut path = get_chrome_user_data_dir()?;
    path.push("Local State");

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    if let Some(info_cache) = json["profile"]["info_cache"].as_object() {
        for (folder, data) in info_cache {
            result.push(Profile {
                folder: folder.clone(),
                name: data["name"].as_str().unwrap_or_default().to_string(),
                email: data["user_name"].as_str().map(|s| s.to_string()),
            });
        }
    }

    Ok(result)
}

#[tauri::command]
fn launch_profile(folder: String) -> Result<(), String> {
    let chrome_path = get_chrome_executable()?;

    Command::new(chrome_path)
        .arg(format!("--profile-directory={}", folder))
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_tags() -> Result<HashMap<String, Vec<String>>, String> {
    let path = get_tags_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_tags(tags: HashMap<String, Vec<String>>) -> Result<(), String> {
    let path = get_tags_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(&tags).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

fn get_tags_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap();
    path.push(".chrome-launcher/tags.json");
    path
}

fn get_chrome_user_data_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let mut path =
            dirs::data_local_dir().ok_or("Cannot find local data dir")?;
        path.push("Google/Chrome/User Data");
        Ok(path)
    }

    #[cfg(target_os = "linux")]
    {
        let mut path =
            dirs::home_dir().ok_or("Cannot find home dir")?;
        path.push(".config/google-chrome");
        Ok(path)
    }
}

fn get_chrome_executable() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(PathBuf::from(
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        ))
    }

    #[cfg(target_os = "linux")]
    {
        Ok(PathBuf::from("google-chrome"))
    }
}

fn main() {
    tauri::Builder::default()
        // .plugin(tauri_plugin_autostart::init(
        //     tauri_plugin_autostart::MacosLauncher::LaunchAgent,
        //     None,
        // ))
        // .on_window_event(|window, event| {
        //     if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        //         api.prevent_close();
        //         let _ = window.hide();
        //     }
        // })
        .invoke_handler(tauri::generate_handler![
            get_profiles,
            launch_profile,
            get_tags,
            save_tags
        ])
        .run(tauri::generate_context!())
        .expect("error");
}
