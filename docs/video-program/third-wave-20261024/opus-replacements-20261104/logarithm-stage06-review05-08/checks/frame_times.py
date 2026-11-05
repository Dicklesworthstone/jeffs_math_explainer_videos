"""Check every picture number and time pair this review cites against each chapter's sampling.json, and list the lint anchors.

Reads only. Standard library. Fails if a cited time is more than 0.06 s from its picture's time.

    python3 -I -B frame_times.py
"""
import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
FRAMES = HERE.parents[5] / "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-frames01"
CITED = {
    "05_complex_turns": [(50, 22.538), (61, 27.433), (134, 60.0), (136, 60.6), (185, 80.5), (215, 94.5), (227, 98.5), (286, 121.113), (297, 126.0), (332, 143.0), (348, 151.0), (362, 155.1), (172, 74.0), (173, 74.5), (374, 159.0), (376, 160.0), (405, 173.5), (406, 174.0), (67, 29.0), (168, 72.733)],
    "06_lifting_loops": [(88, 38.446), (102, 43.5), (103, 43.633), (169, 70.5), (184, 77.156), (196, 82.033), (197, 82.233), (268, 114.0), (271, 115.5), (336, 145.0), (391, 169.5), (394, 170.4)],
    "07_branch_cut": [(96, 41.5), (158, 69.0), (174, 75.741), (177, 76.5), (185, 80.5), (186, 80.633), (187, 80.833), (189, 81.133), (196, 84.5), (218, 95.5), (242, 104.5), (244, 105.5), (262, 114.5), (265, 116.0), (285, 126.0), (294, 129.8), (309, 133.167), (312, 134.0), (330, 142.5), (336, 145.5), (360, 157.5), (361, 158.0), (362, 158.5)],
    "08_product_rule": [(31, 14.0), (44, 20.5), (64, 30.131), (77, 35.1), (78, 35.3), (96, 43.5), (97, 44.0), (117, 54.0), (118, 54.5), (139, 62.5), (140, 63.0), (160, 73.0), (161, 73.5), (162, 74.0), (188, 87.0), (203, 91.5), (213, 96.5), (244, 111.385), (255, 116.0), (256, 116.267), (257, 116.467), (271, 122.5), (276, 125.0), (277, 125.5), (298, 136.0), (325, 148.183)],
}
LOOK_UP = {
    "05_complex_turns": [],
    "06_lifting_loops": [],
    "07_branch_cut": [],
    "08_product_rule": [],
}


def main():
    bad = 0
    for chapter, pairs in CITED.items():
        sampling = json.loads((FRAMES / chapter / "sampling.json").read_text(encoding="utf-8"))
        times = sampling["times"]
        for frame, said in pairs:
            actual = times[frame]
            ok = abs(actual - said) <= 0.06
            bad += not ok
            print(f"{chapter}\tf{frame}\tsaid {said:.3f}\tis {actual:.3f}\t{'ok' if ok else 'DIFFERS'}")
        for t in LOOK_UP[chapter]:
            near = min(range(len(times)), key=lambda i: abs(times[i] - t))
            print(f"{chapter}\tnearest to {t:.3f}\tf{near}\t{times[near]:.3f}")
        print(f"{chapter}\tlint anchors {len(sampling['layout_findings'])}")
        for finding in sampling["layout_findings"]:
            print(f"{chapter}\tanchor\t{json.dumps(finding, ensure_ascii=False)}")
    print("differing pairs:", bad)


if __name__ == "__main__":
    main()
