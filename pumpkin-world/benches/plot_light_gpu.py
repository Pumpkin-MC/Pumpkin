"""Plots light_gpu bench results. Usage: python plot_light_gpu.py results.json out.png"""

import json
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

SURFACE = "#fcfcfb"
INK = "#0b0b0b"
MUTED = "#52514e"
COLORS = {"cpu": "#2a78d6", "igpu": "#eb6834", "dgpu": "#1baf7a"}
LABELS = {"cpu": "CPU (BFS)", "igpu": "Intel iGPU", "dgpu": "NVIDIA dGPU"}

rows = json.load(open(sys.argv[1]))
out = sys.argv[2]

by_backend = {}
for r in rows:
    by_backend.setdefault(r["backend"], []).append(r)
for v in by_backend.values():
    v.sort(key=lambda r: r["voxels"])

sizes = sorted({r["chunks"] for r in rows})
xs_labels = [f"{c}x{c}\n{next(r['voxels'] for r in rows if r['chunks'] == c) / 1e6:.1f}M" for c in sizes]

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12.5, 5.4), facecolor=SURFACE)

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
    ax.set_yscale("log")


def draw(ax, backend, ys, dashed=False):
    x = range(len(ys))
    ax.plot(
        x, ys, color=COLORS[backend], linewidth=2, marker="o", markersize=8,
        markeredgecolor=SURFACE, markeredgewidth=2,
        linestyle="--" if dashed else "-",
    )
    return ys


# Left: total wall clock, what a server would actually pay.
for backend in ("cpu", "igpu", "dgpu"):
    ys = [r["mean_ms"] for r in by_backend[backend]]
    draw(ax1, backend, ys)
    ax1.annotate(
        LABELS[backend], (len(ys) - 1, ys[-1]), textcoords="offset points",
        xytext=(9, 0), color=MUTED, fontsize=9.5, va="center",
    )

ax1.set_title("Total wall clock per region", color=INK, fontsize=12.5, pad=12, loc="left")
ax1.set_ylabel("mean time (ms, log scale)", color=MUTED, fontsize=9.5)
ax1.set_xlabel("region size (chunks square / voxels)", color=MUTED, fontsize=9.5)

# Right: kernel time alone, no host transfer.
cpu_ys = [r["mean_ms"] for r in by_backend["cpu"]]
draw(ax2, "cpu", cpu_ys)
ax2.annotate(
    "CPU (whole BFS)", (len(cpu_ys) - 1, cpu_ys[-1]), textcoords="offset points",
    xytext=(9, 0), color=MUTED, fontsize=9.5, va="center",
)
for backend in ("igpu", "dgpu"):
    ys = [r["compute_ms"] for r in by_backend[backend]]
    draw(ax2, backend, ys, dashed=True)
    ax2.annotate(
        LABELS[backend] + " kernel", (len(ys) - 1, ys[-1]), textcoords="offset points",
        xytext=(9, 0), color=MUTED, fontsize=9.5, va="center",
    )

ax2.set_title("Compute only, transfer excluded", color=INK, fontsize=12.5, pad=12, loc="left")
ax2.set_xlabel("region size (chunks square / voxels)", color=MUTED, fontsize=9.5)

fig.suptitle(
    "Block-light propagation: CPU vs Vulkan compute",
    color=INK, fontsize=15, x=0.055, ha="left", y=0.975,
)
fig.text(
    0.055, 0.915,
    "Mean of 5 runs. GPU output matched the CPU BFS exactly on these "
    "uniform-luminance scenes. Intel Arrow Lake-S iGPU and RTX 5070 Laptop dGPU, Vulkan.",
    color=MUTED, fontsize=9.5, ha="left",
)
fig.subplots_adjust(left=0.075, right=0.86, top=0.79, bottom=0.13, wspace=0.42)
fig.savefig(out, dpi=170, facecolor=SURFACE)
print("wrote", out)
