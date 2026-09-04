import { readFileSync } from "node:fs";

const VERSION_SOURCE = "crates/pumpkin-util/src/version.rs";

function highest(pattern: RegExp, source: string): string {
  const seen = [...source.matchAll(pattern)].map((m) => m.slice(1).map(Number));
  if (seen.length === 0) {
    throw new Error(`no variant matched ${pattern} in ${VERSION_SOURCE}`);
  }
  const top = seen.sort((a, b) => a[0] - b[0] || a[1] - b[1]).at(-1)!;
  return top.join(".");
}

function supportedGameVersions(): string {
  const source = readFileSync(VERSION_SOURCE, "utf8");
  const java = highest(/^\s{4}V_(\d+)_(\d+),$/gm, source);
  const bedrock = highest(/^\s{4}V_1_(\d+)_(\d+),$/gm, source);
  return `${java}-${bedrock}`;
}

export default {
  workspace: {
    tagTemplate: "{name}@v{version}",
    versioning: "semver",
    releaseCommitMode: "commit",
    releaseCommitBody: "summary",
    updateLockfiles: true,
    updateDependents: true,
    changelog: {
      sections: {
        feat: "Features",
        fix: "Bug Fixes",
        perf: "Performance",
        security: "Security",
        refactor: "Code Refactoring",
        docs: "Documentation",
      },
      groupByScope: true,
      includeCommitLinks: true,
      includeCompareLink: true,
    },
    hooks: {
      preBump: "cargo fmt --all -- --check",
      postBump: async (ctx: { newVersion: string; packagePath: string }) => {
        const { readFileSync, writeFileSync } = await import("node:fs");
        const manifest = "Cargo.toml";
        const suffix = supportedGameVersions();
        const before = readFileSync(manifest, "utf8");
        const after = before.replace(
          /^version = "[^"]+"$/m,
          `version = "${ctx.newVersion}+${suffix}"`,
        );
        if (before === after) {
          throw new Error("workspace version line not found in Cargo.toml");
        }
        writeFileSync(manifest, after);
      },
      onFailure: "abort",
    },
  },
  package: [
    {
      name: "pumpkin",
      path: ".",
      changelog: "CHANGELOG.md",
      versionedFiles: [{ path: "Cargo.toml", format: "toml" }],
    },
    {
      name: "pumpkin-plugin-api",
      path: "crates/pumpkin-plugin-api",
      changelog: "crates/pumpkin-plugin-api/CHANGELOG.md",
      versionedFiles: [
        { path: "crates/pumpkin-plugin-api/Cargo.toml", format: "toml" },
      ],
    },
    {
      name: "pumpkin-plugin-utils",
      path: "crates/pumpkin-plugin-utils",
      changelog: "crates/pumpkin-plugin-utils/CHANGELOG.md",
      versionedFiles: [
        { path: "crates/pumpkin-plugin-utils/Cargo.toml", format: "toml" },
      ],
    },
  ],
};
