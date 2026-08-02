#!/usr/bin/env python3
"""
Compute the conformance score from catalog.json (mechanical coverage) and
fidelity_results.json (sampled behavioral comparison). Two numbers, not one:

  coverage  = does a Pumpkin analogue exist at all for this vanilla class?
              (mechanical, full population, cheap, no AI)
  fidelity  = given an analogue exists, does its behavior match vanilla?
              (sampled via AI comparison, stated n, NOT full population)

composite = coverage_pct * fidelity_pass_rate -- an estimate, not a measurement,
because fidelity is sampled. Report the sample size every time this number is used.
"""
import json
from pathlib import Path

HERE = Path(__file__).parent


def main():
    catalog = json.loads((HERE / "catalog.json").read_text())
    fidelity = json.loads((HERE / "fidelity_results.json").read_text())

    coverage_pct = catalog["coverage_pct"]
    total_classes = catalog["total_classes"]
    covered_classes = catalog["covered_classes"]

    results = fidelity["results"]
    n = len(results)
    passed = sum(1 for r in results if r["verdict"] == "pass")
    failed = sum(1 for r in results if r["verdict"] == "fail")
    fidelity_pass_rate = round(100 * passed / n, 2) if n else None

    by_subsystem = {}
    for r in results:
        d = by_subsystem.setdefault(r["subsystem"], {"n": 0, "pass": 0, "fail": 0})
        d["n"] += 1
        d[r["verdict"]] += 1

    composite = round(coverage_pct * fidelity_pass_rate / 100, 2) if fidelity_pass_rate is not None else None

    checked_subsystems = sorted(by_subsystem.keys())
    all_subsystems = sorted(catalog["by_subsystem"].keys())
    unchecked_subsystems = [s for s in all_subsystems if s not in checked_subsystems]

    report = {
        "milestone": catalog["milestone"],
        "coverage": {
            "pct": coverage_pct,
            "covered": covered_classes,
            "total": total_classes,
            "method": "mechanical name match (exact struct/enum name, case-insensitive struct name, or exact snake_case filename) across the full enumerated vanilla class list -- no AI, no sampling",
            "by_subsystem": catalog["by_subsystem"],
        },
        "fidelity_sample": {
            "pass_rate_pct": fidelity_pass_rate,
            "n": n,
            "passed": passed,
            "failed": failed,
            "method": "stratified random sample (seed 262, up to 6 per subsystem) of COVERED classes only, compared against decompiled vanilla by an AI agent reading both sides",
            "checked_subsystems": checked_subsystems,
            "unchecked_subsystems": unchecked_subsystems,
            "by_subsystem": by_subsystem,
            "caveat": "This is a partial sample, not a full measurement. Do not report fidelity_pass_rate_pct as if it applies to unchecked_subsystems.",
        },
        "composite_estimate_pct": composite,
        "composite_caveat": (
            f"coverage_pct ({coverage_pct}%) is a real full-population measurement. "
            f"fidelity_pass_rate_pct ({fidelity_pass_rate}%) is from a {n}-unit sample "
            f"covering only {checked_subsystems}, not {unchecked_subsystems}. "
            "composite_estimate_pct = coverage * fidelity is therefore an ESTIMATE that "
            "assumes the sampled fidelity rate generalizes to unchecked subsystems and to "
            "match-kind categories not yet spot-checked. State the sample size whenever this "
            "number is quoted."
        ),
    }

    out_path = HERE / "score.json"
    out_path.write_text(json.dumps(report, indent=1))

    print(f"Coverage (mechanical, full population): {coverage_pct}% ({covered_classes}/{total_classes})")
    print(f"Fidelity (sampled, n={n}, subsystems={checked_subsystems}): {fidelity_pass_rate}%")
    print(f"  unchecked subsystems: {unchecked_subsystems}")
    print(f"Composite estimate: {composite}%  <- estimate, see composite_caveat in {out_path}")


if __name__ == "__main__":
    main()
