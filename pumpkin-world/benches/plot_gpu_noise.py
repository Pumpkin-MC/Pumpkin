"""Plots gpu_noise bench results. Usage: python plot_gpu_noise.py results.json out.png"""

import json
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

SURFACE = "#fcfcfb"
INK = "#0b0b0b"
MUTED = "#52514e"

COLORS = {
    "cpu-serial": "#9fb8d4",
    "cpu-rayon": "#2a78d6",
    "igpu": "#eb6834",
    "dgpu": "#1baf7a",
}
LABELS = {
    "cpu-serial": "CPU, single thread",
    "cpu-rayon": "CPU, all cores (rayon)",
    "igpu": "Intel iGPU",
    "dgpu": "NVIDIA dGPU",
}
ORDER = ["cpu-serial", "cpu-rayon", "igpu", "dgpu"]

rows = json.load(open(sys.argv[1]))
out = sys.argv[2]

PANEL_OCTAVES = 6
octaves_all = sorted({r["octaves"] for r in rows})
sizes = sorted({r["chunks"] for r in rows})


def series(backend, octaves):
    got = [r for r in rows if r["backend"] == backend and r["octaves"] == octaves]
    got.sort(key=lambda r: r["voxels"])
    return got


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

ax1.set_yscale("log")
ax1.set_xticks(range(len(sizes)))
ax1.set_xticklabels(xs_labels)

for backend in ORDER:
    got = series(backend, PANEL_OCTAVES)
    if not got:
        continue
    ys = [r["mean_ms"] for r in got]
    ax1.plot(
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

ax1.set_title(
    f"Density field, {PANEL_OCTAVES} octaves", color=INK, fontsize=12.5, pad=12, loc="left"
)
ax1.set_ylabel("mean time (ms, log scale)", color=MUTED, fontsize=9.5)
ax1.set_xlabel("region size (chunks square / voxels)", color=MUTED, fontsize=9.5)
ax1.legend(frameon=False, fontsize=8.8, labelcolor=MUTED, loc="upper left")

biggest = max(sizes)
ax2.set_xticks(range(len(octaves_all)))
ax2.set_xticklabels([str(o) for o in octaves_all])

base = {
    o: next(
        r["mean_ms"]
        for r in rows
        if r["backend"] == "cpu-rayon" and r["octaves"] == o and r["chunks"] == biggest
    )
    for o in octaves_all
}
for backend in ("igpu", "dgpu"):
    ys = []
    for o in octaves_all:
        got = [
            r
            for r in rows
            if r["backend"] == backend and r["octaves"] == o and r["chunks"] == biggest
        ]
        ys.append(base[o] / got[0]["mean_ms"] if got else 0.0)
    ax2.plot(
        range(len(ys)),
        ys,
        color=COLORS[backend],
        linewidth=2,
        marker="o",
        markersize=7,
        markeredgecolor=SURFACE,
        markeredgewidth=2,
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
    "parity with all CPU cores",
    (0, 1.0),
    textcoords="offset points",
    xytext=(4, 6),
    color=MUTED,
    fontsize=8.5,
)
ax2.set_title(
    f"Speedup vs rayon at {biggest}x{biggest} chunks, by arithmetic intensity",
    color=INK,
    fontsize=12.5,
    pad=12,
    loc="left",
)
ax2.set_ylabel("times faster than all CPU cores", color=MUTED, fontsize=9.5)
ax2.set_xlabel("noise octaves per voxel (more work per byte moved)", color=MUTED, fontsize=9.5)

fig.suptitle(
    "Terrain density noise on the GPU: the compute-bound case",
    color=INK,
    fontsize=15,
    x=0.05,
    ha="left",
    y=0.975,
)
fig.text(
    0.05,
    0.895,
    "Mean of 5 runs. This workload uploads 32 bytes of parameters and downloads one byte per voxel, so\n"
    "unlike block light there is nothing to transfer in. Classification matched the CPU on every voxel at\n"
    "every size. Intel Arrow Lake-S iGPU and RTX 5070 Laptop dGPU, Vulkan.",
    color=MUTED,
    fontsize=9.2,
    ha="left",
    va="top",
)
fig.subplots_adjust(left=0.065, right=0.855, top=0.74, bottom=0.13, wspace=0.30)
fig.savefig(out, dpi=170, facecolor=SURFACE)
print("wrote", out)
