#!/usr/bin/env python3
"""
Step 3 (Phase 3): Generate all 39 principal STRATO graphs (A01-A24, B01-B06,
C01-C03, D01-D06) plus 10 supporting diagnostics (S01-S10) in PNG (300 dpi),
SVG, and CSV formats under `graphs/study39/`.

Strict PowerPoint Graph B convention:
  - Method A (FDM-BTCS): red (#DC2626; distinct red shades/line styles when multiple A stages share a panel)
  - Method B (Sturm-Liouville): blue (#2563EB)
  - Experimental measurements: black (#111111) circular markers
  - Experimental upper bounds: black (#111111) downward triangles
  - Non-comparable degraded curves: grey (#6B7280) hollow markers
  - Horizontal axis: Time (hours)
  - Vertical axis: Cumulative Cs-137 release (%)
  - Every applicable A-, B-, and D-series graph overlays the same canonical experimental
    dataset and shows ONE primary final-endpoint Delta annotation per model curve.
"""

import json
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

REPO = Path(__file__).resolve().parent.parent
DATA = REPO / "data" / "study39"
EXP = DATA / "experimental"
OUT = REPO / "graphs" / "study39"
TEMPS = [1200, 1350, 1500, 1600, 1700, 1800]

RED_A = "#DC2626"
RED_A_DARK = "#991B1B"
RED_A_MID = "#B91C1C"
RED_A_LIGHT = "#EF4444"
RED_A_SOFT = "#F87171"
BLUE_B = "#2563EB"
BLACK_EXP = "#111111"
GREY_NC = "#6B7280"
ORANGE_C = "#D97706"
GREEN_D = "#059669"
PURPLE_D = "#7C3AED"

plt.rcParams.update({
    "font.family": "DejaVu Sans",
    "font.size": 10,
    "axes.titlesize": 11.2,
    "axes.titleweight": "bold",
    "axes.labelsize": 10.5,
    "legend.fontsize": 8.0,
    "xtick.labelsize": 9.5,
    "ytick.labelsize": 9.5,
    "figure.dpi": 150,
    "savefig.dpi": 300,
    "axes.grid": True,
    "grid.alpha": 0.25,
    "grid.linestyle": "--",
})

EP_DF = pd.read_csv(DATA / "endpoint_deviations.csv")


def load_curve(tc: int, stage_key: str) -> pd.DataFrame:
    return pd.read_csv(DATA / "curves" / f"{tc}c_{stage_key}.csv")


def load_meta(tc: int, stage_key: str) -> dict:
    with open(DATA / "curves" / f"{tc}c_{stage_key}.json", encoding="utf-8") as f:
        return json.load(f)


def get_ep(tc: int, cfg_id: str, method: str = "Method A") -> dict | None:
    sub = EP_DF[(EP_DF["temperature_c"] == tc) & (EP_DF["config_id"] == cfg_id) & (EP_DF["method"] == method)]
    if sub.empty:
        return None
    return sub.iloc[0].to_dict()


def save_fig(fig, gid: str, csv_df: pd.DataFrame):
    OUT.mkdir(parents=True, exist_ok=True)
    fig.savefig(OUT / f"{gid}.png", dpi=300, bbox_inches="tight", pad_inches=0.14)
    fig.savefig(OUT / f"{gid}.svg", bbox_inches="tight", pad_inches=0.14)
    plt.close(fig)
    csv_df.to_csv(OUT / f"{gid}.csv", index=False)


def overlay_experiment(ax, tc: int, log_y: bool = False):
    """Plot the single immutable canonical experimental evidence at temperature tc."""
    if tc in (1200, 1350, 1500):
        exp = pd.read_csv(EXP / f"exp_{tc}c_cs.csv")
        ub = exp[exp["series_kind"] == "upper_bound"]
        for idx_r, (_, r) in enumerate(ub.iterrows()):
            is_primary_uco = str(r["series"]).endswith("Cs-UCO")
            lbl = (
                f"Upper bound UCO (<{100*r['release_fraction']:.3g}%, Bullock/TE-2090)"
                if is_primary_uco
                else f"Upper bound {r['series'].replace('exp:', '')} (<{100*r['release_fraction']:.3g}%)"
            )
            ax.scatter(
                [r["time_h"]],
                [100.0 * r["release_fraction"]],
                marker="v",
                s=78 if is_primary_uco else 58,
                color=BLACK_EXP if is_primary_uco else "#374151",
                zorder=6,
                label=lbl,
            )
            if pd.notna(r.get("alt_bound_fraction")):
                ax.scatter(
                    [r["time_h"]],
                    [100.0 * float(r["alt_bound_fraction"])],
                    marker="v",
                    s=56,
                    facecolors="none",
                    edgecolors=BLACK_EXP,
                    lw=1.3,
                    zorder=6,
                    label=f"Collector bound (<{100*float(r['alt_bound_fraction']):.3g}%)",
                )
        nc = exp[exp["series_kind"] == "not_comparable_plotted"]
        if not nc.empty:
            for ser, grp in nc.groupby("series"):
                ax.plot(
                    grp["time_h"],
                    100.0 * grp["release_fraction"],
                    marker="o",
                    ms=3.2,
                    mfc="none",
                    mec=GREY_NC,
                    color=GREY_NC,
                    lw=0.9,
                    ls=":",
                    label=f"Degraded UO2/UC2 only ({ser.replace('exp:', '')}, N/C)",
                )
        return f"Bullock (1984) / TE-2090 (UCO <5% bound at {int(ub['time_h'].iloc[0])} h)"

    if tc == 1600:
        hfr = pd.read_csv(EXP / "exp_1600c_hfr_k31_cs137.csv")
        sub = hfr[hfr["time_h"] <= 400.0].sort_values("time_h")
        ax.scatter(
            sub["time_h"],
            100.0 * sub["value"],
            marker="v",
            s=52,
            color=BLACK_EXP,
            zorder=6,
            label=f"Upper bound HFR-K3/1 sphere (N={len(sub)}, IAEA-TECDOC-978)",
        )
        return f"IAEA-TECDOC-978 HFR-K3/1 sphere (upper bound at {sub['time_h'].iloc[-1]:.1f} h; NOT AGR-1)"

    if tc in (1700, 1800):
        exp = pd.read_csv(EXP / f"exp_{tc}c_cs.csv")
        ser = "exp:F125-Cs" if tc == 1700 else "exp:F126-Cs"
        sub = exp[exp["series"] == ser].sort_values("time_h")
        lbl = (
            "Exp: Minato HRB-22 ACT3 (N=8, intact UO2)"
            if tc == 1700
            else "Exp: Minato HRB-22 ACT4 (N=11, w=1/25 failed SiC)"
        )
        ax.scatter(
            sub["time_h"],
            100.0 * sub["release_fraction"],
            marker="o",
            s=46,
            color=BLACK_EXP,
            zorder=6,
            label=lbl,
        )
        return lbl

    return ""


