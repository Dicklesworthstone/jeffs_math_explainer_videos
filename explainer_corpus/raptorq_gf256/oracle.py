#!/usr/bin/env python3
"""Independent oracle for Video B ("The Arithmetic Where 3 x 7 = 9"): every row B-F1 ... B-F28 of the
facts ledger in docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md section 10.5, plus
extra rows B-X1 ... B-X2 for the open items (RAID-6, QR codes). The claim walk (gate G3) added keys under
B-F1, B-F6, B-F9 ... B-F13, B-F15, B-F18, B-F24, B-F26 and the extra rows B-X3 ... B-X6 (ESI range,
small arithmetic said aloud, CRC-8/SAE-J1850, FIPS 197's field); see docs/fact-sheet.md.

Run from this directory:  uv run --quiet --no-project --with numpy python3 -I -B oracle.py
(that is what `./x raptorq_gf256 facts-check` runs).
Output: one line per fact, KEY<TAB>VALUE, on stdout. A run summary goes to stderr.

Value conventions
  * integers are exact;
  * probabilities are computed from exact formulas with fractions.Fraction (or decimal.Decimal at
    50 digits) and printed as repr(float(...)); where an infinite product or sum is truncated, the
    truncation error is bounded in a comment and is far below 1e-15;
  * strings are used only for orderings, lists and provenance labels;
  * keys ending in "_ok" are internal cross-checks (1 = the two independent routes agree).

Provenance
  * Functions are adapted (copied, not imported) from docs/research/raptorq_gf256/scripts/
    (gf256np.py, gf256_facts.py, rank_probs.py, misc_facts.py, toy_examples.py, lt_peel_sim.py) and
    docs/reviews/plan_r1/integration_check.py. They are copied so that this file has no import side
    effects and writes nothing outside this directory.
  * RFC 6330 tables (Table 1, Table 2, OCT_EXP, OCT_LOG) are transcribed below from
    https://www.rfc-editor.org/rfc/rfc6330.txt (sha256 87f6da89cc325987cb2910d1c124f74eaeac5c86a83f962cf347c0cc66321ace,
    fetched 2026-10-10) by a parser written for this oracle (not the repo's rfc6330_data.py).
  * RECORDED values: B-F21 and B-F22 are Monte Carlo counts from runs that this oracle does not repeat
    (B-F22 needs the PyPI package raptorq==2.0.0, and both runs took 15+ minutes). They are pinned
    constants, labelled "recorded", with the output file they came from; the oracle re-reads that
    file when it exists and reports whether the pinned constant still matches it.
"""
import bisect
import hashlib
import math
import os
import random
import re
import sys
import time
from decimal import Decimal, getcontext, localcontext
from fractions import Fraction

import numpy as np

T0 = time.time()
_KEYS = {}


def _fmt(v):
    if isinstance(v, bool):
        return "1" if v else "0"
    if isinstance(v, (int, np.integer)):
        return str(int(v))
    if isinstance(v, Fraction):
        return str(v.numerator) if v.denominator == 1 else repr(float(v))
    if isinstance(v, Decimal):
        return repr(float(v))
    if isinstance(v, (float, np.floating)):
        return repr(float(v))
    if isinstance(v, (list, tuple)):
        return ",".join(_fmt(x) for x in v)
    return str(v)


def emit(key, value):
    if key in _KEYS:
        raise SystemExit(f"duplicate key {key}")
    s = _fmt(value)
    _KEYS[key] = s
    print(f"{key}\t{s}")


# the repository root: the nearest ancestor holding docs/research (the oracle
# lives in explainer_corpus/<slug>/)
REPO = next(str(d) for d in __import__("pathlib").Path(__file__).resolve().parents if (d / "docs" / "research").is_dir())
RESULTS = os.path.join(REPO, "docs", "research", "raptorq_gf256", "scripts", "results")

# ----------------------------------------------------------------------------------------------
# RFC 6330 data (transcribed; see module docstring)
# ----------------------------------------------------------------------------------------------
# Section 5.3.5.2, Table 1: f[d], d = 0..30
RFC_TABLE1 = [0, 5243, 529531, 704294, 791675, 844104, 879057, 904023, 922747, 937311, 948962, 958494, 966438,
              973160, 978921, 983914, 988283, 992138, 995565, 998631, 1001391, 1003887, 1006157, 1008229, 1010129,
              1011876, 1013490, 1014983, 1016370, 1017662, 1048576]
# Section 5.6, Table 2: rows "K' J(K') S(K') H(K') W(K')", 477 rows
RFC_TABLE2_TEXT = """
10 254 7 10 17  12 630 7 10 19  18 682 11 10 29  20 293 11 10 31  26 80 11 10 37  30 566 11 10 41  32 860 11
10 43  36 267 11 10 47  42 822 11 10 53  46 506 13 10 59  48 589 13 10 61  49 87 13 10 61  55 520 13 10 67
60 159 13 10 71  62 235 13 10 73  69 157 13 10 79  75 502 17 10 89  84 334 17 10 97  88 583 17 10 101  91 66
17 10 103  95 352 17 10 107  97 365 17 10 109  101 562 17 10 113  114 5 19 10 127  119 603 19 10 131  125
721 19 10 137  127 28 19 10 139  138 660 19 10 149  140 829 19 10 151  149 900 23 10 163  153 930 23 10 167
160 814 23 10 173  166 661 23 10 179  168 693 23 10 181  179 780 23 10 191  181 605 23 10 193  185 551 23 10
197  187 777 23 10 199  200 491 23 10 211  213 396 23 10 223  217 764 29 10 233  225 843 29 10 241  236 646
29 10 251  242 557 29 10 257  248 608 29 10 263  257 265 29 10 271  263 505 29 10 277  269 722 29 10 283
280 263 29 10 293  295 999 29 10 307  301 874 29 10 313  305 160 29 10 317  324 575 31 10 337  337 210 31 10
349  341 513 31 10 353  347 503 31 10 359  355 558 31 10 367  362 932 31 10 373  368 404 31 10 379  372 520
37 10 389  380 846 37 10 397  385 485 37 10 401  393 728 37 10 409  405 554 37 10 421  418 471 37 10 433
428 641 37 10 443  434 732 37 10 449  447 193 37 10 461  453 934 37 10 467  466 864 37 10 479  478 790 37 10
491  486 912 37 10 499  491 617 37 10 503  497 587 37 10 509  511 800 37 10 523  526 923 41 10 541  532 998
41 10 547  542 92 41 10 557  549 497 41 10 563  557 559 41 10 571  563 667 41 10 577  573 912 41 10 587  580
262 41 10 593  588 152 41 10 601  594 526 41 10 607  600 268 41 10 613  606 212 41 10 619  619 45 41 10 631
633 898 43 10 647  640 527 43 10 653  648 558 43 10 661  666 460 47 10 683  675 5 47 10 691  685 895 47 10
701  693 996 47 10 709  703 282 47 10 719  718 513 47 10 733  728 865 47 10 743  736 870 47 10 751  747 239
47 10 761  759 452 47 10 773  778 862 53 10 797  792 852 53 10 811  802 643 53 10 821  811 543 53 10 829
821 447 53 10 839  835 321 53 10 853  845 287 53 10 863  860 12 53 10 877  870 251 53 10 887  891 30 53 10
907  903 621 53 10 919  913 555 53 10 929  926 127 53 10 941  938 400 53 10 953  950 91 59 10 971  963 916
59 10 983  977 935 59 10 997  989 691 59 10 1009  1002 299 59 10 1021  1020 282 59 10 1039  1032 824 59 10
1051  1050 536 59 11 1069  1074 596 59 11 1093  1085 28 59 11 1103  1099 947 59 11 1117  1111 162 59 11 1129
1136 536 59 11 1153  1152 1000 61 11 1171  1169 251 61 11 1187  1183 673 61 11 1201  1205 559 61 11 1223
1220 923 61 11 1237  1236 81 67 11 1259  1255 478 67 11 1277  1269 198 67 11 1291  1285 137 67 11 1307  1306
75 67 11 1327  1347 29 67 11 1367  1361 231 67 11 1381  1389 532 67 11 1409  1404 58 67 11 1423  1420 60 67
11 1439  1436 964 71 11 1459  1461 624 71 11 1483  1477 502 71 11 1499  1502 636 71 11 1523  1522 986 71 11
1543  1539 950 71 11 1559  1561 735 73 11 1583  1579 866 73 11 1601  1600 203 73 11 1621  1616 83 73 11 1637
1649 14 73 11 1669  1673 522 79 11 1699  1698 226 79 11 1723  1716 282 79 11 1741  1734 88 79 11 1759  1759
636 79 11 1783  1777 860 79 11 1801  1800 324 79 11 1823  1824 424 79 11 1847  1844 999 79 11 1867  1863 682
83 11 1889  1887 814 83 11 1913  1906 979 83 11 1931  1926 538 83 11 1951  1954 278 83 11 1979  1979 580 83
11 2003  2005 773 83 11 2029  2040 911 89 11 2069  2070 506 89 11 2099  2103 628 89 11 2131  2125 282 89 11
2153  2152 309 89 11 2179  2195 858 89 11 2221  2217 442 89 11 2243  2247 654 89 11 2273  2278 82 97 11 2311
2315 428 97 11 2347  2339 442 97 11 2371  2367 283 97 11 2399  2392 538 97 11 2423  2416 189 97 11 2447
2447 438 97 11 2477  2473 912 97 11 2503  2502 1 97 11 2531  2528 167 97 11 2557  2565 272 97 11 2593  2601
209 101 11 2633  2640 927 101 11 2671  2668 386 101 11 2699  2701 653 101 11 2731  2737 669 101 11 2767
2772 431 101 11 2801  2802 793 103 11 2833  2831 588 103 11 2861  2875 777 107 11 2909  2906 939 107 11 2939
2938 864 107 11 2971  2979 627 107 11 3011  3015 265 109 11 3049  3056 976 109 11 3089  3101 988 113 11 3137
3151 507 113 11 3187  3186 640 113 11 3221  3224 15 113 11 3259  3265 667 113 11 3299  3299 24 127 11 3347
3344 877 127 11 3391  3387 240 127 11 3433  3423 720 127 11 3469  3466 93 127 11 3511  3502 919 127 11 3547
3539 635 127 11 3583  3579 174 127 11 3623  3616 647 127 11 3659  3658 820 127 11 3701  3697 56 127 11 3739
3751 485 127 11 3793  3792 210 127 11 3833  3840 124 127 11 3881  3883 546 127 11 3923  3924 954 131 11 3967
3970 262 131 11 4013  4015 927 131 11 4057  4069 957 131 11 4111  4112 726 137 11 4159  4165 583 137 11 4211
4207 782 137 11 4253  4252 37 137 11 4297  4318 758 137 11 4363  4365 777 137 11 4409  4418 104 139 11 4463
4468 476 139 11 4513  4513 113 149 11 4567  4567 313 149 11 4621  4626 102 149 11 4679  4681 501 149 11 4733
4731 332 149 11 4783  4780 786 149 11 4831  4838 99 149 11 4889  4901 658 149 11 4951  4954 794 149 11 5003
5008 37 151 11 5059  5063 471 151 11 5113  5116 94 157 11 5171  5172 873 157 11 5227  5225 918 157 11 5279
5279 945 157 11 5333  5334 211 157 11 5387  5391 341 157 11 5443  5449 11 163 11 5507  5506 578 163 11 5563
5566 494 163 11 5623  5637 694 163 11 5693  5694 252 163 11 5749  5763 451 167 11 5821  5823 83 167 11 5881
5896 689 167 11 5953  5975 488 173 11 6037  6039 214 173 11 6101  6102 17 173 11 6163  6169 469 173 11 6229
6233 263 179 11 6299  6296 309 179 11 6361  6363 984 179 11 6427  6427 123 179 11 6491  6518 360 179 11 6581
6589 863 181 11 6653  6655 122 181 11 6719  6730 522 191 11 6803  6799 539 191 11 6871  6878 181 191 11 6949
6956 64 191 11 7027  7033 387 191 11 7103  7108 967 191 11 7177  7185 843 191 11 7253  7281 999 193 11 7351
7360 76 197 11 7433  7445 142 197 11 7517  7520 599 197 11 7591  7596 576 199 11 7669  7675 176 211 11 7759
7770 392 211 11 7853  7855 332 211 11 7937  7935 291 211 11 8017  8030 913 211 11 8111  8111 608 211 11 8191
8194 212 211 11 8273  8290 696 211 11 8369  8377 931 223 11 8467  8474 326 223 11 8563  8559 228 223 11 8647
8654 706 223 11 8741  8744 144 223 11 8831  8837 83 223 11 8923  8928 743 223 11 9013  9019 187 223 11 9103
9111 654 227 11 9199  9206 359 227 11 9293  9303 493 229 11 9391  9400 369 233 11 9491  9497 981 233 11 9587
9601 276 239 11 9697  9708 647 239 11 9803  9813 389 239 11 9907  9916 80 239 11 10009  10017 396 241 11
10111  10120 580 251 11 10223  10241 873 251 11 10343  10351 15 251 11 10453  10458 976 251 11 10559  10567
584 251 11 10667  10676 267 257 11 10781  10787 876 257 11 10891  10899 642 257 12 11003  11015 794 257 12
11119  11130 78 263 12 11239  11245 736 263 12 11353  11358 882 269 12 11471  11475 251 269 12 11587  11590
434 269 12 11701  11711 204 269 12 11821  11829 256 271 12 11941  11956 106 277 12 12073  12087 375 277 12
12203  12208 148 277 12 12323  12333 496 281 12 12451  12460 88 281 12 12577  12593 826 293 12 12721  12726
71 293 12 12853  12857 925 293 12 12983  13002 760 293 12 13127  13143 130 293 12 13267  13284 641 307 12
13421  13417 400 307 12 13553  13558 480 307 12 13693  13695 76 307 12 13829  13833 665 307 12 13967  13974
910 307 12 14107  14115 467 311 12 14251  14272 964 311 12 14407  14415 625 313 12 14551  14560 362 317 12
14699  14713 759 317 12 14851  14862 728 331 12 15013  15011 343 331 12 15161  15170 113 331 12 15319  15325
137 331 12 15473  15496 308 331 12 15643  15651 800 337 12 15803  15808 177 337 12 15959  15977 961 337 12
16127  16161 958 347 12 16319  16336 72 347 12 16493  16505 732 347 12 16661  16674 145 349 12 16831  16851
577 353 12 17011  17024 305 353 12 17183  17195 50 359 12 17359  17376 351 359 12 17539  17559 175 367 12
17729  17742 727 367 12 17911  17929 902 367 12 18097  18116 409 373 12 18289  18309 776 373 12 18481  18503
586 379 12 18679  18694 451 379 12 18869  18909 287 383 12 19087  19126 246 389 12 19309  19325 222 389 12
19507  19539 563 397 12 19727  19740 839 397 12 19927  19939 897 401 12 20129  20152 409 401 12 20341  20355
618 409 12 20551  20564 439 409 12 20759  20778 95 419 13 20983  20988 448 419 13 21191  21199 133 419 13
21401  21412 938 419 13 21613  21629 423 431 13 21841  21852 90 431 13 22063  22073 640 431 13 22283  22301
922 433 13 22511  22536 250 439 13 22751  22779 367 439 13 22993  23010 447 443 13 23227  23252 559 449 13
23473  23491 121 457 13 23719  23730 623 457 13 23957  23971 450 457 13 24197  24215 253 461 13 24443  24476
106 467 13 24709  24721 863 467 13 24953  24976 148 479 13 25219  25230 427 479 13 25471  25493 138 479 13
25733  25756 794 487 13 26003  26022 247 487 13 26267  26291 562 491 13 26539  26566 53 499 13 26821  26838
135 499 13 27091  27111 21 503 13 27367  27392 201 509 13 27653  27682 169 521 13 27953  27959 70 521 13
28229  28248 386 521 13 28517  28548 226 523 13 28817  28845 3 541 13 29131  29138 769 541 13 29423  29434
590 541 13 29717  29731 672 541 13 30013  30037 713 547 13 30323  30346 967 547 13 30631  30654 368 557 14
30949  30974 348 557 14 31267  31285 119 563 14 31583  31605 503 569 14 31907  31948 181 571 14 32251  32272
394 577 14 32579  32601 189 587 14 32917  32932 210 587 14 33247  33282 62 593 14 33601  33623 273 593 14
33941  33961 554 599 14 34283  34302 936 607 14 34631  34654 483 607 14 34981  35031 397 613 14 35363  35395
241 619 14 35731  35750 500 631 14 36097  36112 12 631 14 36457  36479 958 641 14 36833  36849 524 641 14
37201  37227 8 643 14 37579  37606 100 653 14 37967  37992 339 653 14 38351  38385 804 659 14 38749  38787
510 673 14 39163  39176 18 673 14 39551  39576 412 677 14 39953  39980 394 683 14 40361  40398 830 691 15
40787  40816 535 701 15 41213  41226 199 701 15 41621  41641 27 709 15 42043  42067 298 709 15 42467  42490
368 719 15 42899  42916 755 727 15 43331  43388 379 727 15 43801  43840 73 733 15 44257  44279 387 739 15
44701  44729 457 751 15 45161  45183 761 751 15 45613  45638 855 757 15 46073  46104 370 769 15 46549  46574
261 769 15 47017  47047 299 787 15 47507  47523 920 787 15 47981  48007 269 787 15 48463  48489 862 797 15
48953  48976 349 809 15 49451  49470 103 809 15 49943  49978 115 821 15 50461  50511 93 821 16 50993  51017
982 827 16 51503  51530 432 839 16 52027  52062 340 853 16 52571  52586 173 853 16 53093  53114 421 857 16
53623  53650 330 863 16 54163  54188 624 877 16 54713  54735 233 877 16 55259  55289 362 883 16 55817  55843
963 907 16 56393  56403 471 907 16 56951
"""
RFC_TABLE2_SHA256 = "0f6ec89eb20470f26ca78c10d6e6fddd953cdab13d9eb9212a9af4fec5d9a7f0"  # sha256 of the rows joined by single spaces, computed at transcription
# Section 5.7.3, OCT_EXP (510 entries) and Section 5.7.4, OCT_LOG (entries for octets 1..255)
RFC_OCT_EXP_TEXT = """
1 2 4 8 16 32 64 128 29 58 116 232 205 135 19 38 76 152 45 90 180 117 234 201 143 3 6 12 24 48 96 192 157 39
78 156 37 74 148 53 106 212 181 119 238 193 159 35 70 140 5 10 20 40 80 160 93 186 105 210 185 111 222 161
95 190 97 194 153 47 94 188 101 202 137 15 30 60 120 240 253 231 211 187 107 214 177 127 254 225 223 163 91
182 113 226 217 175 67 134 17 34 68 136 13 26 52 104 208 189 103 206 129 31 62 124 248 237 199 147 59 118
236 197 151 51 102 204 133 23 46 92 184 109 218 169 79 158 33 66 132 21 42 84 168 77 154 41 82 164 85 170 73
146 57 114 228 213 183 115 230 209 191 99 198 145 63 126 252 229 215 179 123 246 241 255 227 219 171 75 150
49 98 196 149 55 110 220 165 87 174 65 130 25 50 100 200 141 7 14 28 56 112 224 221 167 83 166 81 162 89 178
121 242 249 239 195 155 43 86 172 69 138 9 18 36 72 144 61 122 244 245 247 243 251 235 203 139 11 22 44 88
176 125 250 233 207 131 27 54 108 216 173 71 142 1 2 4 8 16 32 64 128 29 58 116 232 205 135 19 38 76 152 45
90 180 117 234 201 143 3 6 12 24 48 96 192 157 39 78 156 37 74 148 53 106 212 181 119 238 193 159 35 70 140
5 10 20 40 80 160 93 186 105 210 185 111 222 161 95 190 97 194 153 47 94 188 101 202 137 15 30 60 120 240
253 231 211 187 107 214 177 127 254 225 223 163 91 182 113 226 217 175 67 134 17 34 68 136 13 26 52 104 208
189 103 206 129 31 62 124 248 237 199 147 59 118 236 197 151 51 102 204 133 23 46 92 184 109 218 169 79 158
33 66 132 21 42 84 168 77 154 41 82 164 85 170 73 146 57 114 228 213 183 115 230 209 191 99 198 145 63 126
252 229 215 179 123 246 241 255 227 219 171 75 150 49 98 196 149 55 110 220 165 87 174 65 130 25 50 100 200
141 7 14 28 56 112 224 221 167 83 166 81 162 89 178 121 242 249 239 195 155 43 86 172 69 138 9 18 36 72 144
61 122 244 245 247 243 251 235 203 139 11 22 44 88 176 125 250 233 207 131 27 54 108 216 173 71 142
"""
RFC_OCT_LOG_TEXT = """
0 1 25 2 50 26 198 3 223 51 238 27 104 199 75 4 100 224 14 52 141 239 129 28 193 105 248 200 8 76 113 5 138
101 47 225 36 15 33 53 147 142 218 240 18 130 69 29 181 194 125 106 39 249 185 201 154 9 120 77 228 114 166
6 191 139 98 102 221 48 253 226 152 37 179 16 145 34 136 54 208 148 206 143 150 219 189 241 210 19 92 131 56
70 64 30 66 182 163 195 72 126 110 107 58 40 84 250 133 186 61 202 94 155 159 10 21 121 43 78 212 229 172
115 243 167 87 7 112 192 247 140 128 99 13 103 74 222 237 49 197 254 24 227 165 153 119 38 184 180 124 17 68
146 217 35 32 137 46 55 63 209 91 149 188 207 205 144 135 151 178 220 252 190 97 242 86 211 171 20 42 93 158
132 60 57 83 71 109 65 162 31 45 67 216 183 123 164 118 196 23 73 236 127 12 111 246 108 161 59 82 41 157 85
170 251 96 134 177 187 204 62 90 203 89 95 176 156 169 160 81 11 245 22 235 122 117 44 215 79 174 213 233
230 231 173 232 116 214 244 234 168 80 88 175
"""


