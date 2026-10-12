# The Arithmetic Where 3 × 7 = 9: YouTube package

What goes on the upload page. The description is cut from `docs/description.md` (whose
numbers are oracle keys and fact-sheet rows) and links back to it for the full notes; the
chapter times come from the final cut (`./x raptorq_gf256 chapters`).

- Title: The Arithmetic Where 3 × 7 = 9
- Visibility: unlisted until the owner has listened to it, then public
- Category: Education. Audience: not made for kids.
- Altered or synthetic content: No (a generic synthetic narrator; no real person, place or event is depicted). The description says the narration is synthetic.
- Thumbnail: `renders/raptorq_gf256_thumbnail.png` (`./x raptorq_gf256 thumbnail`)
- Captions: English, `renders/final/raptorq_gf256.en.srt` (`./x raptorq_gf256 captions`)
- Tags: RaptorQ, fountain code, erasure code, finite field, Galois field, GF(256), XOR, polynomials, error correction, RFC 6330, Reed-Solomon, network coding, linear algebra, math explainer

## Description

```text
A server sends one file to a million phones, and every phone loses its own share of the packets. RaptorQ, the fountain code in RFC 6330 and in ATSC 3.0 broadcasting, lets each phone rebuild the file from almost any packets it happens to receive, without asking for anything again. This video builds the arithmetic that makes that work.

Every packet is an equation: some pieces of the file, mixed with ex-or (XOR). With ex-or alone, a random set of exactly enough equations fails to pin the file down about 71% of the time (model), and almost all of the trouble is in the last few equations. RaptorQ closes those last few with 10 to 16 equations over a byte field of 256 elements, where bytes multiply as polynomials folded by one that cannot be factored. In that field, 3 × 7 = 9.

Chapters
{CHAPTERS}

What the promise averages over
RFC 6330 says a decoder fails at most 1 time in 100 with exactly enough packets, 1 in 10,000 with one extra and 1 in 1,000,000 with two. Those are averages over packets chosen at random. A phone that receives the originals first and loses random packets gets a different mix, so we measured that case too: with 1,002 pieces, each packet lost with probability 1 in 100, 3 of 5,000,000 simulated phones failed with two packets more than they needed (98 with one extra, 21,186 with none). Numbers marked "model" in the video are computed under stated assumptions; measured numbers carry their sample sizes.

The exact version of every simplification, every source, and the code that computes each number and checks it against an independent oracle:
https://github.com/Dicklesworthstone/jeffs_math_explainer_videos/blob/main/explainer_corpus/raptorq_gf256/docs/description.md

Sources
RFC 6330, "RaptorQ Forward Error Correction Scheme for Object Delivery" (Luby, Shokrollahi, Watson, Stockhammer, Minder; IETF, 2011): https://www.rfc-editor.org/rfc/rfc6330
ATSC A/331:2021, "Signaling, Delivery, Synchronization, and Error Protection"
Byers, Luby, Mitzenmacher, Rege, "A Digital Fountain Approach to Reliable Distribution of Bulk Data", ACM SIGCOMM 1998
Ho, Médard, Koetter, Karger, Effros, Shi, Leong, "A Random Linear Network Coding Approach to Multicast", IEEE Trans. Inf. Theory 52(10), 2006
H. P. Anvin, "The mathematics of RAID-6"
ISO/IEC 18004 (QR Code), §6.5.2; FIPS 197 (AES), §4.2

Narration: synthetic speech (FrankenTTS). Animation: franken_manim, a Rust rewrite of manim.
```

## Chapter titles (in order; the times come from the final cut)

A million phones | Bits and ex-or | Packets are equations | The cube | The ex-or lottery |
Spares and rings | Why not wrap around at 256? | Prime clocks | Bytes are polynomials | The fold |
The byte clock | The times table | The byte lottery | The price of density | Where the danger lives |
Parking a piece | Two packets
