use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_tazama(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::TazamaProject { action, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "tazama_project",
                a,
                format!(
                    "Tazama project: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "info" | "list" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} a Tazama video project via MCP bridge",
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

        Intent::TazamaTimeline {
            action,
            clip_id,
            position,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "clip_id", clip_id);
            if let Some(p) = position {
                a.insert(
                    "position".to_string(),
                    serde_json::Value::String(p.to_string()),
                );
            }
            Ok(mcp_call(
                "tazama_timeline",
                a,
                format!(
                    "Tazama timeline: {}{}",
                    action,
                    clip_id
                        .as_ref()
                        .map_or(String::new(), |c| format!(" clip '{}'", c))
                ),
                if action == "list" {
                    PermissionLevel::Safe
                } else {
                    PermissionLevel::SystemWrite
                },
                format!(
                    "{} clips on Tazama timeline via MCP bridge",
                    match action.as_str() {
                        "add" => "Adds",
                        "remove" => "Removes",
                        "split" => "Splits",
                        "trim" => "Trims",
                        "list" => "Lists",
                        _ => "Manages",
                    }
                ),
            ))
        }

        Intent::TazamaEffects {
            action,
            effect_type,
            clip_id,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "effect_type", effect_type);
            insert_opt(&mut a, "clip_id", clip_id);
            Ok(mcp_call(
                "tazama_effects",
                a,
                format!(
                    "Tazama effects: {}{}",
                    action,
                    effect_type
                        .as_ref()
                        .map_or(String::new(), |e| format!(" '{}'", e))
                ),
                if action == "list" {
                    PermissionLevel::Safe
                } else {
                    PermissionLevel::SystemWrite
                },
                format!(
                    "{} effects in Tazama via MCP bridge",
                    match action.as_str() {
                        "apply" => "Applies",
                        "remove" => "Removes",
                        "list" => "Lists",
                        "preview" => "Previews",
                        _ => "Manages",
                    }
                ),
            ))
        }

        Intent::TazamaAi { action, options } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "options", options);
            Ok(mcp_call(
                "tazama_ai",
                a,
                format!("Tazama AI: {}", action),
                PermissionLevel::SystemWrite,
                format!("Runs AI {} on Tazama video via MCP bridge", action),
            ))
        }

        Intent::TazamaExport { path, format } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "path", path);
            insert_opt(&mut a, "format", format);
            Ok(mcp_call(
                "tazama_export",
                a,
                format!(
                    "Tazama export{}",
                    format
                        .as_ref()
                        .map_or(String::new(), |f| format!(" as {}", f))
                ),
                PermissionLevel::SystemWrite,
                "Exports/renders Tazama video project via MCP bridge".to_string(),
            ))
        }

        Intent::TazamaMedia { action, path } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "path", path);
            Ok(mcp_call(
                "tazama_media",
                a,
                format!(
                    "Tazama media: {}{}",
                    action,
                    path.as_ref().map_or(String::new(), |p| format!(" '{}'", p))
                ),
                match action.as_str() {
                    "list" | "info" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages media library via Tazama".to_string(),
            ))
        }

        Intent::TazamaSubtitles { action, language } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "language", language);
            Ok(mcp_call(
                "tazama_subtitles",
                a,
                format!(
                    "Tazama subtitles: {}{}",
                    action,
                    language
                        .as_ref()
                        .map_or(String::new(), |l| format!(" ({})", l))
                ),
                match action.as_str() {
                    "list" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages subtitles via Tazama".to_string(),
            ))
        }

        _ => unreachable!("translate_tazama called with non-tazama intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tazama_project_create() {
        let intent = Intent::TazamaProject {
            action: "create".to_string(),
            name: Some("my-video".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_project");
        assert_eq!(mcp.arguments["action"], "create");
        assert_eq!(mcp.arguments["name"], "my-video");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_project_list() {
        let intent = Intent::TazamaProject {
            action: "list".to_string(),
            name: None,
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_project");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tazama_project_info() {
        let intent = Intent::TazamaProject {
            action: "info".to_string(),
            name: Some("proj1".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tazama_project_open() {
        let intent = Intent::TazamaProject {
            action: "open".to_string(),
            name: Some("proj1".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_project_save() {
        let intent = Intent::TazamaProject {
            action: "save".to_string(),
            name: None,
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_project_close() {
        let intent = Intent::TazamaProject {
            action: "close".to_string(),
            name: None,
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_timeline_add() {
        let intent = Intent::TazamaTimeline {
            action: "add".to_string(),
            clip_id: Some("clip-001".to_string()),
            position: Some(10.5),
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_timeline");
        assert_eq!(mcp.arguments["action"], "add");
        assert_eq!(mcp.arguments["clip_id"], "clip-001");
        assert_eq!(mcp.arguments["position"], "10.5");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_timeline_list() {
        let intent = Intent::TazamaTimeline {
            action: "list".to_string(),
            clip_id: None,
            position: None,
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_timeline");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tazama_timeline_remove() {
        let intent = Intent::TazamaTimeline {
            action: "remove".to_string(),
            clip_id: Some("clip-002".to_string()),
            position: None,
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_timeline_split() {
        let intent = Intent::TazamaTimeline {
            action: "split".to_string(),
            clip_id: Some("clip-001".to_string()),
            position: Some(5.0),
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_timeline_trim() {
        let intent = Intent::TazamaTimeline {
            action: "trim".to_string(),
            clip_id: Some("clip-001".to_string()),
            position: None,
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_effects_apply() {
        let intent = Intent::TazamaEffects {
            action: "apply".to_string(),
            effect_type: Some("blur".to_string()),
            clip_id: Some("clip-001".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_effects");
        assert_eq!(mcp.arguments["action"], "apply");
        assert_eq!(mcp.arguments["effect_type"], "blur");
        assert_eq!(mcp.arguments["clip_id"], "clip-001");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_effects_list() {
        let intent = Intent::TazamaEffects {
            action: "list".to_string(),
            effect_type: None,
            clip_id: None,
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tazama_effects_remove() {
        let intent = Intent::TazamaEffects {
            action: "remove".to_string(),
            effect_type: Some("blur".to_string()),
            clip_id: Some("clip-001".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_effects_preview() {
        let intent = Intent::TazamaEffects {
            action: "preview".to_string(),
            effect_type: Some("color_grade".to_string()),
            clip_id: None,
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_ai() {
        let intent = Intent::TazamaAi {
            action: "auto_cut".to_string(),
            options: Some("fast".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_ai");
        assert_eq!(mcp.arguments["action"], "auto_cut");
        assert_eq!(mcp.arguments["options"], "fast");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_ai_no_options() {
        let intent = Intent::TazamaAi {
            action: "stabilize".to_string(),
            options: None,
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_ai");
        assert!(mcp.arguments.get("options").is_none());
    }

    #[test]
    fn test_tazama_export_with_format_and_path() {
        let intent = Intent::TazamaExport {
            path: Some("/tmp/output.mp4".to_string()),
            format: Some("mp4".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_export");
        assert_eq!(mcp.arguments["path"], "/tmp/output.mp4");
        assert_eq!(mcp.arguments["format"], "mp4");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_export_no_options() {
        let intent = Intent::TazamaExport {
            path: None,
            format: None,
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_export");
        assert!(mcp.arguments.get("path").is_none());
        assert!(mcp.arguments.get("format").is_none());
    }

    #[test]
    fn test_tazama_media_import() {
        let intent = Intent::TazamaMedia {
            action: "import".to_string(),
            path: Some("/tmp/clip.mov".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_media");
        assert_eq!(mcp.arguments["action"], "import");
        assert_eq!(mcp.arguments["path"], "/tmp/clip.mov");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_media_list() {
        let intent = Intent::TazamaMedia {
            action: "list".to_string(),
            path: None,
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tazama_media_info() {
        let intent = Intent::TazamaMedia {
            action: "info".to_string(),
            path: Some("/tmp/clip.mov".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tazama_subtitles_generate() {
        let intent = Intent::TazamaSubtitles {
            action: "generate".to_string(),
            language: Some("en".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tazama_subtitles");
        assert_eq!(mcp.arguments["action"], "generate");
        assert_eq!(mcp.arguments["language"], "en");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tazama_subtitles_list() {
        let intent = Intent::TazamaSubtitles {
            action: "list".to_string(),
            language: None,
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tazama_subtitles_remove() {
        let intent = Intent::TazamaSubtitles {
            action: "remove".to_string(),
            language: Some("fr".to_string()),
        };
        let t = translate_tazama(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }
}
