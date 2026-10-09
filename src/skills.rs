use std::fs;
use std::path::{Path, PathBuf};

/// Returns the complete, production-grade SKILL.md definition for AI coding agents.
pub fn generate_skill_content() -> &'static str {
    r#"---
name: botox
description: >-
  Use Botox to compile Markdown files into publication-quality LaTeX PDF documents
  or interactive HTML presentation slides. Provides comprehensive guidance on document
  frontmatter, slide creation, styling themes, math typesetting, Kroki/Mermaid diagrams,
  callouts, bibliographies, and CLI flags.
---

# Botox: Publication-Grade Typesetting & Presentations

Botox is a high-performance, single native binary document compiler written in pure Rust. It transforms standard Markdown into LaTeX-quality PDF documents or interactive HTML slide decks using an in-process Typst engine.

## Command Line Usage

### Basic Commands
```bash
# 1. Initialize a new document or presentation
botox init document.md               # Create a new academic PDF document template
botox init slides.md                 # Create a new presentation slide deck template

# 2. Compile documents (PDF is default for documents)
botox document.md                    # Compiles to document.pdf
botox document.md -o custom.pdf      # Explicit output path
botox document.md -o custom.html     # Export document to HTML

# 3. Compile presentations (HTML is default for slides)
botox slides.md                      # Compiles to interactive slides.html
botox slides.md -o deck.pdf          # Export slides to PDF

# 4. Live watch mode (recompiles on change)
botox -w document.md
botox --watch slides.md

# 5. Unix pipeline mode (stream compiled PDF directly to stdout)
cat document.md | botox - -o - > output.pdf

# 6. Configuration & Setup
botox config                         # Show active configuration and loaded sources
botox setup                          # Interactive setup wizard
botox setup --global --author "Name" # Non-interactive configuration
botox --skills                       # Generate this skill file for AI agents
```

### CLI Options Reference
| Flag | Description |
| :--- | :--- |
| `-o, --output <file>` | Destination file (`.pdf`, `.html`, `.svg`, `.json`, or `-` for stdout) |
| `-w, --watch` | Watch input file and directory for changes and recompile automatically |
| `--pdf` | Force PDF output mode |
| `--slides` | Force presentation slide deck mode |
| `--theme <theme>` | Document theme (`academic`, `modern`, `elegant`, `technical`, `compact`, `minimal`) or Slide theme (`default`, `academic`, `gaia`, `uncover`, `nord`, `dark`) |
| `--toc` / `--no-toc` | Enable or disable automatic Table of Contents |
| `--author <name>` | Author name (overrides frontmatter and config) |
| `--font <name>` | Main font (default: New Computer Modern) |
| `-N, --number-sections` | Automatically number section headings |
| `-b, --bibliography` | Transform hyperlinks into an IEEE-standard Bibliography |
| `--no-bibliography` | Disable automatic bibliography generation |
| `--exclude-ref <patterns>` | Exclude matching URL patterns/domains from bibliography |
| `--include-ref <patterns>` | Only include matching URL patterns/domains in bibliography |

---

## Document Frontmatter Reference

Documents use YAML frontmatter at the top of the Markdown file:

```yaml
---
title: "Document Title"
subtitle: "Document Subtitle"
author:
  - name: "Author Name"
    affiliation: "Organization or University"
date: \today
lang: en
papersize: a4                        # a4, us-letter
fontsize: 11pt                       # 10pt, 11pt, 12pt
mainfont: "New Computer Modern"      # Any installed system font
geometry: "margin=2.5cm"
number-sections: true
table-of-contents: true
toc-depth: 3
bibliography: true                   # Turns hyperlinks into IEEE citations
theme: academic                      # academic, modern, elegant, technical, compact, minimal
---

# Introduction

Body text goes here...
```

### Document Themes
* **`academic`** (default): Traditional LaTeX / Computer Modern style with serif typography and clean rules.
* **`modern`**: Clean sans-serif headings, subtle blue accents, and open margins.
* **`elegant`**: Sophisticated editorial layout suitable for books, essays, and long-form writing.
* **`technical`**: Monospace accents, sharp borders, structured headers for RFCs and engineering specs.
* **`compact`**: Space-efficient layout optimized for summaries and multi-column papers.
* **`minimal`**: Pure, distraction-free typography without decorative elements.

