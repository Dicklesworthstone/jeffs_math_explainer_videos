# Research Charter: Differentiable Mathematical Animation in FrankenManim

> **Document Class:** Exploratory Research Charter (§20.3 G5 criterion 6)  
> **Status:** Active Research Track (Non-Product-Gated)  
> **Constitutional Constraints:** D-01..D-25, `#![forbid(unsafe_code)]`, zero external tools beyond ffmpeg  
> **Bead:** `fm-exploratory-tier-charter-711n`

---

## 1. Motivation & Context

In classical computer animation systems (including Grant Sanderson's 3Blue1Brown `manim`), animation parameters—keyframes, coordinate paths, morph interpolation rates, opacity fades, and camera orientations—are specified either manually or procedurally. When tuning a complex mathematical visual (e.g. finding a camera orbit that avoids occluding critical axes while minimizing angular jerk, or aligning a geometric contour to fit empirical data), the author performs iterative manual trial-and-error.

**Differentiable animation** reframes animation synthesis and refinement as mathematical optimization:
$$\min_{\boldsymbol{\theta}} \mathcal{L}(\operatorname{Scene}(\boldsymbol{\theta}), \mathcal{Y}^\star) + \mathcal{R}(\boldsymbol{\theta})$$
where:
- $\boldsymbol{\theta}$ represents continuous scene parameters (geometry control points, transformation matrices, camera trajectories, color palettes, time warp rate functions).
- $\operatorname{Scene}(\boldsymbol{\theta})$ produces a rasterized video frame or sequence of frames.
- $\mathcal{L}$ is an objective function (e.g. geometric alignment loss, perceptual distance, contrast preservation, trajectory smoothness).
- $\mathcal{R}$ is a regularizer enforcing physical or kinematic smoothness.

To solve this via gradient-based optimization (e.g. Adam, L-BFGS), we require tractably computable gradients:
$$\nabla_{\boldsymbol{\theta}} \mathcal{L} = \frac{\partial \mathcal{L}}{\partial \operatorname{Frame}} \cdot \frac{\partial \operatorname{Frame}}{\partial \boldsymbol{\theta}}$$

FrankenManim's sovereign architecture offers a unique advantage: unlike traditional triangle-mesh rasterizers that rely on heuristic soft-rasterization (e.g. SoftRas, PyTorch3D) or Monte Carlo edge sampling (e.g. Mitsuba 3) to smooth over non-differentiable step boundaries, **Lumen (§10.2) evaluates vector coverage analytically on quadratic Bézier curves** via Green's theorem ($\int x\,dy$). This research charter defines the boundaries, theoretical framework, and milestones for differentiable animation within the FrankenSuite.

---

## 2. Mathematical Foundation: Differentiating Analytic Coverage

### 2.1 Forward Formulation (Lumen §10.2)
In FrankenManim's Lumen renderer, every planar vector fill is defined by a closed 2D path $\partial \Omega$ composed of quadratic Bézier segments $\mathbf{B}_k(t) = (x_k(t), y_k(t))$ for $t \in [0, 1]$.

By Green's theorem in the plane, the total area $A$ of region $\Omega$ is evaluated as a closed line integral:
$$A(\boldsymbol{\theta}) = \iint_{\Omega(\boldsymbol{\theta})} dx\,dy = \oint_{\partial \Omega(\boldsymbol{\theta})} x\,dy = \sum_{k} \int_0^1 x_k(t; \boldsymbol{\theta})\,y'_k(t; \boldsymbol{\theta})\,dt$$

On a discrete raster lattice, Lumen evaluates cell coverage $C_{i,j} \in [0, 1]$ over each pixel cell $\mathcal{P}_{i,j} = [x_i, x_{i+1}] \times [y_j, y_{j+1}]$ by accumulating the signed trapezoidal areas under the monotone curve segments clipped to that cell plus the scanline winding carry from the left:
$$C_{i,j}(\boldsymbol{\theta}) = \iint_{\Omega(\boldsymbol{\theta}) \cap \mathcal{P}_{i,j}} dx\,dy$$

### 2.2 Boundary Reynolds Transport Theorem
When differentiating the integrated pixel intensity or cell coverage with respect to a shape parameter $\theta$ (e.g. control point coordinate, scale factor, radius), the classical Leibniz-Reynolds transport theorem in 2D states:
$$\frac{\partial}{\partial \theta} \iint_{\Omega(\boldsymbol{\theta})} f(x, y)\,dA = \iint_{\Omega(\boldsymbol{\theta})} \frac{\partial f}{\partial \theta}\,dA + \oint_{\partial \Omega(\boldsymbol{\theta})} f(\mathbf{x})\,(\mathbf{v}(\mathbf{x}) \cdot \mathbf{n}(\mathbf{x}))\,dl$$
where:
- $\mathbf{v}(\mathbf{x}) = \frac{\partial \mathbf{x}}{\partial \theta}$ is the boundary velocity vector at $\mathbf{x} \in \partial \Omega$.
- $\mathbf{n}(\mathbf{x})$ is the outward unit normal vector.
- $dl$ is the differential arc length element along the boundary.

For flat fills with constant color/intensity ($f = 1$):
$$\frac{\partial}{\partial \theta} C_{i,j}(\boldsymbol{\theta}) = \int_{\partial \Omega(\boldsymbol{\theta}) \cap \mathcal{P}_{i,j}} (\mathbf{v}(\mathbf{x}) \cdot \mathbf{n}(\mathbf{x}))\,dl$$

### 2.3 Exact Closed-Form Boundary Derivative on Quadratic Béziers
A critical insight of this research is that for quadratic Bézier curves, the integrand $(\mathbf{v} \cdot \mathbf{n})\,dl$ **simplifies into a pure polynomial in $t$ without radicals**.

Let a segment be parameterized by $\mathbf{B}(t) = (x(t), y(t))$. The tangent vector is $\mathbf{B}'(t) = (x'(t), y'(t))$. The outward normal and arc length element satisfy:
$$\mathbf{n}(t)\,dl = \begin{pmatrix} y'(t) \\ -x'(t) \end{pmatrix} dt$$

Taking the boundary velocity $\mathbf{v}(t) = \left( \frac{\partial x(t)}{\partial \theta}, \frac{\partial y(t)}{\partial \theta} \right)^T$:
$$(\mathbf{v}(t) \cdot \mathbf{n}(t))\,dl = \left( \frac{\partial x(t)}{\partial \theta} y'(t) - \frac{\partial y(t)}{\partial \theta} x'(t) \right) dt$$

Because quadratic Bézier curves have coordinates that are degree-2 polynomials in $t$:
$$x(t) = (1-t)^2 x_0 + 2t(1-t) x_1 + t^2 x_2$$
$$\frac{\partial x(t)}{\partial \theta} \text{ has degree } \le 2, \quad y'(t) \text{ has degree } 1$$

Therefore, the product $\frac{\partial x(t)}{\partial \theta} y'(t) - \frac{\partial y(t)}{\partial \theta} x'(t)$ is a **polynomial of degree at most 3 in $t$**.
Its definite integral over any parameter sub-interval $[t_a, t_b] \subseteq [0, 1]$:
$$\int_{t_a}^{t_b} \left( \frac{\partial x}{\partial \theta} y' - \frac{\partial y}{\partial \theta} x' \right) dt$$
is computable **exactly in closed form using standard rational polynomial anti-derivatives**, with zero numerical quadrature error and zero transcendental function evaluation!

---

## 3. Architecture & Integration Plan

The differentiable animation stack is organized across three tiers:

```
+-------------------------------------------------------------+
|               Proscenium Animation Optimizer                |
|  - Keyframe trajectory fitting (spline / rate function)     |
|  - Camera path optimization & collision avoidance          |
+-------------------------------------------------------------+
                              | (adjoint loss / gradients)
                              v
+-------------------------------------------------------------+
|          frankentorch (ft) Autograd Graph Substrate         |
|  - Reverse-mode AD for affine transformations & styles      |
|  - Tensor loss functions & Adam / L-BFGS optimizers         |
+-------------------------------------------------------------+
                              | (boundary parameters & dL/dC)
                              v
+-------------------------------------------------------------+
|           Lumen Differentiable Analytic Kernel (Spike)       |
|  - Forward: Exact scanline winding trapezoid coverage       |
|  - Backward: Boundary line integral on quadratic Béziers   |
+-------------------------------------------------------------+
```

1. **Scene Graph Level (`fmn-mobject` / `fmn-anim`):**
   Continuous mobject properties (coordinates, colors, stroke widths) can be wrapped as differentiable leaves in a `frankentorch` computational graph.
2. **Transform Level (`fmn-geom`):**
   Standard linear algebra transformations ($4 \times 4$ camera projection, $3 \times 3$ affine transforms) propagate adjoints via standard matrix calculus in `frankentorch`.
3. **Raster Level (`fmn-render` / exploratory spike):**
   Evaluates $\frac{\partial C_{i,j}}{\partial \mathbf{P}_k}$ where $\mathbf{P}_k$ are the 2D control points of the path.

---

## 4. Explicit Non-Goals & Constitutional Boundaries

To preserve FrankenManim's engineering integrity, the following boundaries are permanent and non-negotiable:

1. **No Product Gate Dependency:**
   The exploratory tier is strictly research. No release gate, milestone (G1–G5), or core build requires differentiable features. If research reveals that differentiable animation is computationally unviable, it can be retired without impacting production rendering.
2. **Certified Determinism Inviolability (D-18, §10.5):**
   The certified CPU engine (`--reproducible`) will **never** incorporate heuristic approximations, stochastic sampling, or fuzzy rasterization. Certified frames remain an exact, bit-identical function of the canonical raster arithmetic.
3. **No Unsafe Code (D-03):**
   `#![forbid(unsafe_code)]` remains strictly enforced. All gradient accumulation and tensor algebra must execute in verified safe Rust.
4. **No External Machine Learning Frameworks (D-01, D-02):**
   No PyTorch, LibTorch, JAX, TensorFlow, or Python runtime dependencies. All automatic differentiation runs through `frankentorch` (`ft`), the sovereign tensor engine.

---

## 5. Failure Modes & Stop Conditions

The exploratory research program must evaluate and document the following hard stop conditions:

| Stop Condition | Criterion | Implication / Decision |
|---|---|---|
| **Topological Discontinuities** | Parameter shifts cause self-intersection changes, edge splitting, or hole birth/death | Boundary integral gradients are undefined at topological bifurcations. If gradient ascent gets trapped in topological local minima, restrict differentiable domain to fixed-topology morphs. |
| **Numerical Gradient Singularities** | Boundary segment length $\to 0$ or curvature $\kappa \to \infty$ produces $|\nabla| > 10^6$ | Requires gradient clipping or minimum segment-length regularization. If stability requires invasive heuristics, document and pause. |
| **Efficiency vs Derivative-Free Methods** | Derivative-free optimization (CMA-ES / Nelder-Mead in `wasm_cmaes` or `fsci-opt`) solves parameter matching faster for $N \le 30$ parameters | For low-dimensional scene animation tuning, derivative-free methods avoid all boundary integral overhead. If CMA-ES outperforms backpropagation across 80% of test cases, prioritize black-box optimization over analytical backprop. |

---

## 6. Next Steps & Deliverables

1. Land the initial feasibility spike in `spikes/g5-differentiable/` validating exact closed-form boundary derivatives on quadratic circular arcs against finite differences.
2. Publish the SME/AMX architectural investigation note (`docs/research/sme-amx-investigation.md`) evaluating hardware matrix acceleration suitability.
3. Keep track of upstream `frankentorch` reverse-mode autograd readiness for future high-level graph binding.
