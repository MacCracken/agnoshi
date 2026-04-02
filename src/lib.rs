//! Agnoshi — AI-native natural language shell for AGNOS.
//!
//! Translates natural language into system commands with human oversight,
//! security approval workflows, and full audit logging.
//!
//! # Public API
//!
//! The stable public surface is intentionally narrow:
//!
//! - [`Interpreter`] — natural language → intent → command translation
//! - [`Session`] — shell session lifecycle (interactive + one-shot)
//! - [`ShellConfig`] — shell configuration
//! - [`Mode`] — operating mode (Human, AiAssisted, AiAutonomous, Strict)
//! - [`SecurityContext`] — user privilege management
//!
//! Domain modules (`prompt`, `history`) are also public for advanced consumers.

pub mod aliases;
pub mod approval;
pub mod audit;
pub mod commands;
pub mod completion;
pub mod config;
pub mod dashboard;
pub mod history;
pub mod interpreter;
pub mod llm;
pub mod mode;
pub mod output;
pub mod permissions;
pub mod prompt;
pub mod sandbox;
pub mod schema_filter;
pub mod security;
pub mod session;
pub mod ui;

pub use config::ShellConfig;
pub use interpreter::{Intent, Interpreter, ListOptions, Translation};
pub use mode::Mode;
pub use security::{PermissionLevel, SecurityContext};
pub use session::Session;
