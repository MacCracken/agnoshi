use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_tarang(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::TarangProbe { path } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "path", path);
            Ok(mcp_call(
                "tarang_probe",
                a,
                format!("Probe media: {path}"),
                PermissionLevel::Safe,
                "Probes a media file for format, codec, and stream info via Tarang".to_string(),
            ))
        }
        Intent::TarangAnalyze { path } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "path", path);
            Ok(mcp_call(
                "tarang_analyze",
                a,
                format!("Analyze media: {path}"),
                PermissionLevel::Safe,
                "AI-powered media content analysis via Tarang".to_string(),
            ))
        }
        Intent::TarangCodecs => {
            let a = serde_json::Map::new();
            Ok(mcp_call(
                "tarang_codecs",
                a,
                "List supported codecs".to_string(),
                PermissionLevel::Safe,
                "Lists all audio and video codecs supported by Tarang".to_string(),
            ))
        }
        Intent::TarangTranscribe { path, language } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "path", path);
            insert_opt(&mut a, "language", language);
            Ok(mcp_call(
                "tarang_transcribe",
                a,
                format!("Transcribe: {path}"),
                PermissionLevel::SystemWrite,
                "Prepares audio transcription request via Tarang (routes to hoosh)".to_string(),
            ))
        }
        Intent::TarangFormats { path } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "path", path);
            Ok(mcp_call(
                "tarang_formats",
                a,
                format!("Detect format: {path}"),
                PermissionLevel::Safe,
                "Detects media container format from file header via Tarang".to_string(),
            ))
        }
        Intent::TarangFingerprintIndex { path } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "path", path);
            Ok(mcp_call(
                "tarang_fingerprint_index",
                a,
                format!("Index fingerprint: {path}"),
                PermissionLevel::SystemWrite,
                "Computes audio fingerprint and indexes in the vector store for similarity search"
                    .to_string(),
            ))
        }
        Intent::TarangSearchSimilar { path, top_k } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "path", path);
            if let Some(k) = top_k {
                a.insert(
                    "top_k".to_string(),
                    serde_json::Value::Number((*k as u64).into()),
                );
            }
            Ok(mcp_call(
                "tarang_search_similar",
                a,
                format!("Find similar to: {path}"),
                PermissionLevel::Safe,
                "Finds media files similar to a given file using audio fingerprint matching"
                    .to_string(),
            ))
        }
        Intent::TarangDescribe { path } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "path", path);
            Ok(mcp_call(
                "tarang_describe",
                a,
                format!("Describe media: {path}"),
                PermissionLevel::Safe,
                "Generates a rich AI content description using LLM analysis via hoosh".to_string(),
            ))
        }
        _ => unreachable!("translate_tarang called with non-tarang intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tarang_probe() {
        let intent = Intent::TarangProbe {
            path: "/tmp/song.mp3".to_string(),
        };
        let t = translate_tarang(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tarang_probe");
        assert_eq!(mcp.arguments["path"], "/tmp/song.mp3");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tarang_analyze() {
        let intent = Intent::TarangAnalyze {
            path: "/tmp/video.mkv".to_string(),
        };
        let t = translate_tarang(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tarang_analyze");
        assert_eq!(mcp.arguments["path"], "/tmp/video.mkv");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tarang_codecs() {
        let intent = Intent::TarangCodecs;
        let t = translate_tarang(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tarang_codecs");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tarang_transcribe_with_language() {
        let intent = Intent::TarangTranscribe {
            path: "/tmp/audio.wav".to_string(),
            language: Some("en".to_string()),
        };
        let t = translate_tarang(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tarang_transcribe");
        assert_eq!(mcp.arguments["path"], "/tmp/audio.wav");
        assert_eq!(mcp.arguments["language"], "en");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tarang_transcribe_no_language() {
        let intent = Intent::TarangTranscribe {
            path: "/tmp/audio.wav".to_string(),
            language: None,
        };
        let t = translate_tarang(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tarang_transcribe");
        assert_eq!(mcp.arguments["path"], "/tmp/audio.wav");
        assert!(mcp.arguments.get("language").is_none());
    }

    #[test]
    fn test_tarang_formats() {
        let intent = Intent::TarangFormats {
            path: "/tmp/clip.avi".to_string(),
        };
        let t = translate_tarang(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tarang_formats");
        assert_eq!(mcp.arguments["path"], "/tmp/clip.avi");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tarang_fingerprint_index() {
        let intent = Intent::TarangFingerprintIndex {
            path: "/tmp/track.flac".to_string(),
        };
        let t = translate_tarang(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tarang_fingerprint_index");
        assert_eq!(mcp.arguments["path"], "/tmp/track.flac");
        assert_eq!(t.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_tarang_search_similar_with_top_k() {
        let intent = Intent::TarangSearchSimilar {
            path: "/tmp/song.mp3".to_string(),
            top_k: Some(5),
        };
        let t = translate_tarang(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tarang_search_similar");
        assert_eq!(mcp.arguments["path"], "/tmp/song.mp3");
        assert_eq!(mcp.arguments["top_k"], 5);
        assert_eq!(t.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_tarang_search_similar_no_top_k() {
        let intent = Intent::TarangSearchSimilar {
            path: "/tmp/song.mp3".to_string(),
            top_k: None,
        };
        let t = translate_tarang(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tarang_search_similar");
        assert!(mcp.arguments.get("top_k").is_none());
    }

    #[test]
    fn test_tarang_describe() {
        let intent = Intent::TarangDescribe {
            path: "/tmp/podcast.mp3".to_string(),
        };
        let t = translate_tarang(&intent).unwrap();
        let mcp = t.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "tarang_describe");
        assert_eq!(mcp.arguments["path"], "/tmp/podcast.mp3");
        assert_eq!(t.permission, PermissionLevel::Safe);
    }
}
