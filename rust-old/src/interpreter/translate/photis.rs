use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_photis(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::TaskList { status } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "status", status);
            Ok(mcp_call(
                "photis_list_tasks",
                a,
                format!(
                    "List tasks{}",
                    status
                        .as_ref()
                        .map_or(String::new(), |s| format!(" with status {}", s))
                ),
                PermissionLevel::Safe,
                "Lists tasks from Photis Nadi via MCP bridge".to_string(),
            ))
        }

        Intent::TaskCreate { title, priority } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "title", title);
            insert_opt(&mut a, "priority", priority);
            Ok(mcp_call(
                "photis_create_task",
                a,
                format!("Create task: {}", title),
                PermissionLevel::SystemWrite,
                "Creates a new task in Photis Nadi via MCP bridge".to_string(),
            ))
        }

        Intent::TaskUpdate { task_id, status } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "task_id", task_id);
            insert_opt(&mut a, "status", status);
            Ok(mcp_call(
                "photis_update_task",
                a,
                format!("Update task {}", task_id),
                PermissionLevel::SystemWrite,
                "Updates a task in Photis Nadi via MCP bridge".to_string(),
            ))
        }

        Intent::RitualCheck { date } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "date", date);
            Ok(mcp_call(
                "photis_get_rituals",
                a,
                "Check daily rituals".to_string(),
                PermissionLevel::Safe,
                "Retrieves ritual/habit status from Photis Nadi via MCP bridge".to_string(),
            ))
        }

        Intent::ProductivityStats { period } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "period", period);
            Ok(mcp_call(
                "photis_analytics",
                a,
                format!(
                    "Productivity analytics{}",
                    period
                        .as_ref()
                        .map_or(String::new(), |p| format!(" for {}", p))
                ),
                PermissionLevel::Safe,
                "Retrieves productivity analytics from Photis Nadi via MCP bridge".to_string(),
            ))
        }

        Intent::PhotoisBoards { action, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "photis_boards",
                a,
                format!(
                    "Photis boards: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "list" | "info" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages boards via Photis Nadi".to_string(),
            ))
        }

        Intent::PhotoisNotes { action, content } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "content", content);
            Ok(mcp_call(
                "photis_notes",
                a,
                format!(
                    "Photis notes: {}{}",
                    action,
                    content
                        .as_ref()
                        .map_or(String::new(), |c| format!(" '{}'", c))
                ),
                match action.as_str() {
                    "list" | "get" | "search" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages notes via Photis Nadi".to_string(),
            ))
        }

        _ => unreachable!("translate_photis called with non-photis intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_list_with_status() {
        let intent = Intent::TaskList {
            status: Some("pending".to_string()),
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_list_tasks");
        assert_eq!(mcp.arguments["status"], "pending");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_task_list_without_status() {
        let intent = Intent::TaskList { status: None };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_list_tasks");
        assert!(mcp.arguments.get("status").is_none());
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_task_create_with_priority() {
        let intent = Intent::TaskCreate {
            title: "Fix bug".to_string(),
            priority: Some("high".to_string()),
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_create_task");
        assert_eq!(mcp.arguments["title"], "Fix bug");
        assert_eq!(mcp.arguments["priority"], "high");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_task_create_without_priority() {
        let intent = Intent::TaskCreate {
            title: "Review PR".to_string(),
            priority: None,
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_create_task");
        assert_eq!(mcp.arguments["title"], "Review PR");
        assert!(mcp.arguments.get("priority").is_none());
    }

    #[test]
    fn test_task_update_with_status() {
        let intent = Intent::TaskUpdate {
            task_id: "task-123".to_string(),
            status: Some("done".to_string()),
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_update_task");
        assert_eq!(mcp.arguments["task_id"], "task-123");
        assert_eq!(mcp.arguments["status"], "done");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_task_update_without_status() {
        let intent = Intent::TaskUpdate {
            task_id: "task-456".to_string(),
            status: None,
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_update_task");
        assert_eq!(mcp.arguments["task_id"], "task-456");
        assert!(mcp.arguments.get("status").is_none());
    }

    #[test]
    fn test_ritual_check_with_date() {
        let intent = Intent::RitualCheck {
            date: Some("2026-04-01".to_string()),
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_get_rituals");
        assert_eq!(mcp.arguments["date"], "2026-04-01");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_ritual_check_without_date() {
        let intent = Intent::RitualCheck { date: None };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_get_rituals");
        assert!(mcp.arguments.get("date").is_none());
    }

    #[test]
    fn test_productivity_stats_with_period() {
        let intent = Intent::ProductivityStats {
            period: Some("week".to_string()),
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_analytics");
        assert_eq!(mcp.arguments["period"], "week");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_productivity_stats_without_period() {
        let intent = Intent::ProductivityStats { period: None };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_analytics");
        assert!(mcp.arguments.get("period").is_none());
    }

    #[test]
    fn test_photis_boards_list() {
        let intent = Intent::PhotoisBoards {
            action: "list".to_string(),
            name: None,
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_boards");
        assert_eq!(mcp.arguments["action"], "list");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_photis_boards_info() {
        let intent = Intent::PhotoisBoards {
            action: "info".to_string(),
            name: Some("sprint-1".to_string()),
        };
        let t = translate_photis(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_photis_boards_create() {
        let intent = Intent::PhotoisBoards {
            action: "create".to_string(),
            name: Some("backlog".to_string()),
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_boards");
        assert_eq!(mcp.arguments["action"], "create");
        assert_eq!(mcp.arguments["name"], "backlog");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_photis_notes_list() {
        let intent = Intent::PhotoisNotes {
            action: "list".to_string(),
            content: None,
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_notes");
        assert_eq!(mcp.arguments["action"], "list");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_photis_notes_search() {
        let intent = Intent::PhotoisNotes {
            action: "search".to_string(),
            content: Some("meeting".to_string()),
        };
        let t = translate_photis(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_photis_notes_get() {
        let intent = Intent::PhotoisNotes {
            action: "get".to_string(),
            content: Some("note-id-1".to_string()),
        };
        let t = translate_photis(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_photis_notes_create() {
        let intent = Intent::PhotoisNotes {
            action: "create".to_string(),
            content: Some("Remember to deploy".to_string()),
        };
        let t = translate_photis(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "photis_notes");
        assert_eq!(mcp.arguments["action"], "create");
        assert_eq!(mcp.arguments["content"], "Remember to deploy");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }
}
