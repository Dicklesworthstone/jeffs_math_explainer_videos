#!/usr/bin/env python3
"""Write chapter 20's wake-ledger01 file from root's W1 manifest: measured entries only.

WhiteSwan white-wake-closing01, Agent Mail 36299. Root runs this; it reads, hashes and writes
one new file, and runs nothing else.

    python3 -I -B PKG/tools/wake_ledger01.py --manifest MANIFEST.json [--compare COMPARE.json] \
        --out LEDGER.tsv
    python3 -I -B PKG/tools/wake_ledger01.py --self-test NEW_DIR

MANIFEST is the manifest `tools/fluid_reference/wake_history.py compare` reads ({"reference",
"rows": [{"role", "analysis"}], "close_enough"}), read from the repository root as that tool reads
it. A row may also name its "receipt"; by default it is resource-receipt.json in the run
directory's parent (the job folder). The manifest may list fewer than five rows: a role it does
not list stays pending. COMPARE is that tool's compare output for the same manifest, once all five
rows are run.

A row's numbers are written only if all of these hold, and the row is pending (with the first
reason that failed) otherwise:

- the analysis is wake_history.py's, status OK, with a table_row;
- its settings are the 2D-2 wake on a fixed grid, and equal the run directory's settings.json case;
- every input file it hashed still has that hash;
- the receipt is terminal and successful: finished, exit 0, the disk guard's exit 0, admitted,
  not preflight only, and its inputs hash unchanged at the end;
- the run's command names its own run directory, and the same tunnel binary as the rows before;
- its analysis parameters and sample spacing equal the rows before it;
- its level and step continue the ladder of the rows before it (grid levels +1 at one step; steps
  2 and 3 on grid 3's level at longer steps): the chapter's preview refuses anything else;
- each of its eight measurements is finite and in range (a cost that is absent or out of range is
  left out, and its cell stays "?").

The compare's curve gaps and the 20_v1 slot are written only if the compare names this manifest,
did not refuse, kept every row this file accepts, tabulates exactly the analyses' rows, declares
the manifest's tolerances, and its verdict is the one recomputed here from its table and curves.
W2 is always pending. Nothing absent is written as zero; values are Python's repr of each float.
Paths in the file are written REPO- and HOME-relative.
"""

import argparse
import hashlib
import json
import math
import os
import pathlib
import sys

SCHEMA = "wake-ledger01"
TOOL = "wake_history.py"
ROLES = ["grid1", "grid2", "grid3", "step2", "step3"]
MEASUREMENTS = ["strouhal", "drag_peak", "drag_middle_ff", "drag_swing", "lift_peak", "lift_middle_ff",
                "lift_swing", "delta_p_st96"]
COSTS = ["cells_wet_leaves_max", "steps_run", "time_on_clock_s", "memory_max_rss_bytes"]
CURVES = {"curve_drag_max_abs": "drag", "curve_lift_max_abs": "lift"}
POSITIVE = {"strouhal", "drag_peak", "drag_middle_ff", "lift_peak", "lift_swing"} | set(COSTS)
NON_NEGATIVE = {"drag_swing"} | set(CURVES)
COUNTS = {"cells_wet_leaves_max", "steps_run"}
TOLERANCE_KEYS = MEASUREMENTS + ["curve_drag", "curve_lift"]
SLOT_KEYS = ["20_v1.St", "20_v1.d1", "20_v1.d2", "20_v1.cells", "20_v1.steps", "20_v1.time_s"]
W2 = "no measured wake field in this package; W2 is a release prerequisite that root qualifies"


class Pending(Exception):
    """A row that is not measured, with the reason."""


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for block in iter(lambda: fh.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def private(text):
    """Write the repository root as REPO and the home directory as HOME; one line, no tabs."""
    text = str(text).replace(os.getcwd(), "REPO").replace(str(pathlib.Path.home()), "HOME")
    return " ".join(text.replace("\t", " ").split())


def load_json(path, what):
    try:
        with open(path) as fh:
            return json.load(fh)
    except (OSError, ValueError) as e:
        raise Pending("%s unreadable (%s)" % (what, type(e).__name__)) from None


def finite(x):
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x)