def style_graph_b_axes(ax, tc: int, curves_pct: list, log_y: bool, x_max: float):
    ax.set_xlabel("Time (hours)")
    ax.set_ylabel("Cumulative Cs-137 release (%)")
    ax.set_xlim(0.0, x_max * 1.03)
    if log_y:
        ax.set_yscale("log")
        flat = np.concatenate([np.asarray(c, dtype=float) for c in curves_pct])
        pos = flat[flat > 0.0]
        y_lo = max(1e-7, float(np.min(pos)) * 0.3) if len(pos) else 1e-6
        y_hi = max(12.0, float(np.max(pos)) * 3.0) if len(pos) else 100.0
        if tc in (1200, 1350):
            y_lo = min(y_lo, 5e-4)
            y_hi = max(y_hi, 12.0)
        elif tc == 1500:
            y_lo = min(y_lo, 5e-3)
            y_hi = max(y_hi, 150.0)
        elif tc == 1600:
            y_lo = min(y_lo, 3e-4)
            y_hi = max(y_hi, 2.0)
        ax.set_ylim(y_lo, y_hi)
    else:
        flat = np.concatenate([np.asarray(c, dtype=float) for c in curves_pct])
        y_hi = float(np.nanmax(flat)) * 1.18 if len(flat) else 35.0
        ax.set_ylim(0.0, max(y_hi, 6.5 if tc == 1700 else 32.0))


def plot_group_a():
    stage_configs = [
        (1, "P01", "stage_a", "Stage A (Baseline: const D, G=0, \u03bb=0)"),
        (7, "P02", "stage_b", "Stage B (+G, Fresh particle, F_gen)"),
        (13, "P05", "stage_c", "Stage C (+G, +\u03bb, Fresh particle, F_gen)"),
        (19, "P04", "stage_d_ramp_decay_off", "Stage D (Heating ramp D(r, T(t)), G=0)"),
    ]
    for base_idx, cid, sk, stage_title in stage_configs:
        for offset, tc in enumerate(TEMPS):
            gid = f"A{base_idx + offset:02d}"
            df = load_curve(tc, sk)
            t = df["time_h"].to_numpy(dtype=float)
            fa_pct = 100.0 * df["f_method_a"].to_numpy(dtype=float)
            ep = get_ep(tc, cid, "Method A")
            log_y = tc in (1200, 1350, 1500, 1600)

            fig, ax = plt.subplots(figsize=(8.2, 5.2))
            if log_y:
                ax.plot(t[fa_pct > 0], fa_pct[fa_pct > 0], color=RED_A, lw=2.5, label="Method A: FDM-BTCS")
            else:
                ax.plot(t, fa_pct, color=RED_A, lw=2.5, label="Method A: FDM-BTCS")

            exp_src = overlay_experiment(ax, tc, log_y=log_y)

            vals_for_lim = [fa_pct]
            if ep is not None:
                vals_for_lim.append(np.array([ep["f_exp_or_u_pct"], ep["f_model_pct"]]))
            if tc in (1200, 1350, 1500):
                vals_for_lim.append(np.array([0.01, 5.0]))
            elif tc == 1600:
                vals_for_lim.append(np.array([0.0016, 0.35]))
            elif tc == 1700:
                vals_for_lim.append(np.array([5.456]))
            elif tc == 1800:
                vals_for_lim.append(np.array([14.71]))
            style_graph_b_axes(ax, tc, vals_for_lim, log_y=log_y, x_max=float(t[-1]))

            if ep is not None:
                note = (
                    f"Method A {ep['label_text']}\n"
                    f"t_final = {ep['t_final_h']:.4g} h  |  {ep['comparison_class']}"
                )
            else:
                note = "Experimental endpoint unavailable"
            box_fc = "#FEF2F2" if ep and abs(ep["delta_pp"]) > 5.0 else "#F8FAFC"
            ax.text(
                0.02,
                0.96,
                note,
                transform=ax.transAxes,
                va="top",
                fontsize=8.3,
                color=RED_A_DARK,
                fontweight="bold",
                bbox=dict(boxstyle="round,pad=0.35", fc=box_fc, ec=RED_A, lw=0.9),
            )
            ax.set_title(f"{gid} ({cid}): {stage_title} \u2014 {tc} \u00b0C (Cs-137)\n{exp_src}", fontsize=10.6)
            ax.legend(loc="lower right" if tc >= 1700 or log_y else "upper left", fontsize=7.8, framealpha=0.93)
            save_fig(
                fig,
                gid,
                pd.DataFrame({
                    "time_h": t,
                    "method_a_pct": fa_pct,
                    "config_id": cid,
                    "t_final_h": ep["t_final_h"] if ep else np.nan,
                    "endpoint_delta_pp": ep["delta_pp"] if ep else np.nan,
                }),
            )


