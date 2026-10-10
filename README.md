# Botox

[![Language: Rust](https://img.shields.io/badge/Language-Rust_2024-dea584.svg)](https://www.rust-lang.org/)
[![Engine: Typst](https://img.shields.io/badge/Engine-Typst_0.15-239dad.svg)](https://typst.app/)
[![Diagrams: Kroki](https://img.shields.io/badge/Diagrams-Kroki-white.svg)](https://kroki.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Botox is a CLI tool written in Rust that compiles Markdown into PDF documents or HTML presentation slides.

It embeds the Typst typesetting engine directly inside the binary, so you don't need to install TeX Live, Pandoc, Node.js, or external PDF engines.

## Features

- **Documents**: Compiles Markdown to PDFs with Computer Modern typography.
- **Slides**: Compiles Markdown to standalone HTML presentation decks with keyboard navigation and incremental steps.
- **Math**: Inline and display LaTeX-style math (`$...$` and `$$...$$`), matrices, and piecewise cases.
- **Diagrams**: Mermaid, PlantUML, UMLet (`.uxf`), and Graphviz diagrams compiled via Kroki with automatic fallbacks and local caching.
- **File Referencing**: Reference external diagram files (`.uxf`, `.puml`, `.mmd`) or code files directly instead of copying their contents into Markdown.
- **Structure**: Tables with alignment and captions, callout boxes, table of contents, and automatic bibliography generation for links.
- **Watch Mode**: `--watch` / `-w` automatically recompiles when you save changes.
- **Piping**: Supports reading from stdin and streaming output to stdout (`-o -`).

## Installation

Until Botox is published to package managers (Homebrew, crates.io, etc.), you can install it using one of the methods below.

### 1. Build from Source (Cargo)

If you have Rust and Cargo installed:

```bash
git clone https://github.com/botoxMD/botox-cli.git
cd botox-cli
cargo install --path .
```

Or build the release binary directly and copy it into your `PATH`:

```bash
cargo build --release
mkdir -p ~/.local/bin
cp target/release/botox ~/.local/bin/
```

Make sure `~/.local/bin` is in your `PATH` (e.g. in your `~/.bashrc` or `~/.zshrc`):

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### 2. Prebuilt Binaries (GitHub Releases)

Precompiled standalone binaries for Linux, macOS, and Windows are available on the [Releases page](https://github.com/botoxMD/botox-cli/releases).

1. Download the archive for your operating system and architecture.
2. Extract the `botox` (or `botox.exe`) executable.
3. Move it to a folder in your system `PATH` (such as `~/.local/bin` on Linux/macOS or `C:\Windows\System32` / user path on Windows).
4. On Linux/macOS, ensure execute permissions are set:
   ```bash
   chmod +x ~/.local/bin/botox
   ```

### 3. VS Code Extension

For live vector preview as you type in VS Code, install the companion extension [`botox-vsc-ext`](https://github.com/botoxMD/botox-vsc-ext).

If you are installing from a `.vsix` package:
- **Via VS Code UI**: Open VS Code -> Extensions view (`Ctrl+Shift+X` or `Cmd+Shift+X`) -> click `...` (Views and More Actions) at the top -> select **Install from VSIX...** and pick the `.vsix` file.
- **Via Terminal** (if the `code` CLI is available in your PATH):
  ```bash
  code --install-extension botox-0.14.5.vsix
  ```
*(Note: If the `code` command is not in your terminal PATH, use the VS Code UI method above or press `F1` in VS Code and run `Shell Command: Install 'code' command in PATH`.)*

## Quick Start

### Compiling a Document

```bash
botox document.md
```

This generates `document.pdf`.

### Compiling Slides

```bash
botox slides.md
```

If the frontmatter contains `type: slides`, Botox outputs `slides.html`. You can also force slides mode:

```bash
botox --slides presentation.md -o presentation.html
```

### Watch Mode

Automatically rebuild on file changes:

```bash
botox -w document.md
```

### Setup Wizard

Configure your default author name, theme, paper size, and settings globally or per-project:

```bash
botox setup
```

## Examples

For complete, working samples that demonstrate how to write documents and presentations, see the [`examples/`](examples/) directory:

- [`examples/document.md`](examples/document.md): Complete PDF document sample with math, diagrams, tables, callouts, and references.
- [`examples/slides.md`](examples/slides.md): Complete HTML presentation slide deck sample with pauses and diagrams.
- [`examples/architecture.uxf`](examples/architecture.uxf): Sample UMLet diagram file referenced in Markdown.
- [`examples/pipeline.puml`](examples/pipeline.puml): Sample PlantUML diagram file referenced in Markdown.
- [`examples/sample.rs`](examples/sample.rs): Sample code file referenced in Markdown code blocks.

## CLI Usage

```text
Usage: botox <input.md> [options]
       botox <pdf|slides> <input.md> [options]
       botox init <filename> [options]
       botox config
       botox setup [options]
       botox --skills [options]

Commands:
  init <filename>         Initialize a new Markdown document or presentation
  config                  Display currently active configuration
  setup                   Interactive configuration wizard
  skills, --skills        Generate agent skill definition for AI coding agents

Options:
  -o, --output <file>     Output path (.pdf, .html, or '-' for stdout)
  -w, --watch             Watch input file and directory for changes
  --config <file>         Custom configuration YAML path
  --pdf                   Force PDF document mode
  --slides                Force presentation slide deck mode
  --theme <theme>         Document theme (academic, modern, elegant, technical, compact, minimal)
                          or slide theme (default, academic, dark, nord)
  --toc                   Generate Table of Contents
  --no-toc                Disable Table of Contents
  --author <name>         Author name
  --font <name>           Font name
  -N, --number-sections   Number section headings
  -b, --bibliography      Generate bibliography from links
  --exclude-ref <pat>     Exclude matching URLs/domains from bibliography
  --include-ref <pat>     Include only matching URLs/domains in bibliography
```

## License

This project is licensed under the [MIT License](LICENSE).