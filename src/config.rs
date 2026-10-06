use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DocumentConfig {
    pub author: Option<String>,
    pub affiliation: Option<String>,
    pub fontsize: Option<String>,
    pub mainfont: Option<String>,
    pub mathfont: Option<String>,
    pub papersize: Option<String>,
    pub columns: Option<usize>,
    pub section_numbering: Option<bool>,
    pub toc: Option<bool>,
    pub margin: Option<MarginConfig>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum MarginConfig {
    Uniform(String),
    Axes { x: Option<String>, y: Option<String> },
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SlidesConfig {
    pub theme: Option<String>,
    pub paginate: Option<bool>,
    pub font: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct BotoxConfig {
    pub document: Option<DocumentConfig>,
    pub slides: Option<SlidesConfig>,
}

impl BotoxConfig {
    pub fn load(custom_path: Option<&Path>) -> (Self, Option<PathBuf>) {
        let mut candidates = Vec::new();

        if let Some(cp) = custom_path {
            candidates.push(cp.to_path_buf());
        }

        candidates.push(PathBuf::from("botox.yaml"));
        candidates.push(PathBuf::from("skygem.yaml"));

        if let Some(home) = dirs_home() {
            candidates.push(home.join(".config").join("botox").join("config.yaml"));
            candidates.push(home.join(".config").join("skygem").join("config.yaml"));
        }

        if let Ok(appdata) = std::env::var("APPDATA") {
            candidates.push(PathBuf::from(appdata).join("botox").join("config.yaml"));
        }

        for path in candidates {
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(cfg) = serde_yaml::from_str::<BotoxConfig>(&content) {
                        return (cfg, Some(path));
                    }
                }
            }
        }

        (BotoxConfig::default(), None)
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .and_then(|h| if h.is_empty() { None } else { Some(PathBuf::from(h)) })
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .and_then(|h| if h.is_empty() { None } else { Some(PathBuf::from(h)) })
        })
}
