from pathlib import Path
from collections import defaultdict
from datetime import datetime
import argparse
import os
import sys
import time


# ============================================================
# CẤU HÌNH
# ============================================================

CODE_EXTENSIONS = {
    # C / C++
    ".c", ".h", ".cc", ".hh", ".cpp", ".cxx", ".hpp", ".hxx", ".ipp", ".tpp",

    # Rust
    ".rs",

    # Python
    ".py", ".pyw", ".pyi",

    # JavaScript / TypeScript
    ".js", ".mjs", ".cjs", ".jsx",
    ".ts", ".mts", ".cts", ".tsx",

    # Java / JVM
    ".java", ".kt", ".kts", ".scala", ".groovy",

    # C#
    ".cs", ".csx",

    # Go
    ".go",

    # PHP
    ".php", ".php3", ".php4", ".php5", ".php7", ".php8", ".phtml",

    # Ruby
    ".rb", ".rake", ".gemspec",

    # Swift / Objective-C
    ".swift", ".m", ".mm",

    # Dart
    ".dart",

    # Lua
    ".lua",

    # Perl
    ".pl", ".pm",

    # R
    ".r",

    # Julia
    ".jl",

    # Haskell
    ".hs", ".lhs",

    # Elixir / Erlang
    ".ex", ".exs", ".erl", ".hrl",

    # Clojure
    ".clj", ".cljs", ".cljc",

    # F#
    ".fs", ".fsi", ".fsx",

    # Visual Basic
    ".vb",

    # Pascal / Delphi
    ".pas", ".pp",

    # Fortran
    ".f", ".f77", ".f90", ".f95", ".f03", ".f08",

    # Assembly
    ".asm", ".s", ".S",

    # Shell
    ".sh", ".bash", ".zsh", ".fish", ".csh", ".ksh",

    # PowerShell
    ".ps1", ".psm1", ".psd1",

    # Windows scripts
    ".bat", ".cmd",

    # Web
    ".html", ".htm", ".xhtml",
    ".css", ".scss", ".sass", ".less",

    # SQL
    ".sql",

    # Solidity
    ".sol",

    # Move
    ".move",

    # Verilog / SystemVerilog / VHDL
    ".v", ".vh", ".sv", ".svh", ".vhd", ".vhdl",

    # CUDA / OpenCL
    ".cu", ".cuh", ".cl",

    # Protobuf / GraphQL
    ".proto", ".graphql", ".gql",

    # Shader
    ".hlsl", ".fx", ".glsl", ".vert", ".frag", ".geom", ".comp",

    # Nix
    ".nix",

    # Zig
    ".zig",

    # Nim
    ".nim", ".nims",

    # Crystal
    ".cr",

    # OCaml
    ".ml", ".mli",

    # Lisp / Scheme
    ".lisp", ".lsp", ".el", ".scm",

    # CMake
    ".cmake",

    # GDScript
    ".gd",

    # QML
    ".qml",

    # Robot Framework
    ".robot",
}


# File code đặc biệt không có extension
SPECIAL_CODE_FILES = {
    "makefile",
    "gnumakefile",
    "cmakelists.txt",
    "dockerfile",
    "jenkinsfile",
    "rakefile",
    "gemfile",
    "vagrantfile",
}


# Thư mục bỏ qua hoàn toàn
IGNORE_DIRS = {
    # Version control
    ".git",
    ".svn",
    ".hg",
    ".bzr",

    # Node / JS
    "node_modules",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".angular",
    ".vite",
    ".parcel-cache",

    # Python
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    ".nox",
    ".venv",
    "venv",
    "env",
    ".env",

    # Rust
    "target",

    # Java / Gradle / Maven
    ".gradle",
    ".m2",

    # IDE
    ".idea",
    ".vscode",
    ".vs",

    # Build output
    "build",
    "builds",
    "dist",
    "out",
    "bin",
    "obj",
    "release",
    "debug",

    # Coverage
    "coverage",
    ".coverage",
    "htmlcov",

    # Cache / temp
    "cache",
    ".cache",
    "tmp",
    "temp",
    ".tmp",

    # Logs
    "logs",
    "log",

    # Dependencies
    "vendor",
    "packages",

    # Generated
    "generated",
    ".generated",
}


