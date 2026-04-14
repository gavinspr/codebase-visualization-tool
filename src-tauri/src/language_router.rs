use tree_sitter::Language;

pub struct ParserConfig {
    pub language: Language,
    pub import_query: &'static str,
}

pub fn get_parser_config(extension: &str) -> Option<ParserConfig> {
    match extension {
        // Pure TypeScript & JavaScript
        "ts" | "js" => {
            let lang: Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
            Some(ParserConfig {
                language: lang,
                import_query: "(import_statement source: (string (string_fragment) @target))",
            })
        }

        // TSX & JSX
        "tsx" | "jsx" => {
            // Note the change here to LANGUAGE_TSX!
            let lang: Language = tree_sitter_typescript::LANGUAGE_TSX.into();
            Some(ParserConfig {
                language: lang,
                import_query: "(import_statement source: (string (string_fragment) @target))",
            })
        }

        // Rust
        "rs" => {
            let lang: Language = tree_sitter_rust::LANGUAGE.into();
            Some(ParserConfig {
                language: lang,
                import_query: "(use_declaration) @target",
            })
        }

        // Java
        "java" => {
            let lang: Language = tree_sitter_java::LANGUAGE.into();
            Some(ParserConfig {
                language: lang,
                import_query: "(import_declaration) @target",
            })
        }

        // Unsupported languages return None. The engine will map the file as a node,
        // but skip the AST parsing step.
        _ => None,
    }
}
