---
name: text-writing
description: >-
  Mastery framework for technical document typography, page layout, and textbook/study guide authoring.
  Applies Edward Tufte's data-ink ratio, Robert Bringhurst's typographic canons, Mayer's spatial
  contiguity, and cognitive layout principles to eliminate visual clutter, optimize line measure,
  scaffold white space, and design publication-grade study materials.
---

# Text Writing & Page Design: The Art of Uncluttered Technical Publishing

> *"Cluttered and confusing pages are not an inevitable consequence of presenting complex information; they are consequences of bad design."*  
> — Adapted from Edward Tufte, *Envisioning Information*

Presenting complex technical and scientific knowledge requires more than accurate facts—it requires **cognitive page engineering**. When a page looks dense, cramped, or disorganized, the reader's brain expends working memory on **extraneous visual friction** (trying to track long lines, parse nested borders, or locate key formulas) rather than absorbing the material.

This skill codifies the layout, typographic, and structural principles used by the world's finest textbook publishers (MIT Press, Cambridge University Press, No Starch Press), mathematical treatises (Knuth, Stewart), and information design pioneers (Edward Tufte, Robert Bringhurst).

---

## 1. The Core Philosophy: The Typography of Cognition

Human working memory can process information only when visual entry points and spatial structure guide the eye effortlessly.

```
Total Mental Effort = Content Complexity (Intrinsic) + Visual Friction (Extraneous)
```

Your objective in text writing and page design is to **drive extraneous visual friction to zero**.

### The 4 Laws of Uncluttered Technical Pages

1. **The Law of the Measure (Optimal Line Length)**: Never stretch body text across the full width of an A4 or US Letter page. Lines longer than 75 characters exhaust the eye's saccadic tracking muscle.
2. **The Data-Ink Ratio (Tufte)**: Every drop of ink on the page must deliver knowledge or provide spatial structure. Remove decorative borders, heavy table gridlines, gratuitous divider rules, and redundant containers.
3. **The Spatial Contiguity Principle (Mayer)**: Text and the diagrams, equations, or code snippets it explains must exist in the immediate visual field. Never force a reader to turn a page or scroll away to inspect an illustration.
4. **The Law of Declarative Diagrams (No Manual Drafting)**: Never draw diagrams by hand using ASCII art (`+---+`, `-->`) or raw SVG XML. Always express visualizations using Botox's native Diagram-as-Code engines (`mermaid`, `dot`, `d2`, `plantuml`, `bytefield`, `wavedrom`, `erd`). Botox compiles them into crisp vector graphics with zero layout breakage.

---

## 2. Page Geometry & The 3 Canonical Layout Grids

Textbooks and high-impact study guides rely on deliberate geometric grids rather than arbitrary default margins.

### Grid A: The Asymmetric Tufte Grid (Main Text + Sidenote Channel)
* **Best for**: Advanced textbooks, comprehensive tutorials, and deep-dive technical guides.
* **Structure**: A readable main column (55–65 characters wide) paired with a generous outer margin (5–7 cm wide).
* **Usage**:
  - The main column carries the core narrative flow uninterrupted.
  - The outer margin houses: definitions, micro-diagrams, historical context, cross-references, and small formulas.
* **Why it works**: Eliminates footnotes and endnotes completely. The reader's eye moves laterally across a single line rather than dropping to the bottom of the page or flipping to an appendix.

### Grid B: The Compact Two-Column Grid (Academic / Exam Reference)
* **Best for**: Exam synthesis sheets, formula compendiums, cheatsheets, and dense revision notes.
* **Structure**: Two equal columns (widths 80–85mm) separated by a generous central gutter (6–8mm).
* **Usage**:
  - Cuts line length down to the golden 40–55 character range.
  - Increases information density by 30–40% without inducing claustrophobia.
  - Large diagrams and wide tables span both columns (`columns: 1` or floating figures), while worked calculations stay within their column.

### Grid C: The Generous Single-Column Grid (Standard Textbook)
* **Best for**: Standard documentation, course handouts, and laboratory guides.
* **Structure**: Centered text block with deep lateral margins (minimum 2.5–3.2 cm on A4/Letter).
* **Rule**: Set `geometry: "margin=2.8cm"` or wider in frontmatter. If margins are too narrow (e.g. `margin=1cm`), body text expands to 110+ characters per line, ruining readability.