def in_range(key, x):
    if not finite(x):
        return False
    if key in POSITIVE and x <= 0:
        return False
    if key in NON_NEGATIVE and x < 0:
        return False
    return not (key in COUNTS and float(x) != int(x))


def shown(key, x):
    return str(int(x)) if key in COUNTS else repr(float(x))


def check_row(row, before):
    """The row's ledger entry {role, level, dt, analysis, receipt, values, binary, params, notes}."""
    path = row.get("analysis")
    if not isinstance(path, str):
        raise Pending("the manifest row names no analysis")
    a = load_json(path, "analysis")
    if a.get("tool") != TOOL:
        raise Pending("analysis not written by %s" % TOOL)
    if a.get("status") != "OK" or not isinstance(a.get("table_row"), dict):
        reasons = "; ".join(a.get("reasons") or []) or "no reason given"
        raise Pending("analysis status %s: %s" % (a.get("status"), reasons))
    settings = a.get("settings") or {}
    if settings.get("flow") != "wake" or settings.get("mode") != "fixed":
        raise Pending("not the 2D-2 wake on a fixed grid (flow %r, mode %r)" % (settings.get("flow"), settings.get("mode")))
    level = settings.get("effective_level", settings.get("level"))
    dt = settings.get("dt")
    if not (isinstance(level, int) and finite(dt) and dt > 0):
        raise Pending("level or step missing from the analysis settings")
    for name, item in sorted((a.get("inputs") or {}).items()):
        if not os.path.isfile(item.get("path", "")) or sha256(item["path"]) != item.get("sha256"):
            raise Pending("input %s changed or missing since the analysis" % name)
    run_dir = a.get("run_dir")
    if not isinstance(run_dir, str) or not os.path.isdir(run_dir):
        raise Pending("run directory missing")
    run_settings = load_json(os.path.join(run_dir, "settings.json"), "run settings.json")
    if run_settings.get("case") != settings:
        raise Pending("the run's settings.json case differs from the analysed settings")
    command = run_settings.get("command") or []
    if os.path.realpath(run_dir) not in [os.path.realpath(c) for c in command if isinstance(c, str)]:
        raise Pending("the run's command does not name its run directory")
    binary = command[2] if len(command) > 2 else None
    receipt_path = row.get("receipt") or os.path.join(os.path.dirname(os.path.normpath(run_dir)), "resource-receipt.json")
    r = load_json(receipt_path, "receipt")
    terminal = [
        ("finished", bool(r.get("finished"))),
        ("exit 0", r.get("exit") == 0),
        ("disk guard exit 0", (r.get("sbh") or {}).get("exit") == 0),
        ("admitted", r.get("admitted") is True),
        ("not preflight only", r.get("preflight_only") is False),
        ("inputs unchanged", bool(r.get("inputs_sha256")) and r.get("inputs_sha256") == r.get("final_inputs_sha256")),
    ]
    failed = [name for name, ok in terminal if not ok]
    if failed:
        raise Pending("receipt not terminal and successful: %s" % ", ".join(failed))
    if before and binary != before[0]["binary"]:
        raise Pending("tunnel binary differs from %s's" % before[0]["role"])
    if before and a.get("params") != before[0]["params"]:
        raise Pending("analysis parameters differ from %s's" % before[0]["role"])
    if before and settings.get("sample_dt") != before[0]["sample_dt"]:
        raise Pending("sample spacing differs from %s's" % before[0]["role"])
    entry = {"role": row["role"], "level": level, "dt": dt}
    wrong = ladder(before + [entry])
    if wrong:
        raise Pending("not on the ladder: %s" % "; ".join(wrong))
    t = a["table_row"]
    bad = [k for k in MEASUREMENTS if not in_range(k, t.get(k))]
    if bad:
        raise Pending("measurement absent or out of range: %s" % ", ".join(bad))
    values = {k: t[k] for k in MEASUREMENTS}
    notes = []
    for k in COSTS:
        if t.get(k) is None:
            notes.append("%s absent from the analysis" % k)
        elif in_range(k, t[k]):
            values[k] = t[k]
        else:
            notes.append("%s out of range (%r), left out" % (k, t[k]))
    entry.update({"analysis": sha256(path), "receipt": sha256(receipt_path), "values": values, "binary": binary,
                  "params": a.get("params"), "sample_dt": settings.get("sample_dt"), "table_row": t,
                  "analysis_path": path, "receipt_path": receipt_path, "notes": notes})
    return entry


