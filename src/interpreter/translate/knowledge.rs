use anyhow::Result;
use serde_json::json;

use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_knowledge(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::KnowledgeSearch { query, source } => {
            let _source_flag = source
                .as_ref()
                .map(|s| format!(" --source {}", s))
                .unwrap_or_default();
            Ok(Translation {
                command: "curl".to_string(),
                args: vec![
                    "-s".to_string(),
                    "-X".to_string(),
                    "POST".to_string(),
                    "http://127.0.0.1:8090/v1/knowledge/search".to_string(),
                    "-H".to_string(),
                    "Content-Type: application/json".to_string(),
                    "-d".to_string(),
                    json!({"query": query, "limit": 10}).to_string(),
                ],
                description: format!("Search knowledge base for: {}", query),
                permission: PermissionLevel::Safe,
                explanation: "Searches the local knowledge base index".to_string(),
                mcp: None,
            })
        }

        Intent::RagQuery { query } => Ok(Translation {
            command: "curl".to_string(),
            args: vec![
                "-s".to_string(),
                "-X".to_string(),
                "POST".to_string(),
                "http://127.0.0.1:8090/v1/rag/query".to_string(),
                "-H".to_string(),
                "Content-Type: application/json".to_string(),
                "-d".to_string(),
                json!({"query": query, "top_k": 5}).to_string(),
            ],
            description: format!("RAG query: {}", query),
            permission: PermissionLevel::Safe,
            explanation: "Retrieves context-augmented results from the RAG pipeline".to_string(),
            mcp: None,
        }),

        _ => unreachable!("translate_knowledge called with non-knowledge intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_search_json_injection_with_quotes() {
        let intent = Intent::KnowledgeSearch {
            query: r#"test","malicious":"true"#.to_string(),
            source: None,
        };
        let result = translate_knowledge(&intent).unwrap();
        // The JSON payload must remain valid and the injected key must not appear
        // as a top-level field.
        let payload = result.args.last().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert_eq!(parsed["query"], r#"test","malicious":"true"#);
        assert!(parsed.get("malicious").is_none());
    }

    #[test]
    fn test_knowledge_search_json_injection_with_backslash_newline() {
        let intent = Intent::KnowledgeSearch {
            query: "test\ninjection".to_string(),
            source: None,
        };
        let result = translate_knowledge(&intent).unwrap();
        let payload = result.args.last().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert_eq!(parsed["query"], "test\ninjection");
        // Ensure the limit field is still intact (payload not broken).
        assert_eq!(parsed["limit"], 10);
    }

    #[test]
    fn test_knowledge_search_normal_query() {
        let intent = Intent::KnowledgeSearch {
            query: "how to configure networking".to_string(),
            source: None,
        };
        let result = translate_knowledge(&intent).unwrap();
        let payload = result.args.last().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert_eq!(parsed["query"], "how to configure networking");
        assert_eq!(parsed["limit"], 10);
    }

    #[test]
    fn test_rag_query_json_injection_with_quotes() {
        let intent = Intent::RagQuery {
            query: r#"test","malicious":"true"#.to_string(),
        };
        let result = translate_knowledge(&intent).unwrap();
        let payload = result.args.last().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert_eq!(parsed["query"], r#"test","malicious":"true"#);
        assert!(parsed.get("malicious").is_none());
        assert_eq!(parsed["top_k"], 5);
    }

    #[test]
    fn test_rag_query_json_injection_with_backslash_newline() {
        let intent = Intent::RagQuery {
            query: "test\\ninjection".to_string(),
        };
        let result = translate_knowledge(&intent).unwrap();
        let payload = result.args.last().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert_eq!(parsed["query"], "test\\ninjection");
        assert_eq!(parsed["top_k"], 5);
    }
}
