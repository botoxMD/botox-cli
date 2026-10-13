# Comprehensive Architectural & Implementation Critique: Botox & Botox-VSC-Ext

## 1. Executive Summary & Core Verdict

### What the Project Does Remarkably Well
* **Compelling Value Proposition:** The vision of delivering publication-grade LaTeX typesetting without TeX Live, Pandoc, or Node.js dependencies via an in-process, pure-Rust Typst compiler (`typst-as-lib`) is brilliant.
* **Vector SVG Preview Model:** Emitting per-page SVGs into JSON instead of rasterizing to PNG gives crisp, zoom-independent rendering and enables pure-vector text selection in the VS Code preview.
* **Feature Completeness:** Seamless support for Marp-style slides, Kroki vector diagrams, IEEE auto-bibliographies, Pandoc fenced divs, and GitHub callouts in a single native binary.

### The Hard Truth: Core Architectural Vulnerabilities
1. **Pervasive String Replacement Architecture:** Rather than manipulating an Abstract Syntax Tree (AST), the compilation pipeline relies heavily on multiple passes of ad-hoc string manipulations, sentinel tokens (`BOTOXCMD...`, `BOTOX_REF_...`), and regular expressions. This creates severe edge-case fragilities and parsing collisions.
2. **Brittle Magic Numbers & Hardcoded Assumptions:** Found throughout the stack (e.g., hardcoded IP address `140.238.215.250.sslip.io` in Kroki endpoints, hardcoded `146` line-offset subtraction in the VS Code diagnostic parser).
3. **Process-Spawning IPC Bottleneck:** Live preview spawns a full native OS process for every debounced keystroke, dumping temporary JSON files to disk and reading them back.
4. **Monolithic Code Smells:** Multiple files exceed 1,300 lines of tightly coupled responsibilities, and the VS Code webview embeds ~1,500 lines of untyped, raw JavaScript inside a TypeScript template string.

---

## 2. Botox CLI (Rust Engine) Deep-Dive

### 2.1 Parsing & Markdown Pipeline Architecture

#### 🛑 Critical Bug: Panic on Empty Callout Title
* **Location:** `src/markdown/callouts.rs` (lines 8–13)
```rust
let q = after_t.chars().next().unwrap_or('"');
if q == '"' || q == '\'' {
    if let Some(end_q) = after_t[1..].find(q) {
        title = after_t[1..=end_q].to_string(); // 💥 PANIC IF TITLE IS EMPTY!
    }
}
```
* **The Bug:** If a user writes `::: {.note title=""}` or `title=''`, `after_t[1..]` starts with the closing quote, so `end_q == 0`. The slice `after_t[1..=0]` attempts an inclusive range where `start > end`, causing an **unrecoverable panic and crash** in Rust.
* **Fix:** Use slice indexing relative to `after_t[1..]`: `title = after_t[1..1 + end_q].to_string()`.

---

#### ⚠️ Sentinel Token Collisions & Lexical Leakage
* **Location:** `src/markdown/escapes.rs` and `src/markdown/mod.rs`
The markdown conversion pipeline uses sentinel strings:
```rust
BOTOXCMDNEWPAGEBOTOX
BOTOXCMDCOLBREAKBOTOX
BOTOXSUBSTART / BOTOXSUBEND
BOTOX_REF_PLACEHOLDER_START:...:BOTOX_REF_PLACEHOLDER_END
```
* **The Flaw:** If a user writes documentation *about* Botox, or a code block / text containing these exact words, the post-processing replacements corrupt the document.
* **Pulldown-Cmark Mismatch:** Pulldown-cmark is an event-based parser, but Botox preprocesses the text with `preprocess_pandoc`, sends it through `pulldown_cmark`, escapes text chunks with `escape_typst_text`, and then performs string replacements on the output.
* **Recommendation:** Transform events cleanly using an event stream filter (`impl Iterator<Item = Event>`) or AST transformation. Do not mix token placeholders with text emitting.