def plot_group_b():
    for idx, tc in enumerate(TEMPS, start=1):
        gid = f"B{idx:02d}"
        df_a = load_curve(tc, "stage_a")
        meta_a = load_meta(tc, "stage_a")
        t = df_a["time_h"].to_numpy(dtype=float)
        fa_pct = 100.0 * df_a["f_method_a"].to_numpy(dtype=float)
        fb_pct = 100.0 * df_a["f_method_b"].to_numpy(dtype=float)
        vb = ~np.isnan(fb_pct)
        diff_pp = np.abs(fa_pct - fb_pct)
        vf = meta_a["numerics"].get("b_valid_from_h") or 0.0
        fl = meta_a["numerics"].get("b_accuracy_floor") or 1e-12
        log_y = tc in (1200, 1350, 1500, 1600)

        ep_a = get_ep(tc, "P01", "Method A")
        ep_b = get_ep(tc, "P01", "Method B")

        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(8.2, 6.4), sharex=True, gridspec_kw={"height_ratios": [1.65, 1.0]})
        if log_y:
            ma = fa_pct > 0
            mb = vb & (fb_pct > 0)
            ax1.plot(t[ma], fa_pct[ma], color=RED_A, lw=2.4, label="Method A: FDM-BTCS (Stage A / P01)")
            if mb.any():
                ax1.plot(t[mb], fb_pct[mb], color=BLUE_B, lw=2.2, ls="--", label="Method B: Sturm–Liouville (Stage A / P01)")
        else:
            ax1.plot(t, fa_pct, color=RED_A, lw=2.4, label="Method A: FDM-BTCS (Stage A / P01)")
            if vb.any():
                ax1.plot(t[vb], fb_pct[vb], color=BLUE_B, lw=2.2, ls="--", label="Method B: Sturm–Liouville (Stage A / P01)")

        exp_src = overlay_experiment(ax1, tc, log_y=log_y)
        vals_lim = [fa_pct]
        if tc in (1200, 1350, 1500):
            vals_lim.append(np.array([0.01, 5.0]))
        elif tc == 1600:
            vals_lim.append(np.array([0.0016, 0.35]))
        elif tc == 1700:
            vals_lim.append(np.array([5.456]))
        elif tc == 1800:
            vals_lim.append(np.array([14.71]))
        style_graph_b_axes(ax1, tc, vals_lim, log_y=log_y, x_max=float(t[-1]))
        ax1.set_xlabel("")
        if vf > 0:
            ax1.axvspan(0, min(vf, t[-1]), color="#E2E8F0", alpha=0.5, label=f"Method B withheld mask (t < {vf:.2f} h)")

        b_line = f"Method B {ep_b['label_text']}" if ep_b else "Method B: N/A"
        a_line = f"Method A {ep_a['label_text']}" if ep_a else ""
        ax1.text(
            0.02,
            0.96,
            f"{b_line}\n{a_line}",
            transform=ax1.transAxes,
            va="top",
            fontsize=8.2,
            color="#1E3A8A",
            fontweight="bold",
            bbox=dict(boxstyle="round,pad=0.32", fc="#EFF6FF", ec=BLUE_B, lw=0.9),
        )
        ax1.set_title(f"{gid}: Method B (Sturm\u2013Liouville) vs Method A & Experiment \u2014 {tc} \u00b0C\n{exp_src}", fontsize=10.5)
        ax1.legend(loc="lower right" if tc >= 1700 or log_y else "upper right", fontsize=7.6, framealpha=0.93)

        ax2.semilogy(t[vb], np.maximum(diff_pp[vb], 1e-16), color=BLUE_B, lw=1.7, label="|Method A \u2212 Method B| (Stage A, pp)")
        ax2.axhline(fl * 100.0, color="#0F172A", ls=":", lw=1.1, label=f"Method B accuracy floor ({fl*100:.2e} pp)")
        ax2.set_xlabel("Time (hours)")
        ax2.set_ylabel("|F_A \u2212 F_B| (pp)")
        ax2.legend(loc="upper right", fontsize=7.8, framealpha=0.92)
        ax2.text(
            0.02,
            0.10,
            "Constant-T P01/P02/P03/P05 applicability governed by Sturm\u2013Liouville drawing mask; ramp T(t) P04/P06/P07/P08 withheld.",
            transform=ax2.transAxes,
            fontsize=7.6,
            bbox=dict(boxstyle="round,pad=0.25", fc="#FFF7ED", ec="#FDBA74"),
        )
        fig.tight_layout()
        save_fig(fig, gid, pd.DataFrame({"time_h": t, "method_a_pct": fa_pct, "method_b_pct": fb_pct, "abs_diff_pp": diff_pp}))


