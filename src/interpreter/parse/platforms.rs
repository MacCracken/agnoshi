use crate::interpreter::Interpreter;
use crate::interpreter::intent::Intent;

use super::{cap_opt, cap_str};

/// Parse consumer platform intents: Agnostic, Edge, SecureYeoman, Delta, Aequi, Photis Nadi
pub(super) fn parse_platforms(
    interp: &Interpreter,
    input: &str,
    input_lower: &str,
) -> Option<Intent> {
    // --- Agnostic QA platform intents ---
    if let Some(caps) = interp.try_captures("agnostic_run", input_lower) {
        let title = cap_str(&caps, 1);
        let target_url = cap_opt(&caps, 3);
        if !title.is_empty() {
            return Some(Intent::AgnosticSubmitTask {
                title: title.clone(),
                description: Some(title),
                target_url,
            });
        }
    }

    if let Some(caps) = interp.try_captures("agnostic_status", input_lower) {
        let task_id = cap_str(&caps, 1);
        if !task_id.is_empty() {
            return Some(Intent::AgnosticTaskStatus { task_id });
        }
    }

    if let Some(caps) = interp.try_captures("agnostic_report", input_lower) {
        let session_id = cap_str(&caps, 1);
        let result_type = cap_opt(&caps, 3);
        if !session_id.is_empty() {
            return Some(Intent::AgnosticStructuredResults {
                session_id,
                result_type,
            });
        }
    }

    if let Some(caps) = interp.try_captures("agnostic_list_suites", input_lower) {
        let domain = cap_opt(&caps, 2);
        return Some(Intent::AgnosticListPresets { domain });
    }

    if interp
        .try_captures("agnostic_agents", input_lower)
        .is_some()
    {
        return Some(Intent::AgnosticAgentStatus);
    }

    if let Some(caps) = interp.try_captures("agnostic_dashboard", input_lower) {
        let section = cap_opt(&caps, 3);
        return Some(Intent::AgnosticDashboard { section });
    }
    if interp
        .try_captures("agnostic_trends", input_lower)
        .is_some()
    {
        return Some(Intent::AgnosticTrends);
    }
    if let Some(caps) = interp.try_captures("agnostic_compare", input_lower) {
        let session_a = cap_str(&caps, 1);
        let session_b = cap_str(&caps, 2);
        if !session_a.is_empty() && !session_b.is_empty() {
            return Some(Intent::AgnosticCompare {
                session_a,
                session_b,
            });
        }
    }

    if let Some(caps) = interp.try_captures("agnostic_run_crew", input_lower) {
        let title = cap_str(&caps, 2);
        let preset = cap_opt(&caps, 4);
        let gpu_required = caps.get(5).is_some();
        if !title.is_empty() {
            return Some(Intent::AgnosticRunCrew {
                title,
                preset,
                gpu_required,
            });
        }
    }

    if let Some(caps) = interp.try_captures("agnostic_crew_status", input_lower) {
        let crew_id = cap_str(&caps, 2);
        if !crew_id.is_empty() {
            return Some(Intent::AgnosticCrewStatus { crew_id });
        }
    }

    if let Some(caps) = interp.try_captures("agnostic_list_crews", input_lower) {
        let status = cap_opt(&caps, 3);
        return Some(Intent::AgnosticListCrews { status });
    }

    if let Some(caps) = interp.try_captures("agnostic_cancel_crew", input_lower) {
        let crew_id = cap_str(&caps, 2);
        if !crew_id.is_empty() {
            return Some(Intent::AgnosticCancelCrew { crew_id });
        }
    }

    if let Some(caps) = interp.try_captures("agnostic_list_presets", input_lower) {
        let domain = cap_opt(&caps, 3);
        return Some(Intent::AgnosticListPresets { domain });
    }

    if let Some(caps) = interp.try_captures("agnostic_list_definitions", input_lower) {
        let domain = cap_opt(&caps, 4);
        return Some(Intent::AgnosticListDefinitions { domain });
    }

    if let Some(caps) = interp.try_captures("agnostic_create_agent", input_lower) {
        let agent_key = cap_str(&caps, 1);
        let name = cap_str(&caps, 2);
        let role = cap_opt(&caps, 3).unwrap_or_default();
        if !agent_key.is_empty() && !name.is_empty() {
            return Some(Intent::AgnosticCreateAgent {
                agent_key,
                name,
                role,
            });
        }
    }

    if interp
        .try_captures("agnostic_gpu_status", input_lower)
        .is_some()
    {
        return Some(Intent::AgnosticGpuStatus);
    }

    if interp
        .try_captures("agnostic_gpu_memory", input_lower)
        .is_some()
    {
        return Some(Intent::AgnosticGpuMemory);
    }

    // --- Edge fleet management intents ---
    if let Some(caps) = interp.try_captures("edge_list", input_lower) {
        let status = cap_opt(&caps, 3);
        return Some(Intent::EdgeListNodes { status });
    }

    if let Some(caps) = interp.try_captures("edge_deploy", input_lower) {
        let task = cap_str(&caps, 1);
        let node = cap_opt(&caps, 2);
        if !task.is_empty() {
            return Some(Intent::EdgeDeploy { task, node });
        }
    }

    if let Some(caps) = interp.try_captures("edge_update", input_lower) {
        let node = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map_or("", |m| m.as_str())
            .trim()
            .to_string();
        let version = cap_opt(&caps, 3);
        if !node.is_empty() {
            return Some(Intent::EdgeUpdate { node, version });
        }
    }

    if let Some(caps) = interp.try_captures("edge_health", input_lower) {
        let node = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty() && s != "fleet" && s != "all" && s != "nodes");
        return Some(Intent::EdgeHealth { node });
    }

    if let Some(caps) = interp.try_captures("edge_decommission", input_lower) {
        let node = cap_str(&caps, 1);
        if !node.is_empty() {
            return Some(Intent::EdgeDecommission { node });
        }
    }

    if let Some(caps) = interp.try_captures("edge_logs", input_lower) {
        let action = cap_str(&caps, 2);
        let node = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::EdgeLogs { action, node });
        }
    }
    if let Some(caps) = interp.try_captures("edge_config", input_lower) {
        let action = cap_str(&caps, 2);
        let node = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::EdgeConfig {
                action,
                node,
                key: None,
            });
        }
    }

    // --- SecureYeoman AI platform intents ---
    if let Some(caps) = interp.try_captures("yeoman_agents", input_lower) {
        let action = cap_str(&caps, 2);
        let agent_id = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::YeomanAgents {
                action,
                agent_id: agent_id.clone(),
                name: agent_id,
            });
        }
    }

    if let Some(caps) = interp.try_captures("yeoman_tasks", input_lower) {
        let action = cap_str(&caps, 2);
        let description = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::YeomanTasks {
                action,
                description: description.clone(),
                task_id: description,
            });
        }
    }

    if let Some(caps) = interp.try_captures("yeoman_tools", input_lower) {
        let action = cap_str(&caps, 2);
        let query = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::YeomanTools { action, query });
        }
    }

    if let Some(caps) = interp.try_captures("yeoman_integrations", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::YeomanIntegrations { action, name });
        }
    }

    if interp.try_captures("yeoman_status", input_lower).is_some() {
        return Some(Intent::YeomanStatus);
    }

    if let Some(caps) = interp.try_captures("yeoman_logs", input_lower) {
        let action = cap_str(&caps, 2);
        let agent_id = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::YeomanLogs { action, agent_id });
        }
    }
    if let Some(caps) = interp.try_captures("yeoman_workflows", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::YeomanWorkflows { action, name });
        }
    }

    if interp
        .try_captures("yeoman_register_tools", input_lower)
        .is_some()
    {
        return Some(Intent::YeomanRegisterTools {
            action: "register".to_string(),
        });
    }

    if let Some(caps) = interp.try_captures("yeoman_tool_execute", input_lower) {
        let tool_name = cap_str(&caps, 1);
        let args = cap_opt(&caps, 2);
        if !tool_name.is_empty() {
            return Some(Intent::YeomanToolExecute { tool_name, args });
        }
    }

    if let Some(caps) = interp.try_captures("yeoman_brain_query", input_lower) {
        let query = cap_str(&caps, 1);
        if !query.is_empty() {
            return Some(Intent::YeomanBrainQuery { query, limit: None });
        }
    }

    if let Some(caps) = interp.try_captures("yeoman_brain_sync", input_lower) {
        let action = cap_str(&caps, 1);
        if !action.is_empty() {
            return Some(Intent::YeomanBrainSync {
                action,
                topic: None,
            });
        }
    }

    if let Some(caps) = interp.try_captures("yeoman_token_budget", input_lower) {
        let action = cap_str(&caps, 1);
        let pool = cap_opt(&caps, 2);
        if !action.is_empty() {
            return Some(Intent::YeomanTokenBudget {
                action,
                pool,
                amount: None,
            });
        }
    }

    if let Some(caps) = interp.try_captures("yeoman_events", input_lower) {
        let action = cap_str(&caps, 1);
        let limit = cap_opt(&caps, 2);
        if !action.is_empty() {
            return Some(Intent::YeomanEvents { action, limit });
        }
    }

    if let Some(caps) = interp.try_captures("yeoman_swarm", input_lower) {
        let action = cap_str(&caps, 1);
        let extra = cap_opt(&caps, 2);
        if !action.is_empty() {
            return Some(Intent::YeomanSwarm {
                action,
                swarm_id: extra.clone(),
                capability: extra,
            });
        }
    }

    // --- Delta code hosting intents ---
    if let Some(caps) = interp.try_captures("delta_create_repo", input_lower) {
        let name = cap_str(&caps, 2);
        let description = cap_opt(&caps, 4);
        if !name.is_empty() {
            return Some(Intent::DeltaCreateRepo { name, description });
        }
    }

    if interp
        .try_captures("delta_list_repos", input_lower)
        .is_some()
    {
        return Some(Intent::DeltaListRepos);
    }

    if let Some(caps) = interp.try_captures("delta_pr", input_lower) {
        let action = caps
            .get(2)
            .map_or("list", |m| m.as_str())
            .trim()
            .to_string();
        let repo = cap_opt(&caps, 4);
        let title = cap_opt(&caps, 6);
        return Some(Intent::DeltaPr {
            action,
            repo,
            title,
        });
    }

    if let Some(caps) = interp.try_captures("delta_push", input_lower) {
        let repo = cap_opt(&caps, 2);
        let branch = cap_opt(&caps, 4);
        return Some(Intent::DeltaPush { repo, branch });
    }

    if let Some(caps) = interp.try_captures("delta_ci", input_lower) {
        let repo = cap_opt(&caps, 4);
        return Some(Intent::DeltaCiStatus { repo });
    }

    if let Some(caps) = interp.try_captures("delta_branches", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::DeltaBranches {
                action,
                repo: None,
                name,
            });
        }
    }
    if let Some(caps) = interp.try_captures("delta_review", input_lower) {
        let action = cap_str(&caps, 2);
        let pr_id = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::DeltaReview { action, pr_id });
        }
    }

    // --- Aequi accounting intents ---
    if let Some(caps) = interp.try_captures("aequi_tax", input_lower) {
        let quarter = cap_opt(&caps, 6);
        return Some(Intent::AequiTaxEstimate { quarter });
    }

    if let Some(caps) = interp.try_captures("aequi_schedule_c", input_lower) {
        let year = cap_opt(&caps, 4);
        return Some(Intent::AequiScheduleC { year });
    }

    if let Some(caps) = interp.try_captures("aequi_import", input_lower) {
        let file_path = cap_str(&caps, 4);
        if !file_path.is_empty() {
            return Some(Intent::AequiImportBank { file_path });
        }
    }

    if interp.try_captures("aequi_balance", input_lower).is_some() {
        return Some(Intent::AequiBalance);
    }

    if let Some(caps) = interp.try_captures("aequi_receipts", input_lower) {
        let status = caps.get(3).map(|m| {
            let s = m.as_str().trim();
            match s {
                "pending" => "pending_review".to_string(),
                "unreviewed" => "pending_review".to_string(),
                other => other.to_string(),
            }
        });
        return Some(Intent::AequiReceipts { status });
    }

    if let Some(caps) = interp.try_captures("aequi_invoices", input_lower) {
        let action = cap_str(&caps, 2);
        let client = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::AequiInvoices { action, client });
        }
    }
    if let Some(caps) = interp.try_captures("aequi_reports", input_lower) {
        let action = caps
            .get(2)
            .map_or("", |m| m.as_str())
            .trim()
            .replace(' ', "_")
            .to_string();
        let period = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::AequiReports { action, period });
        }
    }

    // --- Photis Nadi task management intents ---
    if let Some(caps) = interp.try_captures("task_list", input_lower) {
        let status = cap_opt(&caps, 4);
        return Some(Intent::TaskList { status });
    }

    // Note: task_create uses original-case input for title preservation
    if let Some(caps) = interp.try_captures("task_create", input) {
        let title = cap_str(&caps, 2);
        if !title.is_empty() {
            let priority = cap_opt(&caps, 4);
            return Some(Intent::TaskCreate { title, priority });
        }
    }

    if let Some(caps) = interp.try_captures("task_update", input_lower) {
        let task_id = cap_str(&caps, 2);
        let status = cap_opt(&caps, 3);
        if !task_id.is_empty() {
            return Some(Intent::TaskUpdate { task_id, status });
        }
    }

    if let Some(caps) = interp.try_captures("ritual_check", input_lower) {
        let date = cap_opt(&caps, 2);
        return Some(Intent::RitualCheck { date });
    }

    if let Some(caps) = interp.try_captures("productivity_stats", input_lower) {
        let period = caps.get(2).map(|m| match m.as_str().trim() {
            "daily" => "day".to_string(),
            "weekly" | "this week" => "week".to_string(),
            "monthly" | "this month" => "month".to_string(),
            other => other.to_string(),
        });
        return Some(Intent::ProductivityStats { period });
    }

    if let Some(caps) = interp.try_captures("photis_boards", input_lower) {
        let action = cap_str(&caps, 2);
        let name = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::PhotoisBoards { action, name });
        }
    }
    if let Some(caps) = interp.try_captures("photis_notes", input_lower) {
        let action = cap_str(&caps, 2);
        let content = cap_opt(&caps, 4);
        if !action.is_empty() {
            return Some(Intent::PhotoisNotes { action, content });
        }
    }

    // --- Phylax threat detection intents ---
    if let Some(caps) = interp.try_captures("phylax_scan", input_lower) {
        // Group 1: target from "scan <target> for threats"
        // Group 2: target from "phylax scan <target>"
        // Group 3: mode
        let target = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let mode = cap_opt(&caps, 3);
        if !target.is_empty() {
            return Some(Intent::PhylaxScan { target, mode });
        }
    }

    if let Some(caps) = interp.try_captures("phylax_findings", input_lower) {
        let severity = cap_opt(&caps, 1);
        return Some(Intent::PhylaxFindings { severity });
    }

    if let Some(caps) = interp.try_captures("phylax_history", input_lower) {
        let limit = caps
            .get(1)
            .and_then(|m| m.as_str().trim().parse::<usize>().ok());
        return Some(Intent::PhylaxHistory { limit });
    }

    if interp.try_captures("phylax_status", input_lower).is_some() {
        return Some(Intent::PhylaxStatus);
    }

    if interp.try_captures("phylax_rules", input_lower).is_some() {
        return Some(Intent::PhylaxRules);
    }

    // --- T-Ron security monitor intents ---
    if interp.try_captures("tron_status", input_lower).is_some() {
        return Some(Intent::TronStatus);
    }

    if let Some(caps) = interp.try_captures("tron_risk", input_lower) {
        let agent_id = cap_str(&caps, 1);
        if !agent_id.is_empty() {
            return Some(Intent::TronRisk { agent_id });
        }
    }

    if let Some(caps) = interp.try_captures("tron_audit", input_lower) {
        let agent_id = cap_opt(&caps, 1);
        let limit = caps
            .get(2)
            .and_then(|m| m.as_str().trim().parse::<usize>().ok());
        return Some(Intent::TronAudit { agent_id, limit });
    }

    if let Some(caps) = interp.try_captures("tron_policy", input_lower) {
        let toml = cap_str(&caps, 1);
        if !toml.is_empty() {
            return Some(Intent::TronPolicy { toml });
        }
    }

    None
}