---

## Presentation Slides Reference

Slide decks are separated into individual slides using horizontal rules (`---`) or `\newpage`:

```yaml
---
marp: true
theme: default                       # default, academic, gaia, uncover, nord, dark
paginate: true                       # Show slide numbers
header: "Company Confidential"
footer: "Quarterly Review"
---

# Presentation Title
### Subtitle or Speaker Name

---

# Key Objectives

- High performance native Rust compilation
- Direct conversion to interactive HTML slides
- Full support for mathematics and diagrams

---

# Architecture Overview

```mermaid
graph TD;
  A[Markdown] --> B[Botox Compiler];
  B --> C[Interactive HTML Slides];
  B --> D[Publication PDF];
```
```

### Slide Themes
* **`default`**: Clean, modern light theme.
* **`academic`**: Traditional academic beamer style.
* **`gaia`**: Warm terracotta and earth tones (Marp compatible).
* **`uncover`**: High-contrast minimalist style with centered content.
* **`nord`**: Cool Arctic frost dark theme.
* **`dark`**: Sleek slate and charcoal dark mode.

---

## Mathematics Typesetting

Botox supports native Typst/LaTeX mathematical syntax:

* **Inline Math**: Enclose equations in single dollar signs: `$E = m c^2$`
* **Display Math**: Enclose equations in double dollar signs:
  ```latex
  $$\int_{-\infty}^{+\infty} e^{-x^2} \, dx = \sqrt{\pi}$$
  ```
* **Equation Labels & Cross-References**:
  ```latex
  $$\vec{F} = m \frac{d\vec{v}}{dt}$$ {#eq:force}

  As defined in @eq:force, force equals mass times acceleration.
  ```

---

## Diagram Rendering (Kroki & Mermaid)

Botox natively compiles diagram code blocks into sharp, scalable vector graphics:

````markdown
```mermaid
sequenceDiagram
    Alice->>Bob: Hello Bob!
    Bob-->>Alice: Hello Alice!
```
````

Supported diagram engines:
* `mermaid` (flowcharts, sequence, class, state, gantt, gitGraph)
* `plantuml` / `puml`
* `graphviz` / `dot`
* `d2`
* `bytefield`
* `c4plantuml`

Diagrams can also have captions and reference labels:
````markdown
```{mermaid, caption="System Architecture", id="fig:arch"}
graph LR;
  Client --> Gateway --> Service;
```

See the system architecture in @fig:arch.
````

---

## Callout Advisory Blocks

Botox supports both GitHub-style and Pandoc-style advisory callouts:

### GitHub Style Callouts
```markdown
> [!NOTE]
> Useful background context or explanations.

> [!TIP]
> Pro-tip for optimizing workflow performance.

> [!WARNING]
> Critical warning regarding security or data loss.

> [!IMPORTANT]
> Essential steps that must not be skipped.
```

### Pandoc Style Divs
```markdown
::: note
**Pandoc Style**: Cleanly rendered with theme-aware borders.
:::

::: warning
**Caution**: High voltage area!
:::
```

---

## Bibliographies & Cross-Referencing

* **Automatic Link Bibliography**: Enable with `bibliography: true` in frontmatter or `-b` in CLI. Hyperlinks in the text are transformed into IEEE citation indices `[1]`, with a bibliography at the end of the document.
* **Manual References Command**:
  ```markdown
  Check out [Rust](https://rust-lang.org) and [Typst](https://typst.app).

  \ref Sources and References
  ```
* **Filtering Links**: Use `exclude-references: ["github.com", "x.com"]` or `--exclude-ref` to prevent non-academic links from appearing in the bibliography.
"#
}