def plot_group_c():
    with open(DATA / "diagnostics" / "c01_convergence.json", encoding="utf-8") as f:
        c01 = json.load(f)
    mb = pd.read_csv(DATA / "diagnostics" / "mass_balance_audit.csv")
    sp = pd.DataFrame(c01["spatial"])
    tp = pd.DataFrame(c01["temporal"])

    fig, axes = plt.subplots(1, 3, figsize=(13.6, 4.5))
    axes[0].loglog(sp["step"], sp["error"], "o-", color=RED_A, lw=2.0, ms=6, label="Observed |F_BTCS \u2212 F_exact|")
    h_ref = sp["step"].to_numpy()
    axes[0].loglog(h_ref, sp["error"].iloc[-1] * (h_ref / h_ref[-1]) ** 2, "--", color="#111111", lw=1.2, label=f"O(\u0394r\u00b2) slope (obs = {c01['last_spatial_order']:.3f})")
    axes[0].set_xlabel("Radial spacing \u0394r (\u03bcm / scaled)")
    axes[0].set_ylabel("Absolute error |F_solver \u2212 F_exact|")
    axes[0].set_title("Spatial Convergence (6 levels)")
    axes[0].legend(fontsize=8.0)

    axes[1].loglog(tp["step"], tp["error"], "s-", color=BLUE_B, lw=2.0, ms=5, label="Observed |F_BTCS \u2212 F_exact|")
    dt_ref = tp["step"].to_numpy()
    axes[1].loglog(dt_ref, tp["error"].iloc[-1] * (dt_ref / dt_ref[-1]) ** 1, "--", color="#111111", lw=1.2, label=f"O(\u0394t) slope (obs = {c01['last_temporal_order']:.3f})")
    axes[1].set_xlabel("Time step \u0394t (s)")
    axes[1].set_ylabel("Absolute error |F_solver \u2212 F_exact|")
    axes[1].set_title("Temporal Convergence (9 levels)")
    axes[1].legend(fontsize=8.0)

    axes[2].semilogy(range(len(mb)), mb["max_residual_over_bound"], "o", color=GREEN_D, ms=4.2, label=f"Run max |\u03b5| / bound ({len(mb)} runs)")
    axes[2].axhline(1.0, color=RED_A, ls="--", lw=1.3, label="Derived rounding bound (|\u03b5|/bound = 1)")
    axes[2].set_xlabel("Simulation run index (Stages A\u2013D, P06, Controlled)")
    axes[2].set_ylabel("Max |\u03b5| / rounding bound")
    axes[2].set_title(f"Mass Balance Check ({len(mb)} runs, max = {mb['max_residual_over_bound'].max():.3f})")
    axes[2].legend(fontsize=8.0)

    fig.suptitle("C01: BTCS Numerical Convergence & Mass Conservation Audit", fontsize=12.4, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.94])
    c01_csv = pd.concat([
        sp.assign(kind="spatial")[["kind", "level", "step", "count", "error", "order"]],
        tp.assign(kind="temporal")[["kind", "level", "step", "count", "error", "order"]],
    ], ignore_index=True)
    save_fig(fig, "C01", c01_csv)

    # C02: Method A vs Method B across all 6 temperatures + all-24-slot summary
    with open(DATA / "diagnostics" / "c02_method_a_vs_b.json", encoding="utf-8") as f:
        c02_raw = json.load(f)
    c02 = pd.DataFrame(c02_raw["comparisons"])
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(11.4, 4.7))
    x = np.arange(len(c02))
    w = 0.36
    ax1.bar(x - w / 2, c02["final_f_a"] * 100.0, width=w, color=RED_A, label="Method A: FDM-BTCS")
    ax1.bar(x + w / 2, c02["final_f_b"] * 100.0, width=w, color=BLUE_B, label="Method B: Sturm–Liouville")
    ax1.set_xticks(x)
    ax1.set_xticklabels([f"{int(t)} \u00b0C" for t in c02["temperature_c"]])
    ax1.set_yscale("log")
    ax1.set_ylabel("Final Stage A Cs-137 release (%)")
    ax1.set_title("End-of-Test Release Agreement (Stage A / P01)")
    ax1.legend(fontsize=8.2)

    ax2.semilogy(x, c02["max_abs_diff_pp"], "o-", color=BLUE_B, lw=2.0, ms=6, label="Max |F_A \u2212 F_B| over valid mask (pp)")
    ax2.semilogy(x, c02["accuracy_floor"] * 100.0, "s--", color="#111111", lw=1.2, ms=5, label="Method B accuracy floor (pp)")
    ax2.set_xticks(x)
    ax2.set_xticklabels([f"{int(t)} \u00b0C" for t in c02["temperature_c"]])
    ax2.set_ylabel("Maximum solver difference (pp)")
    ax2.set_title("Analytical vs Numerical Difference (max <= 1.51e-3 pp)")
    ax2.legend(fontsize=8.0)

    fig.suptitle("C02: Method A (BTCS) vs Method B (Sturm\u2013Liouville) Numerical Agreement", fontsize=12.0, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.94])
    save_fig(fig, "C02", c02)

    # C03: Unified Final-Endpoint Deviation Summary Across Stages A-D & All 6 Temperatures (1200-1800 C)
    ep_a = EP_DF[EP_DF["method"] == "Method A"]
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12.8, 5.1))
    cfgs = [("P01", "Stage A\n(Base)", RED_A), ("P03", "Stage C-dec\n(G=0,+\u03bb)", RED_A_MID), ("P04", "Stage D\n(Ramp T(t))", RED_A_DARK), ("P02", "Stage B*\n(+G, F_gen)", RED_A_LIGHT), ("P05", "Stage C*\n(+G,+\u03bb, F_gen)", RED_A_SOFT)]
    x = np.arange(len(cfgs))
    w = 0.36
    d17 = [float(ep_a[(ep_a["temperature_c"] == 1700) & (ep_a["config_id"] == cid)]["delta_pp"].iloc[0]) for cid, _, _ in cfgs]
    d18 = [float(ep_a[(ep_a["temperature_c"] == 1800) & (ep_a["config_id"] == cid)]["delta_pp"].iloc[0]) for cid, _, _ in cfgs]

    ax1.bar(x - w / 2, d17, width=w, color=RED_A, label="1700 \u00b0C ACT3 (t_final = 271 h)")
    ax1.bar(x + w / 2, d18, width=w, color=BLUE_B, label="1800 \u00b0C ACT4 (t_final = 222 h)")
    ax1.axhline(0.0, color="#111111", lw=1.0)
    for i in range(len(x)):
        ax1.text(x[i] - w / 2, d17[i] + (0.45 if d17[i] >= 0 else -1.1), f"{d17[i]:+.2f}", ha="center", fontsize=7.6)
        ax1.text(x[i] + w / 2, d18[i] + (0.45 if d18[i] >= 0 else -1.1), f"{d18[i]:+.2f}", ha="center", fontsize=7.6)
    ax1.set_xticks(x)
    ax1.set_xticklabels([lbl for _, lbl, _ in cfgs], fontsize=8.3)
    ax1.set_ylim(-6.5, 16.5)
    ax1.set_ylabel("Signed final-endpoint \u0394 = F_model \u2212 F_exp (pp)")
    ax1.set_title("Exact Measurement Endpoint Deviations (*F_gen != F_init)")
    ax1.legend(fontsize=8.0, loc="upper right")

    ub_temps = [1200, 1350, 1500, 1600]
    xb = np.arange(len(ub_temps))
    da = [float(ep_a[(ep_a["temperature_c"] == t) & (ep_a["config_id"] == "P01")]["delta_pp"].iloc[0]) for t in ub_temps]
    dc = [float(ep_a[(ep_a["temperature_c"] == t) & (ep_a["config_id"] == "P05")]["delta_pp"].iloc[0]) for t in ub_temps]
    dd = [float(ep_a[(ep_a["temperature_c"] == t) & (ep_a["config_id"] == "P04")]["delta_pp"].iloc[0]) for t in ub_temps]
    w2 = 0.25
    ax2.bar(xb - w2, da, width=w2, color=RED_A, label="Stage A (P01)")
    ax2.bar(xb, dc, width=w2, color=RED_A_LIGHT, label="Stage C* (P05, +G,+\u03bb, F_gen)")
    ax2.bar(xb + w2, dd, width=w2, color=RED_A_DARK, label="Stage D ramp (P04)")
    ax2.axhline(0.0, color="#111111", lw=1.1, ls="--", label="Upper bound (\u0394 = 0 pp)")
    for i in range(len(xb)):
        ax2.text(xb[i] - w2, da[i] + (0.5 if da[i] >= 0 else -1.3), f"{da[i]:+.2f}", ha="center", fontsize=7.5)
        ax2.text(xb[i], dc[i] + (0.5 if dc[i] >= 0 else -1.3), f"{dc[i]:+.2f}", ha="center", fontsize=7.5)
        ax2.text(xb[i] + w2, dd[i] + (0.5 if dd[i] >= 0 else -1.3), f"{dd[i]:+.2f}", ha="center", fontsize=7.5)
    ax2.set_xticks(xb)
    ax2.set_xticklabels(["1200 \u00b0C\nUCO (5%)", "1350 \u00b0C\nUCO (5%)", "1500 \u00b0C\nUCO (5%)", "1600 \u00b0C\nHFR-K3/1"])
    ax2.set_ylim(-8.5, 18.0)
    ax2.set_ylabel("\u0394 vs upper bound = F_model \u2212 U_exp (pp)")
    ax2.set_title("Upper-Bound Endpoint Differences (1200\u20131600 \u00b0C)")
    ax2.legend(fontsize=7.8, loc="upper left")

    fig.suptitle("C03: Unified Final-Endpoint Deviation Summary (All Stages A\u2013D, 1200\u20131800 \u00b0C)", fontsize=12.2, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.94])
    save_fig(fig, "C03", ep_a)


