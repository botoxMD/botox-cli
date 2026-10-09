pub mod yaml;

pub use yaml::generate_config_yaml;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocumentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fontsize: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mainfont: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mathfont: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monofont: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub papersize: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_numbering: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toc: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toc_depth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<MarginConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bibliography: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkcolor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toc_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<ReferencesConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string_or_vec", alias = "exclude-references", alias = "exclude_references", alias = "exclude-links")]
    pub exclude_references: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string_or_vec", alias = "include-references", alias = "include_references", alias = "include-links")]
    pub include_references: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "krocki-url", alias = "krocki_url", alias = "kroki-url")]
    pub kroki_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ReferencesConfig {
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string_or_vec")]
    pub exclude: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_string_or_vec")]
    pub include: Option<Vec<String>>,
}

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Single(String),
        Multiple(Vec<String>),
    }

    let opt = Option::<StringOrVec>::deserialize(deserializer)?;
    Ok(opt.map(|val| match val {
        StringOrVec::Single(s) => vec![s],
        StringOrVec::Multiple(v) => v,
    }))
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self::defaults()
    }
}

impl DocumentConfig {
    pub fn defaults() -> Self {
        Self {
            theme: Some("academic".to_string()),
            author: None,
            affiliation: None,
            fontsize: None,
            mainfont: None,
            mathfont: Some("New Computer Modern Math".to_string()),
            monofont: Some("DejaVu Sans Mono".to_string()),
            papersize: Some("a4".to_string()),
            columns: None,
            section_numbering: None,
            toc: Some(false),
            toc_depth: Some(3),
            toc_title: None,
            margin: None,
            bibliography: Some(false),
            lang: Some("en".to_string()),
            linkcolor: None,
            references: None,
            exclude_references: None,
            include_references: None,
            kroki_url: None,
        }
    }

    pub fn merge_with(&mut self, other: &Self) {
        if other.theme.is_some() { self.theme = other.theme.clone(); }
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
        if other.toc_depth.is_some() { self.toc_depth = other.toc_depth; }
        if other.toc_title.is_some() { self.toc_title = other.toc_title.clone(); }
        if other.margin.is_some() { self.margin = other.margin.clone(); }
        if other.bibliography.is_some() { self.bibliography = other.bibliography; }
        if other.lang.is_some() { self.lang = other.lang.clone(); }
        if other.linkcolor.is_some() { self.linkcolor = other.linkcolor.clone(); }
        if other.references.is_some() { self.references = other.references.clone(); }
        if other.exclude_references.is_some() { self.exclude_references = other.exclude_references.clone(); }
        if other.include_references.is_some() { self.include_references = other.include_references.clone(); }
        if other.kroki_url.is_some() { self.kroki_url = other.kroki_url.clone(); }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paginate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
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
            author: None,
        }
    }

    pub fn merge_with(&mut self, other: &Self) {
        if other.theme.is_some() { self.theme = other.theme.clone(); }
        if other.paginate.is_some() { self.paginate = other.paginate; }
        if other.font.is_some() { self.font = other.font.clone(); }
        if other.background_color.is_some() { self.background_color = other.background_color.clone(); }
        if other.color.is_some() { self.color = other.color.clone(); }
        if other.author.is_some() { self.author = other.author.clone(); }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BotoxConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<DocumentConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slides: Option<SlidesConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<ReferencesConfig>,
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
            references: None,
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
        if let Some(ref other_ref) = other.references {
            if let Some(ref mut r) = self.references {
                if other_ref.exclude.is_some() { r.exclude = other_ref.exclude.clone(); }
                if other_ref.include.is_some() { r.include = other_ref.include.clone(); }
            } else {
                self.references = Some(other_ref.clone());
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
            if path.is_file()
                && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(cfg) = serde_yaml::from_str::<BotoxConfig>(&content) {
                    resolved.merge_with(&cfg);
                    loaded_sources.push(path);
                    break;
                }
        }

        // 2. Current working directory config (./botox.yaml, ./skygem.yaml)
        let cwd_candidates = [PathBuf::from("botox.yaml"), PathBuf::from("skygem.yaml")];
        for path in &cwd_candidates {
            if path.is_file()
                && let Ok(content) = std::fs::read_to_string(path)
                && let Ok(cfg) = serde_yaml::from_str::<BotoxConfig>(&content) {
                    resolved.merge_with(&cfg);
                    let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
                    loaded_sources.push(canon);
                    break;
                }
        }

        // 3. Document directory config (<doc_dir>/botox.yaml, <doc_dir>/skygem.yaml)
        if let Some(dir) = doc_dir {
            let doc_candidates = [dir.join("botox.yaml"), dir.join("skygem.yaml")];
            for path in &doc_candidates {
                if path.is_file() {
                    let canon_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
                    if !loaded_sources.contains(&canon_path)
                        && let Ok(content) = std::fs::read_to_string(path)
                        && let Ok(cfg) = serde_yaml::from_str::<BotoxConfig>(&content) {
                            resolved.merge_with(&cfg);
                            loaded_sources.push(canon_path);
                            break;
                        }
                }
            }
        }

        // 4. Custom config specified via --config <path>
        if let Some(cp) = custom_path
            && cp.is_file()
            && let Ok(content) = std::fs::read_to_string(cp)
            && let Ok(cfg) = serde_yaml::from_str::<BotoxConfig>(&content) {
                resolved.merge_with(&cfg);
                loaded_sources.push(cp.to_path_buf());
            }

        (resolved, loaded_sources)
    }
}