---

## 3. Typographic Rules of Bringhurst: Measure, Rhythm & Scale

Typography is the architecture of the written word. Follow these strict rules derived from Robert Bringhurst's *The Elements of Typographic Style*:

### 1. The Measure (Line Length)
* **The Golden Standard**: **45 to 75 characters per line** (including spaces). 66 characters is widely considered ideal.
* **Symptom of Failure**: Lines with 90–120 characters force the reader's eye to perform a long horizontal sweep to find the start of the next line, causing frequent line-skipping and eye strain.
* **How to Enforce**: Adjust page margins or switch to a two-column layout.

### 2. Vertical Rhythm & Leading (Line Spacing)
* **Body Text**: Line height (leading) must be proportional to font size and line length.
  - For standard 10pt–11pt text: use a line-height multiplier of **1.15 to 1.3**.
  - A measure of 70 characters requires slightly more leading (1.25) than a tight measure of 45 characters (1.15).
* **Headings**: As font size increases, **leading must decrease**. Title line-height should be tight (1.05–1.1) to avoid awkward gaps between wrapped heading lines.

### 3. The Modular Typographic Scale
Use a consistent geometric progression (e.g. Major Third $1.25$ or Perfect Fourth $1.333$) across headings:

| Level | Role | Size (10pt Base) | Weight / Style | Spacing Before / After |
| :--- | :--- | :--- | :--- | :--- |
| **Title** | Document Title | 24pt–28pt | Bold / Display | Page top |
| **Subtitle**| Context / Author | 13pt–14pt | Regular / Muted | 4pt after title |
| **H1** | Major Theme / Chapter | 18pt–20pt | Bold | 18pt before, 8pt after |
| **H2** | Section / Concept | 14pt–15pt | Semibold | 14pt before, 6pt after |
| **H3** | Sub-topic / Step | 11.5pt–12pt | Medium / Bold | 10pt before, 4pt after |
| **Body** | Core Prose | 10pt–10.5pt | Regular | 0 before, 6pt–8pt after |
| **Caption** | Annotations / Notes | 8.5pt–9pt | Italic / Muted | 4pt around figures |

### 4. Paragraph Separation vs. Indentation
* **Choose One, Never Both**:
  - **Block Paragraphs (Technical Publishing Standard)**: Zero first-line indent, separated by $0.65\text{em} - 0.85\text{em}$ vertical space. Provides clean rectangular blocks that align with tables, callouts, and code blocks.
  - **Indented Paragraphs (Literary Fiction)**: First line indented by $1\text{em} - 1.5\text{em}$, zero vertical space between paragraphs. *Avoid in technical texts with heavy math and code.*

---

## 4. Anti-Clutter Anatomy: Transforming the "Wall of Text"

Students and professionals do not read study guides linearly—they **scan** in an F-pattern looking for anchors, principles, formulas, and pitfalls.

### 🔴 The Anti-Pattern: The Unbroken Wall of Text
A monolithic block of 35 lines of uniform text explaining three definitions, an algebraic step, and a caution. Readers scan the first two sentences, feel overwhelmed, and glaze over.

### 🟢 The Solution: The 4-Anchor Scaffolding

Transform dense prose into structured, scannable units using these four visual anchors:

#### 1. Bold Lead-in Anchors (Run-in Heads)
Never present bulleted lists as generic blocks of text. Bold the first 2–4 words to provide an immediate summary:
* 🔴 **Bad**:
  * We assume that all events in the sample space have non-zero probabilities.
  * In order to compute the posterior, you must sum the prior likelihood products.
* 🟢 **Good**:
  * **Strict Positivity**: Every partition subset $A_i$ must satisfy $P(A_i) > 0$.
  * **Exhaustive Recoil**: The sum of all prior-likelihood products forms the normalizing denominator $P(B)$.