# File bỏ qua
IGNORE_FILES = {
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "bun.lock",
    "bun.lockb",
    "cargo.lock",
    "composer.lock",
    "gemfile.lock",
    "poetry.lock",
    "pipfile.lock",
}


CODE_EXTENSIONS_LOWER = {x.lower() for x in CODE_EXTENSIONS}
IGNORE_DIRS_LOWER = {x.lower() for x in IGNORE_DIRS}
IGNORE_FILES_LOWER = {x.lower() for x in IGNORE_FILES}
SPECIAL_CODE_FILES_LOWER = {x.lower() for x in SPECIAL_CODE_FILES}


# ============================================================
# XÁC ĐỊNH KIỂU COMMENT
# ============================================================

C_STYLE = {
    ".c", ".h", ".cc", ".hh", ".cpp", ".cxx", ".hpp", ".hxx",
    ".ipp", ".tpp",
    ".java",
    ".js", ".mjs", ".cjs", ".jsx",
    ".ts", ".mts", ".cts", ".tsx",
    ".cs", ".csx",
    ".go",
    ".rs",
    ".swift",
    ".m", ".mm",
    ".kt", ".kts",
    ".dart",
    ".scala",
    ".groovy",
    ".sol",
    ".cu", ".cuh",
    ".proto",
    ".hlsl", ".fx", ".glsl", ".vert", ".frag", ".geom", ".comp",
    ".qml",
    ".gd",
}


HASH_STYLE = {
    ".py", ".pyw", ".pyi",
    ".rb", ".rake", ".gemspec",
    ".sh", ".bash", ".zsh", ".fish", ".csh", ".ksh",
    ".r",
    ".pl", ".pm",
    ".nix",
}


SQL_STYLE = {
    ".sql",
}


LUA_STYLE = {
    ".lua",
}


LISP_STYLE = {
    ".lisp", ".lsp", ".el", ".scm",
    ".clj", ".cljs", ".cljc",
}


ASM_STYLE = {
    ".asm",
}


HTML_STYLE = {
    ".html", ".htm", ".xhtml",
}


CSS_STYLE = {
    ".css", ".scss", ".sass", ".less",
}


# ============================================================
# COMMENT PARSER
# ============================================================

def has_code_c_style(line: str, state: dict) -> bool:
    """
    Xử lý:
        // comment
        /* comment */
    Không coi # là comment nên #include / #define vẫn được tính.
    """

    i = 0
    n = len(line)

    in_block = state.get("block", False)
    quote = None

    found_code = False

    while i < n:
        if in_block:
            end = line.find("*/", i)

            if end == -1:
                state["block"] = True
                return found_code

            in_block = False
            state["block"] = False
            i = end + 2
            continue

        ch = line[i]

        # Trong string
        if quote:
            found_code = True

            if ch == "\\":
                i += 2
                continue

            if ch == quote:
                quote = None

            i += 1
            continue

        # String
        if ch in ("'", '"', "`"):
            quote = ch
            found_code = True
            i += 1
            continue

        # //
        if i + 1 < n and line[i:i + 2] == "//":
            break

        # /*
        if i + 1 < n and line[i:i + 2] == "/*":
            in_block = True
            state["block"] = True
            i += 2
            continue

        if not ch.isspace():
            found_code = True

        i += 1

    state["block"] = in_block
    return found_code


