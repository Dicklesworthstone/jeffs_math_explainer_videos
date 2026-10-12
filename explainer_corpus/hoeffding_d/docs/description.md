# Hoeffding's D: description (draft)

Draft of the YouTube description for the re-narrated cut (2026-10-12). The chapter timestamps come from the final
cut's cue logs at publish. Every number below is printed by `hoeffding stats`, which computes it from the data the
video draws and checks the worked example first.

---

Here is a ring of points. Anyone can see that X and Y are related, yet the three usual correlations each read about
zero on it: Pearson's r is −0.012, Spearman's ρ is −0.013 and Kendall's τ is −0.010. Hoeffding's D, from 1948, asks a
different question, whether X and Y could be independent at all, and on the same ring it reads 0.0375, where the 99th
percentile of D over shuffles of Y is 0.0131. The video builds D from nothing: why a turn takes four points to see,
how ranks and one count per point stand in for the 26,010,428,123,750 groups of four among 5,000 points, and a worked
example on ten people's heights and weights that comes out to D = 0.4107.

## Chapters

(timestamps at publish)

1. The ring that correlation can't see
2. Six shapes, four measures
3. Why a turn takes four points
4. Ranks
5. The count Q
6. Three sums and one formula
7. What shuffling destroys
8. Recap

Names to search: Hoeffding's D, Hoeffding's test of independence, Pearson's correlation coefficient, Spearman's rank
correlation, Kendall's tau, rank statistics, U-statistic, permutation test.

## Sources

- W. Hoeffding, "A Non-Parametric Test of Independence", *The Annals of Mathematical Statistics* 19(4), December
  1948, pp. 546–557. doi:10.1214/aoms/1177730150
- J. Emanuel, "My Favorite Statistical Measure: Hoeffding's D", the article whose algorithm and worked example the
  video follows: https://github.com/Dicklesworthstone/hoeffdings_d_explainer
- SAS Institute, *Base SAS 9.4 Procedures Guide: Statistical Procedures*, 4th ed., PROC CORR, "Hoeffding Dependence
  Coefficient", for D's scale: between −0.5 and 1 when no values are tied, and possibly lower in a small sample with
  many ties.
  https://support.sas.com/documentation/cdl/en/procstat/68142/HTML/default/procstat_corr_details07.htm

## Code

The video's numbers are computed by the code that draws it:
https://github.com/Dicklesworthstone/jeffs_math_explainer_videos (the video: `explainer_corpus/hoeffding_d/`; the
statistics: `src/stats.rs` there, and `hoeffding stats` prints them).
