---
title: "Botox Markdown Suite"
subtitle: "Native LaTeX-Grade Typesetting & Slide Decks in a Single Binary"
author:
  - name: "Minus"
    affiliation: "Systems Engineering"
  - name: "Core Architecture Team"
    affiliation: "Botox Project"
date: \today
lang: en
abstract: "Botox is a pure Rust, single-binary command-line suite that compiles standard Markdown documents into publication-grade LaTeX-style PDF documentation or presentation slide decks. Featuring an embedded in-process typesetting engine, authentic New Computer Modern typography, advanced TeX mathematics, native IEEE-standard bibliographies, and Pandoc extensions, Botox requires zero external runtime dependencies."
keywords: ["Document Engineering", "LaTeX Typesetting", "Pandoc Compatibility", "Rust"]
papersize: a4
fontsize: 11pt
mainfont: "New Computer Modern"
monofont: "DejaVu Sans Mono"
mathfont: "New Computer Modern Math"
geometry: "margin=2.5cm"
linkcolor: "#0284c7"
number-sections: true
table-of-contents: true
toc-title: "Table of Contents"
toc-depth: 3
bibliography: true
---

\newpage

# Overview and Architecture

Botox provides a single, self-contained native executable (`botox`) that transforms standard Markdown files into either publication-grade LaTeX-style PDF documents or clean 16:9 presentation slide decks.

Mode detection is automatic based on frontmatter structure (or can be explicitly declared with `--pdf` or `--slides`). This `README.md` file itself serves as a comprehensive living specification of all supported features and can be compiled directly:

```bash
botox README.md -o README.pdf
```

## Architectural Pillars

Unlike traditional document compilation pipelines that require a web of external runtimes:

