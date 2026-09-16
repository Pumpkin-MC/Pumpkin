#!/bin/sh
# Prints the supported protocol versions, e.g. `26.2-26.45`. ferrflow appends it
# to the workspace version as semver build metadata; see ferrflow.toml.
set -eu

SOURCE="crates/pumpkin-util/src/version.rs"

[ -f "$SOURCE" ] || { echo "game-versions: $SOURCE not found" >&2; exit 1; }

# Highest variant declared inside one enum. Java variants read V_<major>_<minor>,
# Bedrock ones V_1_<major>_<minor>, so each gets its own pattern. Scoping to the
# enum matters: both spellings appear in the other's block.
highest() {
    awk -v enum="$1" -v pattern="$2" '
        $0 ~ "^pub enum " enum " [{]" { inside = 1; next }
        inside && /^\}/               { inside = 0 }
        inside && $0 ~ pattern {
            line = $0
            gsub(/[ ,]/, "", line)
            n = split(line, part, "_")
            major = part[n - 1] + 0
            minor = part[n] + 0
            if (major > bestMajor || (major == bestMajor && minor > bestMinor)) {
                bestMajor = major
                bestMinor = minor
                found = 1
            }
        }
        END {
            if (!found) exit 1
            print bestMajor "." bestMinor
        }
    ' "$SOURCE"
}

java=$(highest JavaMinecraftVersion '^    V_[0-9]+_[0-9]+,$') ||
    { echo "game-versions: no Java variant matched in $SOURCE" >&2; exit 1; }
bedrock=$(highest BedrockMinecraftVersion '^    V_1_[0-9]+_[0-9]+,$') ||
    { echo "game-versions: no Bedrock variant matched in $SOURCE" >&2; exit 1; }

printf '%s-%s\n' "$java" "$bedrock"