def ladder(entries):
    """The preview's ladder rule over measured rows: (role index, level, dt)."""
    at = {ROLES.index(e["role"]): (e["level"], e["dt"]) for e in entries}
    wrong = []
    for a, b, steps in ((0, 1, 1), (1, 2, 1), (0, 2, 2)):
        if a in at and b in at and (at[b][0] != at[a][0] + steps or at[b][1] != at[a][1]):
            wrong.append("%s to %s: level +%d at the same step" % (ROLES[a], ROLES[b], steps))
    for b in (3, 4):
        for a in (0, 1, 2):
            if a in at and b in at and (at[b][0] != at[a][0] + 2 - a or at[b][1] <= at[a][1]):
                wrong.append("%s to %s: grid 3's level at a longer step" % (ROLES[a], ROLES[b]))
    if 3 in at and 4 in at and (at[4][0] != at[3][0] or at[4][1] <= at[3][1]):
        wrong.append("step2 to step3: the same level at a longer step")
    return wrong


def check_reference(path):
    ref = load_json(path, "reference")
    if ref.get("tool") != TOOL or ref.get("status") != "OK" or not isinstance(ref.get("table_row"), dict):
        raise Pending("reference is not a status-OK %s analysis" % TOOL)
    for name, item in sorted((ref.get("inputs") or {}).items()):
        if not os.path.isfile(item.get("path", "")) or sha256(item["path"]) != item.get("sha256"):
            raise Pending("reference input %s changed or missing" % name)
    return ref


def verdict(table, curves, tol):
    """wake_history.py compare's verdict, recomputed: grid 3 against the reference."""
    if not tol:
        return None
    errors = {k: abs(table["grid3"][k] - table["reference"][k]) for k in MEASUREMENTS}
    errors.update({"curve_" + name: metric["max_abs"]
                   for name, metric in (curves.get("grid3") or {}).items()})
    return {name: name in errors and type(limit) in (int, float)
            and 0 <= errors[name] <= limit < math.inf
            for name, limit in tol.items()}


def check_compare(path, manifest_path, manifest, entries, reference):
    """(state, detail, curves {role: {key: x}}, slot [6] or None)."""
    c = load_json(path, "compare output")
    if c.get("tool") != TOOL or os.path.realpath(str(c.get("manifest"))) != os.path.realpath(manifest_path):
        return "refused", "the compare output is not wake_history.py's for this manifest", {}, None
    if c.get("refusals"):
        return "refused", "; ".join(c["refusals"]), {}, None
    if reference is None:
        return "refused", "the reference is not accepted", {}, None
    kept = [e["role"] for e in entries]
    if kept != ROLES:
        return "refused", "the ledger keeps pending: %s" % ", ".join(r for r in ROLES if r not in kept), {}, None
    table = c.get("table") or {}
    expected = {e["role"]: e["table_row"] for e in entries}
    expected["reference"] = reference["table_row"]
    if table != expected:
        return "refused", "the compare's table differs from the analyses' rows", {}, None
    if c.get("close_enough") != manifest.get("close_enough"):
        return "refused", "the compare's tolerances differ from the manifest's", {}, None
    curves_in = c.get("curve_against_reference") or {}
    curves = {}
    for role in ROLES:
        got = curves_in.get(role)
        if got is None:
            continue
        for key, n in CURVES.items():
            x = (got.get(n) or {}).get("max_abs")
            if in_range(key, x):
                curves.setdefault(role, {})[key] = x
    want = verdict(table, curves_in, manifest.get("close_enough"))
    if want != c.get("meets_declared_tolerance"):
        return "refused", "the compare's verdict differs from the one recomputed from its table", {}, None
    slots = c.get("slots")
    filled = bool(want) and all(want.values())
    if filled != (slots is not None):
        return "refused", "the compare's slot does not follow its verdict", {}, None
    if not filled:
        return "withheld", str(c.get("slots_withheld") or "no slot"), curves, None
    values = [slots.get(k) for k in SLOT_KEYS]
    missing = [k for k, x in zip(SLOT_KEYS, values) if not finite(x)]
    if missing:
        return "withheld", "every tolerance met, but the slot lacks %s" % ", ".join(missing), curves, None
    return "filled", "every declared tolerance met (%s)" % str(slots.get("source")), curves, values