---

#### 🛑 Severe Design Flaw: Multi-Column Pagebreaks
* **Location:** `src/document/columns.rs` (lines 1–13)
```rust
pub fn sanitize_column_pagebreaks(content: &str, columns: usize) -> String {
    if columns > 1 {
        content
            .replace("#pagebreak()", "#colbreak()")
            ...
```
* **The Flaw:** When `columns > 1`, **every single pagebreak is unconditionally replaced by a columnbreak**. A user authoring a two-column paper who actually needs to advance to the next physical page (`\newpage` or page break) is fundamentally prevented from doing so.
* **Recommendation:** Separate `#colbreak()` (for `\columnbreak`) from `#pagebreak()` (for `\newpage`). Typst fully supports `#pagebreak()` inside a `columns` scope.

---

#### ⚠️ Brittle Regex-Free Math Translation
* **Location:** `src/markdown/math.rs`
* Math conversion uses manual substring scanning and replacement.
* While it handles single commands like `\vec{x}`, nested environments (`\begin{bmatrix} ... \begin{pmatrix} ... \end{pmatrix} ... \end{bmatrix}`) or complex macros break easily.
* The brace matcher `find_matching_brace` does not account for escaped braces (`\{` or `\}`) within math strings, which leads to corrupted equations.

---

### 2.2 Compilation & Layout Engine (`src/compiler.rs`)

#### ⚠️ Invisible Text Hack for Slide Pauses
* **Location:** `src/slides/pauses.rs` (line 24) and `src/compiler.rs` (lines 505–521)
```rust
#place(top + left)[#text(size: 0.001pt, fill: rgb(0, 0, 0, 0))[botox-pause-step]]
```
* Botox injects invisible 0.001pt text, compiles the document with Typst, and then inspects every text item in the layout tree for the substring `"botox-pause-step"` to determine whether a page is an incremental step.
* **Risk:** The text item still exists in the DOM/SVG, could be picked up by screen readers or text-extraction tools, and is easily broken if font rendering or culling changes.
* **Better Approach:** Track sub-slide indices in the compiler AST metadata before compiling, or query page labels using Typst metadata introspections (`#metadata("pause") <botox-step>`).

---

#### ⚠️ Synchronous System Font DB Reloading
* **Location:** `src/compiler.rs` (lines 180–198)
* Whenever a custom font is requested (e.g. `font: "Arial"`), `load_needed_fonts` instantiates a brand new `fontdb::Database` and calls `db.load_system_fonts()`.
* On systems with thousands of installed fonts, `load_system_fonts()` can take **300ms to 1.5s**, completely defeating the fast compilation speed of Typst.
* **Fix:** Cache the `fontdb::Database` in a `LazyLock<RwLock<fontdb::Database>>` or initialize it once on startup.

---

### 2.3 Diagrams & Network Architecture (`src/diagrams/`)

#### 🛑 Hardcoded IP Address & Privacy/Security Risk
* **Location:** `src/diagrams/kroki.rs` (lines 37–41)
```rust
pub const DEFAULT_KROKI_ENDPOINTS: &[&str] = &[
    "https://140.238.215.250.sslip.io",
    "https://kroki.io",
    "https://demo.kroki.io",
];
```
* **Critical Issue:** A hardcoded raw IP address with a dynamic DNS service (`140.238.215.250.sslip.io`) is placed as the **primary** diagram rendering endpoint before `kroki.io`.
  1. **Security & Data Exfiltration:** Any diagram code written by users (which may contain proprietary architectures, database credentials, internal system topologies, or private company data) is sent unencrypted or to an unknown personal server by default.
  2. **Reliability:** If this personal Oracle Cloud / VPS instance goes down or IP changes, requests hang until the 3.5s timeout expires before falling back to official Kroki.
  3. **Package/Enterprise Rejection:** This would immediately fail an enterprise security audit or Homebrew/Debian packaging review.
