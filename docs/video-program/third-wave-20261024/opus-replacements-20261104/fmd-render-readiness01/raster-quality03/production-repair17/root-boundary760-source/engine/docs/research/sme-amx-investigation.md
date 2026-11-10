# Hardware Investigation: SME and AMX in FrankenManim

> **Document Class:** Hardware & Platform Investigation Note (§17.6, §20.3 G5 criterion 6)  
> **Status:** Completed Assessment  
> **Target Architectures:** ARM Scalable Matrix Extension (SME / SME2), Intel Advanced Matrix Extensions (AMX-TMUL / AMX-BF16 / AMX-FP16)  
> **Bead:** `fm-exploratory-tier-charter-711n`

---

## 1. Executive Summary

As part of the exploratory tier charter (G5 criterion 6), this note investigates whether modern CPU matrix extensions—specifically **ARM SME/SME2** (available on newer ARMv9 cores and Apple silicon M4/M5) and **Intel AMX** (available on Intel Xeon Scalable Sapphire Rapids / Emerald Rapids / Granite Rapids)—should be integrated into FrankenManim's execution engine hierarchy or tile pool scheduling.

**Key Finding:**
Neither AMX nor SME is appropriate for Lumen's core rendering engine or tile compositing pipeline.
- **Architectural Mismatch:** Lumen's analytic nonzero-winding fill (§10.2) and curve-distance stroke engine (§10.3) are branchy, polygon-boundary-centric algorithms dominated by quadratic root finding, monotone span clipping, and sparse trapezoidal area integration. Matrix systolic arrays require dense $M \times K \times N$ matrix multiplications and suffer catastrophic zero-padding and reformatting penalties when applied to sparse, irregular boundary geometry.
- **Microarchitectural Penalties:** Both AMX and SME require transitions into dedicated matrix execution states (ARM Streaming SVE mode `SMCR_EL1`; Intel AMX `XSTATE` via `ldtilecfg`), which induce CPU core downclocking, elevated wake-up latencies, and thread-context switch overheads.
- **Safety Doctrine Incompatibility:** Neither ISA extension possesses a safe Rust abstraction in `core::arch` or `std::simd`. Utilizing them would require extensive `unsafe` intrinsics or inline assembly, violating FrankenManim's `#![forbid(unsafe_code)]` mandate (D-03).
- **Affirmation of §17.6:** This investigation validates the explicit ruling in plan §17.6:
  > *"SME/AMX and the Neural Engine are exploratory-tier at most: no safe Rust path, and the wrong shape for branchy analytic coverage."*

---

## 2. Hardware Extension Architectural Taxonomy

### 2.1 Intel AMX (Advanced Matrix Extensions)
- **Execution Units:** Tile Matrix Multiply (TMUL) accelerator attached to CPU cores.
- **Register Architecture:** 8 2D tile registers (`TMM0` through `TMM7`), each up to 1 KiB (total 8 KiB register file), configured with dimensions up to 16 rows $\times$ 64 bytes.
- **Supported Types:** Int8 (`tdpbusd`, `tdpbuud`), Bfloat16 (`tdpbf16ps`), and FP16 (`tdpfp16ps` on Emerald Rapids+).
- **Interface & Overhead:** Requires OS-enabled `XSTATE` tracking. Every thread must execute `ldtilecfg` to configure tile dimensions before compute and `tilerelease` when completed. Unconfigured or mismatched operations trigger architectural exceptions.

### 2.2 ARM SME / SME2 (Scalable Matrix Extension)
- **Execution Units:** Outer product engines attached to SVE2 vector pipes.
- **Register Architecture:** The `ZA` storage array, an $SVL \times SVL$ matrix tile, where $SVL$ is the Streaming Vector Length (e.g. 128 to 2048 bits).
- **Execution Modes:** Normal SVE vs. Streaming SVE Mode (`PSTATE.SM = 1`).
- **Interface & Overhead:** Entering Streaming SVE mode (`smstart za`) switches the core into streaming mode, often forcing a transient pipeline flush and clock throttling on heterogeneous big.LITTLE architectures.

---

## 3. Workload Mapping Analysis

### 3.1 Lumen Vector Rasterization (Analytic Fill & Stroke)
FrankenManim's rendering pipeline evaluates geometry analytically rather than approximating curves with fine triangle tessellation:

```
[Mobject Path]
      |
      v
[Quadratic Splitting]  --> dy/dt = 0 (y-monotone) & dx/dt = 0 (x-monotone)
      |
      v
[Tile Binning]         --> Coarse 128x128 macrotiles -> Fine 16x16 tiles
      |
      v
[Scanline Evaluation]  --> Closed-form root: t = (-b +/- sqrt(b^2 - 4ac)) / (2a)
      |
      v
[Trapezoidal Accum.]   --> Signed cell integral: dx * (y_mid - y_baseline)
      |
      v
[Winding Carry]        --> Prefix sum across scanline tiles
```