def build(manifest_path, compare_path):
    """The ledger's lines."""
    manifest = load_json(manifest_path, "manifest")
    rows = manifest.get("rows") or []
    roles = [r.get("role") for r in rows]
    if any(r not in ROLES for r in roles) or len(set(roles)) != len(roles):
        sys.exit("manifest roles must be distinct, from %s; got %r" % (ROLES, roles))
    tol = manifest.get("close_enough") or {}
    if any(k not in TOLERANCE_KEYS or not finite(v) or v <= 0 for k, v in tol.items()):
        sys.exit("close_enough keys must be from %s, each a positive number" % TOLERANCE_KEYS)
    lines = ["schema\t%s" % SCHEMA,
             "# written by tools/wake_ledger01.py from the manifest %s" % private(os.path.basename(manifest_path)),
             "manifest\t%s" % sha256(manifest_path)]
    reference = None
    try:
        reference = check_reference(manifest.get("reference", ""))
        lines += ["reference\t%s" % sha256(manifest["reference"]), "# reference %s" % private(manifest["reference"])]
    except Pending as why:
        lines.append("# reference not accepted: %s" % private(why))
    for k in TOLERANCE_KEYS:
        if k in tol:
            lines.append("close_enough\t%s\t%r" % (k, float(tol[k])))
    entries, pending = [], {}
    by_role = {r["role"]: r for r in rows}
    for role in ROLES:
        if role not in by_role:
            pending[role] = "not in the manifest (not run, or not assigned to this role)"
            continue
        try:
            entries.append(check_row(by_role[role], entries))
        except Pending as why:
            pending[role] = private(why)
    curves, slot = {}, None
    if compare_path:
        state, detail, curves, slot = check_compare(compare_path, manifest_path, manifest, entries, reference)
    else:
        state, detail = "none", "not run" if len(rows) < len(ROLES) else "not given"
    lines.append("compare\t%s\t%s" % (state, private(detail)))
    measured = {e["role"]: e for e in entries}
    for role in ROLES:
        e = measured.get(role)
        if e is None:
            lines.append("row\t%s\tpending\t%s" % (role, pending[role]))
            continue
        lines += ["# %s: analysis %s; receipt %s; tunnel %s" % (role, private(e["analysis_path"]),
                                                               private(e["receipt_path"]), private(e["binary"])),
                  "row\t%s\tmeasured\t%d\t%r\t%s\t%s" % (role, e["level"], float(e["dt"]), e["analysis"], e["receipt"])]
        values = dict(e["values"])
        values.update(curves.get(role, {}))
        for key in MEASUREMENTS[:7] + list(CURVES) + MEASUREMENTS[7:] + COSTS:
            if key in values:
                lines.append("value\t%s\t%s\t%s" % (role, key, shown(key, values[key])))
        lines += ["# %s: %s" % (role, private(n)) for n in e["notes"]]
    if slot:
        lines += ["slot\t%s\t%r" % (k, float(x)) for k, x in zip(SLOT_KEYS, slot)]
    lines.append("w2\tpending\t%s" % W2)
    return lines


def write(lines, out):
    text = "\n".join(lines) + "\n"
    home = pathlib.Path.home()
    if str(home) in text or os.getcwd() in text or (home.name and home.name in text):
        sys.exit("the ledger would name a private path or the user; not written")
    with open(out, "x", encoding="utf-8") as fh:
        fh.write(text)
    return text


