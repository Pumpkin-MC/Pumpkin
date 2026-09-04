#!/bin/sh
# Stamps the supported protocol versions onto the workspace version as build
# metadata, so `0.1.0` becomes `0.1.0+26.2-26.45`.
#
# Run by ferrflow as a postBump hook. FERRFLOW_NEW_VERSION is one of the eleven
# variables ferrflow exports to a hook; see ferrflow.toml.
set -eu

SOURCE="crates/pumpkin-util/src/version.rs"
MANIFEST="Cargo.toml"

: "${FERRFLOW_NEW_VERSION:?must run as a ferrflow hook, FERRFLOW_NEW_VERSION is unset}"
[ -f "$SOURCE" ] || { echo "stamp: $SOURCE not found" >&2; exit 1; }
[ -f "$MANIFEST" ] || { echo "stamp: $MANIFEST not found" >&2; exit 1; }

# Highest variant declared inside one enum. Java variants read V_<major>_<minor>,
# Bedrock ones V_1_<major>_<minor>, so each gets its own pattern. Scoping to the
# enum matters: both spellings appear in the other's block.
highest() {
    awk -v enum="$1" -v pattern="$2" '
        $0 ~ "^pub enum " enum " \\{" { inside = 1; next }
        inside && /^\}/                { inside = 0 }
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
    { echo "stamp: no Java variant matched in $SOURCE" >&2; exit 1; }
bedrock=$(highest BedrockMinecraftVersion '^    V_1_[0-9]+_[0-9]+,$') ||
    { echo "stamp: no Bedrock variant matched in $SOURCE" >&2; exit 1; }

stamped="${FERRFLOW_NEW_VERSION}+${java}-${bedrock}"

# Only the first `version = "..."` at column zero, which is [workspace.package].
# Dependency versions are indented or inline, so they are never touched.
awk -v value="$stamped" '
    !done && /^version = "/ { print "version = \"" value "\""; done = 1; next }
    { print }
    END { if (!done) exit 1 }
' "$MANIFEST" > "$MANIFEST.stamped" ||
    { rm -f "$MANIFEST.stamped"; echo "stamp: no workspace version line in $MANIFEST" >&2; exit 1; }

mv "$MANIFEST.stamped" "$MANIFEST"
echo "stamp: workspace version set to $stamped"
