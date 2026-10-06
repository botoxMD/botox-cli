use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocumentConfig {
    pub author: Option<String>,
    pub affiliation: Option<String>,
    pub fontsize: Option<String>,
    pub mainfont: Option<String>,
    pub mathfont: Option<String>,
    pub monofont: Option<String>,
    pub papersize: Option<String>,
    pub columns: Option<usize>,
    pub section_numbering: Option<bool>,
    pub toc: Option<bool>,
    pub margin: Option<MarginConfig>,
    pub bibliography: Option<bool>,
    pub lang: Option<String>,
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self::defaults()
    }
}

impl DocumentConfig {
    pub fn defaults() -> Self {
        Self {
            author: None,
            affiliation: None,
            fontsize: Some("11pt".to_string()),
            mainfont: Some("New Computer Modern".to_string()),
            mathfont: Some("New Computer Modern Math".to_string()),
            monofont: Some("DejaVu Sans Mono".to_string()),
            papersize: Some("a4".to_string()),
            columns: Some(1),
            section_numbering: Some(false),
            toc: Some(false),
            margin: Some(MarginConfig::Axes {
                x: Some("2.5cm".to_string()),
                y: Some("2.5cm".to_string()),
            }),
            bibliography: Some(false),
            lang: Some("en".to_string()),
        }
    }

