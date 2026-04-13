use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_jalwa(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::JalwaPlay { path } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "path", path);
            Ok(mcp_call(
                "jalwa_play",
                a,
                format!("Play: {path}"),
                PermissionLevel::SystemWrite,
                "Plays a media file via Jalwa media player".to_string(),
            ))
        }
        Intent::JalwaPause => {
            let a = serde_json::Map::new();
            Ok(mcp_call(
                "jalwa_pause",
                a,
                "Pause playback".to_string(),
                PermissionLevel::SystemWrite,
                "Pauses current playback in Jalwa".to_string(),
            ))
        }
        Intent::JalwaStatus => {
            let a = serde_json::Map::new();
            Ok(mcp_call(
                "jalwa_status",
                a,
                "Playback status".to_string(),
                PermissionLevel::Safe,
                "Gets current playback status from Jalwa".to_string(),
            ))
        }
        Intent::JalwaSearch { query } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "query", query);
            Ok(mcp_call(
                "jalwa_search",
                a,
                format!("Search library: {query}"),
                PermissionLevel::Safe,
                "Searches the Jalwa media library".to_string(),
            ))
        }
        Intent::JalwaRecommend { item_id, max } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "item_id", item_id);
            if let Some(m) = max {
                a.insert("max".to_string(), serde_json::Value::Number((*m).into()));
            }
            Ok(mcp_call(
                "jalwa_recommend",
                a,
                "Get recommendations".to_string(),
                PermissionLevel::Safe,
                "Gets AI-powered media recommendations from Jalwa".to_string(),
            ))
        }
        Intent::JalwaQueue { action, item_id } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "item_id", item_id);
            Ok(mcp_call(
                "jalwa_queue",
                a,
                format!("Queue: {action}"),
                PermissionLevel::SystemWrite,
                "Manages the Jalwa play queue".to_string(),
            ))
        }
        Intent::JalwaLibrary { action, path } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "path", path);
            Ok(mcp_call(
                "jalwa_library",
                a,
                format!("Library: {action}"),
                PermissionLevel::Safe,
                "Manages the Jalwa media library".to_string(),
            ))
        }
        Intent::JalwaPlaylist {
            action,
            name,
            item_id,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            insert_opt(&mut a, "item_id", item_id);
            Ok(mcp_call(
                "jalwa_playlist",
                a,
                format!("Playlist: {action}"),
                PermissionLevel::SystemWrite,
                "Manages Jalwa playlists".to_string(),
            ))
        }
        _ => unreachable!("translate_jalwa called with non-jalwa intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play() {
        let intent = Intent::JalwaPlay {
            path: "/music/song.mp3".to_string(),
        };
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "jalwa_play");
        assert_eq!(mcp.arguments["path"], "/music/song.mp3");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_pause() {
        let intent = Intent::JalwaPause;
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "jalwa_pause");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_status() {
        let intent = Intent::JalwaStatus;
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "jalwa_status");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_search() {
        let intent = Intent::JalwaSearch {
            query: "beethoven".to_string(),
        };
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "jalwa_search");
        assert_eq!(mcp.arguments["query"], "beethoven");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_recommend_with_max() {
        let intent = Intent::JalwaRecommend {
            item_id: "item-42".to_string(),
            max: Some(5),
        };
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "jalwa_recommend");
        assert_eq!(mcp.arguments["item_id"], "item-42");
        assert_eq!(mcp.arguments["max"], 5);
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_recommend_without_max() {
        let intent = Intent::JalwaRecommend {
            item_id: "item-99".to_string(),
            max: None,
        };
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["item_id"], "item-99");
        assert!(mcp.arguments.get("max").is_none());
    }

    #[test]
    fn test_queue_with_item() {
        let intent = Intent::JalwaQueue {
            action: "add".to_string(),
            item_id: Some("track-7".to_string()),
        };
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "jalwa_queue");
        assert_eq!(mcp.arguments["action"], "add");
        assert_eq!(mcp.arguments["item_id"], "track-7");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_queue_without_item() {
        let intent = Intent::JalwaQueue {
            action: "clear".to_string(),
            item_id: None,
        };
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["action"], "clear");
        assert!(mcp.arguments.get("item_id").is_none());
    }

    #[test]
    fn test_library_with_path() {
        let intent = Intent::JalwaLibrary {
            action: "scan".to_string(),
            path: Some("/media/music".to_string()),
        };
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "jalwa_library");
        assert_eq!(mcp.arguments["action"], "scan");
        assert_eq!(mcp.arguments["path"], "/media/music");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_library_without_path() {
        let intent = Intent::JalwaLibrary {
            action: "stats".to_string(),
            path: None,
        };
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert!(mcp.arguments.get("path").is_none());
    }

    #[test]
    fn test_playlist_with_all_fields() {
        let intent = Intent::JalwaPlaylist {
            action: "add".to_string(),
            name: Some("favorites".to_string()),
            item_id: Some("track-1".to_string()),
        };
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "jalwa_playlist");
        assert_eq!(mcp.arguments["action"], "add");
        assert_eq!(mcp.arguments["name"], "favorites");
        assert_eq!(mcp.arguments["item_id"], "track-1");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_playlist_minimal() {
        let intent = Intent::JalwaPlaylist {
            action: "list".to_string(),
            name: None,
            item_id: None,
        };
        let result = translate_jalwa(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["action"], "list");
        assert!(mcp.arguments.get("name").is_none());
        assert!(mcp.arguments.get("item_id").is_none());
    }
}