# ------------------------------------------------------------------------------------ self-test

def fixture(root, name, level, dt, status="OK", params=None, receipt=None, binary="BIN/tunnel",
            effective_level=None, **row):
    """A run directory, its analysis and receipt, as wake_history.py and the runner write them."""
    job = root / name
    run = job / "run"
    run.mkdir(parents=True)
    case = {"name": name, "flow": "wake", "mode": "fixed", "level": level, "duration": 12.0, "dt": dt,
            "sample_dt": 0.0125, "capture_after": 6.0, "capture_every": 8}
    if effective_level is not None:
        case["effective_level"] = effective_level
        level = effective_level
    (run / "settings.json").write_text(json.dumps({"case": case, "command": ["/usr/bin/time", "-l", binary, str(run)]}))
    (run / "series.csv").write_text("t,i\n0,0\n")
    inputs = {n: {"path": str(run / n), "sha256": sha256(run / n)} for n in ("series.csv", "settings.json")}
    h = 0.01 * 2.0 ** (8 - level)
    t = {"strouhal": 0.30 + h, "drag_peak": 3.2 + h, "drag_middle_ff": 3.17 + h, "drag_swing": 0.06,
         "lift_peak": 0.98 + h, "lift_middle_ff": -0.01, "lift_swing": 2.0, "drag_time_average": 3.17,
         "lift_time_average": -0.01, "delta_p_st96": 2.47 + h, "curve": "representative_samples",
         "cells_wet_leaves_max": 8960 * 4 ** (level - 8), "cells_wet_leaves_mean": 8960.0,
         "steps_run": int(3200 * 0.0025 / dt) * 2 ** (level - 8), "time_on_clock_s": 1811.3 * 8 ** (level - 8),
         "memory_max_rss_bytes": None}
    a = {"tool": TOOL, "status": status, "run_dir": str(run), "settings": case, "inputs": inputs,
         "params": params or {"min_cycles": 3}, "label": name}
    if status == "OK":
        a["table_row"] = t
        a["analysis"] = {"measurements": {"period": 1 / (10 * t["strouhal"])}, "representative_samples": []}
    else:
        a["reasons"] = ["settled window: 1 trailing cycles agree with their successor and with the last cycle; "
                        "at least 3 required"]
    (job / "analysis.json").write_text(json.dumps(a))
    r = {"finished": "2026-11-21T05:44:27+00:00", "exit": 0, "admitted": True, "preflight_only": False,
         "sbh": {"exit": 0}, "inputs_sha256": "a" * 64, "final_inputs_sha256": "a" * 64}
    r.update(receipt or {})
    (job / "resource-receipt.json").write_text(json.dumps(r))
    return dict({"role": name.split("-")[0], "analysis": str(job / "analysis.json")}, **row), a


def compare_out(manifest_path, manifest, analyses, ref, curves=0.001, **over):
    """wake_history.py compare's output for five OK rows, computed as it computes it."""
    table = {role: analyses[role]["table_row"] for role in ROLES}
    table["reference"] = ref["table_row"]
    curve = {role: {"drag": {"max_abs": curves, "rms": curves / 2}, "lift": {"max_abs": curves, "rms": curves / 2}}
             for role in ROLES}
    tol = manifest.get("close_enough")
    v = verdict(table, curve, tol)
    out = {"tool": TOOL, "manifest": manifest_path, "close_enough": tol, "refusals": [], "table": table,
           "curve_against_reference": curve, "meets_declared_tolerance": v}
    if v and all(v.values()):
        g = table["grid3"]
        out["slots"] = {"20_v1.St": g["strouhal"], "20_v1.d1": table["grid2"]["strouhal"] - table["grid1"]["strouhal"],
                        "20_v1.d2": g["strouhal"] - table["grid2"]["strouhal"], "20_v1.cells": g["cells_wet_leaves_max"],
                        "20_v1.steps": g["steps_run"], "20_v1.time_s": g["time_on_clock_s"], "source": "grid3"}
    else:
        out["slots"], out["slots_withheld"] = None, "no declared tolerance" if not tol else "a declared tolerance is not met"
    out.update(over)
    return out


