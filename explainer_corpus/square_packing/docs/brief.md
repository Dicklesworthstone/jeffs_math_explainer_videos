# Brief — Why Turning a Square Can Make the Whole Box Smaller

Catalog video `square_packing`, revision `v1-michael-20261017`; first-chapter
task `mx-video-program-program-87do.5.1`. Values I chose are marked
`(default)`. Words and pictures: `docs/storyboard.md`; every number and source:
`docs/fact-sheet.md`.

| Field | Value |
|---|---|
| Topic | Five equal square tiles and the smallest square box that holds them (catalog pitch: "construct a side 2 + √2/2 container, distinguish a feasible construction from a lower bound, then explain why local motion and global optimality differ. Contacts and forbidden regions, not a gallery of records"). |
| Working title | **Why Turning a Square Can Make the Whole Box Smaller** (the catalog's). |
| One-sentence takeaway | Turning one tile lets five fit a box 2 + 1/√2 ≈ 2.71 wide instead of 3, because a gap measured along its diagonal is √2 times as long; but a packing that fits shows only that this box works, and that nothing smaller does needs a proof about every arrangement. |
| The hook | Four tiles fill a box two wide; the obvious fix for the fifth is a box three wide, with room left for four more. Is that wasteful room unavoidable? |
| The one picture | Four straight tiles in the corners, the fifth turned an eighth of a turn in the middle, each inner corner touching the middle of a side of the turned tile; the square gap between the inner corners has a diagonal of exactly one tile. |
| Three questions, kept apart | **Fits?** A packing checked exactly: tile against tile, tile against wall (`model.rs`, `oracle.py`). **Can it move there?** The chapter's turn and squeeze, every frame a packing (tested frame by frame, argued between frames). **Could anything smaller fit?** Göbel's theorem (1979), not shown by any picture; built lemma by lemma from chapter 04 (the corners, the side strips in 05, the middle in 06), each boarded only after its proof is reconstructed, and claimed in chapter 07, whose strict step (a supposed narrower packing enlarged until each tile holds a point strictly inside) counts five tiles against four points. |
| What the script must get right | (1) "Fits" is checked exactly, with contacts exactly zero (numbers a + b√2), never "close enough". (2) The squeeze is a family of packings in shrinking boxes, not a motion inside one box; the frames past the stop overlap and are shown in red as what fails. (3) The turn in the box of three never collides: the turning tile stays in the disc of its half-diagonal about the centre, which meets each corner tile only at its inner corner. (4) No claim of optimality from the picture; no claim about rigidity until chapter 04 recomputes it (the case page reports the packing "not infinitesimally rigid but second-order rigid at fixed side"). (5) Göbel is credited for the packing on screen (Friedman's survey, reference [7]); his paper itself was not read. |
| Spoken vocabulary | tile, box, wide, straight, turn, squeeze, touch, overlap, fit, inner corner, gap, proof (storyboard §1). Never "rotate", "tilt", "container", "obviously". |
| Audience | The corpus's three viewers (plan §1.2). |
| Length | Chapter 01: the central scratch draft at 8f154b5 measured 88.3 s (the static storyboard estimate, 80 s, ran low). Trimmed without touching the derivation or the 4 s think window: eight words, the closing hold, and two serial fades. Predicted from that draft's measured gaps: 82.1 s. The central rerun decides. Whole video: 6 chapters, about 8 to 9 minutes (default). |
| Voice | FrankenTTS `michael` with his approved English accent (owner's direction); no narration has been generated for this revision, and no voice approval is inferred. |
| Sound | The score engine with this video's identity (`src/score.rs`, default): B-flat major, 80 beats a minute, kalimba lead; the motif is the ornament called a turn (3̂ 2̂ 1̂ 2̂), left open in the think window. |
| Look | 3Blue1Brown dark ground; the turned tile gold, corner tiles blue, the box ink, touches green dots, overlaps red (storyboard §2). |
| Out of scope | A gallery of record packings; claims about counts beyond ten and eleven without a source audit; any optimality or rigidity claim not reconstructed in code. |