def has_code_hash_style(line: str, state: dict) -> bool:
    """
    Python, shell, ruby...
    Hỗ trợ # comment.
    Python triple quote cũng được xử lý cơ bản.
    """

    stripped = line.strip()

    if not stripped:
        return False

    triple = state.get("triple")

    if triple:
        pos = line.find(triple)

        if pos == -1:
            return False

        state["triple"] = None

        remaining = line[pos + 3:]
        return bool(remaining.strip())

    # Python multiline string / docstring
    for marker in ('"""', "'''"):
        pos = line.find(marker)

        if pos != -1:
            before = line[:pos].strip()

            second = line.find(marker, pos + 3)

            if second != -1:
                after = line[second + 3:].strip()
                return bool(before or after)

            state["triple"] = marker

            return bool(before)

    quote = None
    escaped = False

    for i, ch in enumerate(line):
        if escaped:
            escaped = False
            continue

        if ch == "\\":
            escaped = True
            continue

        if quote:
            if ch == quote:
                quote = None
            continue

        if ch in ("'", '"'):
            quote = ch
            continue

        if ch == "#":
            return bool(line[:i].strip())

    return bool(line.strip())


def has_code_sql(line: str, state: dict) -> bool:
    """
    SQL:
        -- comment
        /* comment */
    """

    i = 0
    n = len(line)

    in_block = state.get("block", False)
    quote = None
    found_code = False

    while i < n:
        if in_block:
            end = line.find("*/", i)

            if end == -1:
                return found_code

            state["block"] = False
            in_block = False
            i = end + 2
            continue

        ch = line[i]

        if quote:
            found_code = True

            if ch == quote:
                if i + 1 < n and line[i + 1] == quote:
                    i += 2
                    continue

                quote = None

            i += 1
            continue

        if ch in ("'", '"'):
            quote = ch
            found_code = True
            i += 1
            continue

        if line[i:i + 2] == "--":
            break

        if line[i:i + 2] == "/*":
            state["block"] = True
            in_block = True
            i += 2
            continue

        if not ch.isspace():
            found_code = True

        i += 1

    return found_code


def has_code_html(line: str, state: dict) -> bool:
    """
    HTML:
        <!-- comment -->
    """

    i = 0
    found_code = False
    in_block = state.get("block", False)

    while i < len(line):

        if in_block:
            end = line.find("-->", i)

            if end == -1:
                return found_code

            in_block = False
            state["block"] = False
            i = end + 3
            continue

        start = line.find("<!--", i)

        if start == -1:
            if line[i:].strip():
                found_code = True
            break

        if line[i:start].strip():
            found_code = True

        end = line.find("-->", start + 4)

        if end == -1:
            state["block"] = True
            break

        i = end + 3

    return found_code


def has_code_lua(line: str, state: dict) -> bool:
    """
    Lua:
        -- comment
        --[[ block ]]
    """

    stripped = line.strip()

    if not stripped:
        return False

    if state.get("block"):
        end = line.find("]]")

        if end == -1:
            return False

        state["block"] = False
        return bool(line[end + 2:].strip())

    block_start = line.find("--[[")

    if block_start != -1:
        before = line[:block_start].strip()

        block_end = line.find("]]", block_start + 4)

        if block_end == -1:
            state["block"] = True
            return bool(before)

        after = line[block_end + 2:].strip()
        return bool(before or after)

    pos = line.find("--")

    if pos != -1:
        return bool(line[:pos].strip())

    return True


def has_code_simple_semicolon(line: str, state: dict) -> bool:
    stripped = line.strip()

    if not stripped:
        return False

    return not stripped.startswith(";")


def has_code_default(line: str, state: dict) -> bool:
    return bool(line.strip())


# ============================================================
# ĐẾM FILE
# ============================================================

def count_code_lines(file: Path) -> int:
    ext = file.suffix.lower()

    state = {}

    count = 0

    try:
        with file.open(
            "r",
            encoding="utf-8",
            errors="ignore"
        ) as f:

            for line in f:

                if not line.strip():
                    continue

                if ext in C_STYLE:
                    is_code = has_code_c_style(line, state)

                elif ext in HASH_STYLE:
                    is_code = has_code_hash_style(line, state)

                elif ext in SQL_STYLE:
                    is_code = has_code_sql(line, state)

                elif ext in HTML_STYLE:
                    is_code = has_code_html(line, state)

                elif ext in CSS_STYLE:
                    is_code = has_code_c_style(line, state)

                elif ext in LUA_STYLE:
                    is_code = has_code_lua(line, state)

                elif ext in LISP_STYLE:
                    is_code = has_code_simple_semicolon(line, state)

                elif ext in ASM_STYLE:
                    is_code = has_code_simple_semicolon(line, state)

                else:
                    is_code = has_code_default(line, state)

                if is_code:
                    count += 1

    except (OSError, UnicodeError):
        return -1

    return count


