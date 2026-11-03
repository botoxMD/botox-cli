use std::fs;
use std::path::{Path, PathBuf};

/// Returns the complete, production-grade SKILL.md definition for AI coding agents.
pub fn generate_skill_content() -> &'static str {
    r##"---
name: botox
description: >-
  Use Botox to compile Markdown files into publication-quality LaTeX PDF documents
  or interactive HTML presentation slides. Provides comprehensive guidance on document
  frontmatter, slide creation, styling themes, math typesetting, Kroki/Mermaid diagrams,
  callouts, bibliographies, and CLI flags.
---

# Botox: Publication-Grade Typesetting & Presentations

Botox is a high-performance, single native binary document compiler written in pure Rust. It transforms standard Markdown into LaTeX-quality PDF documents or interactive HTML slide decks using an in-process Typst typesetting engine—without requiring Pandoc, TeX Live, or Node.js.

---

## 1. Core Principles & Workflow Selection

Botox operates in two primary modes determined automatically from Markdown frontmatter or CLI flags:

### A. Document Mode (Default)
- **Primary Target**: Paginated PDF documents (`.pdf`), with optional HTML (`.html`) or JSON vector pages (`.json`).
- **Use Cases**: Academic papers, technical specifications, books, RFCs, essays, reports, and cheat sheets.
- **Trigger**: Standard Markdown without `marp: true` or `presentation: true`, or when `--pdf` is passed explicitly.

### B. Presentation Slide Deck Mode
- **Primary Target**: Interactive HTML slide decks (`.html`), with optional PDF export (`-o deck.pdf`).
- **Use Cases**: Conference talks, lecture slides, pitch decks, and technical presentations.
- **Trigger**: Markdown containing `marp: true`, `slides: true`, or `presentation: true` in YAML frontmatter, or when `--slides` is passed explicitly.
- **Slide Delimiters**: Slides are separated by horizontal rules (`---`, `===`, or `\newpage`).
- **Stepwise Reveals**: Supports incremental pause points via `<!-- pause -->` or `\pause`.

---

## 2. Command Line Interface (CLI) Reference

### Common Invocation Patterns
```bash
# 1. Initialize new templates
botox init document.md               # Scaffold an academic PDF document template
botox init slides.md                 # Scaffold an interactive presentation slide deck

# 2. Compile documents (defaults to .pdf output)
botox document.md                    # Compiles to document.pdf
botox document.md -o custom.pdf      # Explicit PDF destination
botox document.md -o document.html   # Export document to HTML reader
botox document.md -o pages.json      # Export vector SVG pages JSON for webviews

# 3. Compile presentations (defaults to .html output)
botox slides.md                      # Compiles to interactive slides.html
botox slides.md -o deck.pdf          # Export presentation slides to PDF

# 4. Live watch mode (recompiles automatically on change)
botox -w document.md
botox --watch slides.md

# 5. Unix standard I/O streaming (stream PDF directly to stdout)
cat document.md | botox - -o - > output.pdf

# 6. Configuration inspection & Agent Skills
botox config                         # Display active configuration cascade
botox setup                          # Interactive configuration wizard
botox setup --global --author "Name" # Non-interactive config setup
botox skills                         # Generate .agents/skills/botox/SKILL.md in project
botox skills --global                # Install to ~/.agents/skills/botox/SKILL.md
botox skills -o -                    # Print skill definition directly to stdout
```

### Complete CLI Flag Table
| Flag / Option | Argument | Description | Default |
| :--- | :--- | :--- | :--- |
| `-o, --output` | `<path>` | Output destination (`.pdf`, `.html`, `.svg`, `.json`, or `-` for stdout) | `.pdf` (doc), `.html` (slides) |
| `-w, --watch` | None | Continuously watch input file & directory for changes and recompile | Disabled |
| `--pdf` | None | Force PDF compilation mode regardless of frontmatter | Auto-detected |
| `--slides` | None | Force presentation slide deck mode | Auto-detected |
| `--theme` | `<theme>` | Document theme (`academic`, `modern`, `elegant`, `technical`, `compact`, `minimal`) or Slide theme (`default`, `academic`, `gaia`, `uncover`, `nord`, `dark`) | `academic` / `default` |
| `--toc` | None | Enable automatic Table of Contents | Config/Frontmatter |
| `--no-toc` | None | Explicitly disable Table of Contents | Disabled |
| `--author` | `<name>` | Override primary author name | Config/Frontmatter |
| `--font` | `<name>` | Primary typeface family | `New Computer Modern` |
| `-N, --number-sections`| None | Number headings automatically (1, 1.1, 1.1.1) | Disabled |
| `-b, --bibliography` | None | Transform hyperlinks into IEEE numbered citations `[1]` with an end bibliography | Disabled |
| `--no-bibliography` | None | Disable link-to-citation bibliography transformation | Enabled |
| `--exclude-ref` | `<csv>` | Comma-separated domains/patterns to exclude from bibliography (e.g. `github.com,x.com`) | None |
| `--include-ref` | `<csv>` | Comma-separated domains/patterns to restrict bibliography citations to | None |
| `--markdown, --md` | None | Export to clean, portable standard Markdown with assets & resolved citations | Disabled |
| `--assets-dir` | `<dir>` | Directory for exported diagram images in Markdown export mode | `assets` |
| `--config` | `<path>` | Path to custom `botox.yaml` configuration file | Auto-discovered |
| `-V, --version` | None | Print version information and exit | None |
| `-h, --help` | None | Print CLI help information and exit | None |