#### 2. Semantic Callout Elevators
Elevate high-priority insights into styled callouts to break visual monotony:
* `> [!NOTE]` — Background context, prerequisite reminders.
* `> [!TIP]` — Mental shortcuts, natural frequency intuitions, exam heuristics.
* `> [!IMPORTANT]` — Formal constraints, non-negotiable proof requirements.
* `> [!CAUTION]` / `> [!WARNING]` — Classic traps, false assumptions, non-obvious failure modes.

> [!TIP]
> **Constraint**: Do not stack callouts consecutively. Separate callouts with explanatory prose, worked steps, or diagrams. A page made entirely of callouts is just a different kind of clutter.

#### 3. Micro-Comparison Tables over Descriptive Lists
Whenever comparing two or more entities across identical dimensions (e.g. Incompatible vs. Independent, Series vs. Parallel, Stack vs. Heap), **always use a table**:

| Dimension | Incompatible ($A \cap B = \emptyset$) | Independent ($P(A \cap B) = P(A)P(B)$) |
| :--- | :--- | :--- |
| **Physical Meaning** | Cannot occur simultaneously | Occurrence of one gives zero information about the other |
| **Joint Probability** | $P(A \cap B) = 0$ | $P(A \cap B) = P(A) \cdot P(B)$ |
| **Union Rule** | $P(A \cup B) = P(A) + P(B)$ | $P(A \cup B) = P(A) + P(B) - P(A)P(B)$ |
| **Mutual Compatibility** | Mutually exclusive | Can only coexist if $P(A) = 0$ or $P(B) = 0$ |

#### 4. The Snug Math Formula (No Trapped Numbering)
Mathematical formulas that carry final answers should be boxed cleanly:
* Use snug inline boxing: `\boxed{...}` transpiled to `#box(stroke: 0.5pt, inset: (x: 8pt, y: 6pt))[$...$]`.
* Keep equation counters (`(17)`) outside the stroke on the right margin.

---

## 5. Designing Textbook-Grade Worked Problems

