use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_shruti(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::ShrutiSession { action, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "shruti_session",
                a,
                format!(
                    "Shruti session: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "info" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} a Shruti DAW session via MCP bridge",
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

        Intent::ShrutiTrack { action, name, kind } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            insert_opt(&mut a, "kind", kind);
            Ok(mcp_call(
                "shruti_tracks",
                a,
                format!(
                    "Shruti track: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                if action == "list" {
                    PermissionLevel::Safe
                } else {
                    PermissionLevel::SystemWrite
                },
                format!(
                    "{} track in Shruti via MCP bridge",
                    match action.as_str() {
                        "add" => "Adds a",
                        "remove" => "Removes a",
                        "list" => "Lists",
                        "rename" => "Renames a",
                        _ => "Manages a",
                    }
                ),
            ))
        }

        Intent::ShrutiMixer {
            track,
            gain,
            mute,
            solo,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "track", track);
            if let Some(g) = gain {
                a.insert(
                    "gain".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(*g).unwrap()),
                );
            }
            if let Some(m) = mute {
                a.insert("mute".to_string(), serde_json::Value::Bool(*m));
            }
            if let Some(s) = solo {
                a.insert("solo".to_string(), serde_json::Value::Bool(*s));
            }
            Ok(mcp_call(
                "shruti_mixer",
                a,
                format!("Shruti mixer: {}", track),
                PermissionLevel::SystemWrite,
                format!(
                    "Controls mixer for track '{}' in Shruti via MCP bridge",
                    track
                ),
            ))
        }

        Intent::ShrutiTransport { action, value } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "value", value);
            Ok(mcp_call(
                "shruti_transport",
                a,
                format!(
                    "Shruti transport: {}{}",
                    action,
                    value
                        .as_ref()
                        .map_or(String::new(), |v| format!(" ({})", v))
                ),
                match action.as_str() {
                    "status" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} Shruti transport via MCP bridge",
                    match action.as_str() {
                        "play" => "Starts playback on",
                        "pause" => "Pauses",
                        "stop" => "Stops",
                        "seek" => "Seeks position on",
                        "set_tempo" => "Sets tempo on",
                        _ => "Controls",
                    }
                ),
            ))
        }

        Intent::ShrutiExport { path, format } => {
            let mut a = serde_json::Map::new();
            insert_opt(&mut a, "path", path);
            insert_opt(&mut a, "format", format);
            Ok(mcp_call(
                "shruti_export",
                a,
                format!(
                    "Shruti export{}",
                    format
                        .as_ref()
                        .map_or(String::new(), |f| format!(" as {}", f))
                ),
                PermissionLevel::SystemWrite,
                "Exports/bounces Shruti session to audio file via MCP bridge".to_string(),
            ))
        }

        Intent::ShrutiPlugins { action, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "shruti_plugins",
                a,
                format!(
                    "Shruti plugins: {}{}",
                    action,
                    name.as_ref().map_or(String::new(), |n| format!(" '{}'", n))
                ),
                match action.as_str() {
                    "list" | "info" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                "Manages audio plugins via Shruti".to_string(),
            ))
        }

        Intent::ShrutiAi { action, track } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "track", track);
            Ok(mcp_call(
                "shruti_ai",
                a,
                format!(
                    "Shruti AI: {}{}",
                    action,
                    track
                        .as_ref()
                        .map_or(String::new(), |t| format!(" on '{}'", t))
                ),
                PermissionLevel::SystemWrite,
                "Runs AI audio features via Shruti".to_string(),
            ))
        }

        _ => unreachable!("translate_shruti called with non-shruti intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shruti_session_create() {
        let intent = Intent::ShrutiSession {
            action: "create".to_string(),
            name: Some("my_song".to_string()),
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_session");
        assert_eq!(mcp.arguments["action"], "create");
        assert_eq!(mcp.arguments["name"], "my_song");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_shruti_session_info() {
        let intent = Intent::ShrutiSession {
            action: "info".to_string(),
            name: None,
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_session");
        assert_eq!(mcp.arguments["action"], "info");
        assert!(mcp.arguments.get("name").is_none());
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_shruti_session_open() {
        let intent = Intent::ShrutiSession {
            action: "open".to_string(),
            name: Some("demo".to_string()),
        };
        let t = translate_shruti(&intent).unwrap();
        assert!(t.description.contains("open"));
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_shruti_track_add() {
        let intent = Intent::ShrutiTrack {
            action: "add".to_string(),
            name: Some("vocals".to_string()),
            kind: Some("audio".to_string()),
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_tracks");
        assert_eq!(mcp.arguments["action"], "add");
        assert_eq!(mcp.arguments["name"], "vocals");
        assert_eq!(mcp.arguments["kind"], "audio");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_shruti_track_list() {
        let intent = Intent::ShrutiTrack {
            action: "list".to_string(),
            name: None,
            kind: None,
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_tracks");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_shruti_track_remove() {
        let intent = Intent::ShrutiTrack {
            action: "remove".to_string(),
            name: Some("drums".to_string()),
            kind: None,
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_tracks");
        assert_eq!(mcp.arguments["action"], "remove");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_shruti_mixer_with_gain_mute_solo() {
        let intent = Intent::ShrutiMixer {
            track: "vocals".to_string(),
            gain: Some(-3.5),
            mute: Some(false),
            solo: Some(true),
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_mixer");
        assert_eq!(mcp.arguments["track"], "vocals");
        assert_eq!(mcp.arguments["gain"], -3.5);
        assert_eq!(mcp.arguments["mute"], false);
        assert_eq!(mcp.arguments["solo"], true);
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_shruti_mixer_minimal() {
        let intent = Intent::ShrutiMixer {
            track: "bass".to_string(),
            gain: None,
            mute: None,
            solo: None,
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_mixer");
        assert_eq!(mcp.arguments["track"], "bass");
        assert!(mcp.arguments.get("gain").is_none());
        assert!(mcp.arguments.get("mute").is_none());
        assert!(mcp.arguments.get("solo").is_none());
    }

    #[test]
    fn test_shruti_transport_play() {
        let intent = Intent::ShrutiTransport {
            action: "play".to_string(),
            value: None,
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_transport");
        assert_eq!(mcp.arguments["action"], "play");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_shruti_transport_status() {
        let intent = Intent::ShrutiTransport {
            action: "status".to_string(),
            value: None,
        };
        let t = translate_shruti(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_shruti_transport_set_tempo() {
        let intent = Intent::ShrutiTransport {
            action: "set_tempo".to_string(),
            value: Some("120".to_string()),
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["action"], "set_tempo");
        assert_eq!(mcp.arguments["value"], "120");
    }

    #[test]
    fn test_shruti_transport_seek() {
        let intent = Intent::ShrutiTransport {
            action: "seek".to_string(),
            value: Some("00:01:30".to_string()),
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["action"], "seek");
        assert_eq!(mcp.arguments["value"], "00:01:30");
    }

    #[test]
    fn test_shruti_export_with_format() {
        let intent = Intent::ShrutiExport {
            path: Some("/tmp/output.wav".to_string()),
            format: Some("wav".to_string()),
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_export");
        assert_eq!(mcp.arguments["path"], "/tmp/output.wav");
        assert_eq!(mcp.arguments["format"], "wav");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_shruti_export_minimal() {
        let intent = Intent::ShrutiExport {
            path: None,
            format: None,
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_export");
        assert!(mcp.arguments.get("path").is_none());
        assert!(mcp.arguments.get("format").is_none());
    }

    #[test]
    fn test_shruti_plugins_list() {
        let intent = Intent::ShrutiPlugins {
            action: "list".to_string(),
            name: None,
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_plugins");
        assert_eq!(mcp.arguments["action"], "list");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_shruti_plugins_info() {
        let intent = Intent::ShrutiPlugins {
            action: "info".to_string(),
            name: Some("reverb".to_string()),
        };
        let t = translate_shruti(&intent).unwrap();
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_shruti_plugins_add() {
        let intent = Intent::ShrutiPlugins {
            action: "add".to_string(),
            name: Some("compressor".to_string()),
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_plugins");
        assert_eq!(mcp.arguments["action"], "add");
        assert_eq!(mcp.arguments["name"], "compressor");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_shruti_ai_with_track() {
        let intent = Intent::ShrutiAi {
            action: "denoise".to_string(),
            track: Some("vocals".to_string()),
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_ai");
        assert_eq!(mcp.arguments["action"], "denoise");
        assert_eq!(mcp.arguments["track"], "vocals");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_shruti_ai_without_track() {
        let intent = Intent::ShrutiAi {
            action: "master".to_string(),
            track: None,
        };
        let t = translate_shruti(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "shruti_ai");
        assert_eq!(mcp.arguments["action"], "master");
        assert!(mcp.arguments.get("track").is_none());
    }
}