# ============================================================
# TERMINAL UI
# ============================================================

class Color:
    RESET = "\033[0m"
    BOLD = "\033[1m"
    DIM = "\033[2m"
    RED = "\033[31m"
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    BLUE = "\033[34m"
    MAGENTA = "\033[35m"
    CYAN = "\033[36m"
    WHITE = "\033[37m"
    BRIGHT_RED = "\033[91m"
    BRIGHT_GREEN = "\033[92m"
    BRIGHT_YELLOW = "\033[93m"
    BRIGHT_BLUE = "\033[94m"
    BRIGHT_MAGENTA = "\033[95m"
    BRIGHT_CYAN = "\033[96m"


USE_COLOR = True


def enable_ansi():
    if os.name != "nt":
        return

    try:
        import ctypes

        kernel32 = ctypes.windll.kernel32
        handle = kernel32.GetStdHandle(-11)
        mode = ctypes.c_uint32()

        if kernel32.GetConsoleMode(handle, ctypes.byref(mode)):
            kernel32.SetConsoleMode(handle, mode.value | 0x0004)
    except Exception:
        pass


def supports_unicode():
    try:
        "✓◆●○✖".encode(sys.stdout.encoding or "utf-8")
        return True
    except Exception:
        return False


UNICODE = supports_unicode()

ICON = {
    "scan": "◆" if UNICODE else "*",
    "file": "●" if UNICODE else "+",
    "empty": "○" if UNICODE else "-",
    "error": "✖" if UNICODE else "X",
    "done": "✓" if UNICODE else "OK",
    "folder": "▣" if UNICODE else "#",
    "info": "›" if UNICODE else ">",
}


LANGUAGE_NAMES = {
    ".rs": "Rust",
    ".py": "Python", ".pyw": "Python", ".pyi": "Python",
    ".js": "JavaScript", ".mjs": "JavaScript", ".cjs": "JavaScript", ".jsx": "JavaScript",
    ".ts": "TypeScript", ".mts": "TypeScript", ".cts": "TypeScript", ".tsx": "TypeScript",
    ".c": "C", ".h": "C/C++ Header",
    ".cc": "C++", ".cpp": "C++", ".cxx": "C++",
    ".hh": "C++ Header", ".hpp": "C++ Header", ".hxx": "C++ Header", ".ipp": "C++", ".tpp": "C++",
    ".java": "Java", ".kt": "Kotlin", ".kts": "Kotlin", ".scala": "Scala", ".groovy": "Groovy",
    ".cs": "C#", ".csx": "C#", ".go": "Go",
    ".php": "PHP", ".php3": "PHP", ".php4": "PHP", ".php5": "PHP", ".php7": "PHP", ".php8": "PHP", ".phtml": "PHP",
    ".rb": "Ruby", ".rake": "Ruby", ".gemspec": "Ruby",
    ".swift": "Swift", ".m": "Objective-C", ".mm": "Objective-C++",
    ".dart": "Dart", ".lua": "Lua", ".pl": "Perl", ".pm": "Perl", ".r": "R", ".jl": "Julia",
    ".hs": "Haskell", ".lhs": "Haskell", ".ex": "Elixir", ".exs": "Elixir", ".erl": "Erlang", ".hrl": "Erlang",
    ".clj": "Clojure", ".cljs": "Clojure", ".cljc": "Clojure",
    ".fs": "F#", ".fsi": "F#", ".fsx": "F#", ".vb": "Visual Basic",
    ".pas": "Pascal", ".pp": "Pascal",
    ".f": "Fortran", ".f77": "Fortran", ".f90": "Fortran", ".f95": "Fortran", ".f03": "Fortran", ".f08": "Fortran",
    ".asm": "Assembly", ".s": "Assembly",
    ".sh": "Shell", ".bash": "Shell", ".zsh": "Shell", ".fish": "Shell", ".csh": "Shell", ".ksh": "Shell",
    ".ps1": "PowerShell", ".psm1": "PowerShell", ".psd1": "PowerShell", ".bat": "Batch", ".cmd": "Batch",
    ".html": "HTML", ".htm": "HTML", ".xhtml": "HTML",
    ".css": "CSS", ".scss": "SCSS", ".sass": "Sass", ".less": "Less",
    ".sql": "SQL", ".sol": "Solidity", ".move": "Move",
    ".v": "Verilog", ".vh": "Verilog", ".sv": "SystemVerilog", ".svh": "SystemVerilog", ".vhd": "VHDL", ".vhdl": "VHDL",
    ".cu": "CUDA", ".cuh": "CUDA", ".cl": "OpenCL",
    ".proto": "Protobuf", ".graphql": "GraphQL", ".gql": "GraphQL",
    ".hlsl": "HLSL", ".fx": "HLSL", ".glsl": "GLSL", ".vert": "GLSL", ".frag": "GLSL", ".geom": "GLSL", ".comp": "GLSL",
    ".nix": "Nix", ".zig": "Zig", ".nim": "Nim", ".nims": "Nim", ".cr": "Crystal",
    ".ml": "OCaml", ".mli": "OCaml", ".lisp": "Lisp", ".lsp": "Lisp", ".el": "Emacs Lisp", ".scm": "Scheme",
    ".cmake": "CMake", ".gd": "GDScript", ".qml": "QML", ".robot": "Robot Framework",
    "cmake": "CMake", "makefile": "Makefile", "dockerfile": "Dockerfile",
    "jenkinsfile": "Jenkinsfile", "rakefile": "Rake", "gemfile": "Gemfile", "vagrantfile": "Vagrantfile",
}


