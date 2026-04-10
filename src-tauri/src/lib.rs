use ignore::WalkBuilder;

#[tauri::command]
fn verify_bridge(frontend_status: &str) -> String {
    println!("Message received in Rust terminal: {}", frontend_status);
    format!(
        "Backend confirms receipt of: '{}'. Bridge is active!",
        frontend_status
    )
}

#[tauri::command]
fn traverse_directory(dir_path: &str) -> Result<Vec<String>, String> {
    let mut file_paths = Vec::new();

    // Blocklist of extensions to ignore, in addition to files that are found in the .gitignore
    let blocked_extensions = [
        "png", "jpg", "jpeg", "gif", "ico", "svg", "webp", // Images
        "lock", "bin", "exe", "dll", "so", "dylib", // Binaries/Locks
        "woff", "woff2", "ttf", "eot", // Fonts
        "mp4", "webm", "mp3", "wav", // Media
    ];

    let walker = WalkBuilder::new(dir_path)
        .hidden(true)
        .git_ignore(true)
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();

                if path.is_file() {
                    let mut is_blocked = false;

                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if blocked_extensions.contains(&ext) {
                            is_blocked = true;
                        }
                    }

                    if !is_blocked {
                        file_paths.push(path.to_string_lossy().into_owned());
                    }
                }
            }
            Err(err) => {
                println!("Skipped file due to error: {}", err);
            }
        }
    }

    if file_paths.is_empty() {
        return Err(format!("No parsable source files found in {}", dir_path));
    }

    Ok(file_paths)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![verify_bridge, traverse_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
