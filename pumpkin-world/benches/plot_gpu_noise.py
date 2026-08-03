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
STYLES = {2: ":", 6: "-", 12: "--"}
PANEL_OCTAVES = 6

rows = json.load(open(sys.argv[1]))
out = sys.argv[2]


def voxels_of(region):
    return next(r["voxels"] for r in rows if r["region"] == region)


regions = []
for r in rows:
    if r["region"] not in regions:
        regions.append(r["region"])
regions.sort(key=voxels_of)
octaves_all = sorted({r["octaves"] for r in rows})


def pick(backend, octaves, region):
    for r in rows:
        if r["backend"] == backend and r["octaves"] == octaves and r["region"] == region:
            return r
    return None


xs_labels = [
    f"{name}\n{voxels_of(name):,}"
    if voxels_of(name) < 1_000_000
    else f"{name}\n{voxels_of(name) / 1e6:.1f}M"
    for name in regions
]

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(13.6, 5.8), facecolor=SURFACE)

for ax in (ax1, ax2):
    ax.set_facecolor(SURFACE)
    ax.grid(True, which="both", color="#e6e5e1", linewidth=0.8)
    ax.set_axisbelow(True)
    for s in ("top", "right"):
        ax.spines[s].set_visible(False)
    for s in ("bottom", "left"):
        ax.spines[s].set_color("#d8d7d2")
    ax.tick_params(colors=MUTED, labelsize=8)
    ax.set_xticks(range(len(regions)))
    ax.set_xticklabels(xs_labels, fontsize=7.5)
    ax.set_yscale("log")

for backend in ORDER:
    ys = [
        got["mean_ms"] if (got := pick(backend, PANEL_OCTAVES, name)) else None
        for name in regions
    ]
    ax1.plot(
        range(len(ys)),
        ys,
        color=COLORS[backend],
        linewidth=2,
        marker="o",
        markersize=6,
        markeredgecolor=SURFACE,
        markeredgewidth=1.5,
        label=LABELS[backend],
    )

ax1.set_title(
    f"Density field, {PANEL_OCTAVES} octaves",
    color=INK,
    fontsize=12.5,
    pad=12,
    loc="left",
)
ax1.set_ylabel("mean time (ms, log scale)", color=MUTED, fontsize=9.5)
ax1.set_xlabel("region (voxels)", color=MUTED, fontsize=9.5)
ax1.legend(frameon=False, fontsize=8.5, labelcolor=MUTED, loc="upper left")

for backend in ("igpu", "dgpu"):
    for octaves in octaves_all:
        ys = []
        for name in regions:
            base = pick("cpu-rayon", octaves, name)
            got = pick(backend, octaves, name)
            ys.append(base["mean_ms"] / got["mean_ms"] if base and got else None)
        ax2.plot(
            range(len(ys)),
            ys,
            color=COLORS[backend],
            linewidth=1.9,
            linestyle=STYLES[octaves],
            marker="o",
            markersize=5,
            markeredgecolor=SURFACE,
            markeredgewidth=1.2,
            label=f"{LABELS[backend]}, {octaves} oct",
        )

ax2.axhline(1.0, color="#7a7975", linewidth=1.3)
ax2.annotate(
    "parity with all CPU cores",
    (0, 1.0),
    textcoords="offset points",
    xytext=(4, 6),
    color=MUTED,
    fontsize=8.5,
)
ax2.set_title(
    "Speedup vs rayon: the GPU only wins in bulk",
    color=INK,
    fontsize=12.5,
    pad=12,
    loc="left",
)
ax2.set_ylabel("times faster than all CPU cores", color=MUTED, fontsize=9.5)
ax2.set_xlabel("region (voxels)", color=MUTED, fontsize=9.5)
ax2.legend(frameon=False, fontsize=7.6, labelcolor=MUTED, loc="upper left", ncol=2)

fig.suptitle(
    "Terrain density noise on the GPU: compute-bound, but only worth it in bulk",
    color=INK,
    fontsize=15,
    x=0.05,
    ha="left",
    y=0.975,
)
fig.text(
    0.05,
    0.895,
    "Mean of 5 runs. Uploads 32 bytes of parameters and downloads one byte per voxel, so unlike block light there is\n"
    "nothing to transfer in. Classification matched the CPU on every voxel. A fixed 1.5-3 ms of submit and allocation\n"
    "cost per dispatch is what sinks the small regions: a single chunk sits below the crossover on both devices.",
    color=MUTED,
    fontsize=9.0,
    ha="left",
    va="top",
)
fig.subplots_adjust(left=0.062, right=0.975, top=0.73, bottom=0.16, wspace=0.22)
fig.savefig(out, dpi=170, facecolor=SURFACE)
print("wrote", out)
