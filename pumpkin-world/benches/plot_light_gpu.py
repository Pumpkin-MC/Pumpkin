"""Plots light_gpu bench results. Usage: python plot_light_gpu.py results.json out.png"""

import json
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

SURFACE = "#fcfcfb"
INK = "#0b0b0b"
MUTED = "#52514e"

COLORS = {
    "cpu-full": "#9fb8d4",
    "cpu-delta": "#2a78d6",
    "igpu-full": "#f3b295",
    "igpu-resident": "#eb6834",
    "dgpu-full": "#94d9be",
    "dgpu-resident": "#1baf7a",
}
LABELS = {
    "cpu-full": "CPU, full rebuild",
    "cpu-delta": "CPU, incremental",
    "igpu-full": "iGPU, full re-upload",
    "igpu-resident": "iGPU, resident",
    "dgpu-full": "dGPU, full re-upload",
    "dgpu-resident": "dGPU, resident",
}
ORDER = [
    "cpu-full",
    "igpu-full",
    "dgpu-full",
    "cpu-delta",
    "igpu-resident",
    "dgpu-resident",
]

rows = json.load(open(sys.argv[1]))
out = sys.argv[2]

by_backend = {}
for r in rows:
    by_backend.setdefault(r["backend"], []).append(r)
for v in by_backend.values():
    v.sort(key=lambda r: r["voxels"])

sizes = sorted({r["chunks"] for r in rows})
xs_labels = [
    f"{c}x{c}\n{next(r['voxels'] for r in rows if r['chunks'] == c) / 1e6:.1f}M"
    for c in sizes
]

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(13.0, 5.6), facecolor=SURFACE)

for ax in (ax1, ax2):
    ax.set_facecolor(SURFACE)
    ax.grid(True, which="both", color="#e6e5e1", linewidth=0.8)
    ax.set_axisbelow(True)
    for s in ("top", "right"):
        ax.spines[s].set_visible(False)
    for s in ("bottom", "left"):
        ax.spines[s].set_color("#d8d7d2")
    ax.tick_params(colors=MUTED, labelsize=9)
    ax.set_xticks(range(len(sizes)))
    ax.set_xticklabels(xs_labels)

ax1.set_yscale("log")

for backend in ORDER:
    if backend not in by_backend:
        continue
    ys = [r["mean_ms"] for r in by_backend[backend]]
    dashed = backend.endswith(("-full",))
    ax1.plot(
        range(len(ys)),
        ys,
        color=COLORS[backend],
        linewidth=2,
        marker="o",
        markersize=7,
        markeredgecolor=SURFACE,
        markeredgewidth=2,
        linestyle="--" if dashed else "-",
        label=LABELS[backend],
    )

ax1.set_title(
    "Cost of one tick of light updates", color=INK, fontsize=12.5, pad=12, loc="left"
)
ax1.set_ylabel("mean time per tick (ms, log scale)", color=MUTED, fontsize=9.5)
ax1.set_xlabel("region size (chunks square / voxels)", color=MUTED, fontsize=9.5)
leg = ax1.legend(frameon=False, fontsize=8.8, labelcolor=MUTED, loc="upper left", ncol=2)

cpu = [r["mean_ms"] for r in by_backend["cpu-delta"]]
for backend in ("igpu-resident", "dgpu-resident"):
    if backend not in by_backend:
        continue
    gpu = [r["mean_ms"] for r in by_backend[backend]]
    ys = [c / g for c, g in zip(cpu, gpu)]
    ax2.plot(
        range(len(ys)),
        ys,
        color=COLORS[backend],
        linewidth=2,
        marker="o",
        markersize=7,
        markeredgecolor=SURFACE,
        markeredgewidth=2,
        label=LABELS[backend],
    )
    ax2.annotate(
        LABELS[backend],
        (len(ys) - 1, ys[-1]),
        textcoords="offset points",
        xytext=(9, 0),
        color=MUTED,
        fontsize=9.5,
        va="center",
    )

ax2.axhline(1.0, color="#b9b8b3", linewidth=1.2, linestyle=":")
ax2.annotate(
    "parity with the CPU",
    (0, 1.0),
    textcoords="offset points",
    xytext=(4, 5),
    color=MUTED,
    fontsize=8.5,
)
ax2.set_title(
    "Speedup over the incremental CPU solve",
    color=INK,
    fontsize=12.5,
    pad=12,
    loc="left",
)
ax2.set_ylabel("times faster than CPU (higher is better)", color=MUTED, fontsize=9.5)
ax2.set_xlabel("region size (chunks square / voxels)", color=MUTED, fontsize=9.5)

fig.suptitle(
    "Block light on the GPU: resident buffers and delta uploads",
    color=INK,
    fontsize=15,
    x=0.05,
    ha="left",
    y=0.975,
)
fig.text(
    0.05,
    0.895,
    "Mean over 24 ticks, each placing and removing a few torches. Every path was checked against the same fixed point.\n"
    "Solid lines keep the grid on the device and upload only the changed voxels; dashed lines re-upload and\n"
    "re-download the whole grid every tick. Intel Arrow Lake-S iGPU and RTX 5070 Laptop dGPU, Vulkan.",
    color=MUTED,
    fontsize=9.2,
    ha="left",
    va="top",
)
fig.subplots_adjust(left=0.065, right=0.855, top=0.74, bottom=0.13, wspace=0.30)
fig.savefig(out, dpi=170, facecolor=SURFACE)
print("wrote", out)
