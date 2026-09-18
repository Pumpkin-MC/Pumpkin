#!/usr/bin/env python3
"""Plan and run Cargo commands for the packages affected by a pull request."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from typing import Any


PACKAGE_NAME = re.compile(r"^[A-Za-z0-9_-]+$")

PR_TEST_MATRIX = {
    "include": [
        {"name": "Linux x64", "os": "ubuntu-latest"},
        {"name": "Windows x64", "os": "windows-latest"},
        {"name": "macOS", "os": "macos-latest"},
    ]
}

FULL_TEST_MATRIX = {
    "include": [
        {"name": "Linux x64", "os": "ubuntu-latest"},
        {"name": "Linux ARM64", "os": "ubuntu-26.04-arm"},
        {"name": "Windows x64", "os": "windows-latest"},
        {"name": "Windows ARM64", "os": "windows-11-arm"},
        {"name": "macOS", "os": "macos-latest"},
    ]
}

FULL_REBUILD_PATHS = {
    ".github/scripts/ci_packages.py",
    ".github/workflows/rust.yml",
    ".gitmodules",
    "Cargo.lock",
    "Cargo.toml",
    "rust-toolchain.toml",
}

RUST_CI_IGNORED_ROOTS = {
    ".github/ISSUE_TEMPLATE",
    ".devcontainer",
    "docs",
}

RUST_CI_IGNORED_FILES = {
    ".github/dependabot.yml",
    ".github/FUNDING.yml",
    ".github/PULL_REQUEST_TEMPLATE.md",
    ".github/workflows/nix.yml",
    ".github/workflows/docker.yml",
    ".github/workflows/release.yml",
    ".github/workflows/reviewers.yml",
    ".github/workflows/sync-wit.yml",
    ".github/workflows/typos.yml",
    ".dockerignore",
    ".editorconfig",
    ".envrc",
    ".gitignore",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "Dockerfile",
    "LICENSE",
    "README.md",
    "SECURITY.md",
    "assets/NOTICE.md",
    "assets/bedrock/README.md",
    "default.nix",
    "docker-compose.yml",
    "egg-pumpkin.json",
    "flake.lock",
    "flake.nix",
    "shell.nix",
    "typos.toml",
}

WIT_ROOT = Path("crates/pumpkin-plugin-wit")


def run_command(command: list[str], *, capture: bool = False) -> str:
    result = subprocess.run(
        command,
        check=True,
        text=True,
        stdout=subprocess.PIPE if capture else None,
    )
    return result.stdout if capture else ""


def cargo_metadata() -> tuple[Path, list[dict[str, Any]]]:
    metadata = json.loads(
        run_command(
            ["cargo", "metadata", "--format-version", "1", "--no-deps"],
            capture=True,
        )
    )
    workspace_root = Path(metadata["workspace_root"]).resolve()
    workspace_members = set(metadata["workspace_members"])
    packages = [
        package
        for package in metadata["packages"]
        if package["id"] in workspace_members
    ]
    return workspace_root, packages


def package_graph(
    workspace_root: Path, packages: list[dict[str, Any]]
) -> tuple[dict[Path, str], dict[str, set[str]]]:
    package_by_root = {
        Path(package["manifest_path"]).resolve().parent: package["name"]
        for package in packages
    }
    reverse_dependencies = {package["name"]: set() for package in packages}

    for package in packages:
        dependent = package["name"]
        for dependency in package["dependencies"]:
            dependency_path = dependency.get("path")
            if dependency_path is None:
                continue
            dependency_name = package_by_root.get(Path(dependency_path).resolve())
            if dependency_name is not None:
                reverse_dependencies[dependency_name].add(dependent)

    relative_roots = {
        root.relative_to(workspace_root): name for root, name in package_by_root.items()
    }
    return relative_roots, reverse_dependencies


def is_rust_ci_ignored_change(path: Path) -> bool:
    path_text = path.as_posix()
    if path_text in RUST_CI_IGNORED_FILES:
        return True
    return any(
        path_text == root or path_text.startswith(f"{root}/")
        for root in RUST_CI_IGNORED_ROOTS
    )


def changed_paths(base: str, head: str) -> list[Path]:
    if not base or not head:
        raise ValueError("pull request planning requires both base and head SHAs")
    output = run_command(
        [
            "git",
            "diff",
            "--no-renames",
            "--name-only",
            "--diff-filter=ACMRD",
            f"{base}...{head}",
            "--",
        ],
        capture=True,
    )
    return [Path(line) for line in output.splitlines() if line]


def has_wit_changes(changed: list[Path]) -> bool:
    return any(path == WIT_ROOT or WIT_ROOT in path.parents for path in changed)


def reverse_dependency_closure(
    directly_affected: set[str], reverse_dependencies: dict[str, set[str]]
) -> set[str]:
    affected = set(directly_affected)
    pending = list(directly_affected)
    while pending:
        dependency = pending.pop()
        for dependent in reverse_dependencies[dependency]:
            if dependent not in affected:
                affected.add(dependent)
                pending.append(dependent)
    return affected


def select_packages(
    changed: list[Path],
    package_roots: dict[Path, str],
    reverse_dependencies: dict[str, set[str]],
) -> tuple[list[str], bool, str]:
    all_packages = sorted(reverse_dependencies)
    directly_affected: set[str] = set()

    for path in changed:
        path_text = path.as_posix()
        if path_text in FULL_REBUILD_PATHS or path_text.startswith(".cargo/"):
            return all_packages, True, f"Workspace file changed: {path_text}"

        matching_roots = [
            root for root in package_roots if path == root or root in path.parents
        ]
        if matching_roots:
            package_root = max(matching_roots, key=lambda root: len(root.parts))
            directly_affected.add(package_roots[package_root])
            continue

        if is_rust_ci_ignored_change(path):
            continue

        # Unknown repository-level inputs may be consumed by build scripts or
        # include_* macros, so fall back to the complete workspace.
        return all_packages, True, f"Unmapped file changed: {path_text}"

    affected = sorted(
        reverse_dependency_closure(directly_affected, reverse_dependencies)
    )
    return affected, False, "Changed packages and their dependents"


def write_outputs(output_path: Path, outputs: dict[str, str]) -> None:
    with output_path.open("a", encoding="utf-8") as output_file:
        for name, value in outputs.items():
            output_file.write(f"{name}={value}\n")


def plan(args: argparse.Namespace) -> None:
    workspace_root, packages = cargo_metadata()
    package_roots, reverse_dependencies = package_graph(workspace_root, packages)
    all_packages = sorted(reverse_dependencies)

    if args.event == "pull_request":
        changed = changed_paths(args.base, args.head)
        selected, run_full, reason = select_packages(
            changed, package_roots, reverse_dependencies
        )
        should_check_wit = has_wit_changes(changed)
    else:
        selected = all_packages
        run_full = True
        should_check_wit = True
        reason = {
            "push": "Pushed to master",
            "workflow_dispatch": "Started manually",
        }.get(args.event, f"Triggered by {args.event.replace('_', ' ')}")

    has_code = bool(selected)
    test_matrix = FULL_TEST_MATRIX if run_full else PR_TEST_MATRIX
    outputs = {
        "affected_packages": json.dumps(selected, separators=(",", ":")),
        "has_code": str(has_code).lower(),
        "run_full": str(run_full).lower(),
        "should_check_wit": str(should_check_wit).lower(),
        "test_matrix": json.dumps(test_matrix, separators=(",", ":")),
    }
    write_outputs(Path(args.output), outputs)

    print(f"CI scope: {'full' if run_full else 'affected'}")
    print(f"Reason: {reason}")
    print(f"Packages ({len(selected)}): {', '.join(selected) if selected else 'none'}")


def cargo_package_insert_index(command: list[str]) -> int:
    if len(command) < 2 or command[0] != "cargo":
        raise ValueError("affected commands must start with cargo")

    if command[1] in {"build", "check", "clippy", "test"}:
        return 2
    if len(command) >= 3 and command[1:3] == ["nextest", "run"]:
        return 3
    if command[1] == "ndk" and "build" in command[2:]:
        return command.index("build", 2) + 1
    raise ValueError(f"unsupported Cargo command: {' '.join(command)}")


def run_for_packages(args: argparse.Namespace) -> None:
    packages_json = os.environ.get("CI_AFFECTED_PACKAGES")
    if packages_json is None:
        raise ValueError("CI_AFFECTED_PACKAGES is not set")

    packages = json.loads(packages_json)
    if not isinstance(packages, list) or not packages:
        raise ValueError("CI_AFFECTED_PACKAGES must contain at least one package")
    if not all(
        isinstance(package, str) and PACKAGE_NAME.fullmatch(package)
        for package in packages
    ):
        raise ValueError("CI_AFFECTED_PACKAGES contains an invalid package name")

    package_arguments = [
        argument for package in packages for argument in ("-p", package)
    ]
    insert_at = cargo_package_insert_index(args.command)
    command = args.command[:insert_at] + package_arguments + args.command[insert_at:]
    print("Running:", " ".join(command))
    run_command(command)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="action", required=True)

    plan_parser = subparsers.add_parser("plan")
    plan_parser.add_argument("--event", required=True)
    plan_parser.add_argument("--base", default="")
    plan_parser.add_argument("--head", default="")
    plan_parser.add_argument("--output", required=True)
    plan_parser.set_defaults(func=plan)

    run_parser = subparsers.add_parser("run")
    run_parser.add_argument("command", nargs=argparse.REMAINDER)
    run_parser.set_defaults(func=run_for_packages)

    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.action == "run" and args.command[:1] == ["--"]:
        args.command = args.command[1:]
    try:
        args.func(args)
    except (json.JSONDecodeError, subprocess.CalledProcessError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
