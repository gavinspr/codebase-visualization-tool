use ignore::WalkBuilder;
use serde::Serialize;
use std::fs;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

mod language_router;

#[derive(Serialize)]
struct FileNode {
    id: String,
    name: String,
    imports: Vec<String>,
}

#[derive(Serialize)]
struct GraphPayload {
    nodes: Vec<FileNode>,
}

#[tauri::command]
fn map_codebase(dir_path: &str) -> Result<GraphPayload, String> {
    let mut nodes = Vec::new();

    let walker = WalkBuilder::new(dir_path)
        .hidden(true)
        .git_ignore(true)
        .build();

    for result in walker {
        if let Ok(entry) = result {
            let path = entry.path();

            if path.is_file() {
                let path_str = path.to_string_lossy().into_owned();
                let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
                let mut imports = Vec::new();

                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                if let Some(config) = language_router::get_parser_config(ext) {
                    if let Ok(source_code) = fs::read_to_string(path) {
                        let mut parser = Parser::new();
                        if parser.set_language(&config.language).is_ok() {
                            if let Some(tree) = parser.parse(&source_code, None) {
                                if let Ok(query) = Query::new(&config.language, config.import_query)
                                {
                                    let mut cursor = QueryCursor::new();
                                    let mut matches = cursor.matches(
                                        &query,
                                        tree.root_node(),
                                        source_code.as_bytes(),
                                    );

                                    while let Some(m) = matches.next() {
                                        for capture in m.captures {
                                            if let Ok(text) =
                                                capture.node.utf8_text(source_code.as_bytes())
                                            {
                                                imports.push(text.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // We don't have a parser for this file (e.g., .css, .md, .png).
                    // We deliberately DO NOT read the file to avoid crashing on binaries.
                    // It will be added to the graph as an "orphan" node with 0 imports.
                }

                nodes.push(FileNode {
                    id: path_str,
                    name: file_name,
                    imports,
                });
            }
        }
    }

    if nodes.is_empty() {
        return Err("No files found in directory.".to_string());
    }

    Ok(GraphPayload { nodes })
}

#[tauri::command]
fn verify_bridge(frontend_status: &str) -> String {
    println!("Message received in Rust terminal: {}", frontend_status);
    format!(
        "Backend confirms receipt of: '{}'. Bridge is active!",
        frontend_status
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![verify_bridge, map_codebase])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
