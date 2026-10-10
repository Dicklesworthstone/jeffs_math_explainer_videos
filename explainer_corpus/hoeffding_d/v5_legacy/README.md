# v5 sources, kept for reference (not compiled)

The Hoeffding explainer's own kit, narrator, encoder wrapper and CLI as they
were at v5, before the crate moved onto the shared `explainer_kit` (bead
W3-migrate). Nothing builds from this folder; `src/` uses the shared kit. The
v5 glyph probe scene is here too; the shared CLI's `glyphs` command renders
the same TEXT/TEX lists from `src/chapters/glyphs.rs`.

`Cargo.lock` is the standalone crate's lock at v5; the workspace's root
`Cargo.lock` governs the build now.
