# Native TeX preambles and template packs

`Tex`, `TexText` and the legacy `SingleStringTex` accept native macro definitions
in `additional_preamble`. Expansion uses fmd-math's existing token parser,
non-recursive macro system and budgets. No LaTeX process or Python typesetter is
introduced.

```python
from manimlib import *

preamble = r"\newcommand{\sq}[1]{#1^2}\newcommand{\RR}{\mathbb{R}}"
formula = Tex(r"\sq{x} + 1", additional_preamble=preamble, isolate=["x"])
label = TexText(r"area $\sq{x}$ in $\RR$", additional_preamble=preamble)
```

The `template` names `default`, `basic`, and `empty` select the existing native
preamble packs. The empty template name retains the default. Native primitives
are available in every pack; `default` additionally defines `\minus`. Inline
`\newcommand` refuses to shadow an existing definition, while `\renewcommand`
requires one. Calls can be nested in the expression, such as `\half{\sq{x}}`;
recursion is refused, as is expansion that exceeds fmd-math's token/depth limits.
The native symbol vocabulary includes `\minus` even in the bare packs; the
default pack additionally defines it as a macro, affecting shadowing rules.

**Pinned-engine limitation:** a macro calling another macro from its definition
body can produce invalid spans or fail to substitute forwarded arguments in the
pinned fmd-math version. Those preamble requests are rejected with an explicit
span-provenance error, not rendered with clamped positions or literal `#1`
tokens. Use source-nested calls or define the compound body directly from native
primitives. The upstream correction is tracked in `UPSTREAM_LEDGER.md` row 13;
this change does not silently replace the governed dependency pin.

A preamble must parse independently without producing ink or layout spacing.
It is not a full LaTeX document preamble: package loading, file inclusion,
shell execution, host-font templates, and unsupported constructs remain named
errors. Known unavailable and unknown templates list the available native packs.
A trailing comment cannot consume the expression because the native boundary
inserts a newline before the original formula.

## Source identity and caching

`get_tex()`, selectors, isolation, coloring, matching animations and live numeric
replacement continue to address the **original expression**. Macro-generated
ink belongs to its call site; argument glyphs retain their argument spans.
Preamble bytes never become selectable formula characters. Formula diagnostics
report offsets relative to that expression. Preamble failures are explicitly
labelled `additional_preamble`. Selection uses containment: select `\half{x}`
to include whole-call generated ink, or `x` for its literal argument. The bare
command name `\half` alone does not contain the full expansion-site span.
Primitive commands differ. Ink that a built-in command draws itself (the rule
of `\frac`/`\over`, the radical of `\sqrt`, accents, and over/under lines and
braces) belongs to the command token. So `tex[R"\over"]` selects the fraction
bar and `tex[R"\sqrt"]` the radical sign, never argument glyphs, as in the
Reference (fmn-tex `KEYWORD_INK_COMMANDS`).

A single-use numeric macro argument can become a live readout without removing
generated ink around it. When one argument is expanded into multiple displayed
copies, `make_number_changeable` refuses rather than collapsing those copies
into one number. Author separate numeric source occurrences when each copy
must have a live readout. Ordinary selection and rendering of repeated macro
arguments remain supported.

Cache lookup includes the complete effective source and the native pack's
fingerprint. Different definitions cannot reuse each other's expansion, and
request-local definitions never leak into another object. Copying and pickling
retain the Python configuration and the formula's projected span map.

Additional preambles are limited to 65,536 UTF-8 bytes; preamble, separator and
formula together are limited to 262,144 bytes. Empty preambles retain the
existing ordinary source path. Native expansion limits still apply within
these input bounds. Invalid preambles and templates fail before the Python
constructor installs live object state; failed render generations publish no
output.

## Rust and acceptance

`TexEngine::typeset_with_preamble(mode, source, preamble)` returns native typeset
data with original-source coordinates. Library callers use
`Tex::new(source).preamble(preamble).build(&engine)` or the same builder method on
`TexText`. Engine selection remains explicit for Rust callers.

`crates/fmn-tex/tests/preamble.rs` and
`crates/fmn-library/tests/tex_preamble.rs` cover native scopes, caches, diagnostics,
geometry and source provenance. `crates/fmn-python/tests/tex_preamble.py` covers
real portal construction and selection. The registered
`render_matrix.python_tex_preamble.v1` Gauntlet scenario exercises an eight-frame
native render at 1/4/16 threads, matching transforms, live numeric replacement
and rejection of a recursive-macro generation.