def paint(value, code):
    if not USE_COLOR:
        return str(value)
    return f"{code}{value}{Color.RESET}"


def terminal_width():
    try:
        return max(80, min(os.get_terminal_size().columns, 150))
    except OSError:
        return 100


def rule(char="─"):
    if not UNICODE:
        char = "-"
    return char * terminal_width()


def now():
    return datetime.now().strftime("%H:%M:%S")


def shorten(text, width):
    text = str(text)
    if len(text) <= width:
        return text
    if width < 16:
        return text[:width]
    left = (width - 3) // 2
    right = width - 3 - left
    return text[:left] + "..." + text[-right:]


def language_name(key):
    return LANGUAGE_NAMES.get(key, key.lstrip(".").upper() or "Unknown")


def language_key(file):
    filename = file.name.lower()
    ext = file.suffix.lower()

    if filename == "cmakelists.txt":
        return "cmake"
    if filename in {"makefile", "gnumakefile"}:
        return "makefile"
    if filename == "dockerfile":
        return "dockerfile"
    if filename == "jenkinsfile":
        return "jenkinsfile"
    if filename == "rakefile":
        return "rakefile"
    if filename == "gemfile":
        return "gemfile"
    if filename == "vagrantfile":
        return "vagrantfile"

    return ext


def banner(root):
    print()
    print(paint(rule("═"), Color.BRIGHT_CYAN))
    print(
        paint("  CODE COUNTER", Color.BRIGHT_CYAN + Color.BOLD)
        + paint("  •  LIVE SOURCE SCANNER", Color.DIM)
    )
    print(paint(rule("═"), Color.BRIGHT_CYAN))
    print(f"{paint(ICON['folder'], Color.BRIGHT_BLUE)} Root       : {paint(root, Color.BOLD)}")
    print(f"{paint(ICON['info'], Color.BRIGHT_BLUE)} Chế độ     : {paint('LIVE SCAN', Color.BRIGHT_GREEN + Color.BOLD)}")
    print(f"{paint(ICON['info'], Color.BRIGHT_BLUE)} Dòng đếm   : code thực, bỏ dòng trống + comment")
    print(f"{paint(ICON['info'], Color.BRIGHT_BLUE)} Bỏ qua     : dependency / build / cache / VCS")
    print()