def plot_group_d():
    pts = pd.read_csv(DATA / "deviations_exact_points.csv")
    sum_df = pd.read_csv(DATA / "deviations_exact_summary.csv")

    # D01 (1200 C), D02 (1350 C), D03 (1500 C)
    d_bounds = [
        ("D01", 1200, "Bullock (1984) / TE-2090 Table 22 (t_final = 10,138 h; upper bounds only)"),
        ("D02", 1350, "Bullock (1984) / TE-2090 Table 22 (t_final = 10,685 h; upper bounds only)"),
        ("D03", 1500, "Bullock (1984) / TE-2090 Fig. 124a & Table 22 (t_final = 11,866 h; NO AGR-1 Cs data)"),
    ]
    for gid, tc, subtitle in d_bounds:
        df_a = load_curve(tc, "stage_a")
        df_b = load_curve(tc, "stage_b")
        df_c = load_curve(tc, "stage_c")
        df_cd = load_curve(tc, "stage_c_decay_only")
        df_dr = load_curve(tc, "stage_d_ramp_decay_off")
        t = df_a["time_h"].to_numpy(dtype=float)

        fig, ax = plt.subplots(figsize=(8.4, 5.4))
        for df_s, col, ls, lw, cid, lbl in [
            (df_a, RED_A, "-", 2.5, "P01", "Stage A (P01): Method A"),
            (df_cd, RED_A_MID, "-.", 1.9, "P03", "Stage C-decay (P03, G=0,+\u03bb)"),
            (df_dr, RED_A_DARK, "--", 2.1, "P04", "Stage D ramp (P04, T(t))"),
            (df_b, RED_A_LIGHT, ":", 1.8, "P02", "Stage B (P02*, +G, F_gen)"),
            (df_c, RED_A_SOFT, (0, (3, 1, 1, 1)), 1.7, "P05", "Stage C (P05*, +G,+\u03bb, F_gen)"),
        ]:
            y = 100.0 * df_s["f_method_a"].to_numpy(dtype=float)
            m = y > 0
            ep = get_ep(tc, cid, "Method A")
            d_txt = f" [{ep['label_text']}]" if ep else ""
            ax.plot(t[m], y[m], color=col, ls=ls, lw=lw, label=f"{lbl}{d_txt}")

        vb = df_a["f_method_b"].notna().to_numpy()
        fb = 100.0 * df_a["f_method_b"].to_numpy(dtype=float)
        if vb.any():
            ep_b = get_ep(tc, "P01", "Method B")
            b_txt = f" [{ep_b['label_text']}]" if ep_b else ""
            ax.plot(t[vb], fb[vb], color=BLUE_B, ls="--", lw=1.8, label=f"Method B Stage A{b_txt}")

        overlay_experiment(ax, tc, log_y=True)
        style_graph_b_axes(ax, tc, [100.0 * df_a["f_method_a"], 100.0 * df_b["f_method_a"], [0.01, 5.0]], log_y=True, x_max=float(t[-1]))
        ax.set_title(f"{gid}: Cumulative Cs-137 Release vs Experiment \u2014 {tc} \u00b0C\n{subtitle}", fontsize=10.3)
        ax.legend(loc="lower right", fontsize=7.0, framealpha=0.94)
        save_fig(fig, gid, EP_DF[(EP_DF["temperature_c"] == tc) & (EP_DF["config_id"].isin(["P01", "P02", "P03", "P04", "P05"]))])

    # D04 (1700 C Minato ACT3) & D05 (1800 C Minato ACT4)
    for gid, tc, exp_ser, case_id, title_sub in [
        ("D04", 1700, "exp:F125-Cs", "D04_1700C_ACT3", "Minato et al. (2000) HRB-22 ACT3 (N=8 intact; NOT AGR-1)"),
        ("D05", 1800, "exp:F126-Cs", "D05_1800C_ACT4_N11", "Minato et al. (2000) HRB-22 ACT4 (N=11, w=1/25 failed SiC; NOT AGR-1)"),
    ]:
        df_a = load_curve(tc, "stage_a")
        df_b = load_curve(tc, "stage_b")
        df_c = load_curve(tc, "stage_c")
        df_cd = load_curve(tc, "stage_c_decay_only")
        df_dr = load_curve(tc, "stage_d_ramp_decay_off")
        t = df_a["time_h"].to_numpy(dtype=float)

        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(8.4, 6.6), sharex=True, gridspec_kw={"height_ratios": [1.75, 1.0]})
        for df_s, col, ls, lw, cid, lbl in [
            (df_a, RED_A, "-", 2.4, "P01", "Stage A (P01)"),
            (df_cd, RED_A_MID, "-.", 1.9, "P03", "Stage C-dec (P03, G=0,+\u03bb)"),
            (df_dr, RED_A_DARK, "--", 2.1, "P04", "Stage D ramp (P04, G=0)"),
            (df_b, RED_A_LIGHT, ":", 1.8, "P02", "Stage B (P02*, +G, F_gen)"),
            (df_c, RED_A_SOFT, (0, (3, 1, 1, 1)), 1.7, "P05", "Stage C (P05*, +G,+\u03bb, F_gen)"),
        ]:
            y = 100.0 * df_s["f_method_a"].to_numpy(dtype=float)
            ep = get_ep(tc, cid, "Method A")
            d_txt = f" [{ep['label_text']}]" if ep else ""
            ax1.plot(t, y, color=col, ls=ls, lw=lw, label=f"{lbl}{d_txt}")

        vb = df_a["f_method_b"].notna().to_numpy()
        fb = 100.0 * df_a["f_method_b"].to_numpy(dtype=float)
        if vb.any():
            ep_b = get_ep(tc, "P01", "Method B")
            b_txt = f" [{ep_b['label_text']}]" if ep_b else ""
            ax1.plot(t[vb], fb[vb], color=BLUE_B, ls="--", lw=1.8, label=f"Method B Stage A{b_txt}")

        overlay_experiment(ax1, tc, log_y=False)
        style_graph_b_axes(ax1, tc, [100.0 * df_dr["f_method_a"], [5.456 if tc == 1700 else 14.71]], log_y=False, x_max=float(t[-1]))
        ax1.set_xlabel("")
        ax1.set_title(f"{gid}: Cumulative Cs-137 Release vs Experiment \u2014 {tc} \u00b0C\n{title_sub}", fontsize=10.3)
        ax1.legend(loc="upper left", fontsize=7.2, framealpha=0.94)

        ps = pts[(pts["case_id"] == case_id) & (pts["method"] == "Method A")]
        for sk, col, lbl in [
            ("stage_a", RED_A, "Stage A"),
            ("stage_c_decay_only", RED_A_MID, "Stage C-dec"),
            ("stage_d_ramp_decay_off", RED_A_DARK, "Stage D-ramp"),
            ("stage_c", RED_A_SOFT, "Stage C (*F_gen)"),
        ]:
            pss = ps[ps["stage_key"] == sk]
            srow = sum_df[(sum_df["case_id"] == case_id) & (sum_df["stage_key"] == sk) & (sum_df["method"] == "Method A")].iloc[0]
            ax2.plot(pss["time_h"], pss["delta_pp"], "o-", ms=3.8, lw=1.4, color=col,
                     label=f"{lbl}: RMSE={srow['rmse_pp']:.2f} pp, final \u0394={srow['final_delta_pp']:+.2f} pp")
        ax2.axhline(0.0, color="#111111", lw=1.0, ls="--")
        ax2.set_xlabel("Time (hours)")
        ax2.set_ylabel("\u0394 = F_model \u2212 F_exp (pp)")
        ax2.legend(loc="lower left" if gid == "D04" else "upper left", fontsize=7.2, ncol=2, framealpha=0.92)
        if gid == "D05":
            ax2.set_ylim(-6.2, 18.2)
            ax2.text(0.01, 0.04, "Note: AGR-1 4-4-2 (HUNN16-442, 5 failed SiC) & JAERI-A excluded (incompatible failure/definition).",
                     transform=ax2.transAxes, fontsize=7.3, bbox=dict(boxstyle="round,pad=0.2", fc="#FFF7ED", ec="#FDBA74"))
        fig.tight_layout()
        save_fig(fig, gid, ps)

    # D06: AGR-1 Irradiation 955-1136 C (Unavailable experiment)
    with open(DATA / "diagnostics" / "d06_irradiation.json", encoding="utf-8") as f:
        d06 = json.load(f)
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12.2, 4.9))
    cols = [RED_A_LIGHT, RED_A, RED_A_DARK]
    csv_rows = []
    for c, item in zip(cols, d06["temperatures"]):
        tc = int(item["temperature_c"])
        th = item["time_h"]
        fa = [100.0 * v for v in item["stage_a_f_method_a"]]
        fc = [100.0 * v for v in item["stage_c_f_gen_method_a"]]
        ax1.semilogy(th, np.maximum(fa, 1e-22), color=c, lw=2.0, label=f"{tc} \u00b0C Stage A Method A (end = {fa[-1]:.2e}%)")
        ax1.semilogy(th, np.maximum(fc, 1e-22), color=c, lw=1.5, ls="--", label=f"{tc} \u00b0C Stage C/D F_gen (end = {fc[-1]:.2e}%)")
        for xh, a, cv in zip(th, fa, fc):
            csv_rows.append({"temperature_c": tc, "time_h": xh, "stage_a_pct": a, "stage_c_fgen_pct": cv})

    ax1.set_xlabel("Time (hours)  [620.2 EFPD = 14,884.8 h]")
    ax1.set_ylabel("Cumulative Cs-137 release (%)")
    ax1.set_title("AGR-1 Irradiation Exploration (C15 Cs Laws, 955\u20131136 \u00b0C)")
    ax1.legend(loc="lower right", fontsize=7.4, framealpha=0.93)

    ax2.axis("off")
    txt = (
        "D06: AGR-1 Irradiation Provenance & Deviation Status\n"
        "---------------------------------------------------------\n"
        "Experimental deviation: UNAVAILABLE\n"
        "Reason: No comparable intact-particle Cs-137 irradiation\n"
        "time series exists in published AGR-1/AGR-2 records.\n\n"
        "• Capsule/compact PIE fractions are cumulative over\n"
        "  time-varying daily power & temperature, include matrix\n"
        "  holdup and failed-SiC coatings (~4,100 particles/compact).\n"
        "• Method A exploration (left) with constant TAVA (955,\n"
        "  1028, 1136 °C) and C15 diffusivity laws stays < 10^-2 %\n"
        "  (fractional release < 10^-4) throughout irradiation.\n"
        "• Method B: applicable only at constant T without source;\n"
        "  real daily-varying AGR-1 history is TIME_VARYING_TEMP."
    )
    ax2.text(0.03, 0.94, txt, va="top", family="monospace", fontsize=8.4,
             bbox=dict(boxstyle="round,pad=0.55", fc="#FFF7ED", ec="#FB923C"))
    fig.suptitle("D06: AGR-1 Irradiation-Temperature Exploration (~955\u20131136 \u00b0C, 620.2 EFPD)", fontsize=12.0, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.94])
    save_fig(fig, "D06", pd.DataFrame(csv_rows))