---

## 3. Document Frontmatter & Metadata Reference

Documents declare configuration in YAML frontmatter enclosed by `---`:

```yaml
---
title: "Autonomous Agentic Workflows in Distributed Systems"
subtitle: "An Architectural Analysis and Implementation Survey"
author:
  - name: "Dr. Jane Doe"
    affiliation: "Department of Computer Science, MIT"
    email: "jdoe@mit.edu"
  - name: "John Smith"
    affiliation: "AI Systems Laboratory"
date: \today                         # Or "2026-10-13", or "October 2026"
abstract: >
  This paper explores modern autonomous agent architectures, evaluating memory
  retrieval latency, multi-pass tool invocation, and formal layout engines.
keywords: "distributed systems, agentic coding, typst, rust"
lang: en
papersize: a4                        # a4, us-letter, a5
orientation: portrait                # portrait (default) or landscape
landscape: false                     # true for full-document horizontal orientation
fontsize: 11pt                       # 9pt, 10pt, 11pt, 12pt
mainfont: "New Computer Modern"      # Any system font or embedded font
mathfont: "New Computer Modern Math"
monofont: "DejaVu Sans Mono"
columns: 1                           # 1 (single column) or 2 (two-column layout)
column-gutter: 18pt                  # Gutter between columns in two-column mode
linestretch: 1.15                    # Line spacing multiplier
geometry: "margin=2.5cm"             # Or margin: "1in", or margin: { x: "2cm", y: "2.5cm" }
number-sections: true                # Automatically number section headings
table-of-contents: true              # Generate Table of Contents
toc-depth: 3                         # 1 (H1), 2 (H1-H2), 3 (H1-H3)
toc-title: "Table of Contents"       # Custom outline header
linkcolor: "#2563eb"                 # Hex color or named color (blue, red, teal, etc.)
kroki_url: "https://kroki.io"        # Kroki endpoint for diagram compilation (or kroki-url / kroki_endpoint)
bibliography: true                   # Transform web links into an IEEE bibliography
exclude-references:
  - "github.com"
  - "x.com"
theme: academic                      # academic, modern, elegant, technical, compact, minimal
---
```

### Supported Author Formats
1. **Single String**: `author: "Jane Doe"`
2. **List of Strings**: `author: ["Jane Doe", "John Smith"]`
3. **List of Structured Maps**:
   ```yaml
   author:
     - name: "Jane Doe"
       affiliation: "MIT CSAIL"
     - name: "John Smith"
       affiliation: "Stanford University"
   ```

### Document Themes Detailed
* **`academic`** (Default):
  - Typeset in New Computer Modern with classical serif proportions.
  - Centered title, bold section headers, indented paragraphs, clean horizontal rules.
  - Standard IEEE / ACM / arXiv style.
* **`modern`**:
  - Contemporary sans-serif styling with prominent colored accent bars.
  - Blue accent highlights (`#2563eb`), card-style abstract containers, and crisp headings.
  - Excellent for whitepapers, product briefs, and executive summaries.
* **`elegant`**:
  - Warm editorial aesthetic with warm amber and deep slate accents.
  - Centered titles, generous leading, classical Roman numerals, and subtle borders.
  - Ideal for long-form essays, books, and thought leadership pieces.
* **`technical`**:
  - Engineering specification format with monospace metadata headers.
  - Teal borders (`#0d9488`), grid-aligned author/date blocks, and structured code blocks.
  - Tailored for RFCs, API designs, and systems documentation.
* **`compact`**:
  - Space-optimized multi-column layout with slim margins and dense line spacing.
  - Ideal for two-column conference papers, cheat sheets, and technical summaries.
* **`minimal`**:
  - Distraction-free pure typography with zero decorative rules or background tinting.
  - Maximum typographic focus on content.

---

## 4. Presentation Slide Decks (Marp Compatibility)

To author presentation slides, include `marp: true` or `slides: true` in frontmatter:

