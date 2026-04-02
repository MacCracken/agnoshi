use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_yeoman(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::YeomanAgents {
            action,
            agent_id,
            name,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "agent_id", agent_id);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "yeoman_agents",
                a,
                format!(
                    "SecureYeoman agents: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "list" | "status" | "info" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Lists/Deploys/Stops/Queries agents via SecureYeoman MCP bridge".to_string(),
            ))
        }

        Intent::YeomanTasks {
            action,
            description,
            task_id,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "description", description);
            insert_opt(&mut a, "task_id", task_id);
            Ok(mcp_call(
                "yeoman_tasks",
                a,
                format!(
                    "SecureYeoman task: {}{}",
                    action,
                    task_id
                        .as_ref()
                        .map_or(String::new(), |id| format!(" '{}'", id))
                ),
                match action.as_str() {
                    "list" | "status" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Assigns/Lists/Checks/Cancels tasks via SecureYeoman MCP bridge".to_string(),
            ))
        }

        Intent::YeomanTools { action, query } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "query", query);
            Ok(mcp_call(
                "yeoman_tools",
                a,
                format!(
                    "SecureYeoman tools: {}{}",
                    action,
                    query
                        .as_ref()
                        .map_or(String::new(), |q| format!(" '{}'", q))
                ),
                PermissionLevel::Safe,
                "Queries MCP tools catalog via SecureYeoman MCP bridge".to_string(),
            ))
        }

        Intent::YeomanIntegrations { action, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "yeoman_integrations",
                a,
                format!(
                    "SecureYeoman integration: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "list" | "status" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Lists/Enables/Disables/Checks integrations via SecureYeoman MCP bridge"
                    .to_string(),
            ))
        }

        Intent::YeomanStatus => Ok(mcp_call(
            "yeoman_status",
            serde_json::Map::new(),
            "SecureYeoman status".to_string(),
            PermissionLevel::Safe,
            "Checks SecureYeoman platform health via MCP bridge".to_string(),
        )),

        Intent::YeomanLogs { action, agent_id } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "agent_id", agent_id);
            Ok(mcp_call(
                "yeoman_logs",
                a,
                format!(
                    "SecureYeoman logs: {}{}",
                    action,
                    agent_id
                        .as_ref()
                        .map_or(String::new(), |id| format!(" ({})", id))
                ),
                PermissionLevel::Safe,
                "Queries agent logs via SecureYeoman".to_string(),
            ))
        }

        Intent::YeomanWorkflows { action, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "yeoman_workflows",
                a,
                format!(
                    "SecureYeoman workflow: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "list" | "status" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages workflows via SecureYeoman".to_string(),
            ))
        }

        Intent::YeomanRegisterTools { action } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            Ok(mcp_call(
                "yeoman_register_tools",
                a,
                format!("SecureYeoman register tools: {}", action),
                PermissionLevel::SystemWrite,
                "Registers SecureYeoman MCP tool catalog into daimon registry".to_string(),
            ))
        }

        Intent::YeomanToolExecute { tool_name, args } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "tool_name", tool_name);
            insert_opt(&mut a, "args", args);
            Ok(mcp_call(
                "yeoman_tool_execute",
                a,
                format!("SecureYeoman execute tool: {}", tool_name),
                PermissionLevel::SystemWrite,
                "Executes a SecureYeoman tool by name via bridge".to_string(),
            ))
        }

        Intent::YeomanBrainQuery { query, limit } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "query", query);
            insert_opt(&mut a, "limit", limit);
            Ok(mcp_call(
                "yeoman_brain_query",
                a,
                format!("SecureYeoman brain query: {}", query),
                PermissionLevel::Safe,
                "Queries SecureYeoman knowledge brain".to_string(),
            ))
        }

        Intent::YeomanBrainSync { action, topic } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "topic", topic);
            Ok(mcp_call(
                "yeoman_brain_sync",
                a,
                format!("SecureYeoman brain sync: {}", action),
                PermissionLevel::SystemWrite,
                "Syncs knowledge between SecureYeoman and AGNOS RAG".to_string(),
            ))
        }

        Intent::YeomanTokenBudget {
            action,
            pool,
            amount,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "pool", pool);
            insert_opt(&mut a, "amount", amount);
            Ok(mcp_call(
                "yeoman_token_budget",
                a,
                format!("SecureYeoman token budget: {}", action),
                match action.as_str() {
                    "list" | "check" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages SecureYeoman agent token budgets via hoosh".to_string(),
            ))
        }

        Intent::YeomanEvents { action, limit } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "limit", limit);
            Ok(mcp_call(
                "yeoman_events",
                a,
                format!("SecureYeoman events: {}", action),
                PermissionLevel::Safe,
                "Queries SecureYeoman event stream".to_string(),
            ))
        }

        Intent::YeomanSwarm {
            action,
            swarm_id,
            capability,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "swarm_id", swarm_id);
            insert_opt(&mut a, "capability", capability);
            Ok(mcp_call(
                "yeoman_swarm",
                a,
                format!("SecureYeoman swarm: {}", action),
                match action.as_str() {
                    "handoff" => PermissionLevel::SystemWrite,
                    _ => PermissionLevel::Safe,
                },
                "Queries SecureYeoman swarm topology".to_string(),
            ))
        }

        _ => unreachable!("translate_yeoman called with non-yeoman intent"),
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
    fn test_yeoman_agents_list() {
        let intent = Intent::YeomanAgents {
            action: "list".to_string(),
            agent_id: None,
            name: None,
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_agents");
        assert_eq!(mcp_args(&t)["action"], "list");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_agents_deploy_with_name() {
        let intent = Intent::YeomanAgents {
            action: "deploy".to_string(),
            agent_id: Some("a1".to_string()),
            name: Some("my-agent".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_agents");
        assert_eq!(mcp_args(&t)["action"], "deploy");
        assert_eq!(mcp_args(&t)["agent_id"], "a1");
        assert_eq!(mcp_args(&t)["name"], "my-agent");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_yeoman_tasks_list() {
        let intent = Intent::YeomanTasks {
            action: "list".to_string(),
            description: None,
            task_id: None,
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_tasks");
        assert_eq!(mcp_args(&t)["action"], "list");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_tasks_assign_with_id() {
        let intent = Intent::YeomanTasks {
            action: "assign".to_string(),
            description: Some("do stuff".to_string()),
            task_id: Some("t42".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_tasks");
        assert_eq!(mcp_args(&t)["task_id"], "t42");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_yeoman_tools() {
        let intent = Intent::YeomanTools {
            action: "list".to_string(),
            query: Some("search".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_tools");
        assert_eq!(mcp_args(&t)["action"], "list");
        assert_eq!(mcp_args(&t)["query"], "search");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_integrations_list() {
        let intent = Intent::YeomanIntegrations {
            action: "list".to_string(),
            name: None,
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_integrations");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_integrations_enable() {
        let intent = Intent::YeomanIntegrations {
            action: "enable".to_string(),
            name: Some("slack".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_integrations");
        assert_eq!(mcp_args(&t)["name"], "slack");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_yeoman_status() {
        let intent = Intent::YeomanStatus;
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_status");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_logs() {
        let intent = Intent::YeomanLogs {
            action: "tail".to_string(),
            agent_id: Some("agent-7".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_logs");
        assert_eq!(mcp_args(&t)["agent_id"], "agent-7");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_workflows_list() {
        let intent = Intent::YeomanWorkflows {
            action: "list".to_string(),
            name: None,
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_workflows");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_workflows_create() {
        let intent = Intent::YeomanWorkflows {
            action: "create".to_string(),
            name: Some("my-flow".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_workflows");
        assert_eq!(mcp_args(&t)["name"], "my-flow");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_yeoman_register_tools() {
        let intent = Intent::YeomanRegisterTools {
            action: "register".to_string(),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_register_tools");
        assert_eq!(mcp_args(&t)["action"], "register");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_yeoman_tool_execute() {
        let intent = Intent::YeomanToolExecute {
            tool_name: "scanner".to_string(),
            args: Some("--fast".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_tool_execute");
        assert_eq!(mcp_args(&t)["tool_name"], "scanner");
        assert_eq!(mcp_args(&t)["args"], "--fast");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_yeoman_brain_query() {
        let intent = Intent::YeomanBrainQuery {
            query: "networking setup".to_string(),
            limit: Some("5".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_brain_query");
        assert_eq!(mcp_args(&t)["query"], "networking setup");
        assert_eq!(mcp_args(&t)["limit"], "5");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_brain_sync() {
        let intent = Intent::YeomanBrainSync {
            action: "push".to_string(),
            topic: Some("security".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_brain_sync");
        assert_eq!(mcp_args(&t)["action"], "push");
        assert_eq!(mcp_args(&t)["topic"], "security");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_yeoman_token_budget_list() {
        let intent = Intent::YeomanTokenBudget {
            action: "list".to_string(),
            pool: None,
            amount: None,
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_token_budget");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_token_budget_set() {
        let intent = Intent::YeomanTokenBudget {
            action: "set".to_string(),
            pool: Some("default".to_string()),
            amount: Some("1000".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_token_budget");
        assert_eq!(mcp_args(&t)["pool"], "default");
        assert_eq!(mcp_args(&t)["amount"], "1000");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_yeoman_events() {
        let intent = Intent::YeomanEvents {
            action: "list".to_string(),
            limit: Some("20".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_events");
        assert_eq!(mcp_args(&t)["limit"], "20");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_swarm_query() {
        let intent = Intent::YeomanSwarm {
            action: "status".to_string(),
            swarm_id: Some("swarm-1".to_string()),
            capability: None,
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_swarm");
        assert_eq!(mcp_args(&t)["swarm_id"], "swarm-1");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_yeoman_swarm_handoff() {
        let intent = Intent::YeomanSwarm {
            action: "handoff".to_string(),
            swarm_id: None,
            capability: Some("nlp".to_string()),
        };
        let t = translate_yeoman(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "yeoman_swarm");
        assert_eq!(mcp_args(&t)["capability"], "nlp");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }
}
