use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_synapse(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::SynapseModels {
            action,
            name,
            source,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            insert_opt(&mut a, "source", source);
            Ok(mcp_call(
                "synapse_models",
                a,
                format!(
                    "Synapse models: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "list" | "info" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} model(s) via Synapse MCP bridge",
                    match action.as_str() {
                        "download" => "Downloads",
                        "delete" => "Deletes",
                        "list" => "Lists",
                        _ => "Queries",
                    }
                ),
            ))
        }

        Intent::SynapseServe { action, model } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "model", model);
            Ok(mcp_call(
                "synapse_serve",
                a,
                format!(
                    "Synapse serve: {}{}",
                    action,
                    model
                        .as_ref()
                        .map_or(String::new(), |m| format!(" '{}'", m))
                ),
                match action.as_str() {
                    "status" | "list" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} model serving via Synapse MCP bridge",
                    match action.as_str() {
                        "start" => "Starts",
                        "stop" => "Stops",
                        _ => "Queries",
                    }
                ),
            ))
        }

        Intent::SynapseFinetune {
            action,
            model,
            method,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "model", model);
            insert_opt(&mut a, "method", method);
            Ok(mcp_call(
                "synapse_finetune",
                a,
                format!(
                    "Synapse finetune: {}{}",
                    action,
                    model
                        .as_ref()
                        .map_or(String::new(), |m| format!(" '{}'", m))
                ),
                match action.as_str() {
                    "status" | "list" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} fine-tuning job via Synapse MCP bridge",
                    match action.as_str() {
                        "start" => "Starts",
                        "cancel" => "Cancels",
                        "list" => "Lists",
                        _ => "Queries",
                    }
                ),
            ))
        }

        Intent::SynapseChat { model, prompt } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "model", model);
            insert_opt(&mut a, "prompt", prompt);
            Ok(mcp_call(
                "synapse_chat",
                a,
                format!("Synapse chat: {}", model),
                PermissionLevel::SystemWrite,
                "Runs inference via Synapse MCP bridge".to_string(),
            ))
        }

        Intent::SynapseStatus => Ok(mcp_call(
            "synapse_status",
            serde_json::Map::new(),
            "Synapse status".to_string(),
            PermissionLevel::Safe,
            "Checks Synapse health and GPU status via MCP bridge".to_string(),
        )),

        Intent::SynapseBenchmark { action, models } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "models", models);
            Ok(mcp_call(
                "synapse_benchmark",
                a,
                format!(
                    "Synapse benchmark: {}{}",
                    action,
                    models
                        .as_ref()
                        .map_or(String::new(), |m| format!(" '{}'", m))
                ),
                match action.as_str() {
                    "list" | "status" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Benchmarks/compares models via Synapse".to_string(),
            ))
        }

        Intent::SynapseQuantize {
            action,
            model,
            format,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "model", model);
            insert_opt(&mut a, "format", format);
            Ok(mcp_call(
                "synapse_quantize",
                a,
                format!(
                    "Synapse quantize: {}{}",
                    action,
                    model
                        .as_ref()
                        .map_or(String::new(), |m| format!(" '{}'", m))
                ),
                match action.as_str() {
                    "status" | "list" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Quantizes/converts model via Synapse".to_string(),
            ))
        }

        _ => unreachable!("translate_synapse called with non-synapse intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synapse_models_list() {
        let intent = Intent::SynapseModels {
            action: "list".to_string(),
            name: None,
            source: None,
        };
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "synapse_models");
        assert_eq!(mcp.arguments["action"], "list");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_synapse_models_download_with_name_and_source() {
        let intent = Intent::SynapseModels {
            action: "download".to_string(),
            name: Some("llama3".to_string()),
            source: Some("huggingface".to_string()),
        };
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "synapse_models");
        assert_eq!(mcp.arguments["action"], "download");
        assert_eq!(mcp.arguments["name"], "llama3");
        assert_eq!(mcp.arguments["source"], "huggingface");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_synapse_models_info() {
        let intent = Intent::SynapseModels {
            action: "info".to_string(),
            name: Some("gpt2".to_string()),
            source: None,
        };
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["action"], "info");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_synapse_models_delete() {
        let intent = Intent::SynapseModels {
            action: "delete".to_string(),
            name: Some("old-model".to_string()),
            source: None,
        };
        let t = translate_synapse(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_synapse_serve_start() {
        let intent = Intent::SynapseServe {
            action: "start".to_string(),
            model: Some("llama3".to_string()),
        };
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "synapse_serve");
        assert_eq!(mcp.arguments["action"], "start");
        assert_eq!(mcp.arguments["model"], "llama3");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_synapse_serve_status() {
        let intent = Intent::SynapseServe {
            action: "status".to_string(),
            model: None,
        };
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "synapse_serve");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_synapse_serve_stop() {
        let intent = Intent::SynapseServe {
            action: "stop".to_string(),
            model: Some("llama3".to_string()),
        };
        let t = translate_synapse(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_synapse_serve_list() {
        let intent = Intent::SynapseServe {
            action: "list".to_string(),
            model: None,
        };
        let t = translate_synapse(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_synapse_finetune_start() {
        let intent = Intent::SynapseFinetune {
            action: "start".to_string(),
            model: Some("llama3".to_string()),
            method: Some("lora".to_string()),
        };
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "synapse_finetune");
        assert_eq!(mcp.arguments["action"], "start");
        assert_eq!(mcp.arguments["model"], "llama3");
        assert_eq!(mcp.arguments["method"], "lora");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_synapse_finetune_status() {
        let intent = Intent::SynapseFinetune {
            action: "status".to_string(),
            model: None,
            method: None,
        };
        let t = translate_synapse(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_synapse_finetune_list() {
        let intent = Intent::SynapseFinetune {
            action: "list".to_string(),
            model: None,
            method: None,
        };
        let t = translate_synapse(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_synapse_finetune_cancel() {
        let intent = Intent::SynapseFinetune {
            action: "cancel".to_string(),
            model: Some("llama3".to_string()),
            method: None,
        };
        let t = translate_synapse(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_synapse_chat() {
        let intent = Intent::SynapseChat {
            model: "llama3".to_string(),
            prompt: Some("hello world".to_string()),
        };
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "synapse_chat");
        assert_eq!(mcp.arguments["model"], "llama3");
        assert_eq!(mcp.arguments["prompt"], "hello world");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_synapse_chat_no_prompt() {
        let intent = Intent::SynapseChat {
            model: "gpt2".to_string(),
            prompt: None,
        };
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "synapse_chat");
        assert_eq!(mcp.arguments["model"], "gpt2");
        assert!(mcp.arguments.get("prompt").is_none());
    }

    #[test]
    fn test_synapse_status() {
        let intent = Intent::SynapseStatus;
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "synapse_status");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_synapse_benchmark_run() {
        let intent = Intent::SynapseBenchmark {
            action: "run".to_string(),
            models: Some("llama3,gpt2".to_string()),
        };
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "synapse_benchmark");
        assert_eq!(mcp.arguments["action"], "run");
        assert_eq!(mcp.arguments["models"], "llama3,gpt2");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_synapse_benchmark_list() {
        let intent = Intent::SynapseBenchmark {
            action: "list".to_string(),
            models: None,
        };
        let t = translate_synapse(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_synapse_benchmark_status() {
        let intent = Intent::SynapseBenchmark {
            action: "status".to_string(),
            models: None,
        };
        let t = translate_synapse(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_synapse_quantize_start() {
        let intent = Intent::SynapseQuantize {
            action: "start".to_string(),
            model: Some("llama3".to_string()),
            format: Some("gguf".to_string()),
        };
        let t = translate_synapse(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "synapse_quantize");
        assert_eq!(mcp.arguments["action"], "start");
        assert_eq!(mcp.arguments["model"], "llama3");
        assert_eq!(mcp.arguments["format"], "gguf");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_synapse_quantize_status() {
        let intent = Intent::SynapseQuantize {
            action: "status".to_string(),
            model: None,
            format: None,
        };
        let t = translate_synapse(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_synapse_quantize_list() {
        let intent = Intent::SynapseQuantize {
            action: "list".to_string(),
            model: None,
            format: None,
        };
        let t = translate_synapse(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }
}
