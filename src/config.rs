use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseConfig {
    #[serde(default = "default_status")]
    pub status: u16,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default = "default_body")]
    pub body: String,
}

fn default_status() -> u16 {
    200
}

fn default_body() -> String {
    r#"{"status": "ok"}"#.to_string()
}

impl Default for ResponseConfig {
    fn default() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        Self {
            status: 200,
            headers,
            body: r#"{"status": "ok"}"#.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub path: String,
    #[serde(default)]
    pub method: Option<String>,
    pub response: ResponseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default)]
    pub response: ResponseConfig,
    #[serde(default)]
    pub routes: Vec<RouteConfig>,
}

fn default_port() -> u16 {
    9080
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 9080,
            host: "127.0.0.1".to_string(),
            response: ResponseConfig::default(),
            routes: vec![RouteConfig {
                path: "/health".to_string(),
                method: Some("GET".to_string()),
                response: ResponseConfig {
                    status: 200,
                    headers: HashMap::new(),
                    body: r#"{"healthy": true}"#.to_string(),
                },
            }],
        }
    }
}

impl Config {
    pub fn load() -> Self {
        // Try loading from multiple locations
        let config_paths = vec![
            PathBuf::from("./config.yaml"),
            PathBuf::from("./config.yml"),
            dirs::config_dir()
                .map(|p| p.join("webhook/config.yaml"))
                .unwrap_or_default(),
        ];

        for path in config_paths {
            if path.exists() {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_yaml::from_str::<Config>(&contents) {
                        return config;
                    }
                }
            }
        }

        // Return default config
        Config::default()
    }

    pub fn find_route(&self, method: &str, path: &str) -> Option<&RouteConfig> {
        self.routes.iter().find(|r| {
            r.path == path
                && r.method
                    .as_ref()
                    .map(|m| m.eq_ignore_ascii_case(method))
                    .unwrap_or(true)
        })
    }

    pub fn get_response(&self, method: &str, path: &str) -> &ResponseConfig {
        self.find_route(method, path)
            .map(|r| &r.response)
            .unwrap_or(&self.response)
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