* **Fix:** Use official `https://kroki.io` as default, allow explicit overrides via `botox.yaml` / `KROKI_ENDPOINT`, and warn the user before transmitting diagram code to remote servers.

---

#### ⚠️ Handcrafted XML/SVG "Sanitizer" (`src/diagrams/cache.rs`)
* **Location:** `src/diagrams/cache.rs` (lines 190–355)
* `sanitize_svg_for_typesetting` attempts to parse `<foreignObject>` tags from Mermaid SVGs using string searches (`find("<foreignObject")`, manual quote parsing, stripping HTML tags with character loops).
* It tries to convert HTML `<div><p>text</p></div>` into SVG `<text><tspan>` nodes with guessed `line_height = 18.0` and guessed `font_size = 14`.
* **The Problem:** 
  - Complex styling, flexboxes, multi-line nested spans, or CSS font-sizes in Mermaid diagrams get mangled or overlap.
  - Entities, XML namespaces, and attributes can easily break SVG validation if unescaped text is encountered.
* **Fix:** If transforming SVG is necessary for `resvg`, use a real XML parser (such as `quick-xml` or `roxmltree`) rather than ad-hoc substring manipulation.

---

### 2.4 Configuration & Legacy Code Smells (`src/config/`)

#### ⚠️ Dead Legacy Names ("skygem")
* **Location:** `src/config/mod.rs` (lines 247–277)
```rust
global_candidates.push(home.join(".config").join("skygem").join("config.yaml"));
...
let cwd_candidates = [PathBuf::from("botox.yaml"), PathBuf::from("skygem.yaml")];
```
* The codebase still looks for legacy filenames like `skygem.yaml` and `.config/skygem/`. This creates technical debt, confusion, and potential configuration collisions across different projects.
* **Fix:** Remove legacy "skygem" checks or deprecate with a warning.

---

#### ⚠️ Misspelled Configuration Aliases
* **Location:** `src/config/mod.rs` (line 50) and `src/main.rs` (lines 805–808)
```rust
#[serde(default, alias = "krocki-url", alias = "krocki_url", alias = "kroki-url")]
pub kroki_url: Option<String>,
```
* "krocki" is explicitly aliased and checked throughout frontmatter extraction. This indicates a typo that got codified into the public API instead of being fixed.

---

### 2.5 CLI Interface & Error Handling (`src/main.rs`)

#### ⚠️ Monolithic Argument Parsing Without `clap`
* `src/main.rs` has **over 1,300 lines**, with hundreds of lines dedicated to a manual `while i < args.len()` loop parsing flags.
* **Consequences:**
  - Flag validation is weak (e.g. trailing flags without arguments, unknown flags silently skipped or misidentified as filenames).
  - Shell completions (`bash`, `zsh`, `fish`) cannot be generated automatically.
  - Help text formatting is manually maintained in raw string literals.
* **Recommendation:** Adopt `clap` (derive). It will eliminate ~500 lines of boilerplate, provide robust error messages, support environment variables natively, and generate shell completions.

---

