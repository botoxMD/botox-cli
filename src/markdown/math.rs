//! LaTeX-to-Typst Math Bridge.
//!
//! Uses AST-based parsing via `mitex` to convert standard LaTeX/KaTeX/AMS-LaTeX
//! mathematical formulas into native Typst math syntax with high fidelity.

/// Preamble definitions injected into Typst documents and slides to provide
/// native compatibility for LaTeX math constructs (delimiters, shims, and macros).
pub const TYPST_MATH_PREAMBLE: &str = r#"
// Botox LaTeX Math Bridge Shims
#let diff = sym.partial
#let operatorname(it) = math.op(math.upright(it))
#let operatornamewithlimits(it) = math.op(limits: true, math.upright(it))
#let negthinspace = h(-(3/18) * 1em)
#let negthinmedspace = h(-(3/18) * 1em)
#let negmedspace = h(-(4/18) * 1em)
#let negthickspace = h(-(5/18) * 1em)
#let enspace = h(0.5em)
#let frac(num, den) = $(num)/(den)$
#let sect = math.inter
#let cfrac(num, den) = $display((num)/(den))$
#let dfrac(num, den) = $display((num)/(den))$
#let tfrac(num, den) = $inline((num)/(den))$
#let dbinom(n, k) = $display(binom(#n, #k))$
#let tbinom(n, k) = $inline(binom(#n, #k))$
#let big(it) = math.lr(size: 1.2em, it)
#let Big(it) = math.lr(size: 1.8em, it)
#let bigg(it) = math.lr(size: 2.4em, it)
#let Bigg(it) = math.lr(size: 3em, it)
#let stackrel(sup, base) = $limits(base)^(sup)$
#let overset(sup, base) = $limits(base)^(sup)$
#let underset(sub, base) = $limits(base)_(sub)$
#let substack(it) = it
#let boxed(it) = box(stroke: 0.5pt, inset: (x: 8pt, y: 6pt), $it$)
#let fbox(it) = box(stroke: 0.5pt, $it$)
#let hbox(it) = it
#let phantom = hide
#let hphantom(it) = box(height: 0pt, hide(it))
#let vphantom(it) = box(width: 0pt, hide(it))
#let smash(it) = box(height: 0pt, align(bottom, $#it$))
#let mathclap(it) = box(width: 0pt, $it$)
#let mathring(it) = math.circle(it)
#let mitexnot(it) = math.cancel(angle: 20deg, it)
#let xcancel = math.cancel
#let bcancel(it) = math.cancel(inverted: true, it)
#let sout(it) = math.cancel(angle: 90deg, it)
#let thickapprox = math.bold(math.approx)
#let thicksim = math.bold(math.tilde)
#let smallint = math.inline(math.integral)
#let pmod(it) = $quad (mod thick it)$
#let pod(it) = $quad (it)$
#let mitexset(it) = $\{it\}$
#let Set(it) = $lr(\{it\})$
#let bra(it) = $chevron.l it|$
#let Bra(it) = $lr(chevron.l it|)$
#let ket(it) = $|it chevron.r$
#let Ket(it) = $lr(|it chevron.r)$
#let braket(it) = $chevron.l it chevron.r$
#let Braket(it) = $lr(chevron.l it chevron.r)$
#let lvert = math.class("opening", "|")
#let rvert = math.class("closing", "|")
#let lVert = math.class("opening", "||")
#let rVert = math.class("closing", "||")
#let varDelta = math.italic(math.Delta)
#let varGamma = math.italic(math.Gamma)
#let varLambda = math.italic(math.Lambda)
#let varOmega = math.italic(math.Omega)
#let varPhi = math.italic(math.Phi)
#let varPi = math.italic(math.Pi)
#let varPsi = math.italic(math.Psi)
#let varSigma = math.italic(math.Sigma)
#let varTheta = math.italic(math.Theta)
#let varUpsilon = math.italic(math.Upsilon)
#let varXi = math.italic(math.Xi)
#let arctg = math.op("arctg")
#let arcctg = math.op("arcctg")
#let ch = math.op("ch")
#let cth = math.op("cth")
#let th = math.op("th")
#let cosec = math.op("cosec")
#let cotg = math.op("cotg")
#let sh = math.op("sh")
#let plim = math.op("plim", limits: true)
#let projlim = math.op("proj\u{2009}lim", limits: true)
#let injlim = math.op("inj\u{2009}lim", limits: true)
#let botox_xarrow(arrow_sym, ..args) = {
  let p = args.pos()
  if p.len() == 1 { $limits(stretch(#arrow_sym)^#p.at(0))$ }
  else if p.len() >= 2 { $limits(stretch(#arrow_sym)^#p.at(1)_#p.at(0))$ }
  else { arrow_sym }
}
#let xleftarrow(..args) = botox_xarrow(math.arrow.l, ..args)
#let xrightarrow(..args) = botox_xarrow(math.arrow.r, ..args)
#let xLeftarrow(..args) = botox_xarrow(math.arrow.l.double, ..args)
#let xRightarrow(..args) = botox_xarrow(math.arrow.r.double, ..args)
#let xleftrightarrow(..args) = botox_xarrow(math.arrow.l.r, ..args)
#let xLeftrightarrow(..args) = botox_xarrow(math.arrow.l.r.double, ..args)
#let xhookleftarrow(..args) = botox_xarrow(math.arrow.l.hook, ..args)
#let xhookrightarrow(..args) = botox_xarrow(math.arrow.r.hook, ..args)
#let xtwoheadleftarrow(..args) = botox_xarrow(math.arrow.l.twohead, ..args)
#let xtwoheadrightarrow(..args) = botox_xarrow(math.arrow.r.twohead, ..args)
#let xleftharpoonup(..args) = botox_xarrow(math.harpoon.lt, ..args)
#let xrightharpoonup(..args) = botox_xarrow(math.harpoon.rt, ..args)
#let xleftharpoondown(..args) = botox_xarrow(math.harpoon.lb, ..args)
#let xrightharpoondown(..args) = botox_xarrow(math.harpoon.rb, ..args)
#let xleftrightharpoons(..args) = botox_xarrow(math.harpoons.ltrb, ..args)
#let xrightleftharpoons(..args) = botox_xarrow(math.harpoons.rtlb, ..args)
#let xtofrom(..args) = botox_xarrow(math.arrows.rl, ..args)
#let xmapsto(..args) = botox_xarrow(math.arrow.r.bar, ..args)
#let xlongequal(..args) = botox_xarrow(math.eq, ..args)
#let underline(..args) = {
  let items = args.pos()
  let joined = ()
  for (i, it) in items.enumerate() {
    if i > 0 { joined.push(math.comma); joined.push([ ]) }
    joined.push(it)
  }
  math.underline(joined.join())
}
#let pmatrix(..args) = math.mat(delim: "(", ..args)
#let bmatrix(..args) = math.mat(delim: "[", ..args)
#let Bmatrix(..args) = math.mat(delim: "{", ..args)
#let vmatrix(..args) = math.mat(delim: "|", ..args)
#let Vmatrix(..args) = math.mat(delim: "\u{2016}", ..args)
#let matrix(..args) = math.mat(delim: none, ..args)
#let smallmatrix(..args) = math.inline(math.mat(delim: none, ..args))
#let smallmat(..args) = math.inline(math.mat(delim: none, ..args))
#let aligned(it) = it
#let alignedat(..args) = args.pos().last()
#let textmath(it) = it
#let mitexdisplay(it) = math.display(it)
#let mitexinline(it) = math.inline(it)
#let mitexscript(it) = math.script(it)
#let mitexsscript(it) = math.sscript(it)
#let mitexmathbf(it) = math.bold(math.upright(it))
#let mitexsqrt(..args) = {
  if args.pos().len() == 1 {
    math.sqrt(args.pos().at(0))
  } else if args.pos().len() == 2 {
    math.root(args.pos().at(0), args.pos().at(1))
  }
}
"#;

/// Normalizes AST output produced by `mitex` into native Typst math functions.
fn normalize_mitex_ast(ast: &mut String) {
    if ast.starts_with("aligned(") && ast.ends_with(')') {
        let inner = &ast[8..ast.len() - 1];
        *ast = inner.trim().to_string();
    }
    *ast = ast
        .replace("angle.l", "chevron.l")
        .replace("angle.r", "chevron.r")
        .replace("planck.reduce", "planck")
        .replace("times.circle.big", "times.o.big")
        .replace("times.circle", "times.o")
        .replace("plus.circle.big", "plus.o.big")
        .replace("plus.circle", "plus.o")
        .replace("dot.circle.big", "dot.o.big")
        .replace("dot.circle", "dot.o")
        .replace("minus.circle", "minus.o")
        .replace("diff ", "partial ")
        .replace("diff^", "partial^")
        .replace("diff_", "partial_")
        .replace("diff)", "partial)")
        .replace("diff,", "partial,")
        .replace("mitexsqrt(", "sqrt(")
        .replace("mitexmathbf(", "bold(")
        .replace("sect ", "inter ")
        .replace("sect)", "inter)")
        .replace("sect,", "inter,")
        .replace("pmatrix(", "mat(delim: \"(\", ")
        .replace("bmatrix(", "mat(delim: \"[\", ")
        .replace("vmatrix(", "mat(delim: \"|\", ")
        .replace("Vmatrix(", "mat(delim: \"\u{2016}\", ")
        .replace("Bmatrix(", "mat(delim: \"{\", ")
        .replace("matrix(", "mat(delim: #none, ");
}

/// Converts single-quoted string literals in LaTeX math expressions (`'John'`, `'Smith'`)
/// to double-quoted string literals (`"John"`, `"Smith"`), while preserving prime notation
/// (e.g. `f'`, `y''`, `(x+y)'`).
fn convert_latex_single_quoted_strings(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        let ch = chars[i];
        if ch == '"' {
            result.push('"');
            i += 1;
            while i < n && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < n {
                    result.push(chars[i]);
                    result.push(chars[i + 1]);
                    i += 2;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            if i < n {
                result.push('"');
                i += 1;
            }
            continue;
        }

        if ch == '\'' {
            let prev_non_ws = (0..i).rev().map(|k| chars[k]).find(|c| !c.is_whitespace());
            let is_prime = match prev_non_ws {
                Some(p) => p.is_ascii_alphanumeric() || p == ')' || p == ']' || p == '}' || p == '\'',
                None => false,
            };

            if is_prime {
                result.push('\'');
                i += 1;
                continue;
            }

            // Find closing quote
            let mut j = i + 1;
            let mut found_close = false;
            while j < n {
                if chars[j] == '\\' {
                    j += 2;
                } else if chars[j] == '\'' {
                    found_close = true;
                    break;
                } else {
                    j += 1;
                }
            }

            if found_close {
                result.push('"');
                let inner: String = chars[i + 1..j].iter().collect();
                result.push_str(&inner);
                result.push('"');
                i = j + 1;
                continue;
            }
        }

        result.push(ch);
        i += 1;
    }

    result
}

fn preprocess_smash(s: &str) -> (String, bool) {
    if !s.contains(r"\smash") {
        return (s.to_string(), false);
    }
    let mut result = String::with_capacity(s.len());
    let mut remainder = s;
    let mut found = false;
    while let Some(pos) = remainder.find(r"\smash") {
        result.push_str(&remainder[..pos]);
        let rest = &remainder[pos + 6..];
        let trimmed = rest.trim_start();
        let after_opt = if trimmed.starts_with('[') {
            if let Some(close) = trimmed.find(']') {
                &trimmed[close + 1..]
            } else {
                rest
            }
        } else {
            rest
        };
        result.push_str(r"\underbar");
        remainder = after_opt;
        found = true;
    }
    result.push_str(remainder);
    (result, found)
}

/// Converts LaTeX math syntax into clean Typst math markup.
///
/// Employs `mitex` AST parsing as the primary compiler bridge, normalizing
/// AST output and applying preamble shims where appropriate.
pub fn convert_latex_math_to_typst(math: &str) -> String {
    let trimmed = math.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if matches!(trimmed, "dt" | "dx" | "dy" | "dz" | "dr" | "du" | "dv") {
        return format!("dif {}", &trimmed[1..]);
    }

    let (with_smash, has_smash) = preprocess_smash(trimmed);
    let preprocessed = convert_latex_single_quoted_strings(&with_smash);

    // Primary: AST-based LaTeX math parsing
    if let Ok(mut ast) = mitex::convert_math(&preprocessed, None) {
        normalize_mitex_ast(&mut ast);
        if has_smash {
            ast = ast.replace("underbar(", "smash(");
        }
        return ast.trim().to_string();
    }

    // Fallback on parser error: pass preprocessed input directly so Typst compiler
    // diagnostics and Botox error mappers pinpoint the exact syntax error.
    preprocessed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_boundary_and_shadowing() {
        assert_eq!(convert_latex_math_to_typst(r"\leftarrow"), "<-");
        assert_eq!(convert_latex_math_to_typst(r"\le"), "<=");
        assert_eq!(convert_latex_math_to_typst(r"\tanh"), "tanh");
        assert_eq!(convert_latex_math_to_typst(r"\tan"), "tan");
        assert_eq!(convert_latex_math_to_typst(r"\sinh"), "sinh");
        assert_eq!(convert_latex_math_to_typst(r"\sin"), "sin");
        assert_eq!(convert_latex_math_to_typst(r"\cosh"), "cosh");
        assert_eq!(convert_latex_math_to_typst(r"\cos"), "cos");
        assert_eq!(convert_latex_math_to_typst(r"\subseteq"), "subset.eq");
        assert_eq!(convert_latex_math_to_typst(r"\subset"), "subset");
        assert_eq!(convert_latex_math_to_typst(r"\cdots"), "dots.h.c");
        assert_eq!(convert_latex_math_to_typst(r"\cdot"), "dot.c");
        assert_eq!(convert_latex_math_to_typst(r"\neq"), "!=");
        assert_eq!(convert_latex_math_to_typst(r"\ne"), "!=");
        assert_eq!(convert_latex_math_to_typst(r"\top"), "top");
    }

    #[test]
    fn test_subscripts_descriptive_names() {
        assert_eq!(convert_latex_math_to_typst(r"x_{total}"), "x _(t o t a l )");
        assert_eq!(convert_latex_math_to_typst(r"a_{11}"), "a _(1 1 )");
        assert_eq!(convert_latex_math_to_typst(r"x_{ij}"), "x _(i j )");
        assert_eq!(convert_latex_math_to_typst(r"V_{out}"), "V _(o u t )");
        assert_eq!(convert_latex_math_to_typst(r"T_{air}"), "T _(a i r )");
        assert_eq!(convert_latex_math_to_typst(r"P_{net}"), "P _(n e t )");
        assert_eq!(convert_latex_math_to_typst(r"y + dx"), "y  +  d x");
        assert_eq!(convert_latex_math_to_typst(r"\int f(x)\, dx"), "integral  f \\(x \\)thin  d x");
    }

    #[test]
    fn test_escaped_braces() {
        assert_eq!(convert_latex_math_to_typst(r"\frac{\{x\}}{y}"), r"frac(\{ x \} ,y )");
    }

    #[test]
    fn test_multi_letter_identifiers_split() {
        assert_eq!(
            convert_latex_math_to_typst(r"\mathbf{Relation}(\underline{pk_1}, \dots, attr, fk, \dots)"),
            "bold(R e l a t i o n )\\(underline(p k _(1 ))\\, dots.h \\, a t t r \\, f k \\, dots.h \\)"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"A(\underline{a_1}, a_2, b_1, c_1, type)"),
            "A \\(underline(a _(1 ))\\, a _(2 )\\, b _(1 )\\, c _(1 )\\, t y p e \\)"
        );
        assert_eq!(convert_latex_math_to_typst("pk_1"), "p k _(1 )");
        assert_eq!(convert_latex_math_to_typst("E = mc^2"), "E  =  m c ^(2 )");
        assert_eq!(convert_latex_math_to_typst(r"\mathit{pk_1}"), "italic(p k _(1 ))");
    }

    #[test]
    fn test_feedback_report_math_fixes() {
        assert_eq!(
            convert_latex_math_to_typst(r"\frac{\text{observé}}{\text{attendu}}"),
            "frac(#textmath[observé];,#textmath[attendu];)"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"1\,000\,000"),
            "1 thin 0 0 0 thin 0 0 0"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"\approx 20 \text{ to } 100\ \mu\text{s}"),
            "approx  2 0  #textmath[ to ]; 1 0 0 thick  mu #textmath[s];"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"\lfloor N / K \rfloor"),
            "⌊  N  \\/ K  ⌋"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"\lceil N / K \rceil"),
            "⌈  N  \\/ K  ⌉"
        );
        assert_eq!(convert_latex_math_to_typst(r"N \bmod K"), "N  mod  K");
        assert_eq!(
            convert_latex_math_to_typst(r"K = \min(\text{NB\_THREADS}, N)"),
            "K  =  min \\(#textmath[NB\\_ THREADS];\\, N \\)"
        );
    }

    #[test]
    fn test_relational_algebra_math() {
        assert_eq!(
            convert_latex_math_to_typst(r"\underline{dnumber, dlocation}"),
            "underline(d n u m b e r \\, d l o c a t i o n )"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"\boxed{R \leftarrow ssn\_dept5 \cup superviseurs}"),
            "boxed(R  <-  s s n \\_ d e p t 5  union  s u p e r v i s e u r s )"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"\sigma_{fname='John' \;\wedge\; lname='Smith'}(employee)"),
            "sigma _(f n a m e = \\\"J o h n \\\" thick and thick  l n a m e = \\\"S m i t h \\\")\\(e m p l o y e e \\)"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"\sigma_{dno=5}(employee)"),
            "sigma _(d n o = 5 )\\(e m p l o y e e \\)"
        );
        assert_eq!(
            convert_latex_math_to_typst(r"\pi_{essn, pno}(works\_on)"),
            "pi _(e s s n \\, p n o )\\(w o r k s \\_ o n \\)"
        );
    }
}