```yaml
---
marp: true
theme: default                       # default, academic, gaia, uncover, nord, dark
paginate: true                       # Show slide numbers (e.g. 1 / 15)
header: "Engineering Review Q4"      # Persistent slide header
footer: "Confidential"               # Persistent slide footer
size: 16:9                           # 16:9 (default) or 4:3
color: "#f8fafc"                     # Optional global text color
backgroundColor: "#0f172a"          # Optional global slide background color
---

# Scalable Agent Architecture
### Distributed Systems Group

---

# Architecture Overview

- **Core Compiler**: Pure Rust single native binary
- **Layout Engine**: In-process Typst vector pipeline
- **Latency**: Sub-10ms incremental recompiles

---

# Data Pipeline

```mermaid
graph LR;
  Markdown --> Parser --> TypstEngine --> Output[PDF / HTML];
```

<!-- pause -->

> [!NOTE]
> All diagram compilation is cached locally using content SHA256 hashes.
```

### Slide Mechanics & Controls
* **Slide Dividers**: Use `---`, `===`, or `\newpage` on an isolated line to begin a new slide.
* **Incremental Steps (`\pause` or `<!-- pause -->`)**:
  - Inserts a stepwise reveal point. In the interactive HTML viewer, items after the pause are revealed on the next slide transition or arrow key press.
* **Interactive HTML Presentation Controls**:
  - `Right Arrow` / `Down Arrow` / `Space` / `Enter` / `N`: Next slide or next pause step.
  - `Left Arrow` / `Up Arrow` / `Page Up` / `P`: Previous slide.
  - `Home` / `End`: First / last slide.
  - `F`: Toggle full-screen presentation mode.
  - `O`: Toggle interactive thumbnail overview grid.
  - `Esc`: Close thumbnail overview or exit fullscreen.
  - Touch: Horizontal swipe gestures on mobile and tablets.
* **Slide Themes**:
  - `default`: Clean, bright presentation theme with crisp typography.
  - `academic`: Beamer-inspired presentation theme for academic conferences.
  - `gaia`: Warm terracotta and earth tones (compatible with Marp Gaia).
  - `uncover`: High-contrast minimalist theme with centered slide elements.
  - `nord`: Cool Arctic Frost palette with slate-blue background.
  - `dark`: Sleek charcoal dark mode.

---

## 5. Diagrams & Visualizations (Kroki & Mermaid)

Botox natively converts diagram code fences into sharp, scalable vector graphics. Diagrams are rendered via a Kroki endpoint, cached locally by content SHA256, and embedded as native vector assets.

> [!IMPORTANT]
> **Never Create Diagrams by Hand**:
> * **NO Hand-Drawn ASCII Art**: Do **NOT** draw diagrams by hand using plain text characters (`+---+`, `|   |`, `-->`). ASCII art breaks responsive wrapping, cannot scale, looks amateurish in publication PDFs, and fails on mobile screens.
> * **NO Raw SVG XML or Ad-Hoc Scripts**: Do **NOT** hand-craft raw `<svg>` code or generate ad-hoc image scripts when a declarative diagram syntax exists.
> * **ALWAYS Use Botox Native Declarative Fences**: Write clean Diagram-as-Code fences (`mermaid`, `dot`/`graphviz`, `d2`, `plantuml`, `bytefield`, `wavedrom`, `erd`). Botox automatically transpiles and compiles them into publication-grade vector graphics with automatic local caching.

### Configuring `kroki_url` (Required for Diagram Compilation)
To enable diagram rendering, configure `kroki_url` (also accepts `kroki-url` or `kroki_endpoint`) in either your document's YAML frontmatter or under `document:` in `botox.yaml`:
```yaml
---
kroki_url: "https://kroki.io"
---
```
> [!TIP]
> For complex flowcharts with multi-line node labels (`\n`) or nested subgraphs, **`graphviz` (`dot`)** is strongly recommended over `mermaid`, because Graphviz computes native SVG `<text>` bounding boxes server-side without relying on browser DOM `<foreignObject>` measurements.

### Supported Diagram Engines
| Fence Identifier | Engine | Description |
| :--- | :--- | :--- |
| `mermaid` | Mermaid.js | Flowcharts, sequence diagrams, class, state, gitGraph, erDiagram, gantt |
| `umlet`, `uxf` | UMLet | Fast UML class, sequence, activity, and deployment diagrams |
| `plantuml`, `puml` | PlantUML | Component, sequence, state, class, and deployment diagrams |
| `c4plantuml` | C4-PlantUML | C4 software architecture diagrams |
| `graphviz`, `dot` | Graphviz | Directed graphs, network topologies, finite state machines |
| `d2` | D2 Lang | Declarative modern diagrams with auto-layout |
| `bytefield` | Bytefield | Binary protocol packet structures and memory layouts |
| `ditaa` | Ditaa | ASCII art converted to diagrams |
| `wavedrom` | WaveDrom | Digital timing waveforms |
| `bpmn` | BPMN | Business Process Model and Notation |
| `erd` | ERD | Entity-Relationship Diagrams |
| `nomnoml` | Nomnoml | UML diagrams using simple syntax |
| `svgbob` | Svgbob | ASCII art diagrams to clean SVG |