def self_test(root):
    """Build fixtures under ROOT (new) and check the ledger for each case. Prints one line per case."""
    root = pathlib.Path(root)
    root.mkdir(parents=False, exist_ok=False)
    refdir = root / "reference"
    refdir.mkdir()
    (refdir / "bdforces").write_text("1 2 3\n")
    ref = {"tool": TOOL, "status": "OK", "inputs": {"bdforces": {"path": str(refdir / "bdforces"),
                                                                    "sha256": sha256(refdir / "bdforces")}},
           "table_row": {"strouhal": 0.3018, "drag_peak": 3.2274, "drag_middle_ff": 3.1958, "drag_swing": 0.0631,
                         "lift_peak": 0.9866, "lift_middle_ff": -0.0174, "lift_swing": 2.0079, "delta_p_st96": 2.4852}}
    (refdir / "reference.json").write_text(json.dumps(ref))
    failures = []

    def case(name, rows, expect, compare=None, close_enough=None, overrides=None):
        d = root / name
        d.mkdir()
        manifest_rows, analyses = [], {}
        for spec in rows:
            row, a = fixture(d, **spec)
            manifest_rows.append(row)
            analyses[row["role"]] = a
        manifest = {"reference": str(refdir / "reference.json"), "rows": manifest_rows}
        if close_enough is not None:
            manifest["close_enough"] = close_enough
        mp = str(d / "manifest.json")
        pathlib.Path(mp).write_text(json.dumps(manifest))
        for path, text in (overrides or {}).items():
            (d / path).write_text(text)
        cp = None
        if compare is not None:
            cp = str(d / "compare.json")
            pathlib.Path(cp).write_text(json.dumps(compare_out(mp, manifest, analyses, ref, **compare)))
        text = "\n".join(build(mp, cp)) + "\n"
        (d / "ledger.tsv").write_text(text)
        body = [line for line in text.splitlines() if not line.startswith("#")]
        problems = [want for want in expect if not any(line.startswith(want) for line in body)]
        if str(pathlib.Path.home()) in text or os.getcwd() in text:
            problems.append("a private path in the ledger")
        if any(line.startswith("value") and line.split("\t")[3] in ("0", "0.0") for line in body):
            problems.append("a zero value")
        print("%s %s%s" % ("ok  " if not problems else "FAIL", name, "" if not problems else ": %s" % problems))
        if problems:
            failures.append(name)
        return body

    ladder5 = [dict(name="grid1-a", level=6, dt=0.0025), dict(name="grid2-a", level=7, dt=0.0025),
               dict(name="grid3-a", level=8, dt=0.0025), dict(name="step2-a", level=8, dt=0.005),
               dict(name="step3-a", level=8, dt=0.01)]
    tol = {"strouhal": 0.01, "drag_peak": 0.05, "lift_peak": 0.05, "curve_drag": 0.01, "curve_lift": 0.01}
    case("empty", [], ["row\tgrid1\tpending\tnot in the manifest", "row\tstep3\tpending", "compare\tnone\tnot run",
                       "w2\tpending"])
    case("insufficient", [dict(name="grid3-x", level=8, dt=0.0025, status="INSUFFICIENT")],
         ["row\tgrid3\tpending\tanalysis status INSUFFICIENT: settled window"])
    case("partial", ladder5[:3], ["row\tgrid1\tmeasured\t6\t0.0025", "value\tgrid3\tstrouhal\t0.3",
                                  "value\tgrid3\tcells_wet_leaves_max\t8960", "row\tstep2\tpending"])
    case("effective-wake-level", [dict(name="grid2-refined", level=8, effective_level=9, dt=.000625)],
         ["row\tgrid2\tmeasured\t9\t0.000625", "value\tgrid2\tcells_wet_leaves_max\t35840"])
    case("not-finished", [dict(name="grid1-b", level=6, dt=0.0025, receipt={"finished": None})],
         ["row\tgrid1\tpending\treceipt not terminal and successful: finished"])
    case("failed-exit", [dict(name="grid1-c", level=6, dt=0.0025, receipt={"exit": 1, "sbh": {"exit": 2}})],
         ["row\tgrid1\tpending\treceipt not terminal and successful: exit 0, disk guard exit 0"])
    case("inputs-moved", [dict(name="grid1-d", level=6, dt=0.0025, receipt={"final_inputs_sha256": "b" * 64})],
         ["row\tgrid1\tpending\treceipt not terminal and successful: inputs unchanged"])
    case("series-changed", [dict(name="grid1-e", level=6, dt=0.0025)], ["row\tgrid1\tpending\tinput series.csv changed"],
         overrides={"grid1-e/run/series.csv": "t,i\n0,1\n"})
    case("off-ladder", [ladder5[0], dict(name="grid2-f", level=8, dt=0.0025), ladder5[2]],
         ["row\tgrid1\tmeasured", "row\tgrid2\tpending\tnot on the ladder: grid1 to grid2",
          "row\tgrid3\tmeasured\t8"])
    case("other-binary", [ladder5[0], dict(name="grid2-g", level=7, dt=0.0025, binary="BIN/other")],
         ["row\tgrid2\tpending\ttunnel binary differs from grid1's"])
    case("other-params", [ladder5[0], dict(name="grid2-h", level=7, dt=0.0025, params={"min_cycles": 2})],
         ["row\tgrid2\tpending\tanalysis parameters differ from grid1's"])
    case("withheld", ladder5, ["compare\twithheld\tno declared tolerance", "value\tgrid3\tcurve_drag_max_abs\t0.001"],
         compare={})
    case("filled", ladder5, ["close_enough\tstrouhal\t0.01", "compare\tfilled", "slot\t20_v1.St\t0.3",
                             "slot\t20_v1.cells\t8960.0", "w2\tpending"], compare={}, close_enough=tol)
    case("not-met", ladder5, ["compare\twithheld\ta declared tolerance is not met"],
         compare={"curves": 0.5}, close_enough=tol)
    case("refused", ladder5, ["compare\trefused\tlevels must rise"],
         compare={"refusals": ["levels must rise over grids 1-3 and stay at grid 3 for steps 2-3"]})
    case("table-differs", ladder5, ["compare\trefused\tthe compare's table differs"],
         compare={"table": {"grid1": {}}})
    case("verdict-differs", ladder5, ["compare\trefused\tthe compare's verdict differs"],
         compare={"meets_declared_tolerance": {"strouhal": True}}, close_enough=tol)
    no_curves = {"curve_against_reference": {}}
    scalar_verdicts = {key: True for key in ("strouhal", "drag_peak", "lift_peak")}
    case("missing-declared-curves-false-pass", ladder5,
         ["compare\trefused\tthe compare's verdict differs"],
         compare={**no_curves, "meets_declared_tolerance": scalar_verdicts}, close_enough=tol)
    case("missing-declared-curves-withheld", ladder5,
         ["compare\twithheld\tmissing measured values for curve_drag, curve_lift"],
         compare={**no_curves, "meets_declared_tolerance": {
                      **scalar_verdicts, "curve_drag": False, "curve_lift": False},
                  "slots": None, "slots_withheld": "missing measured values for curve_drag, curve_lift"},
         close_enough=tol)
    case("row-demoted", ladder5[:4] + [dict(name="step3-i", level=8, dt=0.01, receipt={"exit": 1})],
         ["compare\trefused\tthe ledger keeps pending: step3", "row\tstep3\tpending"], compare={})
    print("self-test: %d of %d cases failed" % (len(failures), 20) if failures else "self-test: all 20 cases pass")
    return 1 if failures else 0


def main(argv):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--manifest")
    p.add_argument("--compare")
    p.add_argument("--out")
    p.add_argument("--self-test", dest="self_test")
    a = p.parse_args(argv)
    if a.self_test:
        return self_test(a.self_test)
    if not (a.manifest and a.out):
        p.error("--manifest and --out are required")
    if os.path.exists(a.out):
        sys.exit("refusing to overwrite %s" % a.out)
    text = write(build(a.manifest, a.compare), a.out)
    print(text, end="")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
