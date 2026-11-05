# Botox Publishing Domain

Botox publishes LaTeX-quality documents, interactive presentation slide decks, and clean standard Markdown from extended Markdown sources.

## Language

**PublishingEngine**:
The core module that coordinates document intake, configuration cascading, preprocessing, and backend layout into compiled artifacts.
_Avoid_: Compiler runner, worker, orchestrator service

**DocumentIntent**:
The resolved target format, theme, geometry, and styling configuration derived from document frontmatter, cascading configuration files, and invocation options.
_Avoid_: Options bundle, config context, settings bag

**TypesettingBackend**:
The in-process layout and vector font compilation engine (Typst) that compiles enriched markup into pages and PDF bytes.
_Avoid_: Typst service, renderer component

**MarkdownEventTransformer**:
The stream processor that enriches standard CommonMark events with publication directives (callouts, equations, cross-references, and figure captions).
_Avoid_: Pandoc preprocessor, regex filter, sentinel replacer

**DocumentEmitter**:
The typesetting adapter seam that translates transpiled Markdown markup into target-specific Typst preambles and layout hierarchies (`ArticleEmitter` for papers, `SlideEmitter` for presentation decks).
_Avoid_: Wrap functions, template renderer, layout builder

**LayoutCoordinator**:
The deep layout engine that manages multi-column flow, title/abstract headers, slide deck segmentation, pause reveals, and title-slide hero formatting (`DocumentLayout` and `SlideDeckLayout`).
_Avoid_: Page splitter, pause processor, string decorator

