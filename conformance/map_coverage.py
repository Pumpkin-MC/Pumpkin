#!/usr/bin/env python3
"""
Mechanically determine, per enumerated vanilla class, whether an analogue exists
anywhere in the Pumpkin workspace. No AI calls -- pure name normalization + lookup
against an index of Rust struct/enum names and file stems. This is the "coverage"
half of the conformance score; it answers "does Pumpkin even attempt this," not
"does it behave the same" (that's fidelity, scored separately and only on covered
units).
"""
import json
import re
import subprocess
from pathlib import Path

REPO_ROOT = Path("/home/eshanki/Pumpkin")
CRATES = [
    "pumpkin", "pumpkin-data", "pumpkin-world", "pumpkin-inventory",
    "pumpkin-protocol", "pumpkin-util", "pumpkin-nbt", "pumpkin-config",
]

# suffixes vanilla appends that Pumpkin sometimes drops/renames (Block -> nothing,
# Item -> nothing, Entity kept). Try the class name as-is first, then with these
# suffixes stripped/swapped, in order.
SUFFIX_VARIANTS = ["", "Block", "Item", "Entity", "Impl"]


def camel_to_snake(name: str) -> str:
    s = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", name)
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s)
    return s.lower()


def strip_suffix(name: str, suffix: str) -> str:
    if suffix and name.endswith(suffix) and len(name) > len(suffix):
        return name[: -len(suffix)]
    return name


def build_struct_index():
    out = subprocess.run(
        ["rg", "-o", "--no-filename", "-g", "*.rs",
         r"^\s*pub (?:struct|enum) (\w+)", "-r", "$1"] + CRATES,
        cwd=REPO_ROOT, capture_output=True, text=True, check=False,
    )
    names = set(out.stdout.split())
    return names


def build_filestem_index():
    out = subprocess.run(
        ["find"] + CRATES + ["-name", "*.rs"],
        cwd=REPO_ROOT, capture_output=True, text=True, check=False,
    )
    stems = {}
    for line in out.stdout.splitlines():
        p = Path(line)
        stem = p.stem
        if stem == "mod":
            stem = p.parent.name
        stems.setdefault(stem, line)
    return stems


def candidate_names(class_name: str):
    seen = set()
    for suf in SUFFIX_VARIANTS:
        base = strip_suffix(class_name, suf)
        for n in (class_name, base):
            if n not in seen:
                seen.add(n)
                yield n


def find_coverage(class_name: str, struct_index, struct_index_ci, stem_index):
    for cand in candidate_names(class_name):
        if cand in struct_index:
            return "struct_match", cand, None
    for cand in candidate_names(class_name):
        if cand.lower() in struct_index_ci:
            return "struct_match_ci", struct_index_ci[cand.lower()], None
    for cand in candidate_names(class_name):
        snake = camel_to_snake(cand)
        if snake in stem_index:
            return "filename_match", cand, stem_index[snake]
    return "none", None, None


def main():
    units_path = Path(__file__).parent / "vanilla_units.json"
    data = json.loads(units_path.read_text())

    struct_index = build_struct_index()
    struct_index_ci = {s.lower(): s for s in struct_index}
    stem_index = build_filestem_index()
    print(f"indexed {len(struct_index)} rust structs/enums, {len(stem_index)} file stems")

    for unit in data["units"]:
        kind, matched_name, path = find_coverage(
            unit["vanilla_class"], struct_index, struct_index_ci, stem_index
        )
        unit["coverage"] = "covered" if kind != "none" else "not_covered"
        unit["coverage_match_kind"] = kind
        unit["pumpkin_ref"] = path
        unit["pumpkin_matched_name"] = matched_name
        unit["fidelity"] = "unchecked"

    covered = sum(1 for u in data["units"] if u["coverage"] == "covered")
    total = len(data["units"])
    data["coverage_pct"] = round(100 * covered / total, 2)
    data["covered_classes"] = covered

    by_subsystem = {}
    for u in data["units"]:
        s = u["subsystem"]
        d = by_subsystem.setdefault(s, {"total": 0, "covered": 0})
        d["total"] += 1
        if u["coverage"] == "covered":
            d["covered"] += 1
    for s, d in by_subsystem.items():
        d["coverage_pct"] = round(100 * d["covered"] / d["total"], 2) if d["total"] else 0.0
    data["by_subsystem"] = by_subsystem

    out_path = Path(__file__).parent / "catalog.json"
    out_path.write_text(json.dumps(data, indent=1))
    print(f"coverage: {covered}/{total} = {data['coverage_pct']}%")
    for s, d in sorted(by_subsystem.items()):
        print(f"  {s}: {d['covered']}/{d['total']} = {d['coverage_pct']}%")
    print(f"-> {out_path}")


if __name__ == "__main__":
    main()