/// Handle `botox --skills` or `botox skills` invocation.
pub fn handle_skills(args: &[String]) {
    let mut target_dir: Option<PathBuf> = None;
    let mut is_global = false;
    let mut to_stdout = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                println!(r#"Usage: botox --skills [options] [directory]
       botox skills [options] [directory]

Generates a standardized SKILL.md definition for AI coding agents (Antigravity, Claude Code, Cursor, Copilot).

Options:
  --global                Install into user's global skills directory (~/.agents/skills/botox/)
  --dir <path>            Target directory where skills/botox/SKILL.md will be generated
  -o, --stdout            Print the SKILL.md contents directly to stdout
  -h, --help              Show this help message

Default:
  Creates '.agents/skills/botox/SKILL.md' in the current project directory.
"#);
                return;
            }
            "--global" => {
                is_global = true;
            }
            "--stdout" | "-o" => {
                if args.get(i + 1).map(|s| s.as_str()) == Some("-") {
                    i += 1;
                }
                to_stdout = true;
            }
            "--dir" => {
                i += 1;
                if i < args.len() {
                    target_dir = Some(PathBuf::from(&args[i]));
                }
            }
            other => {
                if !other.starts_with('-') && target_dir.is_none() && other != "skills" && other != "--skills" {
                    target_dir = Some(PathBuf::from(other));
                }
            }
        }
        i += 1;
    }

    let content = generate_skill_content();

    if to_stdout {
        print!("{content}");
        return;
    }

    let mut destinations: Vec<PathBuf> = Vec::new();

    if let Some(custom) = target_dir {
        let dest = if custom.file_name().and_then(|n| n.to_str()) == Some("SKILL.md") {
            custom
        } else if custom.file_name().and_then(|n| n.to_str()) == Some("botox") {
            custom.join("SKILL.md")
        } else {
            custom.join("skills").join("botox").join("SKILL.md")
        };
        destinations.push(dest);
    } else if is_global {
        if let Some(home) = crate::config::dirs_home() {
            destinations.push(home.join(".agents").join("skills").join("botox").join("SKILL.md"));
            destinations.push(home.join(".gemini").join("skills").join("botox").join("SKILL.md"));
        } else {
            destinations.push(PathBuf::from(".agents").join("skills").join("botox").join("SKILL.md"));
        }
    } else {
        // Project-local: standard agent skills location
        destinations.push(PathBuf::from(".agents").join("skills").join("botox").join("SKILL.md"));
        
        // Also place into .gemini if .gemini directory exists in project root
        if Path::new(".gemini").is_dir() {
            destinations.push(PathBuf::from(".gemini").join("skills").join("botox").join("SKILL.md"));
        }
    }

    let mut success_count = 0;
    for dest in &destinations {
        if let Some(parent) = dest.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            eprintln!("Warning: Failed to create directory '{}': {e}", parent.display());
            continue;
        }

        match fs::write(dest, content) {
            Ok(()) => {
                println!("Generated agent skill -> '{}'", dest.display());
                success_count += 1;
            }
            Err(e) => {
                eprintln!("Error writing skill to '{}': {e}", dest.display());
            }
        }
    }

    if success_count > 0 {
        println!("AI coding agents can now automatically read this skill to author documents and presentations with Botox.");
    } else {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_skill_content_has_frontmatter_and_docs() {
        let content = generate_skill_content();
        assert!(content.starts_with("---\nname: botox\n"));
        assert!(content.contains("description:"));
        assert!(content.contains("# Botox: Publication-Grade Typesetting"));
        assert!(content.contains("botox document.md"));
        assert!(content.contains("botox slides.md"));
        assert!(content.contains("marp: true"));
        assert!(content.contains("```mermaid"));
    }

    #[test]
    fn test_handle_skills_custom_dir() {
        let temp_dir = std::env::temp_dir().join("botox_skill_test");
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);

        let args = vec![
            "--dir".to_string(),
            temp_dir.to_string_lossy().to_string(),
        ];
        handle_skills(&args);

        let expected_file = temp_dir.join("skills").join("botox").join("SKILL.md");
        assert!(expected_file.is_file(), "Skill file should be created");
        let content = fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("name: botox"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
