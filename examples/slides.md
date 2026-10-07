---
marp: true
title: "Next-Generation Systems with Botox"
author: "Systems Architecture Group"
theme: gaia
paginate: true
---

# Next-Generation Systems
### High-Performance Scientific Presentations & Typesetting
**Presenter:** Systems Engineering Research Lab  
**Date:** \today

---

# Typography & Text Formatting

Botox provides full Markdown and LaTeX styling for presentations:

- **Emphasis**: Use **bold**, *italics*, or ***bold-italics*** for key terms.
- **Formulas & Chemistry**: Water H~2~O, energy $E = m c^2$, and powers 2^10^.
- **Revisions**: Flag deprecated items with ~~strikethrough~~.
- **Commands**: Run inline code with `botox slides.md -o slides.pdf`.
- **Workflow**:
  1. Initialize workspace with `botox init --slides`
  2. Live preview with synchronized bidirectional cursor tracking

---

# Callout Boxes & Admonitions

Visual callouts highlight critical insights and best practices:

> [!NOTE]
> Botox compiles Markdown to vector Typst slides in milliseconds with zero external toolchain dependencies.

> [!TIP]
> Switch themes instantly between `default`, `academic`, `nord`, and `dark` via the frontmatter `theme` property.

---

# Mathematical Formulations

Native equations render with Computer Modern vector precision:

Scaled Dot-Product Attention maps queries $Q$ and keys $K$ to values $V$:

$$ \text{Attn}(Q, K, V) = \text{softmax}\left(\frac{Q K^T}{\sqrt{d_k}}\right) V $$

- Dimension scaling $\frac{1}{\sqrt{d_k}}$ avoids vanishing gradients in large models
- Fully vector-rendered with zero pixelation at any display scale

---

# Matrix Models & Linear Algebra

High-dimensional state representations and parameter covariance:

$$ \mathbf{\Sigma} = \begin{bmatrix} \sigma_x^2 & \rho_{x y} & 0 \\ \rho_{y x} & \sigma_y^2 & 0 \\ 0 & 0 & \sigma_z^2 \end{bmatrix}, \quad \nabla_{\mathbf{\theta}} \mathcal{L}(\mathbf{\theta}) \in \mathbb{R}^d $$

- Seamless LaTeX matrix syntax with `\begin{bmatrix} ... \end{bmatrix}`
- Complete support for Greek symbols, subscripts, superscripts, and norms

---

# Code Blocks & Syntax Highlighting

Vector monospace rendering with automated contrast and framing:

```rust
use botox::compiler::compile_typst;

// High-speed slide compilation directly to vector PDF
pub fn build_deck(src: &str, out: &std::path::Path) -> Result<(), String> {
    compile_typst(src, out, None)
}
```

---

# Quantitative Performance

Compilation speed and resource efficiency comparisons:

| Pipeline Engine | Compilation | Memory | Output Format | Math Quality |
| :--- | :---: | :---: | :---: | :---: |
| **Botox (Native)** | **18 ms** | **~24 MB** | **PDF & SVG** | **Exact Vector** |
| Pandoc + LaTeX | 1,850 ms | ~320 MB | PDF only | Native TeX |
| Marp CLI + Chrome | 2,400 ms | ~450 MB | HTML / PDF | Rasterized |
| Quarto + Typst | 420 ms | ~110 MB | PDF / HTML | Native Typst |

---

# Interactive Citations & References

Hyperlinks automatically transform into IEEE-formatted citations:

- Scalable transformer models introduced by [Vaswani et al.](https://arxiv.org/abs/1706.03762).
- Deep residual learning architectures evaluated in [He et al.](https://arxiv.org/abs/1512.03385).
- Sparse mixture-of-experts routing detailed in [Fedus et al.](https://arxiv.org/abs/2101.03961).
- Native layout compiler design inspired by [Typst Project](https://typst.app).

All citation links remain clickable in exported PDF and SVG presentations.

---

# Summary & Deployment

### Key Takeaways

- **Unified Format**: Share syntax between academic papers and slide decks.
\pause
- **Sub-Second Feedback**: Instant live reload during writing and editing.
\pause
- **Publish Anywhere**: One-command export to PDF, SVG, and standalone HTML.
\pause

> [!IMPORTANT]
> Try other built-in slide themes by changing `theme: nord` or `theme: dark` in the frontmatter!
