---
title: "Botox Markdown Suite"
author:
  - name: "Minus"
    affiliation: "Systems Engineering"
date: "October 2026"
abstract: "A self-contained, single-binary command-line tool that converts standard Markdown documents into publication-grade LaTeX-style PDF documentation or presentation slide decks. The frontmatter defines document metadata and typography, while slide decks are delimited by standard horizontal rules. Built in Rust with an embedded in-process typesetting engine, requiring zero external runtime dependencies."
papersize: a4
fontsize: 11pt
mainfont: "New Computer Modern"
mathfont: "New Computer Modern Math"
columns: 1
margin:
  x: 2.5cm
  y: 2.5cm
toc: true
---
\newpage
# Overview

Botox provides a single, self-contained native binary (`botox`) to compile standard Markdown files into:

1. **LaTeX-Grade PDF Documentation**: Typeset via an embedded, in-process Typst engine with authentic New Computer Modern typography, micro-typography, and TeX mathematical notation.
2. **Presentation Slides**: Slide decks partitioned by horizontal rules (`---`) compiled directly into clean 16:9 presentation PDFs.

Mode detection is automatic based on the file frontmatter structure (or can be explicitly declared with `--pdf` or `--slides`).

This `README.md` file is itself an example document. You can compile it directly into a PDF using:

```bash
botox README.md -o README.pdf
```

# Architecture

Unlike traditional document compilation pipelines that require a web of external runtimes:

- **Zero Runtime Dependencies**: No Node.js, no Python, no Pandoc, no Headless Chrome, and no gigabyte-scale TeX Live installations.
- **Embedded Typographic Engine**: The Typst layout engine and New Computer Modern font family (Roman, Bold, Italic, and Math) are compiled directly into the single executable.
- **Instant Compilation**: Typical compilation times range between 300ms and 1.5s.
- **Cross-Platform**: Compiles to a single standalone binary for Linux, macOS, and Windows.

# Installation

## Using the Compiled Binary

The executable can be placed anywhere on your system `PATH`:

```bash
# User-level binary location
cp target/release/botox ~/.local/bin/botox
```

## Building from Source

To compile the standalone binary from source using the Rust toolchain:

```bash
cd botox
cargo build --release
strip target/release/botox
cp target/release/botox ~/.local/bin/botox
```

# Unified Command Usage

The single command `botox` handles both documents and presentations:

```bash
# Compile a documentation PDF (auto-detected)
botox document.md -o document.pdf

# Compile a slide deck (auto-detected)
botox presentation.md -o presentation.pdf

# Generate document with an automatic Table of Contents
botox document.md --toc -o document.pdf

# Explicit mode declaration (optional)
botox pdf document.md -o document.pdf
botox slides presentation.md -o presentation.pdf
```

# PDF Documentation

## Frontmatter and Styling

The opening YAML block controls the metadata and styling of the document. You never need to construct a table of contents manually; it is generated dynamically with dotted leader lines and page numbers.

```yaml
---
title: "Document Title"
author:
  - name: "Author Name"
    affiliation: "Organization or Lab"
date: "October 2026"
abstract: "Summary of the document."
mainfont: "New Computer Modern"
mathfont: "New Computer Modern Math"
fontsize: 11pt
papersize: a4
columns: 1
margin:
  x: 2.5cm
  y: 2.5cm
toc: true
---
```

## Table of Contents Control

A Table of Contents is generated automatically when requested:

- In frontmatter: `toc: true` or `toc: false`
- Via command line: `--toc` or `--no-toc`
- In configuration: `toc: true` in `botox.yaml` to enable it by default across all documents

## Automatic Bibliography Transformation

Normal Markdown hyperlinks can be transformed into an IEEE-standard numbered bibliography. In the body text, links receive bracketed citation numbers (e.g., `Rust Language [1]`), and an unnumbered References section is generated at the end of the document with hanging indents and full electronic resource citations. Repeated links to the same URL automatically share the same citation index.

- In frontmatter: `bibliography: true` (or `bibliography: "ieee"`, `links-as-references: true`)
- Custom title: `biblio-title: "Webography"` (defaults to localized titles like *References*, *Références*, *Literaturverzeichnis*)
- Via command line: `-b` or `--bibliography` (override with `--no-bibliography`)
- In configuration: `bibliography: true` in `botox.yaml` under `document`

## Command Line Options

- `-o, --output <file>`: Output PDF path (defaults to `<input>.pdf`).
- `--toc` / `--no-toc`: Enable or disable the automatic Table of Contents.
- `-b, --bibliography`: Transform web links into an IEEE-standard Bibliography.
- `--no-bibliography`: Disable automatic Bibliography generation.
- `--author <name>`: Override author name.
- `--font <name>`: Override main typeface (e.g. `--font "Libertinus Serif"`).
- `-N, --number-sections`: Enable numbered headings (`1.`, `1.1.`, etc.).
- `--config <path>`: Path to a custom YAML configuration file.

# Presentation Slides

## Slide Syntax and Styling

Slides are divided by standard Markdown horizontal rules (`---`). The frontmatter block defines theme, background color, text color, and pagination:

```markdown
---
marp: true
theme: default
paginate: true
backgroundColor: "#f8fafc"
color: "#0f172a"
---

# Slide 1 Title
Author Name

---

# Key Findings
- Sub-second single-binary compilation
- Embedded New Computer Modern fonts
- Zero runtime dependencies
```

Supported theme presets include `default`, `academic`, `dark`, and `nord`. Custom hex values can also be supplied via `backgroundColor` and `color`.

# Configuration and Defaults

Default values for authors, fonts, margins, table of contents, and slide themes can be configured in a YAML file so they do not need to be specified in every document.

## Configuration Search Order

1. Path supplied via `--config <path>`
2. `./botox.yaml` (project-level configuration in current working directory)
3. `~/.config/botox/config.yaml` (user-level global configuration)

## Configuration Schema

```yaml
document:
  author: "Minus"
  affiliation: "Systems Engineering"
  fontsize: "11pt"
  mainfont: "New Computer Modern"
  mathfont: "New Computer Modern Math"
  papersize: "a4"
  columns: 1
  margin:
    x: "2.5cm"
    y: "2.5cm"
  section_numbering: false
  toc: false

slides:
  theme: "default"
  paginate: true
```

Inspect active configuration settings:

```bash
botox config
```

# Mathematical Typography

Mathematical formulas render with full TeX fidelity:

$$\mathcal{L}_{G} = \mathbb{E}_{x \sim p_{\text{data}}} [\log D(x)] + \mathbb{E}_{z \sim p_{z}} [\log (1 - D(G(z)))]$$

In-line formulas like $f(x) = \sum_{i=0}^n a_i x^i$ integrate seamlessly into text.
