use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_edge(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::EdgeListNodes { status } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "status", status);
            Ok(mcp_call(
                "edge_list",
                a,
                format!(
                    "List edge nodes{}",
                    status
                        .as_ref()
                        .map_or(String::new(), |s| format!(" ({})", s))
                ),
                PermissionLevel::Safe,
                "Lists edge nodes in the fleet via MCP bridge".to_string(),
            ))
        }

        Intent::EdgeDeploy { task, node } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "task", task);
            insert_opt(&mut a, "node_id", node);
            Ok(mcp_call(
                "edge_deploy",
                a,
                format!("Deploy to edge: {}", task),
                PermissionLevel::SystemWrite,
                "Deploys a task to an edge node via MCP bridge".to_string(),
            ))
        }

        Intent::EdgeUpdate { node, version } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "node_id", node);
            if let Some(v) = version {
                a.insert("version".to_string(), serde_json::Value::String(v.clone()));
            } else {
                a.insert(
                    "version".to_string(),
                    serde_json::Value::String("latest".to_string()),
                );
            }
            Ok(mcp_call(
                "edge_update",
                a,
                format!("Update edge node: {}", node),
                PermissionLevel::SystemWrite,
                "Triggers OTA update on an edge node via MCP bridge".to_string(),
            ))
        }

        Intent::EdgeHealth { node } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "node_id", node);
            Ok(mcp_call(
                "edge_health",
                a,
                format!(
                    "Edge health{}",
                    node.as_ref().map_or(String::new(), |n| format!(": {}", n))
                ),
                PermissionLevel::Safe,
                "Gets edge node health status via MCP bridge".to_string(),
            ))
        }

        Intent::EdgeDecommission { node } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "node_id", node);
            Ok(mcp_call(
                "edge_decommission",
                a,
                format!("Decommission edge node: {}", node),
                PermissionLevel::SystemWrite,
                "Decommissions an edge node via MCP bridge".to_string(),
            ))
        }

        Intent::EdgeLogs { action, node } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "node_id", node);
            Ok(mcp_call(
                "edge_logs",
                a,
                format!(
                    "Edge logs: {}{}",
                    action,
                    node.as_ref().map_or(String::new(), |n| format!(" ({})", n))
                ),
                PermissionLevel::Safe,
                "Queries edge node logs".to_string(),
            ))
        }

        Intent::EdgeConfig { action, node, key } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "node_id", node);
            insert_opt(&mut a, "key", key);
            Ok(mcp_call(
                "edge_config",
                a,
                format!(
                    "Edge config: {}{}",
                    action,
                    key.as_ref().map_or(String::new(), |k| format!(" '{}'", k))
                ),
                match action.as_str() {
                    "get" | "list" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages edge node config".to_string(),
            ))
        }

        _ => unreachable!("translate_edge called with non-edge intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mcp_tool_name(t: &Translation) -> &str {
        t.mcp.as_ref().unwrap().tool_name.as_str()
    }

    fn mcp_args(t: &Translation) -> &serde_json::Value {
        &t.mcp.as_ref().unwrap().arguments
    }

    #[test]
    fn test_edge_list_nodes_no_filter() {
        let intent = Intent::EdgeListNodes { status: None };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_list");
        assert!(mcp_args(&t).get("status").is_none());
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_edge_list_nodes_with_status() {
        let intent = Intent::EdgeListNodes {
            status: Some("online".to_string()),
        };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_list");
        assert_eq!(mcp_args(&t)["status"], "online");
    }

    #[test]
    fn test_edge_deploy() {
        let intent = Intent::EdgeDeploy {
            task: "inference-v2".to_string(),
            node: Some("node-3".to_string()),
        };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_deploy");
        assert_eq!(mcp_args(&t)["task"], "inference-v2");
        assert_eq!(mcp_args(&t)["node_id"], "node-3");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_edge_deploy_no_node() {
        let intent = Intent::EdgeDeploy {
            task: "monitor".to_string(),
            node: None,
        };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_deploy");
        assert_eq!(mcp_args(&t)["task"], "monitor");
        assert!(mcp_args(&t).get("node_id").is_none());
    }

    #[test]
    fn test_edge_update_with_version() {
        let intent = Intent::EdgeUpdate {
            node: "node-5".to_string(),
            version: Some("2.1.0".to_string()),
        };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_update");
        assert_eq!(mcp_args(&t)["node_id"], "node-5");
        assert_eq!(mcp_args(&t)["version"], "2.1.0");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_edge_update_default_version() {
        let intent = Intent::EdgeUpdate {
            node: "node-1".to_string(),
            version: None,
        };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_update");
        assert_eq!(mcp_args(&t)["version"], "latest");
    }

    #[test]
    fn test_edge_health_specific_node() {
        let intent = Intent::EdgeHealth {
            node: Some("node-9".to_string()),
        };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_health");
        assert_eq!(mcp_args(&t)["node_id"], "node-9");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_edge_health_fleet() {
        let intent = Intent::EdgeHealth { node: None };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_health");
        assert!(mcp_args(&t).get("node_id").is_none());
    }

    #[test]
    fn test_edge_decommission() {
        let intent = Intent::EdgeDecommission {
            node: "node-old".to_string(),
        };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_decommission");
        assert_eq!(mcp_args(&t)["node_id"], "node-old");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_edge_logs() {
        let intent = Intent::EdgeLogs {
            action: "tail".to_string(),
            node: Some("node-2".to_string()),
        };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_logs");
        assert_eq!(mcp_args(&t)["action"], "tail");
        assert_eq!(mcp_args(&t)["node_id"], "node-2");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_edge_config_get() {
        let intent = Intent::EdgeConfig {
            action: "get".to_string(),
            node: Some("node-4".to_string()),
            key: Some("max_workers".to_string()),
        };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_config");
        assert_eq!(mcp_args(&t)["action"], "get");
        assert_eq!(mcp_args(&t)["key"], "max_workers");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_edge_config_set() {
        let intent = Intent::EdgeConfig {
            action: "set".to_string(),
            node: Some("node-4".to_string()),
            key: Some("max_workers".to_string()),
        };
        let t = translate_edge(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "edge_config");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }
}
