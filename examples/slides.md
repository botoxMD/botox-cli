---
author: Minus
marp: true
theme: gaia
_class: lead
paginate: true
backgroundColor: #f8fafc
color: #0f172a
style: |
  section {
    font-family: 'Liberation Sans', -apple-system, sans-serif;
  }
  h1 {
    color: #1e293b;
  }
---

# Autonomous Document Toolchain
### Transforming Markdown into LaTeX-Quality PDFs & Slides

---

# Design Philosophy

- **Slides**: Divided cleanly by `---`, first section configures style and theme.
- **Documents**: LaTeX-grade typesetting with Computer Modern fonts and mathematical precision.
- **Zero Friction**: No manual table of contents needed.
- **Configurable Defaults**: Set author, fonts, and styles globally or per project.

---

# Slide Architecture

- Use standard Markdown formatting for content
- Slides are separated by standard `---` horizontal rules
- Interactive HTML output includes presenter notes (`P`) and overview grid (`O`)
- Native PDF slide export powered by headless Chrome

---

# Code Snippet

```bash
# Compile documentation
botox paper.md -o paper.pdf

# Compile slide presentation
botox presentation.md -o presentation.pdf
```

---

# Summary

- Fast compilation in milliseconds
- Real LaTeX typography using Typst under the hood
- Built for engineers and researchers