#### ⚠️ Watch Mode via Naive Polling
* **Location:** `src/main.rs` (lines 1189–1242)
```rust
while running.load(std::sync::atomic::Ordering::SeqCst) {
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let current_snapshot = take_watch_snapshot(...);
    ...
```
* The watch loop reads directory metadata every 250ms via polling (`take_watch_snapshot`).
* **The Problem:** It doesn't use OS-native file event notifications (e.g. `inotify` on Linux, `kqueue` on macOS, `ReadDirectoryChangesW` on Windows).
* **Fix:** Use the standard Rust crate [`notify`](https://crates.io/crates/notify). It reduces CPU wakeups to 0% when idle and triggers instantaneous recompilation upon file write events.

---

## 3. Botox VS Code Extension (`botox-vsc-ext`) Deep-Dive

### 3.1 Architecture & IPC Model (`src/compiler.ts`)

#### 🛑 Heavyweight Subprocess & Temp File I/O on Every Keystroke
* **Location:** `src/compiler.ts` (lines 134–260)
1. For every keystroke (debounced by only 120ms), the extension spawns a brand new `botox` process via `child_process.spawn`.
2. It generates a temporary file path: `botox_preview_${Date.now()}_${process.pid}.json`.
3. The native binary compiles Typst, serializes a massive JSON payload with SVGs for every page of the document, and writes it to disk.
4. Node.js detects process exit, reads the JSON from disk with `fs.readFileSync`, parses it with `JSON.parse`, and deletes the file with `fs.unlinkSync`.
5. It then sends the parsed pages across the Webview message bridge.

* **Performance & Wear:** On a 20-page document with diagrams, this generates tens of megabytes of disk writes per minute during active editing.
* **Process Spawning Overhead:** Spawning native processes in Node.js on Windows/macOS has non-trivial fork/exec overhead (50–100ms per spawn just for OS process creation).
* **Architectural Fix:**
  - Introduce a persistent daemon or JSON-RPC / Language Server / stdio server mode (`botox lsp` or `botox preview-server`). The process stays in memory with loaded font databases and typst caches, accepting updated file contents over stdin and streaming vector page deltas over stdout.

---

### 3.2 Error Mapping & Diagnostic Heuristics (`src/extension.ts`)

#### 🛑 Hardcoded Preamble Magic Number
* **Location:** `src/extension.ts` (lines 171–179)
```typescript
if (foundLine !== undefined && foundLine >= lineCount) {
    const preambleCorrected = foundLine - 146; // 💥 MAGIC NUMBER 146!
    if (preambleCorrected >= 0 && preambleCorrected < lineCount) {
        foundLine = preambleCorrected;
    }
}
```
* **The Flaw:** When Typst errors occur, the extension tries to map line numbers back to the Markdown source by subtracting **146**.
* **Why this is fragile:** The Typst preamble generated in `src/document/mod.rs` varies in length depending on whether Table of Contents is enabled, what theme is chosen (Modern vs Academic vs Technical adds different CSS/preludes), whether callout functions are defined, whether metadata is set, etc.
* **Consequence:** Error squiggles will frequently point to the wrong line in Markdown files.
* **Fix:** The compiler binary already knows the exact preamble line count (`typst_markup.lines().take_while(|l| !l.starts_with("// BOTOX_BODY_START")).count()`). The compiler should return source-mapped diagnostics directly with accurate `(line, column)` in the JSON output, rather than making the VS Code extension guess offsets.

---

### 3.3 Webview Architecture (`src/preview.ts`)

#### ⚠️ Monolithic 1,500-Line Embedded Template Literal
* `src/preview.ts` (lines 350–1860) embeds **over 1,500 lines of raw HTML, CSS, and client-side JavaScript** directly within a TypeScript string literal.
* **Consequences:**
  - **No TypeScript type checking:** Typos, undefined variables, and broken function calls inside the webview script will not be caught at compile time.
  - **No IDE linting, formatting, or syntax highlighting** for that giant chunk of JavaScript.
  - **Noticeably harder to test:** Cannot unit test the webview client logic (zoom math, page scrolling, sync heuristics) using Jest / Vitest / Playwright.
* **Fix:** Split the Webview into a clean client-side bundle (e.g. in `web/` or `src/webview/`) written in TypeScript with its own CSS/HTML files, built via Vite or esbuild into `out/webview.js`.

---

#### ⚠️ Brittle Document Modification on Theme Change
* **Location:** `src/preview.ts` (lines 164–188)
```typescript
const fmRegex = /^---\r?\n([\s\S]*?)\r?\n---/;
const match = text.match(fmRegex);
...
const themeLineRegex = /^[ \t]*theme:\s*.*$/m;
```
* When a user changes the theme dropdown in the preview toolbar, the extension executes regex replacements on the active editor buffer to mutate the frontmatter YAML.
* **Risk:** If the frontmatter is formatted differently (e.g., uses quotes, inline comments `# my theme`, or nested YAML dictionaries), regexes can corrupt the YAML or produce duplicate `theme:` keys.
* **Fix:** Use a proper YAML parser (`yaml` package) to parse and mutate AST, or manage preview theme preferences as preview state without directly modifying user source files unless explicitly requested.

---

#### ⚠️ Follow-Cursor / Synchronous Scroll Heuristics
* The synchronized scrolling implementation in `src/preview.ts` (lines 1416–1518) is over 100 lines of complex heuristic interpolation (`queryText`, `headingText`, `prevHeading`, `nextHeading`, TOC skipping, frontmatter fractioning).
* While heading-anchored scrolling is clever, without Typst source-to-layout mapping (Source Maps), long sections without subheadings will desynchronize.
* Typst provides native source span tagging (`typst::syntax::Span`). If Botox attaches source line IDs as SVG `data-source-line="N"` attributes to headings or blocks during SVG generation, the webview can perform exact `querySelector('[data-source-line="..."]')` lookups without guessing.

---

## 4. Comprehensive Comparison & Gap Analysis

| Feature / Domain | Current Implementation | Industry Standard / Recommended State |
| :--- | :--- | :--- |
| **CLI Argument Parsing** | Custom manual loop in `main.rs` (~500 lines) | `clap` (derive) with auto completions and validation |
| **File Watching** | Sleep + metadata snapshot polling (250ms) | OS-native event notifications (`notify` crate) |
| **Diagram Endpoints** | Hardcoded Oracle Cloud IP / sslip.io | Official `kroki.io` by default + user config |
| **SVG Sanitization** | Manual string search and heuristic replacements | Real XML tree parser (`roxmltree` / `quick-xml`) |
| **IPC with VS Code** | Disk file write -> read -> unlink per keystroke | In-memory stdio stream or persistent LSP/daemon |
| **Source Mapping / Sync** | String matching on heading text in client JS | Compiler-emitted `data-source-line` attributes in SVG |
| **Webview Codebase** | 1,500 lines of untyped JS inside template string | Modular TypeScript bundle built with Vite/esbuild |
| **Multi-column breaks** | Destructive global replace `#pagebreak()` -> `#colbreak()` | Distinct `#colbreak()` and `#pagebreak()` semantics |

---

## 5. Prioritized Action Plan

### Priority 1: High Severity / Immediate Fixes
1. **Fix Callout Title Panic:** Patch `src/markdown/callouts.rs` line 11 to prevent panicking on empty `title=""`.
2. **Remove Hardcoded IP Address:** Remove `140.238.215.250.sslip.io` from `DEFAULT_KROKI_ENDPOINTS` in `src/diagrams/kroki.rs`. Default to `https://kroki.io`.
3. **Fix Multi-Column Pagebreaks:** Stop replacing all `#pagebreak()` with `#colbreak()` in `src/document/columns.rs`.

### Priority 2: Architectural Improvements (Medium Term)
1. **Source-Mapped Line Numbers:** Emit Typst/Markdown line numbers directly in JSON headings and SVGs, eliminating the magic number `146` and fuzzy text matching in the VS Code extension.
2. **Modularize the VS Code Webview:** Extract the 1,500 lines of embedded JavaScript into a standalone TypeScript project under `webview/`.
3. **Adopt `clap` and `notify` in CLI:** Replace manual argument parsing and polling loops with production-grade crates.

### Priority 3: Long-Term Enhancements
1. **Persistent Daemon / LSP Mode:** Implement `botox server` / `botox lsp` to eliminate OS process spawn overhead and temp file I/O for live preview.
2. **Font Database Caching:** Cache the system `fontdb::Database` to eliminate the 500ms+ font reload penalty when custom fonts are specified.