### Diagram Code Fence Attributes
Pass attributes inside `{...}` on the fence header:
````markdown
```{mermaid, caption="Microservices Event Mesh", id="fig:event-mesh", width="85%"}
sequenceDiagram
    autonumber
    Client->>API Gateway: POST /orders
    API Gateway->>EventBus: OrderCreated
    EventBus->>PaymentService: Process Payment
    PaymentService-->>EventBus: PaymentCompleted
    EventBus->>InventoryService: Reserve Stock
```

As illustrated in @fig:event-mesh, orders are processed asynchronously.
````

### External Diagram & Code File Referencing
Include external files directly to prevent code duplication:
````markdown
<!-- 1. Diagram file referencing in code fence -->
```{mermaid file="architecture.mmd", caption="System Topology", id="fig:topo"}
```

<!-- 2. UMLet file referencing -->
```{umlet file="models/classes.uxf", caption="Domain Model", id="fig:domain"}
```

<!-- 3. Image syntax for diagram files -->
![Architecture Blueprint](diagrams/network.uxf){#fig:network width=90%}

<!-- 4. Source code file embedding -->
```rust file="src/main.rs"
```
````

### Diagram Design Principles: Good vs. Bad Examples

Effective diagrams provide an immediate, accurate mental model of a system. Poor diagrams induce cognitive overload with tangled lines, ambiguous labels, and mixed levels of abstraction.

#### 1. System Architecture & Flowcharts: Spaghetti vs. Bounded Flow

##### 🔴 Anti-Pattern: The "Hairball / Spaghetti" Diagram
- **Symptoms**: Criss-crossing arrows, mixed directional flow, ambiguous generic labels (*"connects to"*, *"data"*), missing tier boundaries, and inconsistent node shapes.
````markdown
```mermaid
graph TD
    Client --> API
    API --> DB
    Worker --> API
    API --> Cache
    Worker --> DB
    Client --> Cache
    API --> ThirdParty
    ThirdParty --> DB
    Cache --> API
```
````
*Why it fails*:
- **Visual Friction**: Intersecting edges force the reader's eye to untangle lines rather than understand the architecture.
- **Mixed Abstraction**: A high-level client device is placed on the same level as internal in-memory caches and background queue workers.
- **Zero Protocol Insight**: Arrows provide no information on whether communication is synchronous HTTPS, asynchronous pub/sub, or a direct database query.

##### 🟢 Best Practice: Structured Tiers & Unidirectional Flow
- **Pattern**: Strict left-to-right (`flowchart LR`) or top-to-bottom (`flowchart TD`) flow, semantic subgraphs, explicit protocols, and consistent node shapes.
````markdown
```mermaid
flowchart LR
    subgraph Ingestion["Client & Edge Tier"]
        Browser["Web Client<br/><i>(React SPA)</i>"]
        Gateway["API Gateway / Envoy<br/><i>(TLS Termination)</i>"]
    end

    subgraph ServiceMesh["Core Microservices Tier"]
        AuthSvc["Auth Service<br/><i>(OAuth2 / JWT)</i>"]
        OrderSvc["Order Service<br/><i>(Rust / Axum)</i>"]
    end

    subgraph Storage["Persistence & Event Tier"]
        DB[("Order DB<br/><i>(PostgreSQL Primary)</i>")]
        Broker(["Kafka Event Bus<br/><i>(Topic: orders-v1)</i>"])
    end

    Browser -->|HTTPS POST /checkout| Gateway
    Gateway -->|mTLS /validate| AuthSvc
    Gateway -->|gRPC CreateOrder| OrderSvc
    OrderSvc -->|SQL Transaction| DB
    OrderSvc -.->|Async Event| Broker
```
````
*Why it works*:
- **Unidirectional Layout**: Left-to-right movement mirrors the natural processing lifecycle (Client $\to$ Gateway $\to$ Microservice $\to$ Storage).
- **Grouped Bounded Contexts**: `subgraph` blocks clearly isolate networking and security zones.
- **Protocol Precision**: Connections explicitly document the transport (`HTTPS`, `mTLS`, `gRPC`, `SQL`, `Kafka`).
- **Semantic Shapes**: Standard rectangles for services, cylinders `[(...)]` for persistence, and rounded stadiums `([...])` for event queues.

---

#### 2. Sequence Diagrams: Unbounded Blast Radius vs. Scoped Workflows

##### 🔴 Anti-Pattern: The Uncontrolled Blast Radius
- **Symptoms**: Dozens of actors crammed onto one diagram, no step numbers (`autonumber`), synchronous arrows used for background tasks, missing execution lifelines, and zero error handling.
````markdown
```mermaid
sequenceDiagram
    User->>Frontend: Click Buy
    Frontend->>Backend: Process
    Backend->>Payment: Pay
    Backend->>Warehouse: Stock
    Backend->>Email: Send
    Payment->>Backend: OK
    Warehouse->>Backend: OK
    Backend->>Frontend: Done
    Frontend->>User: Success
```
````
*Why it fails*:
- Without `autonumber` and execution activations (`activate`/`deactivate`), it is impossible to tell which service is waiting and which is working in parallel.
- Fails to model the reality of real systems: network failures, declined cards, and timeouts.

##### 🟢 Best Practice: Scoped Lifelines, Activations & Error Paths
- **Pattern**: Use `autonumber`, distinct sync (`->>`) vs async (`--)`) lines, execution activations (`+`/`-`), and explicit `alt ... else ... end` branches.
````markdown
```mermaid
sequenceDiagram
    autonumber
    actor Customer as User
    participant Web as Web Frontend
    participant API as Order Service
    participant Pay as Payment Gateway
    participant Bus as Kafka Broker

    Customer->>Web: Submit Checkout
    activate Web
    Web->>+API: POST /api/checkout (Payload)
    API->>+Pay: ChargeCustomer(Amount, Token)
    
    alt Payment Succeeded
        Pay-->>-API: 200 OK (ChargeID)
        API-)Bus: Publish OrderPaidEvent (Async)
        API-->>-Web: 201 Created (OrderRef)
        Web-->>Customer: Display Order Confirmation
    else Payment Declined / Timeout
        Pay-->>API: 402 Card Declined
        API-->>Web: 400 Bad Request (ErrorCode)
        Web-->>-Customer: Show Retry Payment Prompt
    end
    deactivate Web
```
````
*Why it works*:
- **Temporal Clarity**: `autonumber` allows accompanying prose to reference specific operations (e.g., *"In step 4, the event is emitted asynchronously"*).
- **Blocking vs Non-Blocking**: Synchronous remote calls (`->>`) are clearly distinguished from asynchronous event dispatches (`-)`).
- **Resilient Modeling**: Models alternative flows and failures explicitly.

---

#### 3. Scientific, Mathematical & Engineering Diagrams: Topological Congruence

Diagrams representing mathematics, probability, set theory, or physical systems must preserve **topological congruence**: the visual grammar (boxes, arrows, spatial enclosures) must match the underlying mathematical reality.

##### 🔴 Anti-Pattern 1: The "Tree Fallacy" for Sets & Partitions
- **Symptoms**: Modeling a static set system, partition, or Venn diagram as a directed tree flowchart with arrows (`Universe -> E_only, Both, F_only, Neither`).
- **Why it fails**:
  - **Category Error**: Arrows denote time, state transitions, or causality. Sets are static spatial containment regions.
  - **Destroys Additive Intuition**: The reader cannot visually see that $E$ is composed of $(E \cap F) \cup (E \cap \bar{F})$.

##### 🟢 Best Practice: 2×2 Partition Contingency Matrix
- **Pattern**: Use a 2D spatial matrix (`shape=none, label=<<table ...>>`) aligning intersections with row and column marginal sums.
````markdown
```dot
digraph PartitionMatrix {
    bgcolor="transparent";
    node [shape=none, fontname="DejaVu Sans", fontsize=10];

    Matrix [label=<
        <table border="0" cellborder="1" cellspacing="0" cellpadding="6">
            <tr>
                <td bgcolor="#f8fafc" border="0"><b>Univers &Omega;</b></td>
                <td bgcolor="#dbeafe"><b>Événement <i>F</i></b></td>
                <td bgcolor="#f1f5f9"><b>Contraire <i>F&#772;</i></b></td>
                <td bgcolor="#f8fafc"><b>Total Marginal</b></td>
            </tr>
            <tr>
                <td bgcolor="#dbeafe"><b>Événement <i>E</i></b></td>
                <td bgcolor="#93c5fd"><i>E &cap; F</i><br/><font point-size="8" color="#1e3a8a">Intersection</font></td>
                <td bgcolor="#eff6ff"><i>E &cap; F&#772;</i><br/><font point-size="8" color="#1e3a8a"><i>E</i> seulement</font></td>
                <td bgcolor="#dbeafe"><b>P(<i>E</i>)</b></td>
            </tr>
            <tr>
                <td bgcolor="#f1f5f9"><b>Contraire <i>E&#772;</i></b></td>
                <td bgcolor="#eff6ff"><i>E&#772; &cap; F</i><br/><font point-size="8" color="#1e3a8a"><i>F</i> seulement</font></td>
                <td bgcolor="#f8fafc"><i>E&#772; &cap; F&#772;</i><br/><font point-size="8" color="#475569">Extérieur</font></td>
                <td bgcolor="#f1f5f9"><b>P(<i>E&#772;</i>)</b></td>
            </tr>
            <tr>
                <td bgcolor="#f8fafc"><b>Total Marginal</b></td>
                <td bgcolor="#dbeafe"><b>P(<i>F</i>)</b></td>
                <td bgcolor="#f1f5f9"><b>P(<i>F&#772;</i>)</b></td>
                <td bgcolor="#bbf7d0"><b>P(&Omega;) = 1</b></td>
            </tr>
        </table>
    >];
}
```
````
*Why it works*: Shows the exact geometric summation: rows sum to $P(E)$, columns sum to $P(F)$, and the whole space sums to 1.

---

##### 🔴 Anti-Pattern 2: System Reliability Spurious Body-Port Splitting
- **Symptoms**: Branching parallel edges directly out of a single component box face (`C1 -> C2; C1 -> C3;`), creating angled diagonal wires.
- **Why it fails**: Components are two-terminal blocks ($In \to Out$). Fanning out directly from a component implies the component itself is a demultiplexer. Diagonal lines clutter the schematic.

##### 🟢 Best Practice: Orthogonal Bus Routing with Explicit Junction Nodes
- **Pattern**: Use `splines=ortho`, explicit junction dots (`shape=point, width=0.08`), and component probability annotations.
````markdown
```dot
digraph MixedReliabilityCircuit {
    rankdir=LR;
    bgcolor="transparent";
    splines=ortho;
    nodesep=0.5;
    ranksep=0.6;

    node [shape=box, style="filled,rounded", fontname="DejaVu Sans", fontsize=10];
    edge [color="#0284c7", penwidth=1.5, arrowhead=none];

    In  [shape=circle, label="I", fillcolor="#e0f2fe", color="#0284c7", width=0.4];
    Out [shape=circle, label="O", fillcolor="#e0f2fe", color="#0284c7", width=0.4];

    C1 [label="Composant 1\nP(A1) = 0.50", fillcolor="#eff6ff", color="#3b82f6"];
    C2 [label="Composant 2\nP(A2) = 0.45", fillcolor="#eff6ff", color="#3b82f6"];
    C3 [label="Composant 3\nP(A3) = 0.50", fillcolor="#eff6ff", color="#3b82f6"];

    J1 [shape=point, width=0.08, color="#0284c7"];
    J2 [shape=point, width=0.08, color="#0284c7"];

    In -> C1;
    C1 -> J1;
    J1 -> C2 -> J2;
    J1 -> C3 -> J2;
    J2 -> Out [arrowhead=vee];
}
```
````

---

##### 🔴 Anti-Pattern 3: The Cluttered Probability Tree & ASCII Math
- **Symptoms**: Duplicating probabilities on both branch edges and leaf nodes, wrapping every leaf in heavy rectangular boxes, using ASCII approximations (`E n F_bar`), and failing to visually emphasize the Bayesian evidence vs target hypothesis.

##### 🟢 Best Practice: Two-Stage Bayesian Tree with Evidence Highlighting
- **Pattern**: Clean stage transitions (Priors on stage 1 edges $\to$ Conditionals on stage 2 edges $\to$ Joint probabilities on leaves), Unicode math symbols ($\cap$, $\bar{A}$), dimmed eliminated paths, and highlighted Bayesian target paths.
````markdown
```dot
digraph BayesTree {
    rankdir=LR;
    bgcolor="transparent";
    nodesep=0.35;
    ranksep=0.8;

    node [fontname="DejaVu Sans", fontsize=10];
    edge [fontname="DejaVu Sans", fontsize=9, color="#64748b"];

    Root [shape=circle, label="Source", fillcolor="#f1f5f9", style="filled", color="#94a3b8", width=0.5];

    A   [shape=box, style="filled,rounded", label="A", fillcolor="#eff6ff", color="#3b82f6"];
    nA  [shape=box, style="filled,rounded", label="A&#772;", fillcolor="#fef3c7", color="#f59e0b", penwidth=2];

    AB  [shape=box, style="filled,rounded", label="B\nP(A &cap; B)", fillcolor="#dbeafe", color="#3b82f6"];
    AnB [shape=box, style="filled,dashed,rounded", label="B&#772;\n[Eliminated]", fillcolor="#f8fafc", color="#cbd5e1", fontcolor="#94a3b8"];

    nAB [shape=box, style="filled,rounded", label="B (Target)\nP(A&#772; &cap; B)", fillcolor="#fef08a", color="#d97706", penwidth=2];
    nAnB [shape=box, style="filled,dashed,rounded", label="B&#772;\n[Eliminated]", fillcolor="#f8fafc", color="#cbd5e1", fontcolor="#94a3b8"];

    Root -> A  [label=" P(A) ", color="#3b82f6"];
    Root -> nA [label=" P(A&#772;) ", color="#f59e0b", penwidth=2];

    A -> AB  [label=" P(B|A) ", color="#3b82f6"];
    A -> AnB [label=" P(B&#772;|A) ", style="dashed", color="#cbd5e1"];

    nA -> nAB  [label=" P(B|A&#772;) ", color="#d97706", penwidth=2];
    nA -> nAnB [label=" P(B&#772;|A&#772;) ", style="dashed", color="#cbd5e1"];
}
```
````

---

#### 4. Diagram Engine Selection: Avoiding Common Pitfalls

| Use Case | Recommended Engine | Avoid | Rationale |
| :--- | :--- | :--- | :--- |
| **State Machines & Automata** | `graphviz` (`dot`) | `mermaid` | Graphviz natively computes exact vector bounding boxes server-side without relying on browser `<foreignObject>` font metrics, preventing text clipping. |
| **Dense Network Topologies** | `graphviz` (`dot`) or `d2` | `mermaid` | Graphviz's Neato/Dot routing engines resolve complex multi-edge crossings cleanly without line collisions. |
| **Microservice Architecture** | `d2` or `mermaid` | ASCII art | `d2` and `mermaid` provide structured container nesting and clean orthogonal arrow routing. |
| **Sequence Interactions** | `mermaid` or `plantuml` | `graphviz` | Sequence diagrams require linear actor lifelines and activation boxes designed natively into Mermaid/PlantUML. |
| **Database Schemas & ERDs** | `erd` or `mermaid` | Manual flowcharts | Native ERD engines automatically format primary keys, foreign keys, and cardinalities (`1--*`, `0..1`). |

---

## 6. Mathematics & Equations

Botox supports both LaTeX mathematical macros and native Typst math syntax with LaTeX-quality typography:

### Syntax Rules
* **Inline Math**: Wrap in single dollar signs: `$f(x) = \alpha x + \beta$`
* **Display Math**: Wrap in double dollar signs:
  ```latex
  $$\int_{-\infty}^{+\infty} e^{-x^2} \, dx = \sqrt{\pi}$$
  ```
* **Equation Numbering & Cross-References**:
  ```latex
  $$\mathcal{L}(\theta) = \mathbb{E}_{x \sim p}[ \log p_\theta(x) ]$$ {#eq:loss}

  Minimizing the objective function defined in @eq:loss yields optimal parameters.
  ```

### Critical Syntax Difference: Multi-letter Variables
> [!IMPORTANT]
> In Typst math, juxtaposed characters represent multi-letter variable names rather than implicit multiplication.
> - **Incorrect**: `$E = mc^2$` (Error: unknown variable `mc`)
> - **Correct**: `$E = m c^2$` (Variable `m` multiplied by `c`)

### Supported LaTeX Mathematical Macros
* **Fractions & Roots**: `\frac{a}{b}`, `\sqrt{x}`, `\sqrt[n]{x}`
* **Calculus & Operators**: `\sum_{i=1}^n`, `\prod_{i=1}^n`, `\int_a^b`, `\oint`, `\partial`, `\nabla`, `\lim_{x \to 0}`
* **Accents & Vectors**: `\vec{v}`, `\mathbf{x}`, `\hat{y}`, `\bar{z}`, `\tilde{a}`, `\dot{x}`, `\ddot{x}`
* **Greek Alphabet**: `\alpha`, `\beta`, `\gamma`, `\delta`, `\epsilon`, `\theta`, `\lambda`, `\mu`, `\pi`, `\sigma`, `\tau`, `\phi`, `\omega`, `\Delta`, `\Gamma`, `\Theta`, `\Lambda`, `\Sigma`, `\Phi`, `\Omega`
* **Relations & Logic**: `\leq`, `\geq`, `\neq`, `\approx`, `\equiv`, `\sim`, `\forall`, `\exists`, `\in`, `\notin`, `\subset`, `\cup`, `\cap`
* **Arrows**: `\to`, `\rightarrow`, `\leftarrow`, `\Rightarrow`, `\iff`, `\mapsto`
* **Matrices**:
  ```latex
  $$\begin{bmatrix} a & b \\ c & d \end{bmatrix} \quad \begin{pmatrix} 1 & 0 \\ 0 & 1 \end{pmatrix}$$
  ```

---

## 7. Advisory Callout Blocks

Botox renders distinct, styled callout advisory boxes:

### GitHub-Style Callouts
```markdown
> [!NOTE]
> Informational context or background notes.

> [!TIP]
> Actionable recommendations for performance or best practices.

> [!IMPORTANT]
> Essential prerequisites or crucial steps.

> [!WARNING]
> Warnings regarding breaking changes or potential issues.

> [!CAUTION]
> High-risk alerts regarding security or data destruction.
```

### Custom Titles on Callouts
```markdown
> [!TIP] Optimizing Query Latency
> Adding an index on `(tenant_id, created_at)` reduces index scans by 90%.
```

### Pandoc-Style Divs
```markdown
::: note
**Pandoc Callout**: Seamlessly supported with clean theme styling.
:::

::: warning
**Caution**: Changes take effect immediately upon deployment.
:::
```

---

## 8. Tables, Cross-References & Formatting Extensions

### Pipe Tables with Captions
```markdown
| Method | Complexity | Memory | Throughput |
| :--- | :---: | :---: | ---: |
| Linear Scan | $O(N)$ | $O(1)$ | 12 MB/s |
| B-Tree Index | $O(\log N)$ | $O(N)$ | 480 MB/s |
| Hash Index | $O(1)$ | $O(N)$ | 1,250 MB/s |
Table: Performance Comparison of Index Structures {#tbl:perf}

See the comparative results in @tbl:perf.
```

### Section Headings & Numbering
* **Custom Section IDs**: `# Background & Prior Work {#sec:background}`
* **Unnumbered Headings**: `# Preface {-}` or `# Preface {.unnumbered}`
* **Referencing Sections**: `As discussed in @sec:background...`

### Page Breaks & Page Orientation (Landscape / Horizontal Pages)
Force page breaks in documents using any of the following:
* `\newpage`
* `\pagebreak`
* `\columnbreak` (breaks to the next column in multi-column layouts)
* `<div style="page-break-after: always"></div>`

#### Individual Landscape / Flipped Pages
Rotate individual pages (for wide tables, large architectural diagrams, or multi-column data matrices) while keeping the rest of the document in portrait:
* **Pandoc Fenced Div**:
  ```markdown
  ::: landscape
  | Long Table | Column 2 | Column 3 | Column 4 |
  |---|---|---|---|
  :::
  ```
* **LaTeX Environment**:
  ```latex
  \begin{landscape}
  ...
  \end{landscape}
  ```
* **HTML Directives**:
  ```html
  <!-- landscape -->
  ...
  <!-- /landscape -->
  ```

### Footnotes
```markdown
Modern systems employ content-addressable storage[^cas] for deduplication.

[^cas]: Content-addressable storage computes a cryptographic hash of content to use as its key.
```

---

## 9. Bibliographies & Citations

Botox provides automatic web link transformation into IEEE citation indices:

### Automatic Web Link Citations
Set `bibliography: true` in frontmatter or pass `-b` via CLI:
```markdown
The [Rust programming language](https://rust-lang.org) guarantees memory safety, while [Typst](https://typst.app) provides ultra-fast typesetting.
```
* **Output in Text**: "The Rust programming language [1] guarantees memory safety, while Typst [2] provides ultra-fast typesetting."
* **Generated End Section**:
  ```
  References
  [1] "Rust programming language", https://rust-lang.org
  [2] "Typst", https://typst.app
  ```

### Explicit References Heading
Place references anywhere in your document using `\ref`:
```markdown
\ref Selected Bibliography and Sources
```

### Filtering Citation Domains
Exclude non-academic links or include only specific domains in `botox.yaml` or frontmatter:
```yaml
exclude-references:
  - "github.com"
  - "twitter.com"
  - "x.com"
```

---

## 10. Configuration Cascade (`botox.yaml`)

Botox loads configuration hierarchically:
1. **Command Line Flags** (highest precedence)
2. **Document YAML Frontmatter**
3. **Project Local Config**: `./botox.yaml` or `./.botox.yaml`
4. **Parent Directory Configs** (walks up directory tree)
5. **Global User Config**: `~/.config/botox/config.yaml`
6. **Internal System Defaults** (lowest precedence)

### Sample `botox.yaml`
```yaml
document:
  theme: academic
  papersize: a4
  fontsize: 11pt
  mainfont: "New Computer Modern"
  kroki_url: "https://kroki.io"
  section_numbering: true
  toc: false
  bibliography: true
  exclude_references:
    - "github.com"
    - "x.com"

slides:
  theme: default
  paginate: true
  font: "New Computer Modern"
```

---

## 11. Troubleshooting & Diagnostics

Botox emits structured, source-mapped error messages pointing directly to your Markdown file:

* **`[LaTeX Error]`**: Syntax error in mathematical formulas.
  - *Fix*: Check for multi-letter variables without spaces (`$m c$` instead of `$mc$`), unmatched brackets `\left( ... \right)`, or misspelled macros.
* **`[Resource Error]`**: Missing local asset, diagram, or include file.
  - *Fix*: Verify relative path to image, `.uxf`, or `.mmd` file relative to the document directory.
* **`[Typst Error]`**: Layout or typesetting constraint error.
  - *Fix*: Inspect the exact line and snippet indicated in the diagnostic.
"##
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
        assert!(content.contains("Never Create Diagrams by Hand"));
        assert!(content.contains("Diagram Design Principles: Good vs. Bad Examples"));
        assert!(content.contains("The \"Hairball / Spaghetti\" Diagram"));
        assert!(content.contains("Structured Tiers & Unidirectional Flow"));
        assert!(content.contains("Scientific, Mathematical & Engineering Diagrams"));
        assert!(content.contains("The \"Tree Fallacy\" for Sets & Partitions"));
        assert!(content.contains("Partition Contingency Matrix"));
        assert!(content.contains("System Reliability Spurious Body-Port Splitting"));
        assert!(content.contains("Orthogonal Bus Routing with Explicit Junction Nodes"));
        assert!(content.contains("Two-Stage Bayesian Tree with Evidence Highlighting"));
        assert!(content.contains("Diagram Engine Selection"));
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
