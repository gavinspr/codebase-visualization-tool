use ignore::WalkBuilder;
use serde::Serialize;
use std::fs;
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

mod language_router;

struct RawNode {
    id: String,
    name: String,
    imports: Vec<String>,
}

#[derive(Serialize, Clone)]
struct FileNode {
    id: String,
    name: String,
    imports: Vec<String>,
    in_degree: usize,
    out_degree: usize,
    centrality_role: String,
    is_entry_point: bool,
}

#[derive(Serialize)]
struct GraphPayload {
    nodes: Vec<FileNode>,
}

#[tauri::command]
fn map_codebase(dir_path: &str) -> Result<GraphPayload, String> {
    let mut raw_nodes: Vec<RawNode> = Vec::new();

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

                // Dynamic routing based on extension
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
                }

                raw_nodes.push(RawNode {
                    id: path_str,
                    name: file_name,
                    imports,
                });
            }
        }
    }

    if raw_nodes.is_empty() {
        return Err("No files found in directory.".to_string());
    }

    let mut temp_nodes = Vec::new();
    let mut max_in_degree = 0;
    let mut max_out_degree = 0;

    for node in &raw_nodes {
        let out_degree = node.imports.len();
        let mut in_degree = 0;

        // Get the filename without the extension
        let file_stem = std::path::Path::new(&node.name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Cross-reference against every other node
        for other_node in &raw_nodes {
            if other_node.id == node.id {
                continue;
            }

            if other_node
                .imports
                .iter()
                .any(|import_str| import_str.contains(&file_stem))
            {
                in_degree += 1;
            }
        }

        if in_degree > max_in_degree {
            max_in_degree = in_degree;
        }
        if out_degree > max_out_degree {
            max_out_degree = out_degree;
        }

        // Store the node with its calculated math
        temp_nodes.push((node, in_degree, out_degree));
    }

    // Define dynamic thresholds
    let high_in_threshold = ((max_in_degree as f32 * 0.15).ceil() as usize).max(2);
    let high_out_threshold = ((max_out_degree as f32 * 0.15).ceil() as usize).max(2);

    let mut final_nodes: Vec<FileNode> = Vec::new();

    for (node, in_degree, out_degree) in temp_nodes {
        let is_entry_point = in_degree == 0 && out_degree > 0;

        let centrality_role = if in_degree == 0 && out_degree == 0 {
            "Isolated / Dead Code".to_string()
        } else if in_degree >= high_in_threshold && out_degree == 0 {
            "Primitive / Foundation".to_string()
        } else if in_degree >= high_in_threshold && out_degree >= high_out_threshold {
            "Core Orchestrator".to_string()
        } else if is_entry_point {
            "Application Entry".to_string()
        } else {
            "Standard Node".to_string()
        };

        // Construct the final, serialized node
        final_nodes.push(FileNode {
            id: node.id.clone(),
            name: node.name.clone(),
            imports: node.imports.clone(),
            in_degree,
            out_degree,
            centrality_role,
            is_entry_point,
        });
    }

    Ok(GraphPayload { nodes: final_nodes })
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
