use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_mneme(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::MnemeNotebook { action, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "mneme_notebook",
                a,
                format!(
                    "Mneme notebook: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "list" | "info" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} Mneme notebooks via MCP bridge",
                    match action.as_str() {
                        "create" => "Creates",
                        "open" => "Opens",
                        "delete" => "Deletes",
                        "list" => "Lists",
                        _ => "Queries",
                    }
                ),
            ))
        }

        Intent::MnemeNotes {
            action,
            title,
            notebook_id,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "title", title);
            insert_opt(&mut a, "notebook_id", notebook_id);
            Ok(mcp_call(
                "mneme_notes",
                a,
                format!(
                    "Mneme notes: {}{}",
                    action,
                    title
                        .as_ref()
                        .map_or(String::new(), |t| format!(" '{}'", t))
                ),
                match action.as_str() {
                    "list" | "get" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} notes in Mneme via MCP bridge",
                    match action.as_str() {
                        "create" => "Creates",
                        "edit" => "Edits",
                        "delete" => "Deletes",
                        "list" => "Lists",
                        _ => "Manages",
                    }
                ),
            ))
        }

        Intent::MnemeSearch { query, mode } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "query", query);
            insert_opt(&mut a, "mode", mode);
            Ok(mcp_call(
                "mneme_search",
                a,
                format!(
                    "Mneme search: '{}'{}",
                    query,
                    mode.as_ref().map_or(String::new(), |m| format!(" ({})", m))
                ),
                PermissionLevel::Safe,
                "Searches Mneme knowledge base via MCP bridge".to_string(),
            ))
        }

        Intent::MnemeAi { action, note_id } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "note_id", note_id);
            Ok(mcp_call(
                "mneme_ai",
                a,
                format!(
                    "Mneme AI: {}{}",
                    action,
                    note_id
                        .as_ref()
                        .map_or(String::new(), |id| format!(" ({})", id))
                ),
                PermissionLevel::SystemWrite,
                format!("Runs AI {} on Mneme knowledge via MCP bridge", action),
            ))
        }

        Intent::MnemeGraph { action, node_id } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "node_id", node_id);
            Ok(mcp_call(
                "mneme_graph",
                a,
                format!(
                    "Mneme graph: {}{}",
                    action,
                    node_id
                        .as_ref()
                        .map_or(String::new(), |id| format!(" ({})", id))
                ),
                match action.as_str() {
                    "view" | "stats" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages Mneme knowledge graph via MCP bridge".to_string(),
            ))
        }

        Intent::MnemeImport { action, path } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "path", path);
            Ok(mcp_call(
                "mneme_import",
                a,
                format!(
                    "Mneme import: {}{}",
                    action,
                    path.as_ref().map_or(String::new(), |p| format!(" '{}'", p))
                ),
                match action.as_str() {
                    "status" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Imports documents into Mneme".to_string(),
            ))
        }

        Intent::MnemeTags { action, tag } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "tag", tag);
            Ok(mcp_call(
                "mneme_tags",
                a,
                format!(
                    "Mneme tags: {}{}",
                    action,
                    tag.as_ref().map_or(String::new(), |t| format!(" '{}'", t))
                ),
                match action.as_str() {
                    "list" | "search" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages tags via Mneme".to_string(),
            ))
        }

        _ => unreachable!("translate_mneme called with non-mneme intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notebook_list() {
        let intent = Intent::MnemeNotebook {
            action: "list".to_string(),
            name: None,
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "mneme_notebook");
        assert_eq!(mcp.arguments["action"], "list");
        assert!(mcp.arguments.get("name").is_none());
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_notebook_create() {
        let intent = Intent::MnemeNotebook {
            action: "create".to_string(),
            name: Some("research".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["action"], "create");
        assert_eq!(mcp.arguments["name"], "research");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_notebook_info() {
        let intent = Intent::MnemeNotebook {
            action: "info".to_string(),
            name: Some("notes".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_notebook_delete() {
        let intent = Intent::MnemeNotebook {
            action: "delete".to_string(),
            name: Some("old".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_notes_list() {
        let intent = Intent::MnemeNotes {
            action: "list".to_string(),
            title: None,
            notebook_id: Some("nb-1".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "mneme_notes");
        assert_eq!(mcp.arguments["action"], "list");
        assert_eq!(mcp.arguments["notebook_id"], "nb-1");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_notes_create() {
        let intent = Intent::MnemeNotes {
            action: "create".to_string(),
            title: Some("My Note".to_string()),
            notebook_id: Some("nb-2".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["action"], "create");
        assert_eq!(mcp.arguments["title"], "My Note");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_notes_get() {
        let intent = Intent::MnemeNotes {
            action: "get".to_string(),
            title: Some("existing".to_string()),
            notebook_id: None,
        };
        let result = translate_mneme(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_notes_edit() {
        let intent = Intent::MnemeNotes {
            action: "edit".to_string(),
            title: Some("draft".to_string()),
            notebook_id: None,
        };
        let result = translate_mneme(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_notes_delete() {
        let intent = Intent::MnemeNotes {
            action: "delete".to_string(),
            title: None,
            notebook_id: None,
        };
        let result = translate_mneme(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_search_with_mode() {
        let intent = Intent::MnemeSearch {
            query: "rust async".to_string(),
            mode: Some("semantic".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "mneme_search");
        assert_eq!(mcp.arguments["query"], "rust async");
        assert_eq!(mcp.arguments["mode"], "semantic");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_search_without_mode() {
        let intent = Intent::MnemeSearch {
            query: "tokio".to_string(),
            mode: None,
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["query"], "tokio");
        assert!(mcp.arguments.get("mode").is_none());
    }

    #[test]
    fn test_ai_with_note_id() {
        let intent = Intent::MnemeAi {
            action: "summarize".to_string(),
            note_id: Some("note-42".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "mneme_ai");
        assert_eq!(mcp.arguments["action"], "summarize");
        assert_eq!(mcp.arguments["note_id"], "note-42");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_ai_without_note_id() {
        let intent = Intent::MnemeAi {
            action: "generate".to_string(),
            note_id: None,
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert!(mcp.arguments.get("note_id").is_none());
    }

    #[test]
    fn test_graph_view() {
        let intent = Intent::MnemeGraph {
            action: "view".to_string(),
            node_id: Some("node-1".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "mneme_graph");
        assert_eq!(mcp.arguments["action"], "view");
        assert_eq!(mcp.arguments["node_id"], "node-1");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_graph_stats() {
        let intent = Intent::MnemeGraph {
            action: "stats".to_string(),
            node_id: None,
        };
        let result = translate_mneme(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_graph_link() {
        let intent = Intent::MnemeGraph {
            action: "link".to_string(),
            node_id: Some("node-5".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_import_with_path() {
        let intent = Intent::MnemeImport {
            action: "file".to_string(),
            path: Some("/docs/paper.pdf".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "mneme_import");
        assert_eq!(mcp.arguments["action"], "file");
        assert_eq!(mcp.arguments["path"], "/docs/paper.pdf");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_import_status() {
        let intent = Intent::MnemeImport {
            action: "status".to_string(),
            path: None,
        };
        let result = translate_mneme(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tags_list() {
        let intent = Intent::MnemeTags {
            action: "list".to_string(),
            tag: None,
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "mneme_tags");
        assert_eq!(mcp.arguments["action"], "list");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tags_search() {
        let intent = Intent::MnemeTags {
            action: "search".to_string(),
            tag: Some("rust".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tags_add() {
        let intent = Intent::MnemeTags {
            action: "add".to_string(),
            tag: Some("important".to_string()),
        };
        let result = translate_mneme(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["tag"], "important");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }
}
