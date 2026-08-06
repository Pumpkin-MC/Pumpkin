#!/usr/bin/env bash
# Rebuild the Mojang-named decompiled 26.2 server source used as this repo's parity oracle.
#
# The output is Mojang's proprietary source and must never be committed or redistributed.
# It is written outside the repo by default for that reason.
#
# Usage: tools/decompile-vanilla.sh [output-dir]
#   default output-dir: ~/pumpkin-vanilla-26.2

set -euo pipefail

OUT_DIR="${1:-$HOME/pumpkin-vanilla-26.2}"
WORK="$OUT_DIR/.work"

# Pinned 26.2 server jar, verified against the Mojang piston manifest.
JAR_URL="https://piston-data.mojang.com/v1/objects/823e2250d24b3ddac457a60c92a6a941943fcd6a/server.jar"
JAR_SHA1="823e2250d24b3ddac457a60c92a6a941943fcd6a"
JAR_SHA256="cdacdfb25898de5e4b4b0e5ddcc2722f77067e46605709c2d886c000ebb63ec5"
# SHA-256 of META-INF/versions/26.2/server-26.2.jar inside the bundler jar.
PAYLOAD_SHA256="183c0499c5f855570ee487dd38e141a53f0121f83a0b07a3bac2d8b6698823e8"

VF_VERSION="1.12.0"
VF_URL="https://github.com/Vineflower/vineflower/releases/download/${VF_VERSION}/vineflower-${VF_VERSION}.jar"
VF_SHA256="1dfcfe974395734fa467ce620661c7623d05ba83670de0529b1fbd63ff548b9d"

mkdir -p "$WORK"

check() { # file expected-hash algo
  local actual
  actual=$("$3sum" "$1" | cut -d' ' -f1)
  [ "$actual" = "$2" ] || { echo "hash mismatch for $1: got $actual, want $2" >&2; exit 1; }
}

if [ ! -f "$WORK/server.jar" ]; then
  echo "==> downloading 26.2 server jar"
  curl -sL -o "$WORK/server.jar" "$JAR_URL"
fi
check "$WORK/server.jar" "$JAR_SHA1" sha1
check "$WORK/server.jar" "$JAR_SHA256" sha256

# 26.2 ships as a bundler jar; the real server is a nested jar under META-INF/versions.
# Mojang publishes no mappings for 26.2, but the server payload is NOT obfuscated -- its
# classes already carry real net.minecraft names, so no remapping step is needed.
echo "==> extracting bundler payload"
rm -rf "$WORK/jarx"
unzip -q -o "$WORK/server.jar" -d "$WORK/jarx"
PAYLOAD="$WORK/jarx/META-INF/versions/26.2/server-26.2.jar"
check "$PAYLOAD" "$PAYLOAD_SHA256" sha256

if [ ! -f "$WORK/vineflower.jar" ]; then
  echo "==> downloading vineflower ${VF_VERSION}"
  curl -sL -o "$WORK/vineflower.jar" "$VF_URL"
fi
check "$WORK/vineflower.jar" "$VF_SHA256" sha256

command -v java >/dev/null || {
  echo "java not found. Install a JDK 21+, or fetch a portable one:" >&2
  echo "  curl -sL 'https://api.adoptium.net/v3/binary/latest/21/ga/linux/x64/jdk/hotspot/normal/eclipse' | tar xz" >&2
  exit 1
}

# The bundled libraries go on the decompiler classpath so generic signatures and field
# types resolve; without them large swathes come out as raw Object.
mapfile -t LIB_ARGS < <(find "$WORK/jarx/META-INF/libraries" -name '*.jar' | sed 's/^/-e=/')
echo "==> decompiling with ${#LIB_ARGS[@]} libraries on the classpath (several minutes)"

rm -rf "$OUT_DIR/decompiled"
mkdir -p "$OUT_DIR/decompiled"
# Vineflower 1.12.0 logs one INFO line per class regardless of --log-level/--silent
# (both were tried and neither suppresses it), so this is loud. Redirect if that matters;
# the class-count assertion below is the real success gate.
java -Xmx6G -jar "$WORK/vineflower.jar" "${LIB_ARGS[@]}" \
  "$PAYLOAD" "$OUT_DIR/decompiled"

COUNT=$(find "$OUT_DIR/decompiled" -name '*.java' | wc -l)
EXPECTED=$(unzip -l "$PAYLOAD" | awk '{print $4}' | grep '\.class$' | grep -vc '\$')
echo "==> decompiled $COUNT java files (expected $EXPECTED top-level classes)"
[ "$COUNT" -eq "$EXPECTED" ] || { echo "class count mismatch" >&2; exit 1; }
echo "==> output: $OUT_DIR/decompiled"