- **Zero Runtime Dependencies**: No Node.js, no Python, no Pandoc runtime, no headless Chrome, and no multi-gigabyte TeX Live distributions.
- **Embedded Typesetting Engine**: Built on the [Rust Language](https://www.rust-lang.org) with the [Typst](https://typst.app) layout engine and New Computer Modern font family (Roman, Bold, Italic, and Math) compiled directly into the binary[^engine].
- **Instantaneous Compilation**: Sub-second compilation times (typically 400ms to 900ms) for multi-page documents.
- **Cross-Platform Portability**: Builds as a single static executable for Linux, macOS, and Windows.

[^engine]: Embedded font assets include 17 New Computer Modern and DejaVu typeface variants loaded directly from in-memory binary slices.

## Feature Matrix

- [x] Standalone native single binary (`botox`) with zero external requirements
- [x] Complete Pandoc frontmatter compatibility (`geometry`, `lang`, `toc-depth`, `number-sections`)
- [x] Typography extensions: subscripts (`H~2~O`), superscripts (`10^6^`), strikethrough (`~~text~~`)
- [x] Native multi-pass footnotes with bottom-of-page rule separation
- [x] Definition lists and GFM task checklists
- [x] Advanced TeX mathematical notation (matrices, piecewise systems, partial derivatives, accents)
- [x] Pipe tables with column alignments (`:---`, `:---:`, `---:`) and cell formatting
- [x] Automatic IEEE-standard numbered bibliography generated from hyperlinks
- [x] High-resolution figures and vector graphics (PNG, JPEG, SVG, WebP, GIF) with automated captions and numbering
- [x] Marp-compatible 16:9 presentation slide decks partitioned by `---`
- [x] Cascading configuration hierarchy (`botox.yaml`)

# Installation and Quick Start

## System Binary Installation

The pre-compiled binary can be copied directly to any folder on your system `PATH`:

```bash
# Install to user local binary path
cp bin/botox ~/.local/bin/botox
```

## Building from Source

To compile the standalone binary from source using the Rust toolchain:

```bash
cd botox
cargo build --release
strip target/release/botox
cp target/release/botox ~/.local/bin/botox
```

## Unified CLI Invocation

The single executable `botox` handles both documents and presentations:

```bash
# Initialize a new document or presentation with active default settings
botox init document.md
botox init slides.md

# Compile documentation PDF (auto-detected)
botox README.md -o README.pdf

# Compile presentation slides (auto-detected)
botox slides.md -o slides.pdf

# Transform hyperlinks into an IEEE Bibliography
botox document.md --bib -o document.pdf

# Toggle automatic Table of Contents
botox document.md --toc -o document.pdf
botox document.md --no-toc -o document.pdf

# Inspect active configuration settings
botox config
```

# Typography and Pandoc Extensions

Botox adheres to standard Pandoc Markdown conventions and extensions.

## Inline Styling and Formatting

Standard emphasis, bold, code, and strikethrough are fully supported alongside intra-word subscript and superscript delimiters:

- **Chemical Formulas & Exponents**: Water is written as H~2~O, glucose as C~6~H~12~O~6~, and astronomical orders of magnitude as 10^6^ or 10^-19^.
- **Strikethrough**: Deprecated workflows are marked with ~~legacy pipelines~~ and replaced with instant native compilation.
- **Code Spans**: Variables and registers like `reg_ex_mem` and `instruction_pointer` are rendered with monospace background badges.

## Definition Lists

Pandoc-style definition lists format technical glossaries with bold headwords and indented definitions:

Zero-Dependency Architecture
: A self-contained executable that bundles all fonts, engines, and converters without external interpreters.

Forwarding Unit
: A microarchitectural hazard-mitigation bypass that routes execution results directly between pipeline stages.

Automatic Bibliography
: In-text citations and end-of-document reference sections generated automatically from standard Markdown links.

## Blockquotes

Academic citations and callouts render with subtle left border rules and margins:

> "Simplicity is prerequisite for reliability."
> --- Edsger W. Dijkstra, *Selected Writings on Computing*

## Callout Divs

Pandoc-style fenced divs (`:::`) render as color-coded advisory blocks with distinct visual borders:

::: note
**Pandoc `link_attributes`**: The `{width=50%}` syntax originates from Pandoc's `link_attributes` extension. Botox natively parses and renders image attributes without requiring any external preprocessors.
:::

::: tip
Use cross-references like `@fig:arch`, `@tbl:perf`, `@sec:math`, or `@eq:energy` to generate clickable hyperlinks that automatically track section and asset numbering.
:::

::: warning
Never mix unnumbered headings with strict numerical cross-references. Unnumbered headings (`{-}` or `{.unnumbered}`) are excluded from the numbering counter.
:::

## Figures, Graphics & Attributes

Raster and vector image assets (PNG, JPEG, SVG, WebP, GIF) are rendered natively without external decoders. Standard Markdown syntax with alt text generates numbered LaTeX figures with captions:

```markdown
![Botox Architecture Pipeline](figures/architecture.png){width=65% #fig:arch}
![](logo.svg){width=10cm height=5cm}
![Relative Scaling](banner.png){width=0.8\linewidth}
```

Attributes inside `{...}` support:
- Dimensions: `%` (e.g. `width=50%`), absolute units (`10cm`, `4in`, `200pt`, `100mm`), pixels (`300px`), and TeX factors (`0.8\linewidth`, `\textwidth`).
- Identifiers: `#fig:id` or `id=fig:id` for cross-referencing with `@fig:id`.

## Cross-Referencing

Botox implements Pandoc cross-referencing for figures, tables, sections, and equations:

- `@fig:arch` references Figure 1
- `@tbl:perf` references Table 1 (see @tbl:perf)
- `@sec:math` references Section 4 (see @sec:math)
- `@eq:energy` references Equation 1 (see @eq:energy)

## Page Breaks

LaTeX and Pandoc page break commands placed anywhere in the document are translated into native page divisions:

- `\newpage`
- `\pagebreak`
- `\clearpage`
- `<!-- pagebreak -->`

# Advanced Mathematical Typography {#sec:math}

Mathematical expressions render with full LaTeX fidelity using embedded New Computer Modern Math fonts.

## Fractions, Accents, and Operators

Nested braces in fractions and roots resolve recursively with proper font scaling:

$$E = m c^2 + \frac{1}{2} m v^2$$ {#eq:energy}

$$\vec{F} \approx m \frac{\partial^2 \vec{r}}{\partial t^2} + \sqrt[3]{\frac{\alpha_{11}}{\beta + \gamma}} \pm \vec{\epsilon}$$

Definite integrals, infinite series, and limits preserve standard limits and display styles:

$$\int_{-\infty}^{+\infty} e^{-x^2} \, dx = \sqrt{\pi}, \qquad \sum_{k=0}^{\infty} \frac{x^k}{k!} = e^x, \qquad \lim_{x \to 0} \frac{\sin(x)}{x} = 1$$

## Matrix Environments

Parenthesized (`pmatrix`), bracketed (`bmatrix`), and determinant (`vmatrix`) matrices are fully supported:

$$\begin{pmatrix} 1 & 0 \\ 0 & 1 \end{pmatrix} \qquad \begin{bmatrix} a_{11} & a_{12} & \dots & a_{1n} \\ a_{21} & a_{22} & \dots & a_{2n} \\ \vdots & \vdots & \ddots & \vdots \\ a_{m1} & a_{m2} & \dots & a_{mn} \end{bmatrix} \qquad \begin{vmatrix} \lambda - a & b \\ c & \lambda - d \end{vmatrix}$$

## Piecewise Cases

Piecewise function definitions with conditions and text descriptions render using the `cases` environment:

$$f(x) = \begin{cases} \sqrt{x} & \text{if } x \ge 0 \\ -\sqrt{-x} & \text{if } x < 0 \end{cases}$$

# Structured Tables

Pipe tables support header alignment markers (`:---` for left, `:---:` for center, and `---:` for right), automated captions, and cross-reference labels:

Table: Pipeline execution comparison across toolchains. {#tbl:perf}

| Tool | Pipeline Stage | Target Output | Performance |
| :--- | :---: | :---: | ---: |
| Botox Native Engine | Typst Layout | PDF Vector | $< 1.0\text{ s}$ |
| [Rust Compiler](https://www.rust-lang.org) | Codegen | Native ELF / PE | $O(N)$ |
| [LLVM Backend](https://llvm.org) | Optimization | Machine Code | Parallel |
| [Typst Engine](https://typst.app) | Typesetting | PDF / SVG | Real-time |

Cells seamlessly support inline code, formatting, mathematics, and web links.

# Automatic Bibliography Transformation

When `bibliography: true` is set in the frontmatter (or enabled via the `-b` / `--bibliography` command-line flag), all standard web links are transformed into an **IEEE-standard numbered bibliography**:

1. **In-Text Citations**: Hyperlinks in the body receive bracketed citations (e.g. `[1]`, `[2]`). Clicking the citation number jumps directly to the corresponding entry in the bibliography.
2. **Deduplication**: Repeated links to the same destination automatically share the same citation number across the entire document.
3. **References Section**: An unnumbered References section is generated at the end of the document featuring hanging indents and electronic resource citations (e.g. including the [GitHub Platform](https://github.com)).
4. **Localization**: Respects the document `lang` parameter (e.g., *References*, *Références*, or *Literaturverzeichnis*).

# Presentation Slide Decks

Slide decks are authored in the same Markdown files, using horizontal rules (`---`) as slide boundaries. When `marp: true` or slide themes are detected in the frontmatter, Botox compiles the document into a 16:9 presentation PDF:

```markdown
---
marp: true
theme: default
paginate: true
backgroundColor: "#f8fafc"
color: "#0f172a"
---

# Microarchitecture Hazards
Eliott JAQUIER, Shanshe GUNDISHVILI

---

# Forwarding Unit Implementation
- Direct bypass from EX/MEM to ID/EX
- Zero stall cycles for ALU dependencies
- Fallback stall insertion for Load-Use hazards
```

Built-in slide themes include `default`, `academic`, `dark`, and `nord`, with optional custom `backgroundColor` and `color` hex overrides.

# Configuration Reference

Botox loads configuration settings from a cascading hierarchy:

1. CLI argument `--config <path>`
2. Project-level configuration file: `./botox.yaml`
3. User-level global configuration file: `~/.config/botox/config.yaml`

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
  section_numbering: true
  toc: true
  bibliography: true

slides:
  theme: "default"
  paginate: true
```

## CLI Option Summary

| Flag | Parameter | Description |
| :--- | :--- | :--- |
| `-o, --output` | `<file>` | Output PDF path (default: `<input>.pdf`) |
| `-b, --bib` | None | Enable automatic IEEE Bibliography generation |
| `--no-bib` | None | Disable automatic Bibliography generation |
| `--toc` | None | Force generation of Table of Contents |
| `--no-toc` | None | Disable Table of Contents |
| `--author` | `<name>` | Override author name |
| `--font` | `<name>` | Override primary typeface |
| `-N, --number-sections` | None | Enable numbered section headings |
| `--pdf` | None | Force PDF documentation mode |
| `--slides` | None | Force presentation slide deck mode |
| `--config` | `<file>` | Supply custom configuration YAML path |
| `init` | `<filename>` | Initialize new document or slide deck with default settings |
| `config` | Subcommand | Display currently active configuration |

# Appendix {-}

This appendix section demonstrates an unnumbered heading created with Pandoc's `{-}` attribute syntax. It appears in the document structure without incrementing the section counter.