def log_file(index, total, status, lang, lines, rel_path):
    percent = (index / total * 100.0) if total else 100.0
    path_width = max(28, terminal_width() - 61)
    rel_path = shorten(rel_path, path_width)

    time_part = paint(now(), Color.DIM)
    index_part = paint(f"[{index:04d}/{total:04d}]", Color.DIM)
    percent_part = paint(f"{percent:6.2f}%", Color.CYAN)

    if status == "OK":
        icon = paint(ICON["file"], Color.BRIGHT_GREEN)
        state = paint("COUNT", Color.BRIGHT_GREEN + Color.BOLD)
        line_text = paint(f"+{lines:>8,}", Color.BRIGHT_GREEN)
        lang_text = paint(f"{lang:<14.14}", Color.BRIGHT_MAGENTA)
        path_text = paint(rel_path, Color.WHITE)
    elif status == "EMPTY":
        icon = paint(ICON["empty"], Color.DIM)
        state = paint("EMPTY", Color.DIM)
        line_text = paint(f"{0:>9}", Color.DIM)
        lang_text = paint(f"{lang:<14.14}", Color.DIM)
        path_text = paint(rel_path, Color.DIM)
    else:
        icon = paint(ICON["error"], Color.BRIGHT_RED)
        state = paint("ERROR", Color.BRIGHT_RED + Color.BOLD)
        line_text = paint(f"{'-':>9}", Color.BRIGHT_RED)
        lang_text = paint(f"{lang:<14.14}", Color.BRIGHT_RED)
        path_text = paint(rel_path, Color.BRIGHT_RED)

    print(
        f"{time_part} {index_part} {percent_part} {icon} "
        f"{state:<16} {lang_text:<24} {line_text}  {path_text}",
        flush=True,
    )


def discover_code_files(root):
    candidates = []
    ignored_dirs = 0

    for current_dir, dirs, files in os.walk(root):
        before = len(dirs)
        dirs[:] = [d for d in dirs if d.lower() not in IGNORE_DIRS_LOWER]
        ignored_dirs += before - len(dirs)

        current_path = Path(current_dir)

        for filename in files:
            filename_lower = filename.lower()

            if filename_lower in IGNORE_FILES_LOWER:
                continue

            file = current_path / filename
            ext = file.suffix.lower()

            if ext in CODE_EXTENSIONS_LOWER or filename_lower in SPECIAL_CODE_FILES_LOWER:
                candidates.append(file)

    return candidates, ignored_dirs


def print_summary(root, stats, total_files, total_lines, empty_files, errors, ignored_dirs, elapsed):
    print()
    print(paint(rule("═"), Color.BRIGHT_CYAN))
    print(paint(f"  {ICON['done']} SCAN COMPLETE", Color.BRIGHT_GREEN + Color.BOLD))
    print(paint(rule("═"), Color.BRIGHT_CYAN))

    file_rate = total_files / elapsed if elapsed > 0 else 0.0
    line_rate = total_lines / elapsed if elapsed > 0 else 0.0

    print(f"  Root             : {paint(root, Color.BOLD)}")
    print(f"  File có code     : {paint(f'{total_files:,}', Color.BRIGHT_GREEN + Color.BOLD)}")
    print(f"  Tổng dòng code   : {paint(f'{total_lines:,}', Color.BRIGHT_GREEN + Color.BOLD)}")
    print(f"  File rỗng/comment: {paint(f'{empty_files:,}', Color.DIM)}")
    print(f"  File lỗi đọc     : {paint(f'{errors:,}', Color.BRIGHT_RED if errors else Color.DIM)}")
    print(f"  Thư mục bỏ qua   : {paint(f'{ignored_dirs:,}', Color.BLUE)}")
    print(f"  Thời gian        : {paint(f'{elapsed:.3f}s', Color.BRIGHT_MAGENTA)}")
    print(f"  Tốc độ           : {paint(f'{file_rate:,.1f} file/s', Color.CYAN)}  |  {paint(f'{line_rate:,.0f} dòng/s', Color.CYAN)}")

    print()
    print(paint(rule(), Color.BRIGHT_CYAN))
    print(f"{'NGÔN NGỮ':<24}{'FILE':>12}{'DÒNG CODE':>20}{'TỶ LỆ':>14}")
    print(paint(rule(), Color.DIM))

    merged = defaultdict(lambda: {"files": 0, "lines": 0})

    for key, data in stats.items():
        name = language_name(key)
        merged[name]["files"] += data["files"]
        merged[name]["lines"] += data["lines"]

    for name, data in sorted(merged.items(), key=lambda x: x[1]["lines"], reverse=True):
        percent = data["lines"] / total_lines * 100 if total_lines else 0.0
        row = f"{name:<24}{data['files']:>12,}{data['lines']:>20,}{percent:>13.2f}%"
        print(paint(row, Color.WHITE if name != "Rust" else Color.BRIGHT_GREEN))

    print(paint(rule("═"), Color.BRIGHT_CYAN))
    print()