def _ints(text):
    return [int(x) for x in text.split()]


_t2 = _ints(RFC_TABLE2_TEXT)
RFC_TABLE2 = [tuple(_t2[i:i + 5]) for i in range(0, len(_t2), 5)]
RFC_OCT_EXP = _ints(RFC_OCT_EXP_TEXT)
RFC_OCT_LOG = _ints(RFC_OCT_LOG_TEXT)

# ----------------------------------------------------------------------------------------------
# GF(2)[x] and GF(256) arithmetic (adapted from gf256_facts.py / e_gf.py)
# ----------------------------------------------------------------------------------------------
P_RQ = 0x11D   # x^8+x^4+x^3+x^2+1 (RaptorQ, RAID-6, QR)
P_AES = 0x11B  # x^8+x^4+x^3+x+1   (AES / FIPS-197)


def clmul(a, b):
    r = 0
    while b:
        if b & 1:
            r ^= a
        a <<= 1
        b >>= 1
    return r


def pmod(a, p):
    dp = p.bit_length() - 1
    while a and a.bit_length() - 1 >= dp:
        a ^= p << (a.bit_length() - 1 - dp)
    return a


def gmul(a, b, p=P_RQ):
    return pmod(clmul(a, b), p)


def rust_xtime(a, fold=0x1D):  # the plan's code panel, 05: (a << 1) ^ if a & 0x80 != 0 { 0x1D } else { 0 }  (u8)
    return ((a << 1) & 0xFF) ^ (fold if a & 0x80 else 0)


def rust_mul(a, b, fold=0x1D):  # the plan's code panel, 05: shift-and-add, no tables (10_7 swaps 0x1D for 0x1B)
    acc = 0
    while b:
        if b & 1:
            acc ^= a
        a = rust_xtime(a, fold)
        b >>= 1
    return acc


def order_of(g, p):
    x, k = g, 1
    while x != 1:
        x = gmul(x, g, p)
        k += 1
        if k > 300:
            return None
    return k


def ginv(a, p=P_RQ):
    for b in range(1, 256):
        if gmul(a, b, p) == 1:
            return b
    return None


EXP = []
_x = 1
for _i in range(510):
    EXP.append(_x)
    _x = gmul(_x, 2)
LOG = {EXP[i]: i for i in range(255)}
MUL = [[gmul(a, b) for b in range(256)] for a in range(256)]
INV = [0] + [ginv(a) for a in range(1, 256)]


def rank_gf256_py(M):
    """Pure-Python Gaussian elimination over GF(256) (0x11D); a 0/1 matrix gives its GF(2) rank."""
    A = [list(r) for r in M]
    m, n = len(A), len(A[0])
    r = 0
    for c in range(n):
        piv = next((i for i in range(r, m) if A[i][c]), None)
        if piv is None:
            continue
        A[r], A[piv] = A[piv], A[r]
        s = INV[A[r][c]]
        A[r] = [MUL[s][v] for v in A[r]]
        for i in range(m):
            if i != r and A[i][c]:
                f = A[i][c]
                A[i] = [v ^ MUL[f][w] for v, w in zip(A[i], A[r])]
        r += 1
        if r == m:
            break
    return r


# numpy rank, adapted from gf256np.py (second, independent implementation of the same rank)
_NP_EXP = np.array(EXP + EXP[:20], dtype=np.int32)
_NP_LOG = np.zeros(256, dtype=np.int32)
for _u in range(1, 256):
    _NP_LOG[_u] = LOG[_u]
_NP_MUL = np.zeros((256, 256), dtype=np.uint8)
for _u in range(1, 256):
    _NP_MUL[_u, 1:] = _NP_EXP[_NP_LOG[_u] + _NP_LOG[1:]]
_NP_INV = np.zeros(256, dtype=np.uint8)
for _u in range(1, 256):
    _NP_INV[_u] = _NP_EXP[255 - _NP_LOG[_u]]


def rank256_np(M):
    A = np.array(M, dtype=np.uint8).copy()
    m, n = A.shape
    r = 0
    for c in range(n):
        if r == m:
            break
        nz = np.nonzero(A[r:, c])[0]
        if nz.size == 0:
            continue
        p = r + nz[0]
        if p != r:
            A[[r, p]] = A[[p, r]]
        A[r] = _NP_MUL[_NP_INV[A[r, c]]][A[r]]
        rows = np.nonzero(A[:, c])[0]
        rows = rows[rows != r]
        if rows.size:
            f = A[rows, c]
            A[rows] ^= _NP_MUL[f[:, None], A[r][None, :]]
        r += 1
    return r


def rank_gf2(rows_bits):
    """GF(2) rank of 0/1 rows (lists)."""
    m = [list(r) for r in rows_bits]
    r = 0
    ncols = len(m[0])
    for c in range(ncols):
        piv = next((i for i in range(r, len(m)) if m[i][c] & 1), None)
        if piv is None:
            continue
        m[r], m[piv] = m[piv], m[r]
        for i in range(len(m)):
            if i != r and m[i][c] & 1:
                m[i] = [(x ^ y) & 1 for x, y in zip(m[i], m[r])]
        r += 1
    return r


# ----------------------------------------------------------------------------------------------
# Exact random-matrix rank probabilities (adapted from rank_probs.py, exact via Fraction)
# ----------------------------------------------------------------------------------------------
def p_full_finite(q, k, h=0):
    """P(k unknowns, k+h uniform random equations over GF(q) have rank k) = prod_{i=h+1}^{k+h} (1-q^-i)."""
    p = Fraction(1)
    for i in range(h + 1, k + h + 1):
        p *= 1 - Fraction(1, q ** i)
    return p


