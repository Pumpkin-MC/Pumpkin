#!/usr/bin/env python3
"""
Mechanically enumerate conformance units (vanilla classes + their public/protected
methods) from the decompiled 26.2 vanilla source. No AI calls. This defines the
denominator for the conformance catalog -- it must be built before any fidelity
comparison so the catalog isn't self-selected from prior audits.
"""
import json
import re
import sys
from pathlib import Path

DECOMP_ROOT = Path("/tmp/pumpkin-vanilla-26.2/decompiled/net/minecraft")
MILESTONE = "26.2"

SUBSYSTEMS = {
    "world/level/block": "block",
    "world/entity": "entity",
    "world/item": "item",
    "world/inventory": "inventory",
    "world/level/material": "material",
    "world/food": "food",
    "world/level/border": "border",
    "world/level/gameevent": "gameevent",
}

CLASS_RE = re.compile(
    r"^\s*(?:public|protected)\s+(?:abstract\s+|final\s+|static\s+)*(?:class|interface|enum|record)\s+(\w+)",
    re.MULTILINE,
)
METHOD_RE = re.compile(
    r"^\s*(?:public|protected)\s+"
    r"(?:static\s+|final\s+|abstract\s+|synchronized\s+|default\s+)*"
    r"(?:<[^>]+>\s+)?"
    r"[\w\[\]<>,.? ]+?\s+"
    r"(\w+)\s*\([^;{]*\)\s*"
    r"(?:throws\s+[\w.,\s]+)?\s*\{",
    re.MULTILINE,
)
# things that look like methods but aren't (constructors handled separately, control keywords)
KEYWORD_FALSE_POSITIVES = {
    "if", "for", "while", "switch", "catch", "synchronized", "return",
}


def extract_units(java_path: Path, subsystem: str):
    try:
        text = java_path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []

    class_match = CLASS_RE.search(text)
    class_name = class_match.group(1) if class_match else java_path.stem

    methods = set()
    for m in METHOD_RE.finditer(text):
        name = m.group(1)
        if name in KEYWORD_FALSE_POSITIVES or name == class_name:
            continue
        methods.add(name)

    rel_path = str(java_path.relative_to(DECOMP_ROOT.parent.parent))
    return {
        "milestone": MILESTONE,
        "subsystem": subsystem,
        "vanilla_class": class_name,
        "vanilla_path": rel_path,
        "method_count": len(methods),
        "methods": sorted(methods),
    }


def main():
    if not DECOMP_ROOT.exists():
        print(f"decomp root missing: {DECOMP_ROOT}", file=sys.stderr)
        sys.exit(1)

    units = []
    for subdir, subsystem in SUBSYSTEMS.items():
        pkg_dir = DECOMP_ROOT / subdir
        if not pkg_dir.exists():
            continue
        for java_file in sorted(pkg_dir.rglob("*.java")):
            unit = extract_units(java_file, subsystem)
            if unit:
                units.append(unit)

    out = {
        "milestone": MILESTONE,
        "generated_by": "conformance/enumerate.py",
        "total_classes": len(units),
        "total_methods": sum(u["method_count"] for u in units),
        "units": units,
    }
    out_path = Path(__file__).parent / "vanilla_units.json"
    out_path.write_text(json.dumps(out, indent=1))
    print(f"enumerated {out['total_classes']} classes, {out['total_methods']} methods -> {out_path}")


if __name__ == "__main__":
    main()
