use ignore::WalkBuilder;
use serde::Serialize;
use std::fs;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

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

    let mut parser = Parser::new();
    let ts_lang: tree_sitter::Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
    parser.set_language(&ts_lang).map_err(|_| "Syntax Error")?;

    let query_str = "(import_statement source: (string (string_fragment) @import_target))";
    let query = Query::new(&ts_lang, query_str).map_err(|e| e.to_string())?;
    let mut cursor = QueryCursor::new();

    let walker = WalkBuilder::new(dir_path)
        .hidden(true)
        .git_ignore(true)
        .build();

    for result in walker {
        if let Ok(entry) = result {
            let path = entry.path();

            if path.is_file() {
                // temp: Only parse TypeScript files to match our current grammar
                let is_ts = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "ts" || e == "tsx")
                    .unwrap_or(false);

                if is_ts {
                    let path_str = path.to_string_lossy().into_owned();
                    let file_name = path.file_name().unwrap().to_string_lossy().into_owned();

                    // Read the file
                    if let Ok(source_code) = fs::read_to_string(path) {
                        // Parse the file
                        if let Some(tree) = parser.parse(&source_code, None) {
                            let mut imports = Vec::new();
                            let mut matches =
                                cursor.matches(&query, tree.root_node(), source_code.as_bytes());

                            // Extract the imports
                            while let Some(m) = matches.next() {
                                for capture in m.captures {
                                    if let Ok(text) = capture.node.utf8_text(source_code.as_bytes())
                                    {
                                        imports.push(text.to_string());
                                    }
                                }
                            }

                            // Build Node and add it to the graph
                            nodes.push(FileNode {
                                id: path_str,
                                name: file_name,
                                imports,
                            });
                        }
                    }
                }
            }
        }
    }

    if nodes.is_empty() {
        return Err("No valid TypeScript files found to parse.".to_string());
    }

    // Return the structured payload
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