# ============================================================
# MAIN
# ============================================================

def main():
    global USE_COLOR

    parser = argparse.ArgumentParser(
        description="Đếm dòng code với live log, bỏ comment/dòng trống và thư mục rác."
    )
    parser.add_argument(
        "path",
        nargs="?",
        default=".",
        help="Thư mục cần quét. Mặc định: thư mục hiện tại.",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="Chỉ hiện bảng kết quả cuối, không hiện log từng file.",
    )
    parser.add_argument(
        "--no-color",
        action="store_true",
        help="Tắt màu terminal.",
    )

    args = parser.parse_args()

    enable_ansi()
    USE_COLOR = not args.no_color and sys.stdout.isatty()

    root = Path(args.path).expanduser().resolve()

    if not root.exists():
        print(f"Không tồn tại: {root}", file=sys.stderr)
        return 1

    if not root.is_dir():
        print(f"Không phải thư mục: {root}", file=sys.stderr)
        return 1

    started = time.perf_counter()
    banner(root)

    print(
        f"{paint(ICON['scan'], Color.BRIGHT_CYAN)} "
        f"{paint('Đang tìm file code...', Color.BRIGHT_CYAN + Color.BOLD)}",
        flush=True,
    )

    candidates, ignored_dirs = discover_code_files(root)
    total_candidates = len(candidates)

    print(
        f"{paint(ICON['done'], Color.BRIGHT_GREEN)} "
        f"Tìm thấy {paint(f'{total_candidates:,}', Color.BOLD)} file cần quét."
    )
    print()

    if not args.quiet:
        print(paint(rule(), Color.BRIGHT_CYAN))
        print(paint("LIVE LOG", Color.BOLD))
        print(paint(rule(), Color.BRIGHT_CYAN))

    stats = defaultdict(lambda: {"files": 0, "lines": 0})
    total_files = 0
    total_lines = 0
    empty_files = 0
    skipped_errors = 0

    for index, file in enumerate(candidates, start=1):
        key = language_key(file)
        lang = language_name(key)
        code_lines = count_code_lines(file)

        try:
            rel = str(file.relative_to(root))
        except ValueError:
            rel = str(file)

        if code_lines < 0:
            skipped_errors += 1
            if not args.quiet:
                log_file(index, total_candidates, "ERROR", lang, 0, rel)
            continue

        if code_lines == 0:
            empty_files += 1
            if not args.quiet:
                log_file(index, total_candidates, "EMPTY", lang, 0, rel)
            continue

        total_files += 1
        total_lines += code_lines
        stats[key]["files"] += 1
        stats[key]["lines"] += code_lines

        if not args.quiet:
            log_file(index, total_candidates, "OK", lang, code_lines, rel)

    elapsed = time.perf_counter() - started

    print_summary(
        root=root,
        stats=stats,
        total_files=total_files,
        total_lines=total_lines,
        empty_files=empty_files,
        errors=skipped_errors,
        ignored_dirs=ignored_dirs,
        elapsed=elapsed,
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