def plot_supporting():
    # S01 & S02: 6-temperature Stage A & Stage D-ramp overviews
    for sid, sk, title in [
        ("S01", "stage_a", "S01: Six-Temperature Cs-137 Stage A Release Overview (1200\u20131800 \u00b0C)"),
        ("S02", "stage_d_ramp_decay_off", "S02: Six-Temperature Cs-137 Stage D Ramp Overview (1200\u20131800 \u00b0C)"),
    ]:
        fig, ax = plt.subplots(figsize=(8.0, 4.9))
        rows = []
        cmap = plt.cm.inferno(np.linspace(0.15, 0.85, len(TEMPS)))
        for col, tc in zip(cmap, TEMPS):
            df = load_curve(tc, sk)
            ax.semilogx(np.maximum(df["time_h"], 0.05), 100 * df["f_method_a"], lw=2.0, color=col, label=f"{tc} \u00b0C (end = {100*df['f_method_a'].iloc[-1]:.3g}%)")
            rows.append(df.assign(temperature_c=tc)[["temperature_c", "time_h", "f_method_a"]])
        ax.set_xlabel("Time (hours, log scale)")
        ax.set_ylabel("Cumulative Cs-137 release (%)")
        ax.set_title(title)
        ax.legend(loc="upper left", fontsize=8.2)
        save_fig(fig, sid, pd.concat(rows, ignore_index=True))

    # S03: 1600 C context (HFR-K3/1 + JAERI-A + AGR-1 unavailable note)
    df16a = load_curve(1600, "stage_a")
    df16d = load_curve(1600, "stage_d_ramp_decay_off")
    hfr = pd.read_csv(EXP / "exp_1600c_hfr_k31_cs137.csv")
    jaeri = pd.read_csv(EXP / "exp_jaeri_sample_a_cs137.csv")
    j16 = jaeri[jaeri["T_C"].astype(str) == "1600"]
    ep16 = get_ep(1600, "P01", "Method A")
    fig, ax = plt.subplots(figsize=(8.0, 5.0))
    ax.semilogy(df16a["time_h"], np.maximum(100 * df16a["f_method_a"], 1e-12), color=RED_A, lw=2.2, label=f"Stage A Method A ({ep16['label_text']})" if ep16 else "Stage A Method A")
    ax.semilogy(df16d["time_h"], np.maximum(100 * df16d["f_method_a"], 1e-12), color=RED_A_DARK, lw=1.8, ls="--", label="Stage D ramp Method A (1600 \u00b0C)")
    ax.scatter(hfr["time_h"], 100 * hfr["value"], marker="v", color=BLACK_EXP, s=38, label="IAEA97-K31 HFR-K3/1 sphere (upper bound, UO2)")
    ax.scatter(j16["time_h"], 100 * j16["value"], marker="o", facecolors="none", edgecolors=GREY_NC, s=48, lw=1.4, label="IAEA97-JAERI-A (PLOT ONLY, incompatible def.)")
    ax.set_title("S03: 1600 \u00b0C Cs-137 Context (No AGR-1 Cs-137 Curve; Sphere Upper Bound Only)")
    ax.set_xlabel("Time (hours)")
    ax.set_ylabel("Cumulative Cs-137 release (%, log scale)")
    ax.legend(loc="best", fontsize=7.8)
    save_fig(fig, "S03", hfr)

    # S04 & S05: Radial concentration profiles C(r) at 1700 C and 1800 C
    with open(DATA / "diagnostics" / "radial_profiles_1700_1800.json", encoding="utf-8") as f:
        profs = json.load(f)["profiles"]
    for sid, tc in [("S04", 1700), ("S05", 1800)]:
        fig, ax = plt.subplots(figsize=(7.8, 4.8))
        p_rows = []
        cols = {"stage_a": RED_A, "stage_b": BLUE_B, "stage_c": ORANGE_C, "stage_d_ramp_decay_off": GREEN_D, "stage_d_combined": PURPLE_D}
        for entry in profs:
            if int(entry["temperature_c"]) != tc:
                continue
            r_um = entry["r_um"]
            snap = entry["snapshots"][-1]
            c_vals = np.array(snap["c_mol_m3"])
            c_norm = c_vals / max(c_vals[0], 1e-30)
            st = entry["stage"]
            ax.semilogy(r_um, np.maximum(c_norm, 1e-12), color=cols.get(st, "#111111"), lw=1.8, label=f"{st} (t={snap['time_h']:.1f} h)")
            p_rows.append(pd.DataFrame({"stage": st, "r_um": r_um, "c_mol_m3": c_vals, "c_norm": c_norm}))
        for rx in [175.0, 278.5, 317.9, 353.2]:
            ax.axvline(rx, color="#94A3B8", ls=":", lw=1.0)
        ax.set_xlabel("Particle radius r (\u03bcm)")
        ax.set_ylabel("Normalised concentration C(r) / C(0)")
        ax.set_title(f"{sid}: End-of-Test Radial Concentration Profiles across TRISO Layers \u2014 {tc} \u00b0C")
        ax.legend(loc="lower left", fontsize=8.0)
        save_fig(fig, sid, pd.concat(p_rows, ignore_index=True))

    # S06: C16 vs C15 Diffusivities
    with open(DATA / "parameters.json", encoding="utf-8") as f:
        params = json.load(f)
    fig, ax = plt.subplots(figsize=(7.8, 4.8))
    c16 = params["c16_diffusivities"]
    t_c16 = [r["temperature_c"] for r in c16]
    invT_16 = [1000.0 / (r["temperature_c"] + 273.15) for r in c16]
    layers = [("Kernel", 0, RED_A), ("IPyC / OPyC", 2, BLUE_B), ("SiC", 3, GREEN_D)]
    d_rows = []
    for lname, idx, col in layers:
        dv = [r["diffusivities_m2_s"][idx] for r in c16]
        ax.semilogy(invT_16, dv, "o-", color=col, lw=1.8, label=f"C16 {lname} (Heating test)")
        for tc, dval in zip(t_c16, dv):
            d_rows.append({"phase": "C16", "layer": lname, "T_C": tc, "D_m2_s": dval})
    c15 = params["c15_diffusivities"]
    invT_15 = [1000.0 / (r["temperature_c"] + 273.15) for r in c15]
    for lname, idx, col in layers:
        dv = [r["diffusivities_m2_s"][idx] for r in c15]
        ax.semilogy(invT_15, dv, "s--", color=col, alpha=0.75, lw=1.4, label=f"C15 {lname} (Irradiation)")
    ax.set_xlabel("1000 / T (K\u207b\u00b9)")
    ax.set_ylabel("Effective diffusivity D (m\u00b2 s\u207b\u00b9)")
    ax.set_title("S06: Cs Diffusivity Laws C16 (Heating Test) vs C15 (Irradiation)")
    ax.legend(loc="best", fontsize=7.8, ncol=2)
    save_fig(fig, "S06", pd.DataFrame(d_rows))

    # S07: Ramp temperature histories
    fig, ax = plt.subplots(figsize=(7.8, 4.6))
    t_rows = []
    for tc in TEMPS:
        ramp1 = (400 - 30) / 120.0
        hold1 = 2.0
        if tc >= 1600:
            ramp2 = (1250 - 400) / 120.0
            hold2 = 12.0
            ramp3 = (tc - 1250) / 50.0
            ts = [0, ramp1, ramp1 + hold1, ramp1 + hold1 + ramp2, ramp1 + hold1 + ramp2 + hold2, ramp1 + hold1 + ramp2 + hold2 + ramp3, ramp1 + hold1 + ramp2 + hold2 + ramp3 + 50.0]
            vs = [30, 400, 400, 1250, 1250, tc, tc]
        else:
            ramp2 = (tc - 400) / 50.0
            ts = [0, ramp1, ramp1 + hold1, ramp1 + hold1 + ramp2, ramp1 + hold1 + ramp2 + 50.0]
            vs = [30, 400, 400, tc, tc]
        ax.plot(ts, vs, lw=1.8, label=f"{tc} \u00b0C ramp (hold begins t={ts[-2]:.1f} h)")
        for xh, tv in zip(ts, vs):
            t_rows.append({"target_T_C": tc, "time_since_start_h": xh, "T_C": tv})
    ax.set_xlabel("Time since furnace ramp start (first 80 h)")
    ax.set_ylabel("Temperature (\u00b0C)")
    ax.set_title("S07: Stage D Heating-Test Temperature Ramps (AGR1-ST-C16 & 50 \u00b0C/h Profiles)")
    ax.legend(loc="lower right", fontsize=8.0)
    save_fig(fig, "S07", pd.DataFrame(t_rows))

    # S08: Operational Scenarios S1-S5 Cs-137
    scen_files = [
        ("S1 Steady-State", "s5.steady-state.cs137.1028c.json"),
        ("S2 Shutdown-Cooldown", "s5.shutdown-cooldown.cs137.1028c.json"),
        ("S3 Startup", "s5.startup.cs137.1028c.json"),
        ("S4 Control-Rod", "s5.control-rod-insertion.cs137.1028c.json"),
        ("S5 Refuelling+Heat1800", "s5.refuelling.heating1800.cs137.1028c.json"),
    ]
    fig, ax = plt.subplots(figsize=(8.0, 4.8))
    s_rows = []
    for lbl, fn in scen_files:
        fp = REPO / "data" / "results" / "curves" / fn
        if not fp.exists():
            continue
        jd = json.load(open(fp, encoding="utf-8"))
        ma = jd["method_a"]
        td = ma.get("time_d") or [x / 24.0 for x in ma.get("time_h", [])]
        fv = ma.get("plotted_release_fraction") or ma.get("release_fraction_of_total") or []
        td_clean = [x for x, y in zip(td, fv) if y is not None]
        fv_clean = [100.0 * y for y in fv if y is not None]
        ax.semilogy(td_clean, np.maximum(fv_clean, 1e-18), lw=1.7, label=lbl)
        for x, y in zip(td_clean, fv_clean):
            s_rows.append({"scenario": lbl, "time_d": x, "release_pct": y})
    ax.set_xlabel("Time (days)")
    ax.set_ylabel("Cumulative Cs-137 release (%)")
    ax.set_title("S08: Prescribed-Input Reactor Scenarios S1\u2013S5 (Cs-137, 1028 \u00b0C Normal Ref.)")
    ax.legend(loc="best", fontsize=8.0)
    save_fig(fig, "S08", pd.DataFrame(s_rows) if s_rows else pd.DataFrame({"empty": []}))

    # S09 & S10: Controlled matched-inventory & pointwise deviation profiles at 1700 C and 1800 C
    pts = pd.read_csv(DATA / "deviations_exact_points.csv")
    for sid, cid, title in [
        ("S09", "D04_1700C_ACT3", "S09: Controlled vs Uncontrolled Absolute Deviation |\u0394(t_i)| \u2014 1700 \u00b0C (ACT3)"),
        ("S10", "D05_1800C_ACT4_N11", "S10: Controlled vs Uncontrolled Absolute Deviation |\u0394(t_i)| \u2014 1800 \u00b0C (ACT4)"),
    ]:
        sub = pts[(pts["case_id"] == cid) & (pts["method"] == "Method A")]
        fig, ax = plt.subplots(figsize=(7.9, 4.7))
        for sk, col, ls, lbl in [
            ("stage_a", RED_A, "-", "Stage A (G=0, F_init)"),
            ("stage_c_decay_only", ORANGE_C, "-.", "Stage C-decay (G=0, +\u03bb)"),
            ("stage_d_ramp_decay_off", GREEN_D, "--", "Stage D-ramp (G=0)"),
            ("ctrl_gen_on_decay_off", "#0F172A", "-", "Controlled (+G matched C_init, common F)"),
            ("stage_b", BLUE_B, ":", "Stage B (Fresh +G, F_gen)"),
        ]:
            ss = sub[sub["stage_key"] == sk]
            if not ss.empty:
                ax.plot(ss["time_h"], ss["abs_delta_pp"], "o", ls=ls, ms=4.2, lw=1.6, color=col, label=lbl)
        ax.set_xlabel("Time (hours)")
        ax.set_ylabel("Absolute deviation |\u0394(t_i)| (pp)")
        ax.set_title(title)
        ax.legend(loc="best", fontsize=7.8)
        save_fig(fig, sid, sub)


def main():
    plot_group_a()
    plot_group_b()
    plot_group_c()
    plot_group_d()
    plot_supporting()
    all_png = sorted(OUT.glob("*.png"))
    print(f"Generated {len(all_png)} PNG graphs (+ SVG + CSV) in {OUT}")


if __name__ == "__main__":
    main()