    pub fn merge_with(&mut self, other: &Self) {
        if other.author.is_some() { self.author = other.author.clone(); }
        if other.affiliation.is_some() { self.affiliation = other.affiliation.clone(); }
        if other.fontsize.is_some() { self.fontsize = other.fontsize.clone(); }
        if other.mainfont.is_some() { self.mainfont = other.mainfont.clone(); }
        if other.mathfont.is_some() { self.mathfont = other.mathfont.clone(); }
        if other.monofont.is_some() { self.monofont = other.monofont.clone(); }
        if other.papersize.is_some() { self.papersize = other.papersize.clone(); }
        if other.columns.is_some() { self.columns = other.columns; }
        if other.section_numbering.is_some() { self.section_numbering = other.section_numbering; }
        if other.toc.is_some() { self.toc = other.toc; }
        if other.margin.is_some() { self.margin = other.margin.clone(); }
        if other.bibliography.is_some() { self.bibliography = other.bibliography; }
        if other.lang.is_some() { self.lang = other.lang.clone(); }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum MarginConfig {
    Uniform(String),
    Axes { x: Option<String>, y: Option<String> },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SlidesConfig {
    pub theme: Option<String>,
    pub paginate: Option<bool>,
    pub font: Option<String>,
    pub background_color: Option<String>,
    pub color: Option<String>,
}

impl Default for SlidesConfig {
    fn default() -> Self {
        Self::defaults()
    }
}

impl SlidesConfig {
    pub fn defaults() -> Self {
        Self {
            theme: Some("default".to_string()),
            paginate: Some(true),
            font: Some("New Computer Modern".to_string()),
            background_color: None,
            color: None,
        }
    }

    pub fn merge_with(&mut self, other: &Self) {
        if other.theme.is_some() { self.theme = other.theme.clone(); }
        if other.paginate.is_some() { self.paginate = other.paginate; }
        if other.font.is_some() { self.font = other.font.clone(); }
        if other.background_color.is_some() { self.background_color = other.background_color.clone(); }
        if other.color.is_some() { self.color = other.color.clone(); }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BotoxConfig {
    pub document: Option<DocumentConfig>,
    pub slides: Option<SlidesConfig>,
}

impl Default for BotoxConfig {
    fn default() -> Self {
        Self::defaults()
    }
}

impl BotoxConfig {
    pub fn defaults() -> Self {
        Self {
            document: Some(DocumentConfig::defaults()),
            slides: Some(SlidesConfig::defaults()),
        }
    }

    pub fn merge_with(&mut self, other: &Self) {
        if let Some(ref other_doc) = other.document {
            if let Some(ref mut doc) = self.document {
                doc.merge_with(other_doc);
            } else {
                self.document = Some(other_doc.clone());
            }
        }
        if let Some(ref other_slides) = other.slides {
            if let Some(ref mut slides) = self.slides {
                slides.merge_with(other_slides);
            } else {
                self.slides = Some(other_slides.clone());
            }
        }
    }

    pub fn load(custom_path: Option<&Path>, doc_dir: Option<&Path>) -> (Self, Vec<PathBuf>) {
        let mut resolved = Self::defaults();
        let mut loaded_sources = Vec::new();

        // 1. User-level global config (~/.config/botox/config.yaml, etc.)
        let mut global_candidates = Vec::new();
        if let Some(home) = dirs_home() {
            global_candidates.push(home.join(".config").join("botox").join("config.yaml"));
            global_candidates.push(home.join(".config").join("skygem").join("config.yaml"));
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            global_candidates.push(PathBuf::from(appdata).join("botox").join("config.yaml"));
        }
        for path in global_candidates {
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(cfg) = serde_yaml::from_str::<BotoxConfig>(&content) {
                        resolved.merge_with(&cfg);
                        loaded_sources.push(path);
                        break;
                    }
                }
            }
        }

        // 2. Current working directory config (./botox.yaml, ./skygem.yaml)
        let cwd_candidates = [PathBuf::from("botox.yaml"), PathBuf::from("skygem.yaml")];
        for path in &cwd_candidates {
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(cfg) = serde_yaml::from_str::<BotoxConfig>(&content) {
                        resolved.merge_with(&cfg);
                        let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
                        loaded_sources.push(canon);
                        break;
                    }
                }
            }
        }

        // 3. Document directory config (<doc_dir>/botox.yaml, <doc_dir>/skygem.yaml)
        if let Some(dir) = doc_dir {
            let doc_candidates = [dir.join("botox.yaml"), dir.join("skygem.yaml")];
            for path in &doc_candidates {
                if path.is_file() {
                    let canon_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
                    if !loaded_sources.contains(&canon_path) {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            if let Ok(cfg) = serde_yaml::from_str::<BotoxConfig>(&content) {
                                resolved.merge_with(&cfg);
                                loaded_sources.push(canon_path);
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 4. Custom config specified via --config <path>
        if let Some(cp) = custom_path {
            if cp.is_file() {
                if let Ok(content) = std::fs::read_to_string(cp) {
                    if let Ok(cfg) = serde_yaml::from_str::<BotoxConfig>(&content) {
                        resolved.merge_with(&cfg);
                        loaded_sources.push(cp.to_path_buf());
                    }
                }
            }
        }

        (resolved, loaded_sources)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = BotoxConfig::defaults();
        let slides = cfg.slides.expect("slides config");
        assert_eq!(slides.theme.as_deref(), Some("default"));
        assert_eq!(slides.paginate, Some(true));

        let doc = cfg.document.expect("doc config");
        assert_eq!(doc.fontsize.as_deref(), Some("11pt"));
        assert_eq!(doc.mainfont.as_deref(), Some("New Computer Modern"));
        assert_eq!(doc.papersize.as_deref(), Some("a4"));
    }

    #[test]
    fn test_config_merging() {
        let mut base = BotoxConfig::defaults();
        let override_yaml = r#"
slides:
  theme: "academic"
document:
  fontsize: "12pt"
  mainfont: "Times New Roman"
"#;
        let parsed: BotoxConfig = serde_yaml::from_str(override_yaml).expect("parse yaml");
        base.merge_with(&parsed);

        let slides = base.slides.expect("slides config");
        // Overridden
        assert_eq!(slides.theme.as_deref(), Some("academic"));
        // Preserved default
        assert_eq!(slides.paginate, Some(true));
        assert_eq!(slides.font.as_deref(), Some("New Computer Modern"));

        let doc = base.document.expect("doc config");
        // Overridden
        assert_eq!(doc.fontsize.as_deref(), Some("12pt"));
        assert_eq!(doc.mainfont.as_deref(), Some("Times New Roman"));
        // Preserved default
        assert_eq!(doc.papersize.as_deref(), Some("a4"));
        assert_eq!(doc.monofont.as_deref(), Some("DejaVu Sans Mono"));
    }
}
