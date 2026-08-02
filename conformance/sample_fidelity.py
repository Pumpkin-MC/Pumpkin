#!/usr/bin/env python3
"""
Draw a stratified random sample of *covered* conformance units for fidelity
checking (does the Pumpkin analogue actually behave like vanilla, not just
exist). Coverage is mechanical; fidelity requires reading both sides, so it's
sampled rather than run over all ~450 covered classes. Fixed seed for
reproducibility -- rerunning without changing catalog.json gives the same draw.
"""
import json
import random
from pathlib import Path

PER_SUBSYSTEM = 6
SEED = 26_2  # milestone-derived, fixed so reruns are reproducible


def main():
    catalog_path = Path(__file__).parent / "catalog.json"
    data = json.loads(catalog_path.read_text())

    rng = random.Random(SEED)
    by_subsystem = {}
    for u in data["units"]:
        if u["coverage"] == "covered":
            by_subsystem.setdefault(u["subsystem"], []).append(u)

    sample = []
    for subsystem, units in sorted(by_subsystem.items()):
        pool = sorted(units, key=lambda u: u["vanilla_class"])
        rng.shuffle(pool)
        take = pool[:PER_SUBSYSTEM]
        for u in take:
            sample.append({
                "milestone": u["milestone"],
                "subsystem": subsystem,
                "vanilla_class": u["vanilla_class"],
                "vanilla_path": u["vanilla_path"],
                "pumpkin_ref": u["pumpkin_ref"],
                "pumpkin_matched_name": u["pumpkin_matched_name"],
                "coverage_match_kind": u["coverage_match_kind"],
                "verdict": "unchecked",
            })

    out_path = Path(__file__).parent / "fidelity_sample.json"
    out_path.write_text(json.dumps({
        "milestone": data["milestone"],
        "seed": SEED,
        "per_subsystem": PER_SUBSYSTEM,
        "sample_size": len(sample),
        "units": sample,
    }, indent=1))
    print(f"drew {len(sample)} units across {len(by_subsystem)} subsystems -> {out_path}")


if __name__ == "__main__":
    main()
