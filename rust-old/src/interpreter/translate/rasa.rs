use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_rasa(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::RasaCanvas { action, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "rasa_canvas",
                a,
                format!(
                    "Rasa canvas: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "info" | "list" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} a Rasa image canvas via MCP bridge",
                    match action.as_str() {
                        "create" => "Creates",
                        "open" => "Opens",
                        "save" => "Saves",
                        "close" => "Closes",
                        _ => "Queries",
                    }
                ),
            ))
        }

        Intent::RasaLayers { action, name, kind } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            insert_opt(&mut a, "kind", kind);
            Ok(mcp_call(
                "rasa_layers",
                a,
                format!(
                    "Rasa layers: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                if action == "list" {
                    PermissionLevel::Safe
                } else {
                    PermissionLevel::SystemWrite
                },
                format!(
                    "{} layers in Rasa via MCP bridge",
                    match action.as_str() {
                        "add" => "Adds",
                        "remove" => "Removes",
                        "reorder" => "Reorders",
                        "merge" => "Merges",
                        "list" => "Lists",
                        _ => "Manages",
                    }
                ),
            ))
        }

        Intent::RasaTools { action, params } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "params", params);
            Ok(mcp_call(
                "rasa_tools",
                a,
                format!(
                    "Rasa tools: {}{}",
                    action,
                    params
                        .as_ref()
                        .map_or(String::new(), |p| format!(" ({})", p))
                ),
                PermissionLevel::SystemWrite,
                format!("Applies {} tool in Rasa via MCP bridge", action),
            ))
        }

        Intent::RasaAi { action, prompt } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "prompt", prompt);
            Ok(mcp_call(
                "rasa_ai",
                a,
                format!(
                    "Rasa AI: {}{}",
                    action,
                    prompt
                        .as_ref()
                        .map_or(String::new(), |p| format!(" '{}'", p))
                ),
                PermissionLevel::SystemWrite,
                format!("Runs AI {} on Rasa image via MCP bridge", action),
            ))
        }

        Intent::RasaExport { path, format } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "path", path);
            insert_opt(&mut a, "format", format);
            Ok(mcp_call(
                "rasa_export",
                a,
                format!(
                    "Rasa export{}",
                    format
                        .as_ref()
                        .map_or(String::new(), |f| format!(" as {}", f))
                ),
                PermissionLevel::SystemWrite,
                "Exports Rasa image via MCP bridge".to_string(),
            ))
        }

        Intent::RasaBatch { action, path } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "path", path);
            Ok(mcp_call(
                "rasa_batch",
                a,
                format!(
                    "Rasa batch: {}{}",
                    action,
                    path.as_ref().map_or(String::new(), |p| format!(" '{}'", p))
                ),
                match action.as_str() {
                    "list" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Batch image operations via Rasa".to_string(),
            ))
        }

        Intent::RasaTemplates { action, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "rasa_templates",
                a,
                format!(
                    "Rasa templates: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "list" | "info" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages design templates via Rasa".to_string(),
            ))
        }

        Intent::RasaAdjustments {
            action,
            adjustment_type,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "type", adjustment_type);
            Ok(mcp_call(
                "rasa_adjustments",
                a,
                format!(
                    "Rasa adjustment: {}{}",
                    action,
                    adjustment_type
                        .as_ref()
                        .map_or(String::new(), |t| format!(" ({})", t))
                ),
                match action.as_str() {
                    "list" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages non-destructive adjustment layers via Rasa".to_string(),
            ))
        }

        _ => unreachable!("translate_rasa called with non-rasa intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_create() {
        let intent = Intent::RasaCanvas {
            action: "create".to_string(),
            name: Some("banner".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_canvas");
        assert_eq!(mcp.arguments["action"], "create");
        assert_eq!(mcp.arguments["name"], "banner");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_canvas_list() {
        let intent = Intent::RasaCanvas {
            action: "list".to_string(),
            name: None,
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_canvas");
        assert!(mcp.arguments.get("name").is_none());
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_canvas_info() {
        let intent = Intent::RasaCanvas {
            action: "info".to_string(),
            name: Some("logo".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_canvas_save() {
        let intent = Intent::RasaCanvas {
            action: "save".to_string(),
            name: Some("draft".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_layers_add() {
        let intent = Intent::RasaLayers {
            action: "add".to_string(),
            name: Some("background".to_string()),
            kind: Some("raster".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_layers");
        assert_eq!(mcp.arguments["action"], "add");
        assert_eq!(mcp.arguments["name"], "background");
        assert_eq!(mcp.arguments["kind"], "raster");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_layers_list() {
        let intent = Intent::RasaLayers {
            action: "list".to_string(),
            name: None,
            kind: None,
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_layers");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_layers_remove() {
        let intent = Intent::RasaLayers {
            action: "remove".to_string(),
            name: Some("layer-1".to_string()),
            kind: None,
        };
        let result = translate_rasa(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tools_action() {
        let intent = Intent::RasaTools {
            action: "brush".to_string(),
            params: Some("size=10".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_tools");
        assert_eq!(mcp.arguments["action"], "brush");
        assert_eq!(mcp.arguments["params"], "size=10");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tools_without_params() {
        let intent = Intent::RasaTools {
            action: "eraser".to_string(),
            params: None,
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert!(mcp.arguments.get("params").is_none());
    }

    #[test]
    fn test_ai_with_prompt() {
        let intent = Intent::RasaAi {
            action: "enhance".to_string(),
            prompt: Some("increase sharpness".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_ai");
        assert_eq!(mcp.arguments["action"], "enhance");
        assert_eq!(mcp.arguments["prompt"], "increase sharpness");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_ai_without_prompt() {
        let intent = Intent::RasaAi {
            action: "upscale".to_string(),
            prompt: None,
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert!(mcp.arguments.get("prompt").is_none());
    }

    #[test]
    fn test_export_with_format() {
        let intent = Intent::RasaExport {
            path: Some("/output/image.png".to_string()),
            format: Some("png".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_export");
        assert_eq!(mcp.arguments["path"], "/output/image.png");
        assert_eq!(mcp.arguments["format"], "png");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_export_minimal() {
        let intent = Intent::RasaExport {
            path: None,
            format: None,
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_export");
        assert!(mcp.arguments.get("path").is_none());
        assert!(mcp.arguments.get("format").is_none());
    }

    #[test]
    fn test_batch_with_path() {
        let intent = Intent::RasaBatch {
            action: "resize".to_string(),
            path: Some("/images".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_batch");
        assert_eq!(mcp.arguments["action"], "resize");
        assert_eq!(mcp.arguments["path"], "/images");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_batch_list() {
        let intent = Intent::RasaBatch {
            action: "list".to_string(),
            path: None,
        };
        let result = translate_rasa(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_templates_list() {
        let intent = Intent::RasaTemplates {
            action: "list".to_string(),
            name: None,
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_templates");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_templates_info() {
        let intent = Intent::RasaTemplates {
            action: "info".to_string(),
            name: Some("poster".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_templates_apply() {
        let intent = Intent::RasaTemplates {
            action: "apply".to_string(),
            name: Some("flyer".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["name"], "flyer");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_adjustments_list() {
        let intent = Intent::RasaAdjustments {
            action: "list".to_string(),
            adjustment_type: None,
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_adjustments");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_adjustments_apply() {
        let intent = Intent::RasaAdjustments {
            action: "apply".to_string(),
            adjustment_type: Some("brightness".to_string()),
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "rasa_adjustments");
        assert_eq!(mcp.arguments["action"], "apply");
        assert_eq!(mcp.arguments["type"], "brightness");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_adjustments_without_type() {
        let intent = Intent::RasaAdjustments {
            action: "reset".to_string(),
            adjustment_type: None,
        };
        let result = translate_rasa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert!(mcp.arguments.get("type").is_none());
    }
}
