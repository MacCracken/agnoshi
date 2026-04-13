//! Checkpoint system for destructive operations.
//!
//! Before executing destructive commands (rm, mv, chmod, chown),
//! creates lightweight checkpoints that enable undo.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// A checkpoint recording a destructive operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub timestamp: String,
    pub command: String,
    pub args: Vec<String>,
    pub operation: CheckpointOp,
}

/// The type of operation checkpointed
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointOp {
    /// File removed — backed up to checkpoint dir
    FileRemoved {
        original_path: String,
        backup_path: String,
    },
    /// File moved — records source and destination
    FileMoved { source: String, destination: String },
    /// Permissions changed — records old permissions
    PermissionsChanged { path: String, old_mode: u32 },
    /// Ownership changed — records old owner
    OwnershipChanged { path: String, old_owner: String },
}

/// Manages checkpoints for undo capability
pub struct CheckpointManager {
    checkpoint_dir: PathBuf,
    checkpoints: Vec<Checkpoint>,
}

impl CheckpointManager {
    pub fn new() -> Self {
        let checkpoint_dir = std::env::temp_dir().join("agnoshi-checkpoints");
        Self {
            checkpoint_dir,
            checkpoints: Vec::new(),
        }
    }

    /// Create a checkpoint before a destructive rm operation
    pub async fn checkpoint_remove(&mut self, path: &Path) -> Result<Option<Checkpoint>> {
        if !path.exists() {
            return Ok(None);
        }

        let _ = tokio::fs::create_dir_all(&self.checkpoint_dir).await;

        let id = uuid::Uuid::new_v4().to_string();
        let backup_path = self.checkpoint_dir.join(&id);

        // Copy file/dir to checkpoint location
        if path.is_dir() {
            debug!("Skipping directory checkpoint for {:?} (too large)", path);
            return Ok(None);
        }

        tokio::fs::copy(path, &backup_path)
            .await
            .context("Failed to create checkpoint backup")?;

        let checkpoint = Checkpoint {
            id,
            timestamp: Utc::now().to_rfc3339(),
            command: "rm".to_string(),
            args: vec![path.to_string_lossy().to_string()],
            operation: CheckpointOp::FileRemoved {
                original_path: path.to_string_lossy().to_string(),
                backup_path: backup_path.to_string_lossy().to_string(),
            },
        };

        info!("Checkpoint created: {}", checkpoint.id);
        self.checkpoints.push(checkpoint.clone());
        Ok(Some(checkpoint))
    }

    /// Create a checkpoint before a mv operation
    pub fn checkpoint_move(&mut self, source: &str, destination: &str) -> Checkpoint {
        let id = uuid::Uuid::new_v4().to_string();
        let checkpoint = Checkpoint {
            id,
            timestamp: Utc::now().to_rfc3339(),
            command: "mv".to_string(),
            args: vec![source.to_string(), destination.to_string()],
            operation: CheckpointOp::FileMoved {
                source: source.to_string(),
                destination: destination.to_string(),
            },
        };
        self.checkpoints.push(checkpoint.clone());
        checkpoint
    }

    /// Undo the most recent checkpoint
    pub async fn undo_last(&mut self) -> Result<Option<String>> {
        let checkpoint = match self.checkpoints.pop() {
            Some(cp) => cp,
            None => return Ok(None),
        };

        match &checkpoint.operation {
            CheckpointOp::FileRemoved {
                original_path,
                backup_path,
            } => {
                let backup = Path::new(backup_path);
                let original = Path::new(original_path);
                if backup.exists() {
                    tokio::fs::rename(backup, original).await?;
                    Ok(Some(format!("Restored: {}", original_path)))
                } else {
                    Ok(Some(format!("Backup not found: {}", backup_path)))
                }
            }
            CheckpointOp::FileMoved {
                source,
                destination,
            } => {
                let src = Path::new(destination);
                let dst = Path::new(source);
                if src.exists() {
                    tokio::fs::rename(src, dst).await?;
                    Ok(Some(format!("Moved back: {} -> {}", destination, source)))
                } else {
                    Ok(Some(format!("File not found at: {}", destination)))
                }
            }
            _ => Ok(Some(
                "Undo not supported for this operation type".to_string(),
            )),
        }
    }

    /// Get number of available checkpoints
    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_checkpoint_remove_then_undo() {
        // Create a temp file
        let dir = std::env::temp_dir().join("agnoshi-test-checkpoint-rm");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("testfile.txt");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all(b"hello checkpoint").unwrap();
        }

        let mut mgr = CheckpointManager::new();
        let cp = mgr.checkpoint_remove(&file_path).await.unwrap();
        assert!(cp.is_some());
        assert_eq!(mgr.checkpoint_count(), 1);

        // Simulate rm by deleting the file
        std::fs::remove_file(&file_path).unwrap();
        assert!(!file_path.exists());

        // Undo should restore it
        let result = mgr.undo_last().await.unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().contains("Restored"));
        assert!(file_path.exists());

        // Verify content
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello checkpoint");

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_checkpoint_move_then_undo() {
        let dir = std::env::temp_dir().join("agnoshi-test-checkpoint-mv");
        let _ = std::fs::create_dir_all(&dir);
        let src = dir.join("source.txt");
        let dst = dir.join("dest.txt");
        {
            let mut f = std::fs::File::create(&src).unwrap();
            f.write_all(b"move me").unwrap();
        }

        let mut mgr = CheckpointManager::new();
        let cp = mgr.checkpoint_move(&src.to_string_lossy(), &dst.to_string_lossy());
        assert_eq!(cp.command, "mv");
        assert_eq!(mgr.checkpoint_count(), 1);

        // Simulate mv
        std::fs::rename(&src, &dst).unwrap();
        assert!(!src.exists());
        assert!(dst.exists());

        // Undo
        let result = mgr.undo_last().await.unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().contains("Moved back"));
        assert!(src.exists());
        assert!(!dst.exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_undo_last_returns_none_when_empty() {
        let mut mgr = CheckpointManager::new();
        let result = mgr.undo_last().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_checkpoint_remove_nonexistent_file() {
        let mut mgr = CheckpointManager::new();
        let result = mgr
            .checkpoint_remove(Path::new("/nonexistent/file.txt"))
            .await
            .unwrap();
        assert!(result.is_none());
        assert_eq!(mgr.checkpoint_count(), 0);
    }

    #[tokio::test]
    async fn test_checkpoint_remove_directory_skipped() {
        let dir = std::env::temp_dir().join("agnoshi-test-checkpoint-dir");
        let _ = std::fs::create_dir_all(&dir);

        let mut mgr = CheckpointManager::new();
        let result = mgr.checkpoint_remove(&dir).await.unwrap();
        assert!(result.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
