#!/bin/sh
set -eu
cd "$(dirname "$0")"
for s in propagate pack patch clear; do
    glslc --target-env=vulkan1.2 -O "$s.comp" -o "$s.spv"
done