The hallmark of elite study guides (e.g. Schaum's Outlines, Halliday & Resnick) is a three-part anatomical structure for every worked problem:

```
┌────────────────────────────────────────────────────────┐
│ 1. Problem Framing & Given Data (Inputs)               │  ← Bulleted parameters, clear event labels
├────────────────────────────────────────────────────────┤
│ 2. Strategy & Applied Law (The "Why")                  │  ← 1-2 sentences stating the theorem BEFORE math
├────────────────────────────────────────────────────────┤
│ 3. Step-by-Step Resolution Trace (Aligned Equations)   │  ← Clear arithmetic progression
├────────────────────────────────────────────────────────┤
│ 4. Highlighted Target Outcome (Final Answer Box)       │  ← Snug box with numeric and percentage values
└────────────────────────────────────────────────────────┘
```

### Example: The Elite Problem Layout

#### Problème 2, Question c : Fiabilité globale du système

**Données extraites de l'énoncé** :
* Structure logique : Composant 1 en série avec le bloc parallèle $\{C_2, C_3\}$.
* Probabilités élémentaires : $P(A_1) = 1/2$, $P(A_2) = 9/20$, $P(A_3) = 1/2$.
* Intersections connues : $P(A_1 \cap A_2) = 9/40$, $P(A_1 \cap A_3) = 1/4$, $P(A_1 \cap A_2 \cap A_3) = 9/800$.

**Stratégie de résolution** :  
Le système fonctionne s'il existe au moins un chemin continu entre l'entrée et la sortie. Comme l'indépendance mutuelle des 3 composants n'est pas vérifiée, nous décomposons le système en ses deux chemins minimaux de succès :
$$S = (A_1 \cap A_2) \cup (A_1 \cap A_3)$$
Puis nous appliquons la formule du crible de Poincaré sur ces deux chemins dépendants (partageant le composant critique $C_1$).

**Résolution pas-à-pas** :
1. *Application du crible de Poincaré* :
   $$P(S) = P(A_1 \cap A_2) + P(A_1 \cap A_3) - P((A_1 \cap A_2) \cap (A_1 \cap A_3))$$
2. *Simplification de l'intersection double* :  
   Puisque $(A_1 \cap A_2) \cap (A_1 \cap A_3) = A_1 \cap A_2 \cap A_3$ :
   $$P(S) = P(A_1 \cap A_2) + P(A_1 \cap A_3) - P(A_1 \cap A_2 \cap A_3)$$
3. *Substitution numérique sur dénominateur commun (800)* :
   $$P(S) = \frac{180}{800} + \frac{200}{800} - \frac{9}{800} = \frac{371}{800}$$

> **Résultat final** :
> $$\boxed{P(S) = \frac{371}{800} = 0.46375 \quad (46.38\%)}$$

---

## 6. The 7 Deadly Sins of Page Design (What to Avoid)

| Anti-Pattern | Description | Cognitive Harm | The Fix |
| :--- | :--- | :--- | :--- |
| 🚫 **The Hand-Drawn ASCII Trap** | Drawing system architectures, workflows, or states by hand using ASCII art (`+---+`, `-->`). | Brittle layout; line-wrapping corrupts shapes; looks unpolished in PDFs. | Replace with declarative Botox diagram fences (`mermaid`, `dot`, `d2`). |
| 🚫 **The Edge-to-Edge Highway** | Single column spanning full page width with 110+ characters per line. | Severe eye fatigue; reader loses track when scanning to the next line. | Increase margins to $\geq 2.5\text{cm}$ or switch to a two-column layout. |
| 🚫 **The Boxing Match** | Enclosing every single paragraph, definition, and equation in nested bordered boxes. | Page looks like a wireframe or jail cell; borders compete with content. | Remove borders. Use subtle background tints (e.g. `#f8fafc`), generous white space, and left accent bars. |
| 🚫 **The Disconnected Figure** | A diagram placed 2 pages after the text that references it ("See Fig 4 on page 12"). | Breaks mental immersion; forces destructive physical page-flipping. | Place diagram immediately adjacent to or below its explanation. |
| 🚫 **The Rainbow Highlighter** | Using 5+ arbitrary font colors across the text. | Destroys visual hierarchy; reader cannot determine what is truly important. | Restrict palette to: 1 dark neutral (text), 1 subtle background tint, 1 primary theme color (links/headers), and 1 accent color (warnings/targets). |
| 🚫 **The Heavy Iron Table Grid** | Heavy black border lines (`border: 2px solid black`) around every cell in a table. | The grid lines dominate the data; text looks trapped inside a cage. | Eliminate vertical lines entirely. Use subtle horizontal rules (`0.5pt #e2e8f0`) under headers and soft alternating row fills. |
| 🚫 **The Monolithic Equation Wall** | Six lines of algebraic derivations crammed without commentary or line breaks. | Reader cannot locate where a transformation happened or which rule was applied. | Annotate each step with a brief justification on the right or above the line. |

---

## 7. The Pre-Flight Page Design & Typesetting Checklist

Before exporting or printing any study guide, textbook chapter, or technical documentation, audit every page spread against these 9 checkpoints:

1. [ ] **Declarative Visuals**: Did I avoid hand-drawn ASCII art and use Botox native diagram fences (`mermaid`, `dot`, `d2`)?
2. [ ] **Measure Check**: Is the body text line length between **45 and 75 characters**? (If >80, increase margins or use two columns).
3. [ ] **White Space Breathing Room**: Does the page contain at least **25–35% negative space** (margins, paragraph spacing, figure paddings)?
4. [ ] **Scannability / Bold Lead-ins**: Does every bulleted item and step start with a bold concept anchor?
5. [ ] **Spatial Contiguity**: Is every diagram, graph, or matrix positioned immediately adjacent to the text discussing it?
6. [ ] **Table Cleanliness**: Are tables free of heavy black vertical gridlines, using clean horizontal header rules and comfortable cell padding?
7. [ ] **Callout Discipline**: Are callouts (`[!NOTE]`, `[!TIP]`, `[!WARNING]`) used judiciously and spaced out, rather than stacked consecutively?
8. [ ] **Formula Framing**: Are final conclusions and key theorems highlighted cleanly with snug inline boxes (`\boxed{...}`) rather than full-width block borders?
9. [ ] **Vertical Rhythm**: Do headings have noticeably more space *before* them than *after* them, visibly anchoring them to the section they introduce?
