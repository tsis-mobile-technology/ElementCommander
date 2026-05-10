pub mod client;
pub mod state;

pub use client::AiClient;
pub use state::AiState;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub thinking: Option<String>,  // CoT thinking process
    pub result: String,             // 최종 결과
    pub is_error: bool,
}

impl AiResponse {
    pub fn new(thinking: Option<String>, result: String) -> Self {
        Self {
            thinking,
            result,
            is_error: false,
        }
    }

    pub fn error(content: String) -> Self {
        Self {
            thinking: None,
            result: content,
            is_error: true,
        }
    }
}
