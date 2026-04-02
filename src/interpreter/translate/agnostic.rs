use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_agnostic(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::AgnosticSubmitTask {
            title,
            description,
            target_url,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "title", title);
            insert_str(
                &mut a,
                "description",
                description.as_deref().unwrap_or(title),
            );
            insert_opt(&mut a, "target_url", target_url);
            Ok(mcp_call(
                "agnostic_submit_task",
                a,
                format!("Submit QA task: {}", title),
                PermissionLevel::SystemWrite,
                "Submits a QA task to Agnostic via MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticTaskStatus { task_id } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "task_id", task_id);
            Ok(mcp_call(
                "agnostic_task_status",
                a,
                format!("Task status: {}", task_id),
                PermissionLevel::Safe,
                "Gets task status from Agnostic via MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticStructuredResults {
            session_id,
            result_type,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "session_id", session_id);
            insert_opt(&mut a, "result_type", result_type);
            Ok(mcp_call(
                "agnostic_structured_results",
                a,
                format!("Results: {}", session_id),
                PermissionLevel::Safe,
                "Gets structured results from Agnostic via MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticListPresets { domain } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "domain", domain);
            Ok(mcp_call(
                "agnostic_list_presets",
                a,
                "Agnostic: list presets".to_string(),
                PermissionLevel::Safe,
                "Lists crew presets from Agnostic via MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticAgentStatus => {
            let a = serde_json::Map::new();
            Ok(mcp_call(
                "agnostic_agent_status",
                a,
                "QA agent status".to_string(),
                PermissionLevel::Safe,
                "Gets QA agent status from Agnostic via MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticDashboard { section } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "section", section);
            Ok(mcp_call(
                "agnostic_dashboard",
                a,
                format!(
                    "Agnostic dashboard{}",
                    section
                        .as_ref()
                        .map_or(String::new(), |s| format!(" ({})", s))
                ),
                PermissionLevel::Safe,
                "Gets QA dashboard snapshot from Agnostic".to_string(),
            ))
        }

        Intent::AgnosticTrends => {
            let a = serde_json::Map::new();
            Ok(mcp_call(
                "agnostic_trends",
                a,
                "Agnostic: quality trends".to_string(),
                PermissionLevel::Safe,
                "Gets quality metric trends from Agnostic test history".to_string(),
            ))
        }

        Intent::AgnosticCompare {
            session_a,
            session_b,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "session_a", session_a);
            insert_str(&mut a, "session_b", session_b);
            Ok(mcp_call(
                "agnostic_compare",
                a,
                format!("Agnostic: compare {} vs {}", session_a, session_b),
                PermissionLevel::Safe,
                "Compares two test sessions side-by-side via Agnostic".to_string(),
            ))
        }

        Intent::AgnosticRunCrew {
            title,
            preset,
            gpu_required,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "title", title);
            insert_str(&mut a, "description", title);
            insert_opt(&mut a, "preset", preset);
            if *gpu_required {
                a.insert("gpu_required".to_string(), serde_json::Value::Bool(true));
            }
            Ok(mcp_call(
                "agnostic_run_crew",
                a,
                format!("Agnostic: run crew '{}'", title),
                PermissionLevel::SystemWrite,
                "Runs an agent crew via Agnostic MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticCrewStatus { crew_id } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "crew_id", crew_id);
            Ok(mcp_call(
                "agnostic_crew_status",
                a,
                format!("Agnostic: crew status {}", crew_id),
                PermissionLevel::Safe,
                "Checks crew run status via Agnostic MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticListCrews { status } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "status", status);
            Ok(mcp_call(
                "agnostic_list_crews",
                a,
                "Agnostic: list crews".to_string(),
                PermissionLevel::Safe,
                "Lists crews with optional status filter via Agnostic MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticCancelCrew { crew_id } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "crew_id", crew_id);
            Ok(mcp_call(
                "agnostic_cancel_crew",
                a,
                format!("Agnostic: cancel crew {}", crew_id),
                PermissionLevel::SystemWrite,
                "Cancels a running or pending crew via Agnostic MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticListDefinitions { domain } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "domain", domain);
            Ok(mcp_call(
                "agnostic_list_definitions",
                a,
                "Agnostic: list agent definitions".to_string(),
                PermissionLevel::Safe,
                "Lists agent definitions via Agnostic MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticCreateAgent {
            agent_key,
            name,
            role,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "agent_key", agent_key);
            insert_str(&mut a, "name", name);
            insert_str(&mut a, "role", role);
            insert_str(&mut a, "goal", role);
            a.insert(
                "backstory".to_string(),
                serde_json::Value::String(format!("Agent specializing in {}", role)),
            );
            Ok(mcp_call(
                "agnostic_create_agent",
                a,
                format!("Agnostic: create agent '{}'", agent_key),
                PermissionLevel::SystemWrite,
                "Creates a new agent definition via Agnostic MCP bridge".to_string(),
            ))
        }

        Intent::AgnosticGpuStatus => {
            let a = serde_json::Map::new();
            Ok(mcp_call(
                "agnos_gpu_probe",
                a,
                "GPU status: probe devices and write gpu.json".to_string(),
                PermissionLevel::Safe,
                "Probes GPU devices via agnosys and writes /var/lib/agnosys/gpu.json".to_string(),
            ))
        }

        Intent::AgnosticGpuMemory => {
            let a = serde_json::Map::new();
            Ok(mcp_call(
                "agnos_gpu_probe",
                a,
                "GPU memory: probe device VRAM".to_string(),
                PermissionLevel::Safe,
                "Probes GPU VRAM via agnosys (returns total and available memory per device)"
                    .to_string(),
            ))
        }

        _ => unreachable!("translate_agnostic called with non-agnostic intent"),
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
    fn test_agnostic_submit_task() {
        let intent = Intent::AgnosticSubmitTask {
            title: "Login test".to_string(),
            description: Some("Test login flow".to_string()),
            target_url: Some("https://example.com".to_string()),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_submit_task");
        assert_eq!(mcp_args(&t)["title"], "Login test");
        assert_eq!(mcp_args(&t)["description"], "Test login flow");
        assert_eq!(mcp_args(&t)["target_url"], "https://example.com");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_agnostic_submit_task_no_description() {
        let intent = Intent::AgnosticSubmitTask {
            title: "Smoke test".to_string(),
            description: None,
            target_url: None,
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_submit_task");
        // When description is None, it falls back to title
        assert_eq!(mcp_args(&t)["description"], "Smoke test");
    }

    #[test]
    fn test_agnostic_task_status() {
        let intent = Intent::AgnosticTaskStatus {
            task_id: "task-99".to_string(),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_task_status");
        assert_eq!(mcp_args(&t)["task_id"], "task-99");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_agnostic_structured_results() {
        let intent = Intent::AgnosticStructuredResults {
            session_id: "sess-1".to_string(),
            result_type: Some("summary".to_string()),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_structured_results");
        assert_eq!(mcp_args(&t)["session_id"], "sess-1");
        assert_eq!(mcp_args(&t)["result_type"], "summary");
    }

    #[test]
    fn test_agnostic_list_presets() {
        let intent = Intent::AgnosticListPresets {
            domain: Some("web".to_string()),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_list_presets");
        assert_eq!(mcp_args(&t)["domain"], "web");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_agnostic_agent_status() {
        let intent = Intent::AgnosticAgentStatus;
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_agent_status");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_agnostic_dashboard_with_section() {
        let intent = Intent::AgnosticDashboard {
            section: Some("failures".to_string()),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_dashboard");
        assert_eq!(mcp_args(&t)["section"], "failures");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_agnostic_dashboard_no_section() {
        let intent = Intent::AgnosticDashboard { section: None };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_dashboard");
        assert!(mcp_args(&t).get("section").is_none());
    }

    #[test]
    fn test_agnostic_trends() {
        let intent = Intent::AgnosticTrends;
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_trends");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_agnostic_compare() {
        let intent = Intent::AgnosticCompare {
            session_a: "s1".to_string(),
            session_b: "s2".to_string(),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_compare");
        assert_eq!(mcp_args(&t)["session_a"], "s1");
        assert_eq!(mcp_args(&t)["session_b"], "s2");
    }

    #[test]
    fn test_agnostic_run_crew() {
        let intent = Intent::AgnosticRunCrew {
            title: "Perf suite".to_string(),
            preset: Some("load-test".to_string()),
            gpu_required: true,
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_run_crew");
        assert_eq!(mcp_args(&t)["title"], "Perf suite");
        assert_eq!(mcp_args(&t)["preset"], "load-test");
        assert_eq!(mcp_args(&t)["gpu_required"], true);
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_agnostic_run_crew_no_gpu() {
        let intent = Intent::AgnosticRunCrew {
            title: "Basic suite".to_string(),
            preset: None,
            gpu_required: false,
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_run_crew");
        assert!(mcp_args(&t).get("gpu_required").is_none());
    }

    #[test]
    fn test_agnostic_crew_status() {
        let intent = Intent::AgnosticCrewStatus {
            crew_id: "crew-42".to_string(),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_crew_status");
        assert_eq!(mcp_args(&t)["crew_id"], "crew-42");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_agnostic_list_crews() {
        let intent = Intent::AgnosticListCrews {
            status: Some("running".to_string()),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_list_crews");
        assert_eq!(mcp_args(&t)["status"], "running");
    }

    #[test]
    fn test_agnostic_cancel_crew() {
        let intent = Intent::AgnosticCancelCrew {
            crew_id: "crew-7".to_string(),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_cancel_crew");
        assert_eq!(mcp_args(&t)["crew_id"], "crew-7");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_agnostic_list_definitions() {
        let intent = Intent::AgnosticListDefinitions {
            domain: Some("testing".to_string()),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_list_definitions");
        assert_eq!(mcp_args(&t)["domain"], "testing");
    }

    #[test]
    fn test_agnostic_create_agent() {
        let intent = Intent::AgnosticCreateAgent {
            agent_key: "qa-bot".to_string(),
            name: "QA Bot".to_string(),
            role: "automated testing".to_string(),
        };
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnostic_create_agent");
        assert_eq!(mcp_args(&t)["agent_key"], "qa-bot");
        assert_eq!(mcp_args(&t)["name"], "QA Bot");
        assert_eq!(mcp_args(&t)["role"], "automated testing");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_agnostic_gpu_status() {
        let intent = Intent::AgnosticGpuStatus;
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnos_gpu_probe");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_agnostic_gpu_memory() {
        let intent = Intent::AgnosticGpuMemory;
        let t = translate_agnostic(&intent).unwrap();
        assert_eq!(mcp_tool_name(&t), "agnos_gpu_probe");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }
}