pub fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .and_then(|h| if h.is_empty() { None } else { Some(PathBuf::from(h)) })
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .and_then(|h| if h.is_empty() { None } else { Some(PathBuf::from(h)) })
        })
}

pub fn global_config_path() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA")
        && !appdata.trim().is_empty() {
            return PathBuf::from(appdata).join("botox").join("config.yaml");
        }
    if let Some(home) = dirs_home() {
        return home.join(".config").join("botox").join("config.yaml");
    }
    PathBuf::from("config.yaml")
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
        assert_eq!(doc.theme.as_deref(), Some("academic"));
        assert_eq!(doc.papersize.as_deref(), Some("a4"));
    }

    #[test]
    fn test_config_merging() {
        let mut base = BotoxConfig::defaults();
        let override_yaml = r#"
slides:
  theme: "academic"
document:
  theme: "modern"
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
        assert_eq!(doc.theme.as_deref(), Some("modern"));
        assert_eq!(doc.fontsize.as_deref(), Some("12pt"));
        assert_eq!(doc.mainfont.as_deref(), Some("Times New Roman"));
        // Preserved default
        assert_eq!(doc.papersize.as_deref(), Some("a4"));
        assert_eq!(doc.monofont.as_deref(), Some("DejaVu Sans Mono"));
    }

    #[test]
    fn test_generate_config_yaml_parseable() {
        let yaml = generate_config_yaml(
            "Test Author",
            Some("Research Lab"),
            "modern",
            "nord",
            "us-letter",
            true,
            false,
        );
        let parsed: BotoxConfig = serde_yaml::from_str(&yaml).expect("parse generated yaml");
        let doc = parsed.document.expect("doc");
        assert_eq!(doc.author.as_deref(), Some("Test Author"));
        assert_eq!(doc.affiliation.as_deref(), Some("Research Lab"));
        assert_eq!(doc.theme.as_deref(), Some("modern"));
        assert_eq!(doc.papersize.as_deref(), Some("us-letter"));
        assert_eq!(doc.toc, Some(true));
        assert_eq!(doc.bibliography, Some(false));

        let slides = parsed.slides.expect("slides");
        assert_eq!(slides.theme.as_deref(), Some("nord"));
        assert_eq!(slides.author.as_deref(), Some("Test Author"));
    }
}