def p_full_limit(q, h=0):
    """k -> infinity: prod_{i>h} (1-q^-i), truncated at i = h+150 for q=2 and h+20 for q=256.
    Truncation: the omitted factor lies in [1 - sum_{i>N} q^-i, 1], i.e. relative error < 2^-149 (q=2),
    256^-19 (q=256), far below double precision."""
    N = h + (150 if q == 2 else 20)
    p = Fraction(1)
    for i in range(h + 1, N + 1):
        p *= 1 - Fraction(1, q ** i)
    return p


def mean_spares(q, N=None):
    """sum_{i>=1} 1/(q^i - 1); truncated at N terms, tail < 2 q^-N."""
    N = N or (160 if q == 2 else 20)
    return sum(Fraction(1, q ** i - 1) for i in range(1, N + 1))


def recorded_check(fname, pattern, expected):
    """Re-read a recorded output file (if present) and compare the pinned constants with it."""
    path = os.path.join(RESULTS, fname)
    if not os.path.exists(path):
        return "file-missing"
    txt = open(path, encoding="utf-8").read()
    found = [tuple(int(g) for g in m) for m in re.findall(pattern, txt)]
    return "match" if found == expected else f"MISMATCH:{found}"


# ==============================================================================================
# B-F1  sticker album (coupon collector), K = 10,000            (misc_facts.py; integration_check 6)
# ==============================================================================================
def bf1():
    K = 10000
    H = sum(Fraction(1, i) for i in range(1, K + 1))
    KH = K * H
    emit("B-F1.K", K)
    emit("B-F1.K_times_H_K", KH)
    emit("B-F1.K_times_H_K_floor", KH.numerator // KH.denominator)
    emit("B-F1.ratio_H_K", H)
    # 01_8 "the last few take forever": with j pieces still missing, the next new one takes K/j draws on average,
    # so the last piece alone takes K draws and the last ten take K*H_10; the first half takes K*(H_K - H_{K/2}).
    H10 = sum(Fraction(1, i) for i in range(1, 11))
    emit("B-F1.draws_for_last_piece", Fraction(K, 1))
    emit("B-F1.draws_for_last_10_pieces", K * H10)
    emit("B-F1.last_10_pieces_share_of_all_draws", H10 / H)
    emit("B-F1.draws_for_first_half_of_pieces", K * sum(Fraction(1, i) for i in range(K // 2 + 1, K + 1)))
    # 01_8 screen: the count above is packets caught (each caught packet is one random piece); at 1% loss the
    # expected packets sent is caught / 0.99 (Wald's identity: whether packet n arrives is independent of n <= N).
    emit("B-F1.K_times_H_K_packets_sent_at_1pct_loss", KH / Fraction(99, 100))


# ==============================================================================================
# B-F2  misses per packet at 1% loss, 10^6 phones                (arithmetic)
# ==============================================================================================
def bf2():
    n, p = 10 ** 6, Fraction(1, 100)
    emit("B-F2.expected_misses_per_packet", n * p)
    emit("B-F2.sd_misses_per_packet", math.sqrt(float(n * p * (1 - p))))
    emit("B-F2.ln_P_packet_reaches_all", n * math.log(0.99))
    # 01_1, 01_5: the hook's model, as said (a million phones, one packet in a hundred lost)
    emit("B-F2.phones", n)
    emit("B-F2.loss_one_in", 1 / p)


# ==============================================================================================
# B-F3  XOR example A=5, B=3, C=7, D=2                            (toy_examples.py)
# ==============================================================================================
def bf3():
    A, B, C, D = 5, 3, 7, 2
    cards = [(1, 0, 1, 0), (0, 1, 1, 1), (1, 1, 0, 0), (1, 1, 1, 1)]
    vals = [A ^ C, B ^ C ^ D, A ^ B, A ^ B ^ C ^ D]
    emit("B-F3.A_xor_C", vals[0])
    emit("B-F3.B_xor_C_xor_D", vals[1])
    emit("B-F3.A_xor_B", vals[2])
    emit("B-F3.A_xor_B_xor_C_xor_D", vals[3])
    emit("B-F3.rank_gf2_4_cards", rank_gf2(cards))
    emit("B-F3.B_xor_C", B ^ C)
    emit("B-F3.card1_xor_card3", vals[0] ^ vals[2])
    emit("B-F3.rank_gf2_with_5th_card_BxC", rank_gf2(cards + [(0, 1, 1, 0)]))
    emit("B-F3.D_from_card2_and_BxC", vals[1] ^ (B ^ C))


# ==============================================================================================
# B-F4  GF(2) square success probability                          (rank_probs.py)
# ==============================================================================================
def bf4():
    for k in (1, 2, 3):
        emit(f"B-F4.success_K{k}", p_full_finite(2, k))
    emit("B-F4.success_K3_check_7_8x6_8x4_8", Fraction(7, 8) * Fraction(6, 8) * Fraction(4, 8))
    emit("B-F4.success_limit", p_full_limit(2, 0))
    emit("B-F4.useless_prob_3pieces_2cards_held", Fraction(4, 8))
    # 05_2 screen: three random cards over three pieces fail 1 - 21/64 of the time
    emit("B-F4.fail_K3", 1 - p_full_finite(2, 3))


# ==============================================================================================
# B-F5  GF(2) failure at K, K+1, K+2, K+3 (K -> infinity)         (rank_probs.py)
# ==============================================================================================
def bf5():
    for h in range(4):
        emit(f"B-F5.fail_gf2_h{h}", 1 - p_full_limit(2, h))
    emit("B-F5.fail_gf2_h0_K10", 1 - p_full_finite(2, 10, 0))
    f = [1 - p_full_limit(2, h) for h in range(5)]
    emit("B-F5.ratio_fail_h0_over_h1", f[0] / f[1])
    emit("B-F5.ratio_fail_h3_over_h4", f[3] / f[4])
    # 05_3-05_4, the danger meter on a ten-slot bucket: holding r useful ex-or cards of ten, a uniform card is
    # useless (one the bucket can already make) with probability 2^r / 2^10
    emit("B-F5.danger_meter_K10", [Fraction(2 ** r, 2 ** 10) for r in range(10)])


# ==============================================================================================
# B-F6  average spares (expected extra equations, K -> infinity)  (rank_probs.py; lt_peel_sim.py)
# ==============================================================================================
def bf6():
    for q in (2, 256):
        s = mean_spares(q)
        alt = sum(1 - p_full_limit(q, h) for h in range(0, 200 if q == 2 else 25))
        emit(f"B-F6.mean_spares_gf{q}", s)
        emit(f"B-F6.mean_spares_gf{q}_two_routes_ok", abs(float(s) - float(alt)) < 1e-15)
        # Erdos 1948 (J. Indian Math. Soc. 12, 63-66) proves f(1/t) = sum x^n/(1-x^n) at x = 1/t irrational for
        # every integer |t| > 1, via the Lambert-series identity sum 1/(t^n - 1) = sum_r d(r)/t^r (d = # divisors).
        ndiv = [0] * 241
        for a in range(1, 241):
            for b in range(a, 241, a):
                ndiv[b] += 1
        lam = sum(Fraction(ndiv[r], q ** r) for r in range(1, 241))   # tail < sum_{r>240} r/q^r < 2^-230
        emit(f"B-F6.erdos_lambert_identity_gf{q}_ok", abs(float(s) - float(lam)) < 1e-15)
    # 13_6: the race is on ten pieces, so its exact means are the finite sums sum_{i=1..10} 1/(q^i - 1)
    # (holding i useful cards, a uniform card is useful with probability 1 - q^(i-K)). Second route: the sum over h
    # of P(K + h uniform cards leave a piece unpinned), truncated at h = 200 (q = 2; tail < 10 * 2^-200) or 25.
    K = 10
    for q in (2, 256):
        s = mean_spares(q, K)
        alt = sum(1 - p_full_finite(q, K, h) for h in range(0, 200 if q == 2 else 25))
        emit(f"B-F6.mean_spares_gf{q}_K10", s)
        emit(f"B-F6.mean_spares_gf{q}_K10_two_routes_ok", abs(float(s) - float(alt)) < 1e-15)
    emit("B-F6.p_any_spare_needed_gf256_K10", 1 - p_full_finite(256, K, 0))
    # R2 (13_6 lower-third, Ruth R7): with ten pieces the sum stops at its tenth term, 1/(2^10 - 1)
    emit("B-F6.last_term_denominator_K10", 2 ** K - 1)
    race_k10()


def race_k10():
    """13_6: the race itself, seeded. Uniform random cards over GF(2)^10 and GF(256)^10 (the zero card
    included, as in the model) are added until the cards pin down all ten pieces, a thousand times per side;
    the spares are the cards beyond ten. The rank is tracked incrementally: a GF(2) basis by leading bit,
    and GF(256) rows reduced to 1 at their pivots."""
    rng = random.Random(1306)   # ex-or races first, then byte races, from one stream
    K, T = 10, 1000

    def race_gf2():
        basis, draws = {}, 0
        while len(basis) < K:
            v = rng.getrandbits(K)
            draws += 1
            while v:
                hb = v.bit_length() - 1
                if hb not in basis:
                    basis[hb] = v
                    break
                v ^= basis[hb]
        return draws - K

    def race_gf256():
        rows, draws = {}, 0
        while len(rows) < K:
            v = [rng.randrange(256) for _ in range(K)]
            draws += 1
            for c, r in rows.items():
                if v[c]:
                    m = MUL[v[c]]
                    v = [x ^ m[y] for x, y in zip(v, r)]
            c = next((i for i, x in enumerate(v) if x), None)
            if c is not None:
                s = MUL[INV[v[c]]]
                rows[c] = [s[x] for x in v]
        return draws - K

    emit("B-F6.race_K10_seeded.seed", 1306)
    emit("B-F6.race_K10_seeded.races_per_side", T)
    within = True
    for q, run in ((2, race_gf2), (256, race_gf256)):
        sp = [run() for _ in range(T)]
        mean = Fraction(sum(sp), T)
        emit(f"B-F6.race_K10_seeded.gf{q}.spares_histogram_0_up", [sp.count(h) for h in range(max(sp) + 1)])
        emit(f"B-F6.race_K10_seeded.gf{q}.mean_spares", mean)
        var = sum((x - mean) ** 2 for x in sp) / (T - 1)
        within &= abs(mean - mean_spares(q, K)) <= 4 * math.sqrt(float(var) / T)
    emit("B-F6.race_K10_seeded.means_within_4se_of_exact_ok", within)


# ==============================================================================================
# B-F7  GF(256) success (model)                                   (rank_probs.py)
# ==============================================================================================
def bf7():
    seq = [p_full_finite(256, k) for k in range(1, 41)]
    emit("B-F7.success_K1", seq[0])
    emit("B-F7.success_limit", p_full_limit(256, 0))
    emit("B-F7.decreasing_in_K_ok", all(seq[i] > seq[i + 1] for i in range(len(seq) - 1)))
    emit("B-F7.min_over_K_ge_0.996078_ok", p_full_limit(256, 0) >= Fraction(996078, 10 ** 6))
    # R2 (13_4 "Fix every amount but one ... exactly one choice of that amount lands in the lit part"; Ruth R18,
    # D1; Dev dev-12). Exhaustive at two pieces: the lit part at the last slot is a line through 0 in GF(256)^2
    # (257 of them). For a piece whose own card is not on the line, fix the other amount: exactly one of the 256
    # values of this amount puts the card on the line.
    s = np.arange(256)
    lines = [(1, t) for t in range(256)] + [(0, 1)]
    ok = True
    for u0, u1 in lines:
        lit = np.zeros((256, 256), dtype=bool)          # lit[c1, c2]: the card (c1, c2) is on the line
        lit[_NP_MUL[s, u0], _NP_MUL[s, u1]] = True
        if u0:   # piece 2's own card (0, 1) is off the line: fix c1, vary c2
            ok &= bool((lit.sum(axis=1) == 1).all())
        if u1:   # piece 1's own card (1, 0) is off the line: fix c2, vary c1
            ok &= bool((lit.sum(axis=0) == 1).all())
    emit("B-F7.last_slot_lit_lines_checked_K2", len(lines))
    emit("B-F7.last_slot_exactly_one_amount_lit_exhaustive_K2_ok", ok)


# ==============================================================================================
# B-F8  GF(256) failure (model) at K, K+1, K+2 (K -> infinity)    (rank_probs.py)
# ==============================================================================================
def bf8():
    for h in range(3):
        emit(f"B-F8.fail_gf256_h{h}", 1 - p_full_limit(256, h))


# ==============================================================================================
# B-F9  pentagon                                                   (toy_examples.py; e_gf.py)
# ==============================================================================================
def bf9():
    A, B, C, D, E = 5, 3, 7, 2, 12
    cyc = [[0] * 5 for _ in range(5)]
    for i in range(5):
        cyc[i][i] = 1
        cyc[i][(i + 1) % 5] = 1
    emit("B-F9.rank_gf2", rank_gf2(cyc))
    emit("B-F9.rank_gf256_same_01_matrix", rank_gf256_py(cyc))
    tw = [row[:] for row in cyc]
    tw[4][0] = 2                                      # last card E + 2A
    r_py, r_np = rank_gf256_py(tw), rank256_np(tw)
    emit("B-F9.rank_gf256_with_E_plus_2A", r_py)
    emit("B-F9.rank_two_implementations_ok", r_py == r_np and rank256_np(cyc) == rank_gf256_py(cyc))
    y = [A ^ B, B ^ C, C ^ D, D ^ E, E ^ gmul(2, A)]
    emit("B-F9.card_values", y)
    S = y[0] ^ y[1] ^ y[2] ^ y[3]
    emit("B-F9.S_xor_of_first_four", S)
    emit("B-F9.y5_xor_S_equals_3A", y[4] ^ S)
    emit("B-F9.one_xor_two", 1 ^ 2)
    emit("B-F9.A_solved_15_div_3", gmul(y[4] ^ S, INV[3]))
    emit("B-F9.three_times_five", gmul(3, 5))
    # 16_5 lower-third: a loop of two-piece cards a_i x_i + b_i x_(i+1) (five cards round five pieces) is stuck
    # exactly when prod b_i / a_i = 1 going around. Seeded random nonzero amounts; both branches occur.
    rng = random.Random(1605)
    agree = stuck = 0
    for _ in range(3000):
        a = [rng.randrange(1, 256) for _ in range(5)]
        b = [rng.randrange(1, 256) for _ in range(5)]
        M = [[0] * 5 for _ in range(5)]
        prod = 1
        for i in range(5):
            M[i][i], M[i][(i + 1) % 5] = a[i], b[i]
            prod = gmul(prod, gmul(b[i], INV[a[i]]))
        singular = rank_gf256_py(M) < 5
        stuck += singular
        agree += singular == (prod == 1)
    emit("B-F9.cycle_rule_random_5cycles", 3000)
    emit("B-F9.cycle_rule_random_5cycles_stuck", stuck)
    emit("B-F9.cycle_rule_stuck_iff_prod_b_over_a_is_1_ok", agree == 3000 and stuck > 0)
    # 13_7: the amounts other than 0 and 1
    emit("B-F9.amounts_other_than_0_and_1", 256 - 2)
    # R2 (Ruth R3): the R1 wording "stuck exactly when its amounts multiply to 1" is false; the ring of cards
    # 2X + 2Y is stuck (rank 4) although its ten amounts multiply to 2^10 = 116. The ratio rule holds (ratios 1).
    ring2 = [[0] * 5 for _ in range(5)]
    for i in range(5):
        ring2[i][i], ring2[i][(i + 1) % 5] = 2, 2
    emit("B-F9.all_2_ring_rank_gf256", rank_gf256_py(ring2))
    emit("B-F9.all_2_ring_product_of_amounts", pow_g(2, 10, P_RQ))
    emit("B-F9.all_2_ring_product_of_ratios", pow_g(gmul(2, INV[2]), 5, P_RQ))
    # R2 (13_7, Ruth R18): a matrix of 0/1 amounts has the same rank over GF(2) and over GF(256), so a card
    # with only zeros and ones gains nothing from the byte field (300 seeded random 6 x 6 matrices)
    rng = random.Random(1307)
    same = True
    for _ in range(300):
        M = [[rng.randrange(2) for _ in range(6)] for _ in range(6)]
        same &= rank_gf2(M) == rank_gf256_py(M) == rank256_np(M)
    emit("B-F9.rank_01_matrices_same_over_gf2_and_gf256_300_random_ok", same)


# ==============================================================================================
# B-F10  Z/256 facts                                               (gf256_facts.py; integration_check 7)
# ==============================================================================================
def bf10():
    emit("B-F10.z256_2x128", (2 * 128) % 256)
    emit("B-F10.z256_2x3", (2 * 3) % 256)
    emit("B-F10.z256_2x131", (2 * 131) % 256)
    emit("B-F10.z256_units", sum(1 for a in range(256) if math.gcd(a, 256) == 1))
    emit("B-F10.z256_nonzero_nonunits", sum(1 for a in range(1, 256) if math.gcd(a, 256) != 1))
    emit("B-F10.z256_2_has_inverse", any((2 * b) % 256 == 1 for b in range(256)))
    ok = all(len({a * b % 256 for b in range(256)}) == 256 // math.gcd(a, 256) for a in range(256))
    emit("B-F10.row_a_has_256_over_gcd_values_all_a_ok", ok)
    emit("B-F10.row2_distinct", len({2 * b % 256 for b in range(256)}))
    emit("B-F10.row128_distinct", len({128 * b % 256 for b in range(256)}))
    # 07_4 screen "2·131 = 262 → 6"; 07_6 lower-third "3 × 171 = 513 = 2·256 + 1" (an odd multiplier undoes)
    emit("B-F10.int_2x131", 2 * 131)
    emit("B-F10.z256_inverse_of_3", pow(3, -1, 256))
    emit("B-F10.int_3x171", 3 * 171)
    emit("B-F10.int_3x171_divmod_256", list(divmod(3 * 171, 256)))
    # R2 (07_6 lower-third "hash mixers use odd multipliers", Dev dev-14): Linux include/linux/hash.h "multiplies
    # the input by a large odd number" (SOURCED constants, B-S10); odd, so multiplying by them is undone mod 2^w
    g32, g64 = 0x61C88647, 0x61C8864680B583EB   # SOURCED: GOLDEN_RATIO_32 and GOLDEN_RATIO_64 in hash.h
    emit("B-F10.sourced.linux_hash_h_GOLDEN_RATIO_32", g32)
    emit("B-F10.sourced.linux_hash_h_GOLDEN_RATIO_64", g64)
    emit("B-F10.linux_golden_ratios_odd_and_invertible_mod_2^w_ok",
         g32 % 2 == 1 and g64 % 2 == 1 and g32 * pow(g32, -1, 2 ** 32) % 2 ** 32 == 1
         and g64 * pow(g64, -1, 2 ** 64) % 2 ** 64 == 1)


# ==============================================================================================
# B-F11  GF(256) products (0x11D unless stated)                   (gf256_facts.py, all 65,536 pairs)
# ==============================================================================================
def bf11():
    for a, b in ((3, 7), (2, 128), (128, 3), (128, 4), (150, 3), (7, 7), (2, 142), (3, 244), (3, 5)):
        emit(f"B-F11.mul_{a}_{b}", gmul(a, b))
    emit("B-F11.mul_0x57_0x83_0x11D", gmul(0x57, 0x83, P_RQ))
    emit("B-F11.mul_0x57_0x83_0x11B_AES", gmul(0x57, 0x83, P_AES))
    emit("B-F11.inv_2", INV[2])
    emit("B-F11.inv_3", INV[3])
    emit("B-F11.inv_2_under_0x11B", ginv(2, P_AES))
    emit("B-F11.foil_3x7_carryless", clmul(3, 7))            # (x+1)(x^2+x+1) = x^3+1, no reduction
    emit("B-F11.fold_x8_residue", 0x11D ^ 0x100)              # x^8 -> x^4+x^3+x^2+1 = 16+8+4+1
    # all 65,536 pairs, three routes: shift-and-add + reduction, RFC OCT_EXP/OCT_LOG tables, plan's Rust code
    def tmul(u, v):
        return 0 if u == 0 or v == 0 else RFC_OCT_EXP[RFC_OCT_LOG[u - 1] + RFC_OCT_LOG[v - 1]]
    ok_t = all(tmul(u, v) == MUL[u][v] for u in range(256) for v in range(256))
    ok_r = all(rust_mul(u, v) == MUL[u][v] for u in range(256) for v in range(256))
    emit("B-F11.rfc_table_mul_equals_poly_mul_65536_ok", ok_t)
    emit("B-F11.plan_rust_mul_equals_poly_mul_65536_ok", ok_r)
    emit("B-F11.plan_rust_mul_0x57_0x83", rust_mul(0x57, 0x83))
    emit("B-F11.plan_rust_mul_2_142", rust_mul(2, 142))
    emit("B-F11.nonzero_rows_are_permutations_ok", all(len(set(MUL[a])) == 256 for a in range(1, 256)))
    # 10_7 code panel: 0x1D swaps to 0x1B, so the comments must flip too (AES's inverse of 2 is 141, not 142)
    emit("B-F11.mul_2_141_0x11B_AES", gmul(2, 141, P_AES))
    emit("B-F11.mul_2_142_0x11B_AES", gmul(2, 142, P_AES))
    emit("B-F11.plan_rust_mul_2_141_fold_0x1B", rust_mul(2, 141, 0x1B))
    emit("B-F11.plan_rust_mul_0x57_0x83_fold_0x1B", rust_mul(0x57, 0x83, 0x1B))
    emit("B-F11.plan_rust_mul_fold_0x1B_equals_poly_mul_0x11B_65536_ok",
         all(rust_mul(u, v, 0x1B) == gmul(u, v, P_AES) for u in range(256) for v in range(256)))
    # 10_5 "two times one twenty-eight is x to the eighth" (before the fold); 12_4 lower-third "by hand
    # (x²+x+1)² = x⁴+x²+1 = 21" (carry-less: the cross terms cancel in pairs, no fold needed)
    emit("B-F11.clmul_2_128_is_x8", clmul(2, 128))
    emit("B-F11.clmul_7_7_no_fold", clmul(7, 7))


# ==============================================================================================
# B-F12  irreducible / primitive degree-8 polynomials over GF(2)  (gf256_facts.py)
# ==============================================================================================
def bf12():
    def is_irred(p):
        d = p.bit_length() - 1
        return all(pmod(p, q) != 0 for q in range(2, 1 << (d // 2 + 1)) if 1 <= q.bit_length() - 1 <= d // 2)
    irr = [p for p in range(256, 512) if is_irred(p)]
    prim = [p for p in irr if order_of(2, p) == 255]
    emit("B-F12.n_irreducible_deg8", len(irr))
    emit("B-F12.n_primitive_deg8", len(prim))
    emit("B-F12.formula_irreducible_(2^8-2^4)/8", (2 ** 8 - 2 ** 4) // 8)
    phi255 = sum(1 for k in range(1, 256) if math.gcd(k, 255) == 1)
    emit("B-F12.formula_primitive_phi(255)/8", phi255 // 8)
    emit("B-F12.0x11D_irreducible", P_RQ in irr)
    emit("B-F12.0x11D_primitive", P_RQ in prim)
    emit("B-F12.0x11B_irreducible", P_AES in irr)
    emit("B-F12.0x11B_primitive", P_AES in prim)
    emit("B-F12.ord2_under_0x11B", order_of(2, P_AES))
    emit("B-F12.ord3_under_0x11B", order_of(3, P_AES))
    emit("B-F12.ord2_under_0x11D", order_of(2, P_RQ))
    hist = {}
    for g in range(1, 256):
        o = order_of(g, P_RQ)
        hist[o] = hist.get(o, 0) + 1
    emit("B-F12.element_orders_0x11D", sorted(hist))
    emit("B-F12.element_order_counts_0x11D", [hist[o] for o in sorted(hist)])
    # 12_4 lower-third: "255 = (2+1)(2²+1)(2⁴+1); together they make 11111111₂ = 255"
    emit("B-F12.product_(2+1)(2^2+1)(2^4+1)", (2 + 1) * (2 ** 2 + 1) * (2 ** 4 + 1))
    emit("B-F12.sum_element_order_counts_0x11D", sum(hist.values()))
    emit("B-F12.binary_11111111", int("11111111", 2))


# ==============================================================================================
# B-F13  discrete logs base 2 (0x11D)                              (gf256_facts.py)
# ==============================================================================================
def bf13():
    for u in (3, 29, 128, 150, 4, 58):
        emit(f"B-F13.log_{u}", LOG[u])
    emit("B-F13.log150_plus_log3", LOG[150] + LOG[3])
    emit("B-F13.exp_205", EXP[205])
    emit("B-F13.mul_150_3_via_logs_equals_direct_ok", EXP[(LOG[150] + LOG[3]) % 255] == gmul(150, 3))
    emit("B-F13.exp_7_plus_2", EXP[7 + 2])
    emit("B-F13.log_ordered_table_is_hankel_ok",
         all(MUL[EXP[i]][EXP[j]] == EXP[(i + j) % 255] for i in range(255) for j in range(255)))
    # 12_4 lower-third "7 × 7: 198 + 198 = 396 → 141 clicks → 21"
    emit("B-F13.log_7", LOG[7])
    emit("B-F13.log7_plus_log7", LOG[7] + LOG[7])
    emit("B-F13.log7_plus_log7_mod_255", (LOG[7] + LOG[7]) % 255)
    emit("B-F13.exp_141", EXP[141])
    # 11_7 picture: each byte's partner sits at its mirror image, log a^-1 = 255 - log a (1 is its own mirror)
    emit("B-F13.log_inverse_is_255_minus_log_all255_ok",
         all(LOG[INV[a]] == (255 - LOG[a]) % 255 for a in range(1, 256)))
    emit("B-F13.mirror_pairs_a_ne_inverse", len({frozenset((a, INV[a])) for a in range(1, 256) if INV[a] != a}))
    emit("B-F13.bytes_that_are_their_own_inverse", [a for a in range(1, 256) if INV[a] == a])
    # 11_5 "two cubed times two to the fourth is two to the seventh", as integers and as bytes
    emit("B-F13.int_2^3_x_2^4", 2 ** 3 * 2 ** 4)
    emit("B-F13.gf256_8_x_16", gmul(8, 16))
    # R2 (11_7 table-multiply lower-third, Dev dev-03): EXP needs 510 entries (largest index 254 + 254 = 508);
    # with LOG stored as u8, LOG[a] + LOG[b] wraps at 256 for these nonzero pairs (debug panics, release is wrong)
    sums = [LOG[a] + LOG[b] for a in range(1, 256) for b in range(1, 256)]
    emit("B-F13.nonzero_pairs", len(sums))
    emit("B-F13.max_log_sum_nonzero_pairs", max(sums))
    emit("B-F13.nonzero_pairs_log_sum_ge_256_u8_overflow", sum(1 for x in sums if x >= 256))
    emit("B-F13.table_mul_u16_logs_equals_poly_mul_65536_ok",
         all((0 if a == 0 or b == 0 else EXP[(LOG[a] + LOG[b]) & 0xFFFF]) == MUL[a][b]
             for a in range(256) for b in range(256)))


# ==============================================================================================
# B-F14  RFC 6330 OCT_EXP / OCT_LOG = powers of x mod 0x11D        (gf256_facts.py --rfc)
# ==============================================================================================
def bf14():
    emit("B-F14.oct_exp_0_to_12_computed", EXP[:13])
    emit("B-F14.rfc_oct_exp_0_to_12", RFC_OCT_EXP[:13])
    emit("B-F14.rfc_oct_exp_len", len(RFC_OCT_EXP))
    emit("B-F14.rfc_oct_exp_equals_powers_of_x_mod_0x11D_all510_ok", RFC_OCT_EXP == EXP)
    emit("B-F14.rfc_oct_log_equals_discrete_log_all255_ok",
         len(RFC_OCT_LOG) == 255 and all(RFC_OCT_LOG[u - 1] == LOG[u] for u in range(1, 256)))
    emit("B-F14.distinct_powers_of_2_in_255_steps_0x11D", len(set(EXP[:255])))
    emit("B-F14.xtime_equals_mul2_all_bytes_ok", all(rust_xtime(a) == gmul(a, 2) for a in range(256)))


# ==============================================================================================
# B-F15  dense elimination cost                                    (arithmetic)
# ==============================================================================================
def bf15():
    K = 50000
    emit("B-F15.K_cubed", K ** 3)
    emit("B-F15.K_cubed_over_3", Fraction(K ** 3, 3))
    # 14_1 "about forty-two trillion steps. At a billion a second, half a day." (screen "≈ 11.6 h at 10⁹/s")
    steps = Fraction(K ** 3, 3)
    emit("B-F15.K_cubed_over_3_in_trillions", steps / 10 ** 12)
    emit("B-F15.seconds_at_1e9_steps_per_s", steps / 10 ** 9)
    emit("B-F15.hours_at_1e9_steps_per_s", steps / 10 ** 9 / 3600)
    emit("B-F15.days_at_1e9_steps_per_s", steps / 10 ** 9 / 86400)
    # R2 (14_1 lower-third, Dev dev-09): one byte per amount; ten times faster; the cube when the pieces double
    emit("B-F15.matrix_bytes_K50000", K * K)
    emit("B-F15.hours_at_1e10_steps_per_s", steps / 10 ** 10 / 3600)
    emit("B-F15.work_ratio_when_pieces_double", Fraction((2 * K) ** 3, K ** 3))


# ==============================================================================================
# B-F16  RFC degree table (Table 1)                                (misc_facts.py)
# ==============================================================================================
def _degree_masses():
    N = 1 << 20
    return {d: Fraction(RFC_TABLE1[d] - RFC_TABLE1[d - 1], N) for d in range(1, 31)}


def bf16():
    N = 1 << 20
    p = _degree_masses()
    emit("B-F16.P_deg1", p[1])
    emit("B-F16.P_deg2", p[2])
    dev = [abs((RFC_TABLE1[d] - RFC_TABLE1[d - 1]) - Fraction(N, d * (d - 1))) for d in range(2, 30)]
    emit("B-F16.max_abs_dev_from_ideal_soliton_d2_29_units_2^-20", max(dev))
    rel = [abs(Fraction(RFC_TABLE1[d] - RFC_TABLE1[d - 1]) / Fraction(N, d * (d - 1)) - 1) for d in range(2, 30)]
    emit("B-F16.max_rel_dev_from_ideal_soliton_d2_29", max(rel))
    emit("B-F16.P_deg30_tail", p[30])
    mean = sum(d * p[d] for d in p)
    emit("B-F16.mean_degree", mean)
    pd_lt4 = p[1] + p[2] + p[3]
    e_d1 = 2 + Fraction(1, 2) * pd_lt4    # d1 = 2 + Rand[X,3,2] if d < 4 (Rand modelled as uniform 0/1)
    emit("B-F16.E_d1_PI_symbols", e_d1)
    emit("B-F16.mean_symbols_per_repair", mean + e_d1)


# ==============================================================================================
# B-F17  never-touched pieces, plain sparse code with N = K       (arithmetic)
# ==============================================================================================
def bf17():
    mean = float(sum(d * w for d, w in _degree_masses().items()))
    emit("B-F17.exp_minus_mean_degree", math.exp(-mean))
    K = 10000
    emit("B-F17.finite_K10000_(1-mean/K)^K", (1 - mean / K) ** K)
    # R2 (14_7 "On average, about seventy of them aren't in any card", Ruth R6, Dev dev-04): at 2% extra
    # (N = 10,200 cards), each card of degree d misses a given piece with probability 1 - d/K, independently
    emit("B-F17.one_in_untouched_exp_N_eq_K", 1 / math.exp(-mean))
    frac = (1 - mean / K) ** 10200
    emit("B-F17.finite_K10000_N10200_fraction_untouched", frac)
    emit("B-F17.expected_untouched_K10000_N10200", K * frac)
    emit("B-F17.one_in_untouched_K10000_N10200", 1 / frac)


# ==============================================================================================
# B-F18  pure LT peeling, RFC degrees, K = 10,000, seeded          (lt_peel_sim.py, same RNG stream)
# ==============================================================================================
def bf18():
    F = RFC_TABLE1

    def deg(rng):
        return bisect.bisect_right(F, rng.randrange(1 << 20))

    def peel(K, N, rng):
        nb = []
        adj = [[] for _ in range(K)]
        for j in range(N):
            d = deg(rng)
            s = rng.sample(range(K), d)
            nb.append(set(s))
            for i in s:
                adj[i].append(j)
        known = [False] * K
        rip = [j for j in range(N) if len(nb[j]) == 1]
        rec = 0
        while rip:
            j = rip.pop()
            if len(nb[j]) != 1:
                continue
            i = next(iter(nb[j]))
            if known[i]:
                continue
            known[i] = True
            rec += 1
            for jj in adj[i]:
                if i in nb[jj]:
                    nb[jj].discard(i)
                    if len(nb[jj]) == 1:
                        rip.append(jj)
        return Fraction(rec, K)

    rng = random.Random(42)   # lt_peel_sim.py seed; eps order 0.00, 0.02, 0.05 matches its first three
    K = 10000
    for tag, eps in (("eps000", 0.0), ("eps002", 0.02), ("eps005", 0.05)):
        res = [peel(K, int(K * (1 + eps)), rng) for _ in range(5)]
        emit(f"B-F18.{tag}.recovered_fractions_5runs", res)
        ok = [r for r in res if r >= Fraction(1, 2)]
        emit(f"B-F18.{tag}.n_collapsed_below_50pct", len(res) - len(ok))
        emit(f"B-F18.{tag}.min_recovered", min(res))
        emit(f"B-F18.{tag}.max_recovered", max(res))
        if ok and len(ok) < len(res):
            emit(f"B-F18.{tag}.min_recovered_non_collapsed", min(ok))
        # 14_6 "over a hundred pieces are missing": the pieces left when the runs that did not collapse stall
        if tag != "eps000":
            missing = [K - r * K for r in ok]
            emit(f"B-F18.{tag}.missing_pieces_non_collapsed_min", min(missing))
            emit(f"B-F18.{tag}.missing_pieces_non_collapsed_max", max(missing))


# ==============================================================================================
# B-F19  RFC 6330 Table 2 parameters                               (misc_facts.py --rfc)
# ==============================================================================================
def bf19():
    flat = " ".join(" ".join(map(str, r)) for r in RFC_TABLE2)
    emit("B-F19.table2_transcription_hash_ok", hashlib.sha256(flat.encode()).hexdigest() == RFC_TABLE2_SHA256)

    def is_prime(n):
        return n >= 2 and all(n % i for i in range(2, int(math.isqrt(n)) + 1))
    emit("B-F19.n_rows", len(RFC_TABLE2))
    emit("B-F19.Kprime_strictly_increasing_ok", all(RFC_TABLE2[i][0] < RFC_TABLE2[i + 1][0]
                                                     for i in range(len(RFC_TABLE2) - 1)))
    emit("B-F19.S_and_W_prime_all_rows_ok", all(is_prime(r[2]) and is_prime(r[4]) for r in RFC_TABLE2))
    Hs = [r[3] for r in RFC_TABLE2]
    emit("B-F19.H_min", min(Hs))
    emit("B-F19.H_max", max(Hs))
    emit("B-F19.Kprime_min", RFC_TABLE2[0][0])
    emit("B-F19.Kprime_max", RFC_TABLE2[-1][0])
    tab = {r[0]: r for r in RFC_TABLE2}
    for Kp in (10, 10017, 56403):
        _, J, S, H, W = tab[Kp]
        L = Kp + S + H
        emit(f"B-F19.K{Kp}.S", S)
        emit(f"B-F19.K{Kp}.H", H)
        emit(f"B-F19.K{Kp}.W", W)
        emit(f"B-F19.K{Kp}.L", L)
        emit(f"B-F19.K{Kp}.P", L - W)


# ==============================================================================================
# B-F20  inactivation toy                                          (toy_examples.py)
# ==============================================================================================
def bf20():
    A, B, C, D, E = 5, 3, 7, 2, 12
    pk = {'y1': (('C',), C), 'y2': (('A', 'C'), A ^ C), 'y3': (('B', 'C', 'D'), B ^ C ^ D),
          'y4': (('A', 'B'), A ^ B), 'y5': (('D', 'E'), D ^ E), 'y6': (('B', 'E'), B ^ E)}

    def peel(pk):
        eqs = {k: [set(s), v] for k, (s, v) in pk.items()}
        known, log = {}, []
        while True:
            ripple = [k for k, (s, v) in eqs.items() if len(s) == 1]
            if not ripple:
                break
            k = sorted(ripple)[0]
            (sym,), v = eqs.pop(k)
            known[sym] = v
            log.append((sym, len(ripple)))
            for e in eqs.values():
                if sym in e[0]:
                    e[0].remove(sym)
                    e[1] ^= v
            eqs = {kk: e for kk, e in eqs.items() if e[0]}
        return known, log

    known, log = peel(pk)
    emit("B-F20.peel_order", [s for s, _ in log])
    emit("B-F20.ripple_sizes", [n for _, n in log])
    emit("B-F20.peel_recovers_all_correctly_ok", known == {'A': A, 'B': B, 'C': C, 'D': D, 'E': E})
    known2, _ = peel({k: v for k, v in pk.items() if k != 'y1'})
    emit("B-F20.recovered_without_y1", len(known2))
    y2, y3, y4, y5, y6 = A ^ C, B ^ C ^ D, A ^ B, D ^ E, B ^ E
    emit("B-F20.y_values_y2_to_y6", [y2, y3, y4, y5, y6])
    c = y6 ^ y5 ^ y3
    emit("B-F20.c_equals_y6_xor_y5_xor_y3", c)
    # check symbolically that c = y6^y5^y3 follows from treating C as unknown: try all 256 c, keep consistent ones
    sols = [cc for cc in range(256)
            if ((y4 ^ y2 ^ cc) ^ (y5 ^ (y3 ^ y4 ^ y2))) == y6]
    emit("B-F20.unique_c_consistent_with_y6", sols)
    Ar = y2 ^ c
    Br = y4 ^ Ar
    Dr = y3 ^ y4 ^ y2
    Er = y5 ^ Dr
    emit("B-F20.back_substituted_A_B_D_E", [Ar, Br, Dr, Er])


# ==============================================================================================
# B-F21  ablation (RECORDED)                                       (rq_ablate.py outputs)
# ==============================================================================================
RECORDED_ABLATION = [
    # (key, file, K', mode, h, trials, fails)
    ("K10_rfc_hdpc_h0", "original_rq_ablate_k10_gf256_0_1e5trials.txt", 10, "gf256", 0, 100000, 646),
    ("K10_gf2_hdpc_h0", "original_rq_ablate_k10_gf2random_0.txt", 10, "gf2random", 0, 10000, 7066),
    ("K10_gf2_hdpc_h1", "original_rq_ablate_k10_gf2random_1.txt", 10, "gf2random", 1, 10000, 4269),
    ("K10_gf2_hdpc_h2", "original_rq_ablate_k10_gf2random_2.txt", 10, "gf2random", 2, 10000, 2293),
    ("K101_gf2_hdpc_h0", "original_rq_ablate_k101_gf2random_0.txt", 101, "gf2random", 0, 1000, 702),
    ("K101_gf2_hdpc_h1", "original_rq_ablate_k101_gf2random_1.txt", 101, "gf2random", 1, 1000, 416),
    ("K101_gf2_hdpc_h2", "original_rq_ablate_k101_gf2random_2.txt", 101, "gf2random", 2, 1000, 244),
    ("K10_gf256random_hdpc_h0", "original_rq_ablate_k10_gf256random_0.txt", 10, "gf256random", 0, 20000, 137),
]


def bf21():
    for key, fname, Kp, mode, h, trials, fails in RECORDED_ABLATION:
        emit(f"B-F21.recorded.{key}.fails", fails)
        emit(f"B-F21.recorded.{key}.trials", trials)
        emit(f"B-F21.recorded.{key}.rate", Fraction(fails, trials))
        emit(f"B-F21.recorded.{key}.source", "docs/research/raptorq_gf256/scripts/results/" + fname)
        pat = rf"K'={Kp} L=\d+(?: H=\d+)? mode={mode} N=K'\+{h} trials=(\d+): fails=(\d+)"
        emit(f"B-F21.recorded.{key}.provenance_check", recorded_check(fname, pat, [(trials, fails)]))
    shape_model()


def _p_rank_eq(q, m, n, r):
    """P(an m x n uniform random matrix over GF(q) has rank exactly r), exact (the standard count of rank-r
    matrices: prod_{i<r} (q^m - q^i)(q^n - q^i)/(q^r - q^i), over q^(mn))."""
    if r < 0 or r > min(m, n):
        return Fraction(0)
    num = Fraction(1)
    for i in range(r):
        num *= Fraction((q ** m - q ** i) * (q ** n - q ** i), q ** r - q ** i)
    return num / Fraction(q ** (m * n))


def _p_span(q, m, d):
    """P(m uniform random vectors span GF(q)^d) = prod_{i<d} (1 - q^(i-m)), exact."""
    if m < d:
        return Fraction(0)
    p = Fraction(1)
    for i in range(d):
        p *= 1 - Fraction(q ** i, q ** m)
    return p


def shape_model():
    """R2 (15 notes, description; Ruth R5) and 17_2 (Maya M1): RaptorQ's shape with random rows at K' = 10. The
    S + K' + h ex-or rows are uniform 0/1 rows over the L unknowns and the H byte rows are uniform over GF(256).
    The 0/1 rows have the same rank r over GF(2) and GF(256); the byte rows then fill the other L - r
    directions with probability _p_span(256, H, L - r). Exact sum over r. At h = 0 it is Ruth's product form
    1 - prod_{i=11..27}(1 - 2^-i) * prod_{i=1..10}(1 - 256^-i)."""
    _, _, S, H, _ = {r[0]: r for r in RFC_TABLE2}[10]
    Kp = 10
    L = Kp + S + H
    fails = []
    for h in range(3):
        m = S + Kp + h
        ok = sum(_p_rank_eq(2, m, L, r) * _p_span(256, H, L - r) for r in range(min(m, L) + 1))
        fails.append(1 - ok)
        emit(f"B-F21.shape_model.K10.fail_h{h}", 1 - ok)
    exor = 1 - math.prod((1 - Fraction(1, 2 ** i) for i in range(H + 1, L + 1)), start=Fraction(1))
    byte = 1 - p_full_finite(256, H)
    emit("B-F21.shape_model.K10.ex_or_part_h0", exor)
    emit("B-F21.shape_model.K10.byte_part_h0", byte)
    emit("B-F21.shape_model.K10.product_form_equals_rank_sum_h0_ok", 1 - (1 - exor) * (1 - byte) == fails[0])
    emit("B-F21.shape_model.K10.ratio_h0_over_h1", fails[0] / fails[1])
    m1 = S + Kp + 1
    p_full = _p_rank_eq(2, m1, L, m1)   # the extra ex-or row takes one more slot: a byte card is left spare
    emit("B-F21.shape_model.K10.h1_P_ex_or_rows_all_useful", p_full)
    emit("B-F21.shape_model.K10.h1_share_of_failures_with_a_spare_byte_card",
         p_full * (1 - _p_span(256, H, L - m1)) / fails[1])


# ==============================================================================================
# B-F22  measured RaptorQ, cberner/raptorq decoder (RECORDED)      (rq_mc.py outputs)
# ==============================================================================================
RECORDED_MC = [
    # (K, file, [(h, trials, failures), ...])
    (10, "original_rq_mc_K10_1e6trials.txt", [(0, 1000000, 6316), (1, 1000000, 24), (2, 1000000, 1)]),
    (101, "original_rq_mc_K101_2e5trials_partial.txt", [(0, 200000, 1262), (1, 200000, 4)]),
    (1002, "original_rq_mc_K1002_2e4trials_partial.txt", [(0, 20000, 108), (1, 20000, 0)]),
]


def clopper_pearson(x, n, alpha=0.05):
    """Exact (Clopper-Pearson) two-sided interval by bisection on the binomial tails."""
    def solve(f, lo, hi):
        for _ in range(100):
            mid = (lo + hi) / 2
            if f(mid):
                hi = mid
            else:
                lo = mid
        return (lo + hi) / 2
    # lower: P(X >= x | p) = alpha/2 ; for x = 0 lower = 0
    lower = 0.0 if x == 0 else solve(lambda p: 1 - _binom_cdf_log(x - 1, n, p) >= alpha / 2, 0.0, 1.0)
    upper = solve(lambda p: _binom_cdf_log(x, n, p) <= alpha / 2, 0.0, 1.0)
    return lower, upper


def _binom_cdf_log(x, n, p):
    if p <= 0:
        return 1.0
    if p >= 1:
        return 0.0 if x < n else 1.0
    tot = 0.0
    for j in range(x + 1):
        tot += math.exp(math.lgamma(n + 1) - math.lgamma(j + 1) - math.lgamma(n - j + 1)
                        + j * math.log(p) + (n - j) * math.log1p(-p))
    return tot


def bf22():
    for K, fname, rows in RECORDED_MC:
        expected = []
        for h, trials, fails in rows:
            emit(f"B-F22.recorded.K{K}.h{h}.failures", fails)
            emit(f"B-F22.recorded.K{K}.h{h}.trials", trials)
            emit(f"B-F22.recorded.K{K}.h{h}.rate", Fraction(fails, trials))
            expected.append((h, trials, fails))
        emit(f"B-F22.recorded.K{K}.source", "docs/research/raptorq_gf256/scripts/results/" + fname)
        pat = rf"K={K} recv=K\+(\d+) trials=(\d+): failures=(\d+)"
        emit(f"B-F22.recorded.K{K}.provenance_check", recorded_check(fname, pat, expected))
    lo, hi = clopper_pearson(1, 10 ** 6)
    emit("B-F22.K10.h2.clopper_pearson95_lower", lo)
    emit("B-F22.K10.h2.clopper_pearson95_upper", hi)
    lo, hi = clopper_pearson(24, 10 ** 6)
    emit("B-F22.K10.h1.clopper_pearson95_lower", lo)
    emit("B-F22.K10.h1.clopper_pearson95_upper", hi)
    lo, hi = clopper_pearson(6316, 10 ** 6)
    emit("B-F22.K10.h0.clopper_pearson95_lower", lo)
    emit("B-F22.K10.h0.clopper_pearson95_upper", hi)
    emit("B-F22.K10.ratio_h0_over_h1", Fraction(6316, 24))
    # R2 (17_3 lower-third, Dev dev-10): dividing by 256 twice predicts about 0.1 failures in 10^6 at K+2; with a
    # Poisson count of that mean, one or more failures happen this often
    pred = Fraction(6316, 256 ** 2)
    emit("B-F22.K10.h2_failures_predicted_by_div256_twice", pred)
    emit("B-F22.K10.h2_P_at_least_one_if_poisson_mean_predicted", -math.expm1(-float(pred)))
    # R2 (Ruth R9): the two "six in a thousand" figures, 646 / 10^5 (B-F21, our RFC 6330 harness) and
    # 6,316 / 10^6 (this decoder), agree: pooled two-proportion z
    x1, n1, x2, n2 = 646, 10 ** 5, 6316, 10 ** 6
    pp = Fraction(x1 + x2, n1 + n2)
    emit("B-F22.K10_h0_vs_B-F21_rfc_rows_two_proportion_z",
         float(Fraction(x1, n1) - Fraction(x2, n2)) / math.sqrt(float(pp * (1 - pp) * (Fraction(1, n1) + Fraction(1, n2)))))


# ==============================================================================================
# B-F23  RFC 6330 section 5.8 decoder requirement (SOURCED constants, not computed)
# ==============================================================================================
def bf23():
    req = [Fraction(1, 100), Fraction(1, 10 ** 4), Fraction(1, 10 ** 6)]
    for h, r in enumerate(req):
        emit(f"B-F23.sourced.rfc6330_s5.8_max_fail_Kprime_plus_{h}", r)
    emit("B-F23.sourced.source", "RFC 6330 section 5.8 (Requirements for a Compliant Decoder), page 65-66")
    meas = [Fraction(6316, 10 ** 6), Fraction(24, 10 ** 6), Fraction(1, 10 ** 6)]
    emit("B-F23.recorded_K10_point_estimates_le_requirement", [m <= r for m, r in zip(meas, req)])
    hi = [clopper_pearson(x, 10 ** 6)[1] for x in (6316, 24, 1)]
    emit("B-F23.recorded_K10_cp95_upper_le_requirement", [u <= float(r) for u, r in zip(hi, req)])


# ==============================================================================================
# B-F24  carousel at 1% loss, K = 10,000, 10^6 phones (exact formula, 50-digit decimals)
# ==============================================================================================
def bf24():
    getcontext().prec = 50
    K, n_phones = 10000, 10 ** 6
    p = Decimal(1) / Decimal(100)

    def P_le(m, copies):   # P(every one of `copies` independent pieces received within m loops)
        if m == 0:
            return Decimal(0)
        return ((Decimal(1) - p ** m).ln() * copies).exp()

    mean = sum(Decimal(1) - P_le(m, K) for m in range(0, 40))   # E[T] = sum_{m>=0} P(T > m); tail < 1e-40
    emit("B-F24.mean_loops_per_phone", mean)
    for m in (1, 2, 3, 4):
        emit(f"B-F24.P_phone_done_by_{m}_loops", P_le(m, K))
    for m in (4, 5, 6, 7):
        emit(f"B-F24.P_all_1e6_phones_done_by_{m}_loops", P_le(m, K * n_phones))
    # 01_6 "Most phones need two or three loops"; picture "after loop 3 a few red stragglers" (the field has
    # about 1,000 glyphs, each standing for 1,000 phones)
    emit("B-F24.P_phone_done_in_loop_2_or_3", P_le(3, K) - P_le(1, K))
    late = Decimal(1) - P_le(3, K)
    emit("B-F24.P_phone_still_waiting_after_3_loops", late)
    emit("B-F24.phones_of_1e6_still_waiting_after_3_loops", late * n_phones)
    emit("B-F24.glyphs_of_1000_still_waiting_after_3_loops", late * 1000)
    carousel_packets(K, p, mean)
    # the drawn field (01_1, 01_6): thresholds P(done by m) for one phone and for the last of a million, as doubles
    phone_field([float(P_le(m, K)) for m in range(1, 13)], [float(P_le(m, K * n_phones)) for m in range(1, 13)])


M64 = (1 << 64) - 1
FIELD_SEED = 20261011   # the storyboard's date; never re-rolled for a nicer draw


def splitmix64(seed):
    """SplitMix64 (Steele, Lea & Flood 2014; Vigna's constants), the portable generator the video's model.rs
    runs, so the scene draws the oracle's sample."""
    state = seed
    while True:
        state = (state + 0x9E3779B97F4A7C15) & M64
        z = state
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & M64
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & M64
        yield z ^ (z >> 31)


def phone_field(thr_phone, thr_million):
    """01_6: the field draws a sample of 1,000 phones (one glyph per 1,000). Each finishes in the first loop m with
    u < P(a phone is done by m), u uniform from the top 53 bits of SplitMix64(FIELD_SEED); the next draw gives the
    loop the unluckiest of the million phones finishes in, by the same rule with P(all 10^6 done by m)."""
    rng = splitmix64(FIELD_SEED)
    uniform = lambda: (next(rng) >> 11) * (1.0 / 9007199254740992.0)
    first = lambda u, thr: next((i + 1 for i, t in enumerate(thr) if u < t), len(thr) + 1)
    loops = [first(uniform(), thr_phone) for _ in range(1000)]
    emit("B-F24.field.seed", FIELD_SEED)
    emit("B-F24.field.glyphs", len(loops))
    emit("B-F24.field.phones_per_glyph", 10 ** 6 // len(loops))
    emit("B-F24.field.glyphs_done_by_loop_1_to_6", [sum(t <= m for t in loops) for m in range(1, 7)])
    emit("B-F24.field.unluckiest_of_1e6_loops", first(uniform(), thr_million))


def carousel_packets(K, p, mean_loops):
    """01_8 screen "an average carousel phone": the expected number of packets SENT until a phone holds every
    piece. The carousel sends pieces 1..K, loop after loop; piece i first arrives in loop T_i (P(T_i <= m) =
    1 - p^m, independently), and the phone is done at N = max_i ((T_i - 1) K + i). For n = mK + j, 0 <= j < K,
    P(N <= n) = a^j b^(K-j) with a = 1 - p^(m+1), b = 1 - p^m, so E[N] = sum_n P(N > n) = sum_m (K - S_m) with
    S_0 = 0 and S_m = sum_j a^j b^(K-j) = b (a^K - b^K) / (a - b). Loops m > 30 add < K^2 p^31 < 1e-53; the sum
    runs at 120 digits because a - b = p^m (1 - p) cancels. A phone finishes partway through its last loop,
    so E[N] is well below mean_loops * K (B-F24.mean_loops_per_phone = 2.642)."""
    with localcontext() as ctx:
        ctx.prec = 120
        pd = Decimal(p)
        en = Decimal(K)
        for m in range(1, 31):
            a, b = 1 - pd ** (m + 1), 1 - pd ** m
            en += K - b * ((a.ln() * K).exp() - (b.ln() * K).exp()) / (a - b)
        caught = en * Decimal(99) / Decimal(100)
    emit("B-F24.carousel_mean_packets_sent_until_done", en)
    # packets caught = 0.99 x sent on average (Wald's identity: packet n's arrival is independent of n <= N)
    emit("B-F24.carousel_mean_packets_caught_until_done", caught)
    emit("B-F24.carousel_whole_loops_x_K_for_contrast", mean_loops * K)
    # second route, the same sum term by term in floating point (expm1/log1p), every n < 31 K
    pf = float(p)
    terms = [1.0] * K
    for m in range(1, 31):
        la, lb = math.log1p(-pf ** (m + 1)), math.log1p(-pf ** m)
        terms.extend(-math.expm1(j * la + (K - j) * lb) for j in range(K))
    emit("B-F24.carousel_mean_packets_sent_two_routes_ok", abs(math.fsum(terms) - float(en)) < 1e-6)
    # third route, independent of the formula: a seeded simulation of 4,000 phones. The pieces still missing
    # after loop t are a Bernoulli(p) thinning of those missing after loop t - 1 (loop 1 by geometric skips).
    rng = random.Random(18)
    lq = math.log1p(-pf)
    sent = []
    for _ in range(4000):
        level, i = [], 0
        while True:
            i += 1 + int(math.log(1.0 - rng.random()) / lq)
            if i > K:
                break
            level.append(i)
        loops = 1
        while level:
            deeper = [j for j in level if rng.random() < pf]
            loops += 1
            if not deeper:
                break
            level = deeper
        sent.append(K if loops == 1 else (loops - 1) * K + max(level))
    mc = Fraction(sum(sent), len(sent))
    sd = math.sqrt(sum((x - float(mc)) ** 2 for x in sent) / (len(sent) - 1))
    emit("B-F24.carousel_mc_seeded_phones", len(sent))
    emit("B-F24.carousel_mc_seeded_mean_packets_sent", mc)
    emit("B-F24.carousel_mc_within_4se_of_exact_ok", abs(float(mc) - float(en)) <= 4 * sd / math.sqrt(len(sent)))
    # the third count on the 01_8 screen: a fountain phone with two extra packets (caught), and the sent equivalent
    emit("B-F24.fountain_packets_caught_K_plus_2", K + 2)
    emit("B-F24.fountain_mean_packets_sent_for_K_plus_2_caught", Fraction(K + 2) / Fraction(99, 100))


# ==============================================================================================
# B-F25  Z/256 lottery: invertible <=> det odd <=> invertible mod 2 (integration_check 7, same RNG)
# ==============================================================================================
def bf25():
    def det_bareiss(M):
        A = [row[:] for row in M]
        n = len(A)
        sign, prev = 1, 1
        for k in range(n - 1):
            if A[k][k] == 0:
                sw = next((i for i in range(k + 1, n) if A[i][k] != 0), None)
                if sw is None:
                    return 0
                A[k], A[sw] = A[sw], A[k]
                sign = -sign
            for i in range(k + 1, n):
                for j in range(k + 1, n):
                    A[i][j] = (A[i][j] * A[k][k] - A[i][k] * A[k][j]) // prev
            prev = A[k][k]
        return sign * A[n - 1][n - 1]

    def inverse_mod256(M):
        n = len(M)
        A = [row[:] + [int(i == j) for j in range(n)] for i, row in enumerate(M)]
        for c in range(n):
            piv = next((i for i in range(c, n) if A[i][c] & 1), None)   # units of Z/256 are the odd bytes
            if piv is None:
                return None
            A[c], A[piv] = A[piv], A[c]
            s = pow(A[c][c], -1, 256)
            A[c] = [(v * s) % 256 for v in A[c]]
            for i in range(n):
                if i != c and A[i][c]:
                    f = A[i][c]
                    A[i] = [(v - f * w) % 256 for v, w in zip(A[i], A[c])]
        return [row[n:] for row in A]

    rng = random.Random(1)   # integration_check.py section 7: random.seed(1); K = 8; T = 40000
    Kk, T, CERT = 8, 40000, 4000
    ok = 0
    cert_parity_agree = cert_inverse_ok = cert_odd = 0
    for t in range(T):
        Mx = [[rng.randrange(256) for _ in range(Kk)] for _ in range(Kk)]
        inv_mod2 = rank_gf2([[x & 1 for x in r] for r in Mx]) == Kk
        ok += inv_mod2
        if t < CERT:   # independent certificate on the first CERT samples
            det_odd = det_bareiss(Mx) % 2 == 1
            cert_parity_agree += det_odd == inv_mod2
            if det_odd:
                cert_odd += 1
                Inv = inverse_mod256(Mx)
                prod = [[sum(Mx[i][k] * Inv[k][j] for k in range(Kk)) % 256 for j in range(Kk)] for i in range(Kk)]
                cert_inverse_ok += prod == [[int(i == j) for j in range(Kk)] for i in range(Kk)]
    emit("B-F25.mc_K8_trials", T)
    emit("B-F25.mc_K8_invertible_count", ok)
    emit("B-F25.mc_K8_invertible_fraction", Fraction(ok, T))
    emit("B-F25.prod_K8_(1-2^-i)", p_full_finite(2, 8))
    emit("B-F25.cert_samples", CERT)
    emit("B-F25.cert_exact_det_parity_agrees_with_mod2_rank", cert_parity_agree)
    emit("B-F25.cert_odd_det_count", cert_odd)
    emit("B-F25.cert_explicit_inverse_mod256_verified", cert_inverse_ok)


def race07():
    """07_6: the race, seeded (lane B-2). Ten random ex-or cards against ten random wrap-around cards on ten pieces,
    2,000 rounds a side, drawn from numpy's PCG64DXSM(706) (the scene draws the same words with franken_manim's
    bit-exact Pcg64Dxsm): the ex-or rounds first, one word per card (its top 10 bits, bit i = piece i), then the
    wrap-around rounds, one word per amount (its top byte), card by card. Ex-or fails when the GF(2) rank is below
    10. Wrap-around is decided here by the exact integer determinant's parity (Bareiss), a route independent of the
    scene's elimination mod 256 with odd pivots, and certified against the mod-2 rank on every round."""
    def det_bareiss(M):
        A = [row[:] for row in M]
        n = len(A)
        sign, prev = 1, 1
        for k in range(n - 1):
            if A[k][k] == 0:
                sw = next((i for i in range(k + 1, n) if A[i][k] != 0), None)
                if sw is None:
                    return 0
                A[k], A[sw] = A[sw], A[k]
                sign = -sign
            for i in range(k + 1, n):
                for j in range(k + 1, n):
                    A[i][j] = (A[i][j] * A[k][k] - A[i][k] * A[k][j]) // prev
            prev = A[k][k]
        return sign * A[n - 1][n - 1]

    K, T, SHOWN = 10, 2000, 4
    words = [int(w) for w in np.random.PCG64DXSM(706).random_raw(T * K + T * K * K)]
    xor_rounds = [[w >> 54 for w in words[t * K:(t + 1) * K]] for t in range(T)]
    base = T * K
    wrap_rounds = [[[words[base + t * K * K + r * K + c] >> 56 for c in range(K)] for r in range(K)] for t in range(T)]
    xor_fail = [rank_gf2([[(w >> c) & 1 for c in range(K)] for w in cards]) < K for cards in xor_rounds]
    wrap_fail = [det_bareiss(M) % 2 == 0 for M in wrap_rounds]
    agree = all(f == (rank_gf2([[x & 1 for x in row] for row in M]) < K) for f, M in zip(wrap_fail, wrap_rounds))
    emit("B-F25.race07.seed", 706)
    emit("B-F25.race07.pieces", K)
    emit("B-F25.race07.rounds_per_side", T)
    emit("B-F25.race07.xor_failures", sum(xor_fail))
    emit("B-F25.race07.wrap_failures", sum(wrap_fail))
    emit("B-F25.race07.first_rounds_xor_fail", [int(f) for f in xor_fail[:SHOWN]])
    emit("B-F25.race07.first_rounds_wrap_fail", [int(f) for f in wrap_fail[:SHOWN]])
    emit("B-F25.race07.wrap_round1_amounts", [x for row in wrap_rounds[0] for x in row])
    emit("B-F25.race07.det_parity_equals_mod2_rank_all_rounds_ok", agree)


def b2_drawn():
    """Numbers chapters 08 and 09 draw beyond the ledger rows (lane B-2): the seven-hour times table (08_2), the
    product the hand counts out, 3 x 5 = 15 = 2 * 7 + 1 (08_2), 1001 (09_5), and the six FOIL terms of (x + 1)(x^2 +
    x + 1) as exponents, before the pairs cancel (09_5)."""
    emit("B-X4.z7_times_table", [a * b % 7 for a in range(7) for b in range(7)])
    emit("B-X4.int_3x5", 3 * 5)
    emit("B-X4.int_3x5_divmod_7", list(divmod(3 * 5, 7)))
    emit("B-X4.bits_of_9", format(9, "04b"))
    p3 = [i for i in range(8) if 3 >> i & 1][::-1]
    p7 = [i for i in range(8) if 7 >> i & 1][::-1]
    emit("B-F11.foil_3x7_terms", [i + j for i in p3 for j in p7])
    # 10_4: the whole fold polynomial x^8 + x^4 + x^3 + x^2 + 1 as a number (0x11D)
    emit("B-F11.fold_polynomial_0x11D_decimal", P_RQ)
    # 11_2 lower-third "(its hand is 3)": the smallest byte whose powers reach all 255 under AES's polynomial
    emit("B-F12.smallest_generator_0x11B", next(g for g in range(2, 256) if order_of(g, P_AES) == 255))


# ==============================================================================================
# B-F26  rotation fold (mod x^8 + 1)                               (verify_claims.py)
# ==============================================================================================
def bf26():
    P_ROT = 0x101
    emit("B-F26.mul_3_255_mod_x8_plus_1", gmul(3, 255, P_ROT))
    emit("B-F26.carryless_3_255", clmul(3, 255))
    emit("B-F26.x8_plus_1_equals_(x+1)^8_ok", clmul(clmul(clmul(3, 3), clmul(3, 3)), clmul(clmul(3, 3), clmul(3, 3))) == P_ROT)
    emit("B-F26.doubling_is_bijection_distinct_values", len({gmul(2, a, P_ROT) for a in range(256)}))
    emit("B-F26.zero_divisor_pairs_nonzero", sum(1 for a in range(1, 256) for b in range(1, 256)
                                                  if gmul(a, b, P_ROT) == 0))
    # 10_1 "The simplest fold drops the fallen bit back in at the bottom. A rotation.": times 2 mod x^8 + 1 is
    # an 8-bit rotate-left for every byte
    emit("B-F26.mul2_mod_x8_plus_1_is_rotate_left_all256_ok",
         all(gmul(2, a, P_ROT) == ((a << 1) | (a >> 7)) & 0xFF for a in range(256)))
    emit("B-F26.mul2_mod_x8_plus_1_of_128", gmul(2, 128, P_ROT))
    # R2 (10_2 hint "Three times a byte is double it, ex-or itself", Maya M4): 3a = 2a xor a under the rotation
    # fold for every byte; doubling 255 rotates it to itself, and 255 xor 255 = 0
    emit("B-F26.three_a_equals_two_a_xor_a_all256_ok", all(gmul(3, a, P_ROT) == gmul(2, a, P_ROT) ^ a for a in range(256)))
    emit("B-F26.mul2_mod_x8_plus_1_of_255", gmul(2, 255, P_ROT))
    emit("B-F26.xor_255_255", 255 ^ 255)


# ==============================================================================================
# B-F27  union bound vs exact, GF(256)                             (verify_claims.py)
# ==============================================================================================
def bf27():
    def sig_digits_agree(x, y):
        n = 0
        for d in range(1, 17):
            if f"{x:.{d - 1}e}" == f"{y:.{d - 1}e}":
                n = d
            else:
                break
        return n
    for h in range(3):
        exact = 1 - p_full_limit(256, h)
        bound = Fraction(1, 256 ** h * 255)
        emit(f"B-F27.h{h}.exact_fail", exact)
        emit(f"B-F27.h{h}.union_bound", bound)
        emit(f"B-F27.h{h}.bound_minus_exact_relative", (bound - exact) / exact)
        emit(f"B-F27.h{h}.significant_digits_agree", sig_digits_agree(float(exact), float(bound)))


# ==============================================================================================
# B-F28  GF(2) spares for 10^-6                                    (integration_check 6)
# ==============================================================================================
def bf28():
    fails = {h: 1 - p_full_limit(2, h) for h in range(0, 40)}
    hmin = min(h for h in fails if fails[h] <= Fraction(1, 10 ** 6))
    emit("B-F28.min_spares_for_fail_le_1e-6", hmin)
    emit("B-F28.fail_gf2_h19", fails[19])
    emit("B-F28.fail_gf2_h20", fails[20])
    emit("B-F28.two_to_minus_20", Fraction(1, 2 ** 20))
    emit("B-F28.phones_stuck_of_1e6_at_h2", fails[2] * 10 ** 6)


# ==============================================================================================
# B-X1  RAID-6 (open item): Q = sum 2^i D_i in GF(2^8)/0x11D; generator 2 under 0x11B gives 51 weights
# ==============================================================================================
def bx1():
    def weights(p):
        w, x = [], 1
        for _ in range(255):
            w.append(x)
            x = gmul(x, 2, p)
        return w
    w_rq, w_aes = weights(P_RQ), weights(P_AES)
    emit("B-X1.raid6_distinct_weights_2^i_i0_254_under_0x11D", len(set(w_rq)))
    emit("B-X1.raid6_distinct_weights_2^i_i0_254_under_0x11B", len(set(w_aes)))
    emit("B-X1.first_i_ge1_with_2^i_eq_1_under_0x11B", next(i for i in range(1, 300) if pow_g(2, i, P_AES) == 1))
    emit("B-X1.anvin_02^8_equals_0x1d", pow_g(2, 8, P_RQ))
    # Linux lib/raid/raid6/int.uc computes Q by Horner: wq = D[z0]; for z = z0-1..0: wq = xtime(wq) ^ D[z].
    rng = random.Random(6)
    ok = True
    for _ in range(200):
        n = rng.randrange(2, 30)
        Dd = [rng.randrange(256) for _ in range(n)]
        wq = Dd[n - 1]
        for z in range(n - 2, -1, -1):
            wq = rust_xtime(wq) ^ Dd[z]
        direct = 0
        for i, d in enumerate(Dd):
            direct ^= gmul(pow_g(2, i, P_RQ), d)
        ok &= wq == direct
    emit("B-X1.linux_horner_Q_equals_sum_2^i_D_i_ok", ok)
    # two-erasure recovery needs g^x != g^y for every pair of data disks
    emit("B-X1.max_data_disks_distinct_weights_0x11D", len(set(w_rq)))
    emit("B-X1.max_data_disks_distinct_weights_0x11B_gen2", len(set(w_aes)))


def pow_g(g, n, p):
    r = 1
    for _ in range(n):
        r = gmul(r, g, p)
    return r


# ==============================================================================================
# B-X2  QR codes (open item): ISO/IEC 18004 "byte-wise modulo 100011101 arithmetic"
# ==============================================================================================
def bx2():
    v = int("100011101", 2)
    emit("B-X2.qr_iso18004_modulus_100011101_decimal", v)
    emit("B-X2.qr_modulus_equals_0x11D", v == P_RQ)


# ==============================================================================================
# B-X3  ESI range (14_8 lower-third "RaptorQ: up to 2^24 ≈ 16.8 million packets per block")
#       SOURCED width: RFC 6330 section 3.2, "Encoding Symbol ID (ESI): 24-bit unsigned integer."
# ==============================================================================================
def bx3():
    bits = 24   # SOURCED constant (RFC 6330 section 3.2); everything below is computed from it
    emit("B-X3.sourced.rfc6330_s3.2_esi_bits", bits)
    emit("B-X3.sourced.source", "RFC 6330 section 3.2 (FEC Payload IDs), page 5")
    emit("B-X3.esi_values_per_source_block", 2 ** bits)
    emit("B-X3.esi_values_per_source_block_in_millions", Fraction(2 ** bits, 10 ** 6))


# ==============================================================================================
# B-X4  small arithmetic said aloud or shown (02 bits and ex-or, 06_3, 08 prime clocks, 09 polynomials)
# ==============================================================================================
def bx4():
    def exps(n):   # the powers of x a byte stands for, highest first (09: bits as polynomial terms)
        return [i for i in range(7, -1, -1) if n >> i & 1]
    emit("B-X4.place_values_8_bits", [2 ** i for i in range(8)])
    emit("B-X4.bits_of_5", format(5, "b"))
    emit("B-X4.bits_of_7", format(7, "b"))
    emit("B-X4.byte_max_2^8_minus_1", 2 ** 8 - 1)
    emit("B-X4.xor_5_7", 5 ^ 7)
    emit("B-X4.bits_of_xor_5_7", format(5 ^ 7, "03b"))
    emit("B-X4.xor_2_7", 2 ^ 7)
    emit("B-X4.xor_twice_restores_all_65536_pairs_ok", all((a ^ b) ^ b == a for a in range(256) for b in range(256)))
    emit("B-X4.halvings_2^10", 2 ** 10)
    emit("B-X4.halvings_2^20", 2 ** 20)
    # 06_5: a byte as the amount, 256 choices
    emit("B-X4.byte_values_2^8", 2 ** 8)
    # 08: the seven-hour clock; 256 does not fit in a byte; the primes either side of 256
    emit("B-X4.z7_inverse_of_3", pow(3, -1, 7))
    emit("B-X4.z7_nonzero_rows_are_permutations_ok",
         all(sorted(a * b % 7 for b in range(7)) == list(range(7)) for a in range(1, 7)))
    emit("B-X4.z7_zero_has_no_partner_ok", all(0 * b % 7 != 1 for b in range(7)))
    emit("B-X4.int_2x128", 2 * 128)
    emit("B-X4.bits_needed_for_256", (256).bit_length())
    primes = [n for n in range(240, 270) if all(n % d for d in range(2, math.isqrt(n) + 1))]
    emit("B-X4.nearest_primes_below_and_above_256",
         [max(n for n in primes if n < 256), min(n for n in primes if n > 256)])
    # 09: bytes as polynomials (powers of x with coefficient 1)
    emit("B-X4.x_powers_of_3", exps(3))
    emit("B-X4.x_powers_of_7", exps(7))
    emit("B-X4.x_powers_of_128", exps(128))
    emit("B-X4.x_powers_of_9", exps(9))


# ==============================================================================================
# B-X5  CRC-8/SAE-J1850 (10_7 lower-third "CRC-8/SAE-J1850 uses this very 0x1D")
#       SOURCED parameters: reveng CRC catalogue, width=8 poly=0x1d init=0xff xorout=0xff (attested)
# ==============================================================================================
def bx5():
    poly, init, xorout = 0x1D, 0xFF, 0xFF   # SOURCED constants (the catalogue's parameter line)

    def crc8(data):   # MSB first; each bit step is the code panel's xtime with the same 0x1D
        c = init
        for byte in data:
            c ^= byte
            for _ in range(8):
                c = rust_xtime(c, poly)
        return c ^ xorout
    # SOURCED codewords: SAE J1850 Table 1 as listed in the catalogue entry (data bytes, then the CRC byte)
    words = ["0000000059", "F2018337", "0FAA005579", "00FF5511B8", "332255AABBCCDDEEFFCB", "926B558C",
             "FFFFFFFF74"]
    ok = [crc8(bytes.fromhex(w[:-2])) == int(w[-2:], 16) for w in words]
    emit("B-X5.sourced.crc8_sae_j1850_poly", poly)
    emit("B-X5.crc8_poly_equals_raptorq_fold_low_byte_ok", poly == P_RQ & 0xFF)
    emit("B-X5.attested_codewords", len(words))
    emit("B-X5.attested_codewords_reproduced", sum(ok))


# ==============================================================================================
# B-X6  AES's field (10_7 "A famous encryption standard picked a different prime polynomial")
#       SOURCED: FIPS 197-upd1 section 4.2, eq. (4.3) m(x) = x^8 + x^4 + x^3 + x + 1; worked example (4.7)
# ==============================================================================================
def bx6():
    emit("B-X6.sourced.fips197_m(x)", P_AES)
    chain = [0x57]
    for _ in range(7):
        chain.append(gmul(chain[-1], 2, P_AES))
    emit("B-X6.fips197_s4.2_57_times_01_02_04_08_10_20_40_80", chain)
    emit("B-X6.fips197_s4.2_xtime_chain_matches_ok", chain == [0x57, 0xae, 0x47, 0x8e, 0x07, 0x0e, 0x1c, 0x38])
    emit("B-X6.fips197_s4.2_57_times_13", gmul(0x57, 0x13, P_AES))
    emit("B-X6.fips197_s4.2_57_times_13_is_0xfe_ok", gmul(0x57, 0x13, P_AES) == 0xFE)


# ==============================================================================================
# B-X7  the precode (R2: 15_1-15_3 and 15_6; Ruth R1, Maya M2/M13, Dev dev-01). RFC 6330 section 5.3.3.3:
#       S LDPC and H HDPC relations among the L = K' + S + H intermediate symbols "evaluate to zero"
# ==============================================================================================
def _precode_cover_counts(Kp, S, H, W):
    """Per intermediate symbol, the number of precode rows it sits in with a nonzero entry, from the LDPC loops
    of section 5.3.3.3 (C[0..B-1] three times each, C[B..W-1] once each, the PI symbols in pairs) and the
    identity part of the HDPC relations (C[L-H..L-1]); the dense MT * GAMMA part (C[0..K'+S-1]) is left out,
    so this undercounts and the coverage below needs no V tables."""
    L = Kp + S + H
    P, B = L - W, W - S
    rows = [dict() for _ in range(L)]          # symbol -> {row: parity}
    for i in range(B):
        a, b = 1 + i // S, i % S
        for _ in range(3):
            rows[i][b] = rows[i].get(b, 0) ^ 1
            b = (b + a) % S
    for i in range(S):
        rows[B + i][i] = rows[B + i].get(i, 0) ^ 1
        a, b = i % P, (i + 1) % P
        rows[W + a][i] = rows[W + a].get(i, 0) ^ 1
        rows[W + b][i] = rows[W + b].get(i, 0) ^ 1
    for i in range(H):
        rows[L - H + i][S + i] = 1
    return [sum(r.values()) for r in rows]


def bx7():
    tab = {r[0]: r for r in RFC_TABLE2}
    # first route: build the rows for the three block sizes the video names, and count
    direct = {}
    for Kp in (10, 10017, 56403):
        _, _, S, H, W = tab[Kp]
        direct[Kp] = min(_precode_cover_counts(Kp, S, H, W)) >= 1
    emit("B-X7.precode_covers_every_intermediate_symbol_K10_K10017_K56403_direct_ok", all(direct.values()))
    # second route, every row of Table 2: C[0..B-1] are added three times (odd, so never cancelled), the LDPC
    # symbols once, the HDPC symbols once; PI symbol j < U is in an LDPC row iff j <= S (i % P or (i+1) % P, i < S)
    cover_all, distinct_all = True, True
    for Kp, _, S, H, W in RFC_TABLE2:
        L = Kp + S + H
        P, B = L - W, W - S
        U = P - H
        cover_all &= P >= 2 and U <= S + 1
        # the three LDPC rows of C[i] are distinct iff S divides neither a nor 2a: S is an odd prime and a < S
        distinct_all &= S % 2 == 1 and 1 + (B - 1) // S < S
    emit("B-X7.precode_covers_every_intermediate_symbol_all_477_ok", cover_all)
    emit("B-X7.ldpc_three_distinct_rows_per_symbol_all_477_ok", distinct_all)
    for Kp in (10, 10017):
        _, _, S, H, W = tab[Kp]
        L = Kp + S + H
        emit(f"B-X7.K{Kp}.extra_pieces_S_plus_H", S + H)
        emit(f"B-X7.K{Kp}.ex_or_cards_L_minus_H", L - H)
        emit(f"B-X7.K{Kp}.ldpc_max_a", 1 + (W - S - 1) // S)
    # model: a received packet's LT part hits a given one of the W LT symbols with probability d/W (b uniform,
    # the d indices distinct because W is prime); N = K' independent packets miss it with (1 - mean_d / W)^N
    _, _, S, H, W = tab[10017]
    mean = float(sum(d * w for d, w in _degree_masses().items()))
    emit("B-X7.K10017.model_expected_LT_symbols_in_no_packet_N_eq_Kprime", W * (1 - mean / W) ** 10017)


# ==============================================================================================
# B-X8  Reed-Solomon over GF(256) (R2: 14_8 "Any set of exactly enough packets rebuilds the picture"; the
#       lower-third "N <= 257 (q + 1)"; Ruth R18, Dev dev-14). Columns (1, a, a^2) for the 256 bytes a plus the
#       point at infinity (0, 0, 1): the doubly extended code with K = 3 and N = 257
# ==============================================================================================
def bx8():
    cols3 = [(1, a, gmul(a, a)) for a in range(256)] + [(0, 0, 1)]
    rng = random.Random(1408)
    ok3 = all(rank_gf256_py([list(cols3[i]) for i in rng.sample(range(257), 3)]) == 3 for _ in range(3000))
    emit("B-X8.rs_K3_N257_seeded_triples_checked", 3000)
    emit("B-X8.rs_K3_N257_every_seeded_triple_rebuilds_ok", ok3)
    # K = 2: the columns (1, a) and (0, 1) are the 257 directions of GF(256)^2, so any two rebuild and no 258th
    # column can be added
    dirs = set()
    for a in range(256):
        for b in range(256):
            if a or b:
                s = INV[a] if a else INV[b]
                dirs.add((gmul(a, s), gmul(b, s)))
    emit("B-X8.directions_in_gf256_squared", len(dirs))
    cols2 = [(1, a) for a in range(256)] + [(0, 1)]
    emit("B-X8.rs_K2_N257_every_pair_rebuilds_ok",
         all(gmul(cols2[i][0], cols2[j][1]) ^ gmul(cols2[i][1], cols2[j][0]) != 0
             for i in range(257) for j in range(i + 1, 257)))


# ==============================================================================================
# B-S1, B-S4  the 03_8 deck and the 03_9 name card
# ==============================================================================================
def bs_cards():
    # 03_8: a toy deck stands in for RFC 6330's Tuple[K', ISI] (out of scope): a packet's number seeds SplitMix64,
    # and its first draw picks a nonempty mix of A-D (bit i = piece i), the same on both sides
    packets = [48213, 48214, 48215]
    emit("B-S4.toy_deck.packets", packets)
    emit("B-S4.toy_deck.masks", [1 + next(splitmix64(FIELD_SEED ^ n)) % 15 for n in packets])
    # 03_9 name card "RFC 6330 · IETF Proposed Standard · August 2011" (RFC 6330 header, p. 1; B-S1)
    emit("B-S1.sourced.rfc6330_number", 6330)
    emit("B-S1.sourced.rfc6330_year", 2011)


# ==============================================================================================
# Chapter 13 (lane B-3): 13_1's byte card and the year 13_6's lower-third names
# ==============================================================================================
def ch13():
    # 13_1: one card with random byte amounts on the pieces A-E: random.Random(1301).randrange(1, 256) five
    # times, its result over GF(256) under RaptorQ's fold
    rng = random.Random(1301)
    amounts = [rng.randrange(1, 256) for _ in range(5)]
    r = 0
    for a, piece in zip(amounts, (5, 3, 7, 2, 12)):
        r ^= gmul(a, piece)
    emit("B-CH13.byte_card.seed", 1301)
    emit("B-CH13.byte_card.amounts_A_to_E", amounts)
    emit("B-CH13.byte_card.result", r)
    # 13_6 lower-third "Erdos proved the full sum irrational in 1948" (J. Indian Math. Soc. 12, 63-66; fact sheet,
    # open item 1, VERIFIED)
    emit("B-F6.sourced.erdos_irrationality_year", 1948)


if __name__ == "__main__":
    for fn in (bf1, bf2, bf3, bf4, bf5, bf6, bf7, bf8, bf9, bf10, bf11, bf12, bf13, bf14, bf15, bf16, bf17,
               bf18, bf19, bf20, bf21, bf22, bf23, bf24, bf25, race07, b2_drawn, bf26, bf27, bf28, bx1, bx2, bx3, bx4, bx5, bx6,
               bx7, bx8, bs_cards, ch13):
        fn()
    sys.stdout.flush()
    print(f"# {len(_KEYS)} keys in {time.time() - T0:.2f} s", file=sys.stderr)
