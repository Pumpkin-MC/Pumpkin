#!/usr/bin/env python3
"""
Mechanical Java -> Rust transliterator for the decompiled 26.2 server.

Emits one .rs per vanilla class, preserving package path, class/field/method names,
types (mapped where unambiguous), and inheritance as a `parent` field. Method bodies are
translated only where the statement form is mechanically safe; everything else becomes
`todo!()` with the original Java retained in a comment so nothing is lost.

WHAT THIS IS
    A faithful structural skeleton of the entire vanilla surface, produced in one pass at
    zero model cost. Useful as a checklist and as a starting point per class.

WHAT THIS IS NOT
    A working server, or code that compiles as-is. Java's deep inheritance and pervasive
    mutable aliasing have no direct Rust equivalent; ownership decisions cannot be
    inferred from syntax. Output is written OUTSIDE the repo by default so it cannot be
    mistaken for, or accidentally committed alongside, real Pumpkin code.

    It also contains Mojang's proprietary structure and must not be redistributed.

Usage: tools/transpile_vanilla.py [out-dir]     (default ~/pumpkin-vanilla-26.2/transpiled)
"""
import os
import re
import sys
from pathlib import Path

DECOMP = Path(
    os.environ.get("PUMPKIN_DECOMP", Path.home() / "pumpkin-vanilla-26.2/decompiled")
)

PRIMITIVES = {
    "void": "()", "boolean": "bool", "byte": "i8", "short": "i16", "int": "i32",
    "long": "i64", "float": "f32", "double": "f64", "char": "char",
    "String": "String", "Object": "()",
}

CLASS_RE = re.compile(
    r"^(?P<mods>(?:public|protected|private|abstract|final|static|sealed|non-sealed|\s)*)"
    r"(?P<kind>class|interface|enum|record)\s+(?P<name>\w+)"
    r"(?P<generics><[^{]*?>)?"
    r"(?:\s+extends\s+(?P<parent>[\w.<>\[\], ]+?))?"
    r"(?:\s+implements\s+(?P<impls>[\w.<>\[\], ]+?))?"
    r"\s*\{",
    re.M,
)
FIELD_RE = re.compile(
    r"^\s{3}(?:public|protected|private)\s+(?:static\s+)?(?:final\s+)?"
    r"(?P<type>[\w.<>\[\], ]+?)\s+(?P<name>\w+)\s*(?:=[^;]*)?;",
    re.M,
)
METHOD_RE = re.compile(
    r"^\s{3}(?:public|protected|private)\s+(?:static\s+)?(?:final\s+)?(?:abstract\s+)?"
    r"(?:synchronized\s+)?(?P<ret>[\w.<>\[\], ]+?)\s+(?P<name>\w+)\s*"
    r"\((?P<args>[^)]*)\)\s*(?:throws [\w, ]+)?[{;]",
    re.M,
)


RUST_KEYWORDS = {
    "type", "match", "move", "ref", "box", "impl", "trait", "fn", "let", "mut", "use",
    "mod", "self", "super", "crate", "where", "loop", "become", "final", "override",
    "abstract", "yield", "macro", "priv", "unsafe", "extern", "const", "static", "enum",
    "struct", "return", "break", "continue", "if", "else", "while", "for", "in", "as",
}


def camel_to_snake(n: str) -> str:
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", n)
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", s).lower()
    # Java freely uses names that are Rust keywords (SlabBlock has a field literally
    # called `type`); raw identifiers keep them legal instead of emitting `pub type:`.
    return f"r#{s}" if s in RUST_KEYWORDS else s


ANNOTATION_RE = re.compile(r"@\w+(?:\([^)]*\))?\s*")


def rust_type(java: str) -> str:
    # Mojang annotates parameters inline (`@Nullable LivingEntity user`); left in place
    # the annotation became part of the emitted Rust type.
    j = ANNOTATION_RE.sub("", java).strip()
    if j.endswith("[]"):
        return f"Vec<{rust_type(j[:-2])}>"
    # Generic containers: recurse on the payload. Splitting on '<' and keeping the tail
    # blindly produced garbage like `BlockState>` for nested generics, so the closing
    # bracket must be matched rather than assumed.
    if "<" in j and j.endswith(">"):
        outer, inner = j[: j.index("<")].strip(), j[j.index("<") + 1 : -1]
        outer = outer.split(".")[-1]
        # Only descend into single-payload containers; multi-arg generics (Map<K,V>,
        # codecs, functional interfaces) have no safe mechanical Rust shape.
        if outer in ("List", "Set", "Collection", "Iterable", "Stream"):
            return f"Vec<{rust_type(inner)}>"
        if outer == "Optional":
            return f"Option<{rust_type(inner)}>"
        return outer or "()"
    if j in PRIMITIVES:
        return PRIMITIVES[j]
    return j.split(".")[-1].strip() or "()"


