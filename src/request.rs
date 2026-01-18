use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedRequest {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl CapturedRequest {
    pub fn new(
        id: u64,
        method: String,
        path: String,
        query: Option<String>,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            method,
            path,
            query,
            headers,
            body,
        }
    }

    pub fn full_path(&self) -> String {
        match &self.query {
            Some(q) if !q.is_empty() => format!("{}?{}", self.path, q),
            _ => self.path.clone(),
        }
    }

    pub fn timestamp_display(&self) -> String {
        self.timestamp.format("%H:%M:%S").to_string()
    }

    pub fn formatted_body(&self) -> Option<String> {
        self.body.as_ref().map(|b| {
            // Try to pretty-print JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(b) {
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| b.clone())
            } else {
                b.clone()
            }
        })
    }
}

impl fmt::Display for CapturedRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} {}",
            self.timestamp_display(),
            self.method,
            self.full_path()
        )
    }
}