#### Why AMX / SME Fail Here:
1. **Irregular Sparsity:** A $16 \times 16$ tile rarely has geometry in all 256 pixels. Even complex vector scenes touch only a fraction of cells per tile. Mapping sparse 1D scanline roots onto a $16 \times 64$ dense matrix tile requires padding $>90\%$ of the matrix entries with zeros, completely nullifying the theoretical 16x compute density.
2. **Control Flow Divergence:** Evaluating whether a segment crosses a pixel boundary, whether a root falls inside $[0, 1]$, and clipping against the $[x_0, x_1] \times [y_0, y_1]$ box requires per-lane branch-free selection (`std::simd::select` or SIMD mask blend). Matrix systolic arrays have no conditional branching or per-element masking within the TMUL tile.
3. **Precision Requirements:** Certified CPU rendering requires exact IEEE-754 64-bit (`f64`) floating point arithmetic for trapezoidal area and winding carry. AMX does **not** support `f64` matrix multiplication (only int8, bf16, and fp16). Using AMX would violate the certified determinism contract (D-18, §10.5).

### 3.2 Affine Point Transforms and Coordinate Projections
In `fmn-geom` and `fmn-mobject`, paths are transformed by $4 \times 4$ camera matrices or $3 \times 3$ affine matrices:
$$\mathbf{p}' = \mathbf{M} \mathbf{p} + \mathbf{t}$$

- **Batch Size:** A typical mobject family contains hundreds to thousands of vertices.
- **Arithmetic Intensity:** A $4 \times 4 \times 4$ matrix-vector multiplication performs 16 multiplications and 12 additions per 3D point ($28 \text{ FLOPs}$ for $32 \text{ bytes}$ of point data).
- **Memory Bandwidth Bottleneck:** The arithmetic intensity is $< 1.0 \text{ FLOP/byte}$. Point transformation is completely memory-bandwidth bound. Accelerating compute with a $2000 \text{ GFLOP}$ systolic array provides zero speedup because the CPU cores are throttled by memory subsystem bandwidth (L1/L2 cache line fill rate).
- **Vector Tiers Suffice:** As documented in `crates/fmn-render/src/engine.rs`, standard SIMD vector tiers (`std::simd::Simd<f64, 4>` / `Simd<f32, 4>`) on AVX2, AVX-512, or NEON already saturate the memory subsystem when transforming arrays of coordinates.

### 3.3 Neural Network Visualization (`NeuralNetworkMobject`)
The only plausible GEMM workload in the library is forward inference visualization in `NeuralNetworkMobject` (W7 lineage). However:
- FrankenManim delegates all tensor computation and neural network operations to **`frankentorch`** (ft).
- `frankentorch` owns its own compute backends (CPU, Metal, CUDA).
- Integrating AMX/SME directly into `franken_manim` would duplicate functionality and violate the "Not Built Here" doctrine (D-04).

---

## 4. Safety & Governance Evaluation

| Principle | Requirement | AMX / SME Status | Verdict |
|---|---|---|---|
| **D-03 (No Unsafe)** | `#![forbid(unsafe_code)]` across all crates | No safe Rust API exists in `std::simd` or `core::arch`. Requires raw intrinsics or asm blocks. | **FAIL** |
| **D-15 (SIMD Tiers)** | Crate-wide build tiers via `cfg(target_feature)` | Dynamic tile configuration (`ldtilecfg`) cannot be expressed as a static target feature without per-function state management. | **FAIL** |
| **D-18 (Parallelism Contract)** | Certified determinism independent of scheduler | AMX/SME support reduced precision (bf16/fp16) with non-IEEE rounding and fused accumulation variations across chip steppings. | **FAIL** |
| **D-04 (Not Built Here)** | Core graphics only; delegate tensor math to `ft` | Dense matrix math belongs in `frankentorch`. | **REJECT** |

---

## 5. Architectural Recommendation

1. **Maintain the Clear Boundary:**
   Keep AMX and SME permanently classified as exploratory/quarantined research. Do not add AMX or SME build tiers or runtime feature gates to `fmn-render` or `fmn-core`.
2. **Preserve Focus on the Proven Architecture:**
   - **Certified Path:** Canonical scalar / `std::simd` portable fallback ensuring bit-exact cross-platform reproducibility (§10.5).
   - **Fast CPU Path:** `std::simd` over x86-64-v3 (AVX2), x86-64-v4 (AVX-512), and aarch64+NEON.
   - **Accelerator Annex:** GPU offload via `frankentorch` Metal and CUDA backends where high FLOP-density and tile-local memory actually pay off (§17.5, §17.6).