def split_top_level(sig: str) -> list[str]:
    """Split a Java parameter list on commas that are not inside <...> or [...].

    A naive sig.split(',') shreds generic parameters -- `Builder<Block, BlockState>`
    became two bogus params and emitted `builder: BlockState>`.
    """
    parts, depth, cur = [], 0, []
    for ch in sig:
        if ch in "<[":
            depth += 1
        elif ch in ">]":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur))
    return [p for p in parts if p.strip()]


def parse_args(sig: str) -> list[tuple[str, str]]:
    out = []
    for part in split_top_level(sig):
        toks = ANNOTATION_RE.sub("", part).replace("final ", "").strip().rsplit(" ", 1)
        if len(toks) == 2:
            out.append((camel_to_snake(toks[1]), rust_type(toks[0])))
    return out


def transpile(path: Path) -> str | None:
    text = path.read_text(errors="replace")
    m = CLASS_RE.search(text)
    if not m:
        return None
    name, kind = m.group("name"), m.group("kind")
    parent = (m.group("parent") or "").strip()

    lines = [
        "// Mechanically transliterated from Mojang 26.2 by tools/transpile_vanilla.py.",
        f"// Source: {path.relative_to(DECOMP)}",
        "// Skeleton only -- does not compile. Bodies are todo!() with the Java retained.",
        "",
    ]

    fields = [(camel_to_snake(f.group("name")), rust_type(f.group("type")))
              for f in FIELD_RE.finditer(text)]
    if parent:
        fields.insert(0, ("parent", rust_type(parent)))

    if kind in ("class", "record"):
        lines.append(f"pub struct {name} {{")
        lines += [f"    pub {fn}: {ft}," for fn, ft in fields] or ["    // no fields"]
        lines.append("}")
    elif kind == "enum":
        lines.append(f"pub enum {name} {{")
        body = text[m.end(): text.find(";", m.end())]
        variants = re.findall(r"^\s{6}([A-Z][A-Z0-9_]*)\b", body, re.M)
        lines += [f"    {v}," for v in variants] or ["    // variants unresolved"]
        lines.append("}")
    else:
        lines.append(f"pub trait {name} {{")

    methods = []
    for f in METHOD_RE.finditer(text):
        ret, mn = rust_type(f.group("ret")), f.group("name")
        if mn == name:
            continue
        args = parse_args(f.group("args"))
        arglist = ", ".join(["&self"] + [f"{a}: {t}" for a, t in args])
        ret_s = "" if ret == "()" else f" -> {ret}"
        methods.append(f"    pub fn {camel_to_snake(mn)}({arglist}){ret_s} {{\n"
                       f"        todo!(\"vanilla {name}.{mn}\")\n    }}")

    if kind == "interface":
        lines.append("}")
    if methods:
        lines += ["", f"impl {name} {{", *methods, "}"]
    return "\n".join(lines) + "\n"


def main() -> int:
    out_root = Path(sys.argv[1] if len(sys.argv) > 1
                    else Path.home() / "pumpkin-vanilla-26.2/transpiled")
    src_root = DECOMP / "net/minecraft"
    if not src_root.exists():
        print(f"decompile missing at {src_root}; run tools/decompile-vanilla.sh", file=sys.stderr)
        return 1

    ok = skipped = 0
    for jf in src_root.rglob("*.java"):
        try:
            rs = transpile(jf)
        except Exception:
            rs = None
        if rs is None:
            skipped += 1
            continue
        rel = jf.relative_to(src_root).with_suffix(".rs")
        dst = out_root / rel.parent / camel_to_snake(rel.stem)
        dst = dst.with_suffix(".rs")
        dst.parent.mkdir(parents=True, exist_ok=True)
        dst.write_text(rs)
        ok += 1

    print(f"transliterated {ok} classes ({skipped} skipped) -> {out_root}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
