use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_delta(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::DeltaCreateRepo { name, description } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "name", name);
            insert_opt(&mut a, "description", description);
            Ok(mcp_call(
                "delta_create_repository",
                a,
                format!("Create Delta repository: {}", name),
                PermissionLevel::SystemWrite,
                "Creates a git repository in Delta via MCP bridge".to_string(),
            ))
        }

        Intent::DeltaListRepos => Ok(mcp_call(
            "delta_list_repositories",
            serde_json::Map::new(),
            "List Delta repositories".to_string(),
            PermissionLevel::Safe,
            "Lists git repositories from Delta via MCP bridge".to_string(),
        )),

        Intent::DeltaPr {
            action,
            repo,
            title,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "repo", repo);
            insert_opt(&mut a, "title", title);
            Ok(mcp_call(
                "delta_pull_request",
                a,
                format!("Delta PR: {}", action),
                if action == "list" {
                    PermissionLevel::Safe
                } else {
                    PermissionLevel::SystemWrite
                },
                format!(
                    "{} pull request in Delta via MCP bridge",
                    match action.as_str() {
                        "create" => "Creates a",
                        "merge" => "Merges a",
                        "close" => "Closes a",
                        _ => "Lists",
                    }
                ),
            ))
        }

        Intent::DeltaPush { repo, branch } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "repo", repo);
            insert_opt(&mut a, "branch", branch);
            Ok(mcp_call(
                "delta_push",
                a,
                format!(
                    "Push to Delta{}",
                    repo.as_ref().map_or(String::new(), |r| format!(": {}", r))
                ),
                PermissionLevel::SystemWrite,
                "Pushes code to a Delta repository via MCP bridge".to_string(),
            ))
        }

        Intent::DeltaCiStatus { repo } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "repo", repo);
            Ok(mcp_call(
                "delta_ci_status",
                a,
                format!(
                    "Delta CI status{}",
                    repo.as_ref()
                        .map_or(String::new(), |r| format!(" for {}", r))
                ),
                PermissionLevel::Safe,
                "Retrieves CI pipeline status from Delta via MCP bridge".to_string(),
            ))
        }

        Intent::DeltaBranches { action, repo, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "repo", repo);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "delta_branches",
                a,
                format!(
                    "Delta branches: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "list" | "info" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages branches via Delta".to_string(),
            ))
        }

        Intent::DeltaReview { action, pr_id } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "pr_id", pr_id);
            Ok(mcp_call(
                "delta_review",
                a,
                format!(
                    "Delta review: {}{}",
                    action,
                    pr_id
                        .as_ref()
                        .map_or(String::new(), |id| format!(" PR #{}", id))
                ),
                match action.as_str() {
                    "list" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages code reviews via Delta".to_string(),
            ))
        }

        _ => unreachable!("translate_delta called with non-delta intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_create_repo_with_description() {
        let intent = Intent::DeltaCreateRepo {
            name: "my-app".to_string(),
            description: Some("A cool app".to_string()),
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_create_repository");
        assert_eq!(mcp.arguments["name"], "my-app");
        assert_eq!(mcp.arguments["description"], "A cool app");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_delta_create_repo_without_description() {
        let intent = Intent::DeltaCreateRepo {
            name: "bare-repo".to_string(),
            description: None,
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_create_repository");
        assert_eq!(mcp.arguments["name"], "bare-repo");
        assert!(mcp.arguments.get("description").is_none());
    }

    #[test]
    fn test_delta_list_repos() {
        let intent = Intent::DeltaListRepos;
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_list_repositories");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_delta_pr_create() {
        let intent = Intent::DeltaPr {
            action: "create".to_string(),
            repo: Some("my-app".to_string()),
            title: Some("Add feature X".to_string()),
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_pull_request");
        assert_eq!(mcp.arguments["action"], "create");
        assert_eq!(mcp.arguments["repo"], "my-app");
        assert_eq!(mcp.arguments["title"], "Add feature X");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_delta_pr_list() {
        let intent = Intent::DeltaPr {
            action: "list".to_string(),
            repo: None,
            title: None,
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_pull_request");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_delta_pr_merge() {
        let intent = Intent::DeltaPr {
            action: "merge".to_string(),
            repo: Some("repo".to_string()),
            title: None,
        };
        let t = translate_delta(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_delta_pr_close() {
        let intent = Intent::DeltaPr {
            action: "close".to_string(),
            repo: Some("repo".to_string()),
            title: None,
        };
        let t = translate_delta(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_delta_push_with_repo_and_branch() {
        let intent = Intent::DeltaPush {
            repo: Some("my-app".to_string()),
            branch: Some("feature-x".to_string()),
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_push");
        assert_eq!(mcp.arguments["repo"], "my-app");
        assert_eq!(mcp.arguments["branch"], "feature-x");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_delta_push_minimal() {
        let intent = Intent::DeltaPush {
            repo: None,
            branch: None,
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_push");
        assert!(mcp.arguments.get("repo").is_none());
        assert!(mcp.arguments.get("branch").is_none());
    }

    #[test]
    fn test_delta_ci_status_with_repo() {
        let intent = Intent::DeltaCiStatus {
            repo: Some("my-app".to_string()),
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_ci_status");
        assert_eq!(mcp.arguments["repo"], "my-app");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_delta_ci_status_without_repo() {
        let intent = Intent::DeltaCiStatus { repo: None };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_ci_status");
        assert!(mcp.arguments.get("repo").is_none());
    }

    #[test]
    fn test_delta_branches_list() {
        let intent = Intent::DeltaBranches {
            action: "list".to_string(),
            repo: Some("my-app".to_string()),
            name: None,
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_branches");
        assert_eq!(mcp.arguments["action"], "list");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_delta_branches_create() {
        let intent = Intent::DeltaBranches {
            action: "create".to_string(),
            repo: Some("my-app".to_string()),
            name: Some("feature-y".to_string()),
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_branches");
        assert_eq!(mcp.arguments["action"], "create");
        assert_eq!(mcp.arguments["name"], "feature-y");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_delta_branches_info() {
        let intent = Intent::DeltaBranches {
            action: "info".to_string(),
            repo: None,
            name: Some("main".to_string()),
        };
        let t = translate_delta(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_delta_review_list() {
        let intent = Intent::DeltaReview {
            action: "list".to_string(),
            pr_id: None,
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_review");
        assert_eq!(mcp.arguments["action"], "list");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_delta_review_approve() {
        let intent = Intent::DeltaReview {
            action: "approve".to_string(),
            pr_id: Some("42".to_string()),
        };
        let t = translate_delta(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "delta_review");
        assert_eq!(mcp.arguments["action"], "approve");
        assert_eq!(mcp.arguments["pr_id"], "42");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }
}
