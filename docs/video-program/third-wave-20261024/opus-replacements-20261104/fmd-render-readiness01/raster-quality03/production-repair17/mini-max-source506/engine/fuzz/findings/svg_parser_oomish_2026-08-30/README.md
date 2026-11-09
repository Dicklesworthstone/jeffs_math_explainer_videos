# Resolved: UTF-8 boundary panic in SVG markup probing

- Parser: `fmn-geom` sovereign SVG processor
- Reproducer: `crash-0b09afb7ad8e1d4bee1ecb502b9670eabb13cecb` (17 bytes)
- Original mode: `cargo fuzz run svg_document` with AddressSanitizer and debug assertions
- Original symptom: exit 77 within 15 seconds
- Resolution commit: `ec3591359b057cc9802f40b6a2c2fba325b1289a`

## Root cause

The tokenizer inspected the case-insensitive `<!doctype` prefix by taking a fixed-width `str` slice. The preserved input places the first byte of the two-byte UTF-8 character `¤` at the final byte of that slice, so indexing the validated UTF-8 string at that byte offset panicked before the malformed declaration could become a typed `SvgError`.

## Resolution

The public SVG admission boundary now validates every fixed-width markup probe against UTF-8 character boundaries before invoking the parser core. It skips comments, CDATA, processing instructions, and quoted attribute contents so the guard follows tokenizer-level markup rather than scanning arbitrary text. The parser's existing size, UTF-8, DOCTYPE, and emit helpers remain the semantic authorities behind the facade.

Permanent regressions cover the exact 17-byte corpus input, case-insensitive DOCTYPE rejection, and markup-like text inside quoted attributes. The finding is resolved in source; a fresh sanitizer replay remains part of the next available fuzz lane rather than being inferred from static inspection.
