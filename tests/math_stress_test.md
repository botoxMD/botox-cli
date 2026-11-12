---
title: Comprehensive LaTeX Math Stress Test
subtitle: Botox Math Engine Robustness and Fidelity Suite
author: Botox QA & Benchmarking Team
date: 2026-11-12
papersize: a4
theme: elegant
toc: true
toc-depth: 3
number-sections: true
---

# Introduction and Scope {#sec:intro}

This document serves as an exhaustive stress-testing benchmark for the LaTeX-to-Typst mathematical bridge within **Botox**. It exercises parsing fidelity, syntax transformations, layout alignment, matrix representations, spacing heuristics, and edge cases across mathematics, physics, engineering, and computer science.

Inline equations like $e^{i\pi} + 1 = 0$, $\lim_{x \to 0} \frac{\sin x}{x} = 1$, and $\nabla \cdot \mathbf{B} = 0$ must render cleanly within flowing prose without corrupting paragraph geometry or baseline metrics.

# Advanced Calculus & Differential Geometry {#sec:calculus}

## Partial Derivatives and Multivariable Differential Operators

Standard partial derivatives, higher-order mixed derivatives, and vector differential operators:

$$
\frac{\partial f}{\partial x}, \quad \frac{\partial^2 \psi}{\partial x^2} + \frac{\partial^2 \psi}{\partial y^2} + \frac{\partial^2 \psi}{\partial z^2} = \nabla^2 \psi = \Delta \psi
$$ {#eq:laplace}

The gradient, divergence, and curl operators acting on scalar fields $\phi(x,y,z)$ and vector fields $\mathbf{F} = \langle F_x, F_y, F_z \rangle$:

$$
\operatorname{grad} \phi = \nabla \phi = \frac{\partial \phi}{\partial x}\mathbf{i} + \frac{\partial \phi}{\partial y}\mathbf{j} + \frac{\partial \phi}{\partial z}\mathbf{k}
$$ {#eq:grad}

$$
\operatorname{div} \mathbf{F} = \nabla \cdot \mathbf{F} = \frac{\partial F_x}{\partial x} + \frac{\partial F_y}{\partial y} + \frac{\partial F_z}{\partial z}
$$ {#eq:div}

$$
\operatorname{curl} \mathbf{F} = \nabla \times \mathbf{F} = \begin{vmatrix} \mathbf{i} & \mathbf{j} & \mathbf{k} \\ \frac{\partial}{\partial x} & \frac{\partial}{\partial y} & \frac{\partial}{\partial z} \\ F_x & F_y & F_z \end{vmatrix}
$$ {#eq:curl}

## Line, Surface, and Volume Integrals

Fundamental integral theorems of multivariable calculus:

### Green's Theorem

Let $C$ be a positively oriented, piecewise smooth, simple closed curve in a plane bounding region $D$:

$$
\oint_C (L \, dx + M \, dy) = \iint_D \left( \frac{\partial M}{\partial x} - \frac{\partial L}{\partial y} \right) dA
$$ {#eq:green}

### Stokes' Theorem

Let $\Sigma$ be a smooth oriented surface in $\mathbb{R}^3$ with boundary $\partial \Sigma$:

$$
\oint_{\partial \Sigma} \mathbf{A} \cdot d\mathbf{r} = \iint_{\Sigma} (\nabla \times \mathbf{A}) \cdot d\mathbf{S}
$$ {#eq:stokes}

### Gauss's (Divergence) Theorem

For a compact volume $V \subset \mathbb{R}^3$ with closed boundary $\partial V$:

$$
\oiint_{\partial V} \mathbf{E} \cdot d\mathbf{S} = \iiint_V (\nabla \cdot \mathbf{E}) \, dV
$$ {#eq:gauss}

## Differential Geometry and Curvature

The metric tensor $g_{\mu\nu}$, Christoffel symbols of the second kind, and the Riemann curvature tensor:

$$
\Gamma^\mu_{\alpha\beta} = \frac{1}{2} g^{\mu\nu} \left( \frac{\partial g_{\beta\nu}}{\partial x^\alpha} + \frac{\partial g_{\alpha\nu}}{\partial x^\beta} - \frac{\partial g_{\alpha\beta}}{\partial x^\nu} \right)
$$ {#eq:christoffel}

$$
R^\rho_{\ \sigma\mu\nu} = \partial_\mu \Gamma^\rho_{\nu\sigma} - \partial_\nu \Gamma^\rho_{\mu\sigma} + \Gamma^\rho_{\mu\lambda} \Gamma^\lambda_{\nu\sigma} - \Gamma^\rho_{\nu\lambda} \Gamma^\lambda_{\mu\sigma}
$$ {#eq:riemann}

Contraction yields the Ricci tensor $R_{\sigma\nu} = R^\mu_{\ \sigma\mu\nu}$ and scalar curvature $R = g^{\sigma\nu} R_{\sigma\nu}$.

# Linear Algebra & Quantum Mechanics {#sec:linalg}

## Dirac Bra-Ket Notation and State Vectors

In a Hilbert space $\mathcal{H}$, state vectors $|\psi\rangle$, dual bras $\langle \phi|$, inner products $\langle \phi | \psi \rangle$, and transition amplitudes:

$$
\langle \phi | \psi \rangle = \int_{-\infty}^{\infty} \phi^*(x) \psi(x) \, dx
$$ {#eq:braket-inner}

Outer products, completeness relations, and expectation values:

$$
\hat{\mathbb{I}} = \sum_n |n\rangle \langle n|, \qquad \langle \hat{A} \rangle = \langle \psi | \hat{A} | \psi \rangle = \operatorname{Tr}(\hat{\rho} \hat{A})
$$ {#eq:completeness}

Density matrix representation:

$$
\hat{\rho} = \sum_i p_i |\psi_i\rangle \langle \psi_i|, \quad \text{with } p_i \ge 0 \text{ and } \sum_i p_i = 1
$$ {#eq:density-matrix}

## Hermitian Operators, Conjugates, and Commutators

Hermitian conjugate (dagger $\dagger$), unitary transformations $U^\dagger U = \mathbb{I}$, and the canonical commutation relation:

$$
[\hat{x}, \hat{p}] = \hat{x}\hat{p} - \hat{p}\hat{x} = i\hbar \hat{\mathbb{I}}
$$ {#eq:canonical-comm}

Anti-commutator relations for fermionic operators $a_i, a_j^\dagger$:

$$
\{a_i, a_j^\dagger\} = a_i a_j^\dagger + a_j^\dagger a_i = \delta_{ij} \hat{\mathbb{I}}, \qquad \{a_i, a_j\} = \{a_i^\dagger, a_j^\dagger\} = 0
$$ {#eq:fermion-anticomm}

Tensor product spaces $\mathcal{H}_A \otimes \mathcal{H}_B$:

$$
|\Psi_{AB}\rangle = \frac{1}{\sqrt{2}} \left( |0\rangle_A \otimes |1\rangle_B - |1\rangle_A \otimes |0\rangle_B \right)
$$ {#eq:bell-state}

## Diverse Matrix Environments and Delimiters

Testing various matrix delimiters: `pmatrix`, `bmatrix`, `vmatrix`, `Vmatrix`, `Bmatrix`, unadorned `matrix`, and inlined `smallmatrix`.

### Pauli Spin Matrices (`pmatrix`) and Inlined `smallmatrix`

$$
\sigma_x = \begin{pmatrix} 0 & 1 \\ 1 & 0 \end{pmatrix}, \quad
\sigma_y = \begin{pmatrix} 0 & -i \\ i & 0 \end{pmatrix}, \quad
\sigma_z = \begin{pmatrix} 1 & 0 \\ 0 & -1 \end{pmatrix}
$$ {#eq:pauli}

Inline compact matrix using `smallmatrix`: $\left( \begin{smallmatrix} a & b \\ c & d \end{smallmatrix} \right)$ embedded in text.

### Bracket Matrix (`bmatrix`) and Determinant (`vmatrix`)

$$
\mathbf{A} = \begin{bmatrix}
a_{11} & a_{12} & a_{13} \\
a_{21} & a_{22} & a_{23} \\
a_{31} & a_{32} & a_{33}
\end{bmatrix}, \qquad
\det(\mathbf{A}) = \begin{vmatrix}
a_{11} & a_{12} & a_{13} \\
a_{21} & a_{22} & a_{23} \\
a_{31} & a_{32} & a_{33}
\end{vmatrix}
$$ {#eq:matrices-3x3}

### Norm Matrix (`Vmatrix`) and Braced Matrix (`Bmatrix`)

$$
\|\mathbf{M}\| = \begin{Vmatrix}
m_{11} & m_{12} \\
m_{21} & m_{22}
\end{Vmatrix}, \qquad
\mathbf{B} = \begin{Bmatrix}
\alpha & \beta \\
\gamma & \delta
\end{Bmatrix}
$$ {#eq:vmatrix-bmatrix}

### Large Sparse Block Matrix with Ellipses

$$
\mathbf{J}_n(\lambda) = \begin{bmatrix}
\lambda & 1 & 0 & \cdots & 0 \\
0 & \lambda & 1 & \cdots & 0 \\
0 & 0 & \lambda & \ddots & \vdots \\
\vdots & \vdots & \ddots & \ddots & 1 \\
0 & 0 & \cdots & 0 & \lambda
\end{bmatrix} \in \mathbb{C}^{n \times n}
$$ {#eq:jordan-block}

# Physics & Engineering Models {#sec:physics}

## Maxwell's Equations of Electrodynamics

In differential and integral representations:

### Differential Form

$$
\begin{aligned}
\nabla \cdot \mathbf{E} &= \frac{\rho}{\varepsilon_0} && \text{(Gauss's Law)} \\
\nabla \cdot \mathbf{B} &= 0 && \text{(Gauss's Law for Magnetism)} \\
\nabla \times \mathbf{E} &= -\frac{\partial \mathbf{B}}{\partial t} && \text{(Faraday's Law of Induction)} \\
\nabla \times \mathbf{B} &= \mu_0 \mathbf{J} + \mu_0 \varepsilon_0 \frac{\partial \mathbf{E}}{\partial t} && \text{(Ampère-Maxwell Law)}
\end{aligned}
$$ {#eq:maxwell}

### Electromagnetic Stress-Energy Tensor

$$
T^{\mu\nu} = \frac{1}{\mu_0} \left[ F^{\mu\alpha} F^\nu_{\ \alpha} - \frac{1}{4}\eta^{\mu\nu} F_{\alpha\beta}F^{\alpha\beta} \right]
$$ {#eq:em-tensor}

## Quantum Mechanics: The Schrödinger Equation

Time-dependent Schrödinger equation in a three-dimensional potential $V(\mathbf{r}, t)$:

$$
i\hbar \frac{\partial}{\partial t}\Psi(\mathbf{r}, t) = \left[ -\frac{\hbar^2}{2m}\nabla^2 + V(\mathbf{r}, t) \right] \Psi(\mathbf{r}, t)
$$ {#eq:schroedinger}

Separation of variables yields the stationary eigenvalue equation:

$$
\hat{H}\psi_n(\mathbf{r}) = E_n \psi_n(\mathbf{r}), \quad \text{where } \hat{H} = -\frac{\hbar^2}{2m}\nabla^2 + V(\mathbf{r})
$$ {#eq:time-indep-schroedinger}

## General Relativity: Einstein Field Equations

The gravitational field equations with cosmological constant $\Lambda$:

$$
G_{\mu\nu} + \Lambda g_{\mu\nu} = \frac{8\pi G}{c^4} T_{\mu\nu}
$$ {#eq:einstein-field}

where the Einstein tensor $G_{\mu\nu} = R_{\mu\nu} - \frac{1}{2} R g_{\mu\nu}$.

## Analytical Mechanics: Lagrangian & Hamiltonian Formalisms

Lagrange's equations of motion for generalized coordinates $q_i$ and velocities $\dot{q}_i$:

$$
\frac{d}{dt}\left( \frac{\partial L}{\partial \dot{q}_i} \right) - \frac{\partial L}{\partial q_i} = 0, \quad \text{with } L(q_i, \dot{q}_i, t) = T - V
$$ {#eq:euler-lagrange}

The Legendre transform defines the Hamiltonian $H(q_i, p_i, t) = \sum_i p_i \dot{q}_i - L$, leading to Hamilton's canonical equations:

$$
\dot{q}_i = \frac{\partial H}{\partial p_i}, \qquad \dot{p}_i = -\frac{\partial H}{\partial q_i}
$$ {#eq:hamilton-canonical}

## Thermodynamics & Thermodynamic Potentials

First and second laws in fundamental differential form:

$$
\mathrm{d}U = T\,\mathrm{d}S - P\,\mathrm{d}V + \sum_{i=1}^k \mu_i\,\mathrm{d}N_i
$$ {#eq:thermo-fundamental}

The enthalpy $H = U + PV$, Helmholtz free energy $F = U - TS$, and Gibbs free energy $G = H - TS$ yield Maxwell relations:

$$
\left(\frac{\partial T}{\partial V}\right)_S = -\left(\frac{\partial P}{\partial S}\right)_V, \qquad
\left(\frac{\partial T}{\partial P}\right)_S = \left(\frac{\partial V}{\partial S}\right)_P
$$ {#eq:maxwell-relations}

# Probability, Statistics, & Measure Theory {#sec:prob}

## Bayes' Theorem and Conditional Probability

$$
P(A \mid B) = \frac{P(B \mid A)\, P(A)}{P(B)} = \frac{P(B \mid A)\, P(A)}{\sum_k P(B \mid A_k)\, P(A_k)}
$$ {#eq:bayes}

Continuous probability density and conditional expectation:

$$
\mathbb{E}[X \mid Y = y] = \int_{-\infty}^{\infty} x \, f_{X|Y}(x \mid y) \, dx
$$ {#eq:conditional-expectation}

## Moments, Variances, and Covariance

Expected value $\mathbb{E}[X]$, variance $\operatorname{Var}(X)$, and covariance matrix $\mathbf{\Sigma}$:

$$
\operatorname{Var}(X) = \mathbb{E}\left[ (X - \mathbb{E}[X])^2 \right] = \mathbb{E}[X^2] - (\mathbb{E}[X])^2
$$ {#eq:variance}

$$
\operatorname{Cov}(X, Y) = \mathbb{E}[(X - \mathbb{E}[X])(Y - \mathbb{E}[Y])] = \mathbb{E}[XY] - \mathbb{E}[X]\mathbb{E}[Y]
$$ {#eq:covariance}

## Multivariate Gaussian Distribution

For a random vector $\mathbf{X} = (X_1, \dots, X_p)^T \sim \mathcal{N}_p(\boldsymbol{\mu}, \mathbf{\Sigma})$:

$$
f_{\mathbf{X}}(\mathbf{x}) = \frac{1}{(2\pi)^{p/2} |\mathbf{\Sigma}|^{1/2}} \exp\left( -\frac{1}{2} (\mathbf{x} - \boldsymbol{\mu})^T \mathbf{\Sigma}^{-1} (\mathbf{x} - \boldsymbol{\mu}) \right)
$$ {#eq:multivariate-normal}

## Asymptotic Theorems and Stochastic Convergence

The Strong Law of Large Numbers (SLLN) and Central Limit Theorem (CLT):

$$
P\left( \lim_{n \to \infty} \frac{1}{n}\sum_{i=1}^n X_i = \mu \right) = 1, \qquad
\sqrt{n} \left( \frac{\bar{X}_n - \mu}{\sigma} \right) \xrightarrow{\mathcal{D}} \mathcal{N}(0, 1)
$$ {#eq:clt}

Infimum, supremum, and limit superior / inferior:

$$
\limsup_{n \to \infty} A_n = \bigcap_{k=1}^\infty \bigcup_{n=k}^\infty A_n, \qquad
\liminf_{n \to \infty} A_n = \bigcup_{k=1}^\infty \bigcap_{n=k}^\infty A_n
$$ {#eq:limsup-liminf}

$$
\sup_{x \in D} f(x) \ge \inf_{x \in D} f(x)
$$ {#eq:sup-inf}

# AMS-LaTeX Constructs & Complex Formatting {#sec:ams}

## Alignment and Gathering Environments

Testing multi-line equations with `aligned` and `gathered`:

$$
\begin{aligned}
(x + y)^3 &= (x + y)(x + y)^2 \\
          &= (x + y)(x^2 + 2xy + y^2) \\
          &= x^3 + 3x^2y + 3xy^2 + y^3
\end{aligned}
$$ {#eq:aligned-cube}

And the `gathered` environment for grouped, centered statements:

$$
\begin{gathered}
2x + y = 10 \\
x - 3y = -2 \\
5x + 2y = 23
\end{gathered}
$$ {#eq:gathered-sys}

## Piecewise Cases and Substack

Piecewise function definition with `cases`:

$$
\operatorname{sgn}(x) = \begin{cases}
1 & \text{if } x > 0, \\
0 & \text{if } x = 0, \\
-1 & \text{if } x < 0.
\end{cases}
$$ {#eq:sgn-cases}

Complex summations using `\substack`:

$$
\sum_{\substack{0 \le i \le m \\ 0 \le j \le n}} a_{ij} x^i y^j \le \left( \sum_{i=0}^m |a_i| \right) \left( \sum_{j=0}^n |b_j| \right)
$$ {#eq:substack-sum}

## Modular Arithmetic: `\pmod`, `\pod`, and `\bmod`

$$
x \equiv y \pmod{m}, \qquad a \equiv b \pod{n^2 - 1}, \qquad r = a \bmod b
$$ {#eq:modular}

## Binomial Coefficients: `\binom`, `\dbinom`, `\tbinom`

$$
\binom{n}{k} = \frac{n!}{k!(n-k)!}, \qquad
\dbinom{n}{k} = \frac{n \times (n-1) \cdots (n-k+1)}{k!}, \qquad
\sum_{k=0}^n \tbinom{n}{k} = 2^n
$$ {#eq:binomials}

## Stacking and Relational Oversets: `\overset`, `\underset`, `\stackrel`

$$
X_n \overset{p}{\longrightarrow} X, \qquad
A \underset{n \to \infty}{\longrightarrow} B, \qquad
f(x) \stackrel{\text{def}}{=} \int_0^x \frac{\sin t}{t} \, dt
$$ {#eq:stacking}

## Continued Fractions: `\cfrac`, `\dfrac`, `\frac`

The golden ratio $\phi$ as an infinite continued fraction:

$$
\phi = 1 + \cfrac{1}{1 + \cfrac{1}{1 + \cfrac{1}{1 + \cfrac{1}{1 + \ddots}}}}
$$ {#eq:golden-ratio}

And Ramanujan's continued fraction:

$$
\frac{1}{1 + \cfrac{e^{-2\pi}}{1 + \cfrac{e^{-4\pi}}{1 + \cfrac{e^{-6\pi}}{\ddots}}}} = \left( \sqrt{\frac{5+\sqrt{5}}{2}} - \frac{\sqrt{5}+1}{2} \right) e^{2\pi/5}
$$ {#eq:ramanujan}

## Text Inside Math with Unicode and Accents

Accented characters, diacritics, and symbols inside `\text{...}`:

$$
\text{énergie cinétique} = \frac{1}{2} m v^2, \qquad
F_{\text{frottement}} = -\mu N, \qquad
\Delta_{\text{système}} = \sum_{k=1}^K \text{coût}_k
$$ {#eq:text-accents}

$$
\mathbf{E}_{\text{D'Alembert}} = \left\{ \mathbf{x} \in \mathbb{R}^3 \mid \text{d'Alembertien } \Box \psi = 0 \right\}
$$ {#eq:text-dalembert}

## Emphasized Equations: `\boxed` and `\underline`

Boxed equations for highlighting key theorems:

$$
\boxed{e^{i\theta} = \cos\theta + i\sin\theta}
$$ {#eq:euler-boxed}

Underlined terms in equations and schemas:

$$
\underline{x^2 + y^2} = r^2, \qquad \mathbf{Schema}(\underline{\text{user\_id}}, \text{email}, \text{created\_at})
$$ {#eq:underlined}

# Spacing, Kerning, & Phantoms {#sec:spacing}

## Spacing and Kerning Operators

LaTeX provides fine-grained horizontal spacing operators:
negative thin space `\!`, thin space `\,`, medium space `\:`, thick space `\;`, unescaped/escaped space `\ `, `\quad`, and `\qquad`:

$$
\Gamma \!\! \int \! f(x) \, dx \; \: \text{and} \quad a \qquad b
$$ {#eq:kerning}

Contrasting spaced versus unspaced differentials:

$$
\iint f(x,y)\,dx\,dy \quad \text{vs} \quad \iint f(x,y) dx dy
$$ {#eq:differential-spacing}

## Phantom Expressions and Smash

Testing `\phantom`, `\hphantom`, `\vphantom`, and `\smash`:

$$
{}^{14}_{\phantom{1}6}\text{C} \quad \text{vs} \quad {}^{14}_{6}\text{C}
$$ {#eq:carbon-isotope}

$$
A = \sqrt{\vphantom{b^2} a} + \sqrt{b^2 + c^2}
$$ {#eq:vphantom-root}

$$
\smash{\sum_{i=1}^n} x_i + \hphantom{xyz} y
$$ {#eq:smash-hphantom}

# Mathematical Logic, Set Theory, & Relational Algebra {#sec:logic}

## First-Order Logic and Quantifiers

$$
\forall \epsilon > 0, \; \exists \delta > 0, \; \forall x \in D : \quad 0 < |x - c| < \delta \implies |f(x) - L| < \epsilon
$$ {#eq:cauchy-limit}

Material implication, equivalence, conjunction, disjunction, negation, and arrows (`\longleftarrow`, `\longrightarrow`):

$$
(P \implies Q) \iff (\neg P \vee Q), \qquad \neg (A \wedge B) \iff (\neg A \vee \neg B)
$$ {#eq:de-morgan}

Sequents, turnstiles, and arrows:

$$
A \longleftarrow B \longrightarrow C, \qquad \Gamma \vdash \phi \implies \Gamma \vDash \phi
$$ {#eq:soundness}

## Set Theory and Number Systems

Set relations, set operations, and cardinalities:

$$
\mathbb{N} \subset \mathbb{Z} \subset \mathbb{Q} \subset \mathbb{R} \subset \mathbb{C}
$$ {#eq:number-subsets}

$$
A \cup (B \cap C) = (A \cup B) \cap (A \cup C), \qquad A \setminus B = A \cap B^c
$$ {#eq:set-distributivity}

$$
\emptyset \subseteq S \subsetneq X, \quad x \in S, \quad y \notin S, \quad \mathcal{P}(S) = \{ T \mid T \subseteq S \}
$$ {#eq:power-set}

## Relational Algebra Operators

Database query optimization and relational calculus primitives:

$$
\sigma_{\text{age} \ge 21 \;\wedge\; \text{status} = \text{'active'}}(\text{Users})
$$ {#eq:rel-select}

$$
\pi_{\text{id}, \text{name}, \text{salary}}(\sigma_{\text{dept} = 5}(\text{Employees}))
$$ {#eq:rel-project}

Natural join $\bowtie$ and theta join $\bowtie_\theta$:

$$
R = \text{Orders} \bowtie_{\text{Orders.cust\_id} = \text{Customers.id}} \text{Customers}
$$ {#eq:rel-join}

Renaming operator $\rho$ and set operations:

$$
\rho_{S(A_1, A_2, \dots, A_n)}(R) \cup \pi_{a, b}(T) \bowtie U
$$ {#eq:rel-rename}

# Extreme Edge Cases & Torture Tests {#sec:torture}

## Deeply Nested Expressions

Deep nesting of fractions, square roots, and parentheses:

$$
\sqrt{1 + \sqrt{2 + \sqrt{3 + \sqrt{4 + \dots}}}}
$$ {#eq:nested-roots}

$$
f(x) = \cfrac{1}{1 + \cfrac{2}{2 + \cfrac{3}{3 + \cfrac{4}{4 + \cfrac{5}{5 + x}}}}}
$$ {#eq:nested-fracs}

## Mixed Multi-Letter Subscripts and Superscripts

Identifiers with multiple letters, subscripts, prime marks, and underscores:

$$
V_{\text{out}}^{(k)} = \alpha_{\text{damping}} \cdot V_{\text{in}}' + \beta_{\text{leak}} \cdot (V_{\text{target}}'')^2
$$ {#eq:sub-super-complex}

$$
\operatorname{Loss}_{\text{total}}(\theta) = \frac{1}{N}\sum_{i=1}^N \mathcal{L}_{\text{BCE}}(y_i, \hat{y}_i) + \lambda \|\mathbf{W}_1\|_F^2
$$ {#eq:loss-func}

## Delimiter Sizing Torture

Automated and explicit delimiter sizing (`\left( \dots \right)`, `\big`, `\Big`, `\bigg`, `\Bigg`):

$$
\Bigg( \bigg[ \Big\{ \big| x \big| \Big\} \bigg] \Bigg)
$$ {#eq:delimiter-sizes}

$$
\left( \frac{\displaystyle \sum_{k=1}^n k^3}{\displaystyle \prod_{j=1}^m (j^2 + 1)} \right)^{\frac{1}{2}}
$$ {#eq:left-right-huge}

# Conclusion {#sec:conclusion}

This completes the mathematical stress test for Botox. All sections verify compliance across AMS-LaTeX syntax, multi-line matrices, spacing shims, physics formalisms, and database relational algebra.
