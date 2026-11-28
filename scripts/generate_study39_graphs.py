#!/usr/bin/env python3
"""
Step 4: Generate all 39 principal STRATO Phase 2 graphs (A01-A24, B01-B06,
C01-C03, D01-D06) plus 10 supporting diagnostics (S01-S10) in PNG (300 dpi),
SVG, and underlying CSV formats under `graphs/study39/`.

Strict styling:
  - Method A (FDM-BTCS): #DC2626 red continuous
  - Method B (Sturm-Liouville): #2563EB blue continuous / dashed markers
  - Experimental measurements: #111111 black circles
  - Experimental upper bounds: #111111 black downward triangles ('v')
  - Non-comparable datasets: #6B7280 grey hollow markers clearly annotated
  - Graph B axes: Time (hours) vs Cumulative release fraction (%)
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
BLUE_B = "#2563EB"
ORANGE_C = "#D97706"
GREEN_D = "#059669"
PURPLE_D = "#7C3AED"
BLACK_EXP = "#111111"
GREY_NC = "#6B7280"

plt.rcParams.update({
    "font.family": "DejaVu Sans",
    "font.size": 10,
    "axes.titlesize": 11.5,
    "axes.titleweight": "bold",
    "axes.labelsize": 10.5,
    "legend.fontsize": 8.5,
    "xtick.labelsize": 9.5,
    "ytick.labelsize": 9.5,
    "figure.dpi": 150,
    "savefig.dpi": 300,
    "axes.grid": True,
    "grid.alpha": 0.25,
    "grid.linestyle": "--",
})


def load_curve(tc: int, stage_key: str) -> pd.DataFrame:
    return pd.read_csv(DATA / "curves" / f"{tc}c_{stage_key}.csv")


def load_meta(tc: int, stage_key: str) -> dict:
    with open(DATA / "curves" / f"{tc}c_{stage_key}.json", encoding="utf-8") as f:
        return json.load(f)


def save_fig(fig, gid: str, csv_df: pd.DataFrame):
    OUT.mkdir(parents=True, exist_ok=True)
    fig.savefig(OUT / f"{gid}.png", dpi=300, bbox_inches="tight", pad_inches=0.12)
    fig.savefig(OUT / f"{gid}.svg", bbox_inches="tight", pad_inches=0.12)
    plt.close(fig)
    csv_df.to_csv(OUT / f"{gid}.csv", index=False)


def add_dual_pct_frac_note(ax):
    ax.set_xlabel("Time (hours)")
    ax.set_ylabel("Cumulative release fraction (%)")
    ax2 = ax.twinx()
    y0, y1 = ax.get_ylim()
    ax2.set_ylim(y0 / 100.0, y1 / 100.0)
    ax2.set_ylabel("Fraction (dimensionless)")
    ax2.grid(False)
    return ax2


def plot_group_a():
    # A01-A06: Stage A
    for idx, tc in enumerate(TEMPS, start=1):
        gid = f"A{idx:02d}"
        df_a = load_curve(tc, "stage_a")
        meta = load_meta(tc, "stage_a")
        t = df_a["time_h"]
        fa_pct = 100.0 * df_a["f_method_a"]
        fb_pct = 100.0 * df_a["f_method_b"]

        fig, ax = plt.subplots(figsize=(7.6, 4.8))
        ax.plot(t, fa_pct, color=RED_A, lw=2.0, label=f"Method A: FDM-BTCS (end = {fa_pct.iloc[-1]:.4g}%)")
        valid_b = fb_pct.notna()
        if valid_b.any():
            ax.plot(t[valid_b], fb_pct[valid_b], color=BLUE_B, lw=1.6, ls="--", label="Method B: Sturm–Liouville")
        ax.set_title(f"{gid}: Stage A (Baseline: Const D, G=0, \u03bb=0) \u2014 {tc} \u00b0C (Cs-137)")
        vf = meta["numerics"].get("b_valid_from_h")
        fl = meta["numerics"].get("b_accuracy_floor")
        res_max = meta["numerics"].get("max_residual_over_bound", 0.0)
        note = f"Max |\u03b5|/bound = {res_max:.2e}  |  Method B floor = {fl:.2e} (valid \u2265 {vf:.2f} h)" if fl else f"Max |\u03b5|/bound = {res_max:.2e}"
        ax.text(0.02, 0.96, note, transform=ax.transAxes, va="top", fontsize=8.2,
                bbox=dict(boxstyle="round,pad=0.3", fc="#F8FAFC", ec="#CBD5E1"))
        ax.legend(loc="lower right", framealpha=0.92)
        add_dual_pct_frac_note(ax)
        save_fig(fig, gid, pd.DataFrame({
            "time_h": t, "stage_a_method_a_pct": fa_pct, "stage_a_method_b_pct": fb_pct
        }))

    # A07-A12: Stage B
    for idx, tc in enumerate(TEMPS, start=7):
        gid = f"A{idx:02d}"
        df_a = load_curve(tc, "stage_a")
        df_b = load_curve(tc, "stage_b")
        meta = load_meta(tc, "stage_b")
        t = df_b["time_h"]
        fa_pct = 100.0 * df_b["f_method_a"]
        fb_pct = 100.0 * df_b["f_method_b"]
        ref_a_pct = 100.0 * df_a["f_method_a"]

        fig, ax = plt.subplots(figsize=(7.6, 4.8))
        ax.plot(t, ref_a_pct, color=GREY_NC, lw=1.2, ls=":", label=f"Stage A ref (G=0, F_init; end = {ref_a_pct.iloc[-1]:.4g}%)")
        ax.plot(t, fa_pct, color=RED_A, lw=2.0, label=f"Stage B Method A (+G_ref, Fresh, F_gen; end = {fa_pct.iloc[-1]:.4g}%)")
        valid_b = fb_pct.notna()
        if valid_b.any():
            ax.plot(t[valid_b], fb_pct[valid_b], color=BLUE_B, lw=1.6, ls="--", label="Stage B Method B: Sturm–Liouville (Duhamel)")
        ax.set_title(f"{gid}: Stage B (+Fission Generation G, Fresh Particle) \u2014 {tc} \u00b0C (Cs-137)")
        fl = meta["numerics"].get("b_accuracy_floor")
        vf = meta["numerics"].get("b_valid_from_h")
        note = f"Fresh start C_init=0 | G_ref=8.987e-6 mol m\u207b\u00b3 s\u207b\u00b9 | Method B valid \u2265 {vf:.2f} h" if vf else "Fresh start C_init=0 | F_gen = M_rel / M_gen"
        ax.text(0.02, 0.96, note, transform=ax.transAxes, va="top", fontsize=8.2,
                bbox=dict(boxstyle="round,pad=0.3", fc="#F8FAFC", ec="#CBD5E1"))
        ax.legend(loc="center right", framealpha=0.92)
        add_dual_pct_frac_note(ax)
        save_fig(fig, gid, pd.DataFrame({
            "time_h": t, "stage_b_method_a_pct": fa_pct, "stage_b_method_b_pct": fb_pct, "stage_a_ref_pct": ref_a_pct
        }))

    # A13-A18: Stage C
    for idx, tc in enumerate(TEMPS, start=13):
        gid = f"A{idx:02d}"
        df_b = load_curve(tc, "stage_b")
        df_c = load_curve(tc, "stage_c")
        df_cd = load_curve(tc, "stage_c_decay_only")
        t = df_c["time_h"]
        fa_c = 100.0 * df_c["f_method_a"]
        fb_c = 100.0 * df_c["f_method_b"]
        fa_b = 100.0 * df_b["f_method_a"]
        fa_cd = 100.0 * df_cd["f_method_a"]

        fig, ax = plt.subplots(figsize=(7.6, 4.8))
        ax.plot(t, fa_c, color=RED_A, lw=2.0, label=f"Stage C Method A (+G, +\u03bb, F_gen; end = {fa_c.iloc[-1]:.4g}%)")
        vb = fb_c.notna()
        if vb.any():
            ax.plot(t[vb], fb_c[vb], color=BLUE_B, lw=1.6, ls="--", label="Stage C Method B: Sturm–Liouville")
        ax.plot(t, fa_b, color=GREY_NC, lw=1.2, ls=":", label=f"Stage B ref (+G, \u03bb=0; end = {fa_b.iloc[-1]:.4g}%)")
        ax.plot(t, fa_cd, color=ORANGE_C, lw=1.4, ls="-.", label=f"Stage C decay-only (G=0, +\u03bb, F_init; end = {fa_cd.iloc[-1]:.4g}%)")
        ax.set_title(f"{gid}: Stage C (+Radioactive Decay \u03bb_Cs137) \u2014 {tc} \u00b0C (Cs-137)")
        lam_T = 7.302187e-10 * (t.iloc[-1] * 3600.0)
        ax.text(0.02, 0.96, f"\u03bb = 7.302e-10 s\u207b\u00b9  |  \u03bb\u00b7T_end = {lam_T:.4e}", transform=ax.transAxes,
                va="top", fontsize=8.2, bbox=dict(boxstyle="round,pad=0.3", fc="#F8FAFC", ec="#CBD5E1"))
        ax.legend(loc="center right", framealpha=0.92)
        add_dual_pct_frac_note(ax)
        save_fig(fig, gid, pd.DataFrame({
            "time_h": t, "stage_c_method_a_pct": fa_c, "stage_c_method_b_pct": fb_c,
            "stage_b_ref_pct": fa_b, "stage_c_decay_only_pct": fa_cd
        }))

    # A19-A24: Stage D
    for idx, tc in enumerate(TEMPS, start=19):
        gid = f"A{idx:02d}"
        df_a = load_curve(tc, "stage_a")
        df_di = load_curve(tc, "stage_d_isothermal")
        df_dro = load_curve(tc, "stage_d_ramp_decay_off")
        df_dc = load_curve(tc, "stage_d_combined")
        meta = load_meta(tc, "stage_d_ramp_decay_off")
        t = df_a["time_h"]
        fa_a = 100.0 * df_a["f_method_a"]
        fa_di = 100.0 * df_di["f_method_a"]
        fa_dro = 100.0 * df_dro["f_method_a"]
        fa_dc = 100.0 * df_dc["f_method_a"]

        fig, ax = plt.subplots(figsize=(7.6, 4.8))
        ax.plot(t, fa_dro, color=RED_A, lw=2.0, label=f"Stage D Method A (D(T(t)) ramp+hold, G=0; end = {fa_dro.iloc[-1]:.4g}%)")
        ax.plot(t, fa_di, color=PURPLE_D, lw=1.4, ls="--", label=f"Stage D isothermal D(T_nom) check (end = {fa_di.iloc[-1]:.4g}%)")
        ax.plot(t, fa_a, color=GREY_NC, lw=1.2, ls=":", label=f"Stage A ref (const D, end = {fa_a.iloc[-1]:.4g}%)")
        ax.plot(t, fa_dc, color=GREEN_D, lw=1.5, ls="-.", label=f"Stage D combined (+G, +\u03bb, D(T(t)), F_gen; end = {fa_dc.iloc[-1]:.4g}%)")
        ax.set_title(f"{gid}: Stage D (D(r, T(t)) Heating Ramp & Combined Physics) \u2014 {tc} \u00b0C")
        hist_id = meta.get("history_id", "AGR1-ST-C16")
        ax.text(0.02, 0.96, f"Ramp: {hist_id}  |  Method B: N/A (TIME_VARYING_TEMPERATURE)\nPre-hold diff at t=0: {fa_dro.iloc[0]:.4g}%",
                transform=ax.transAxes, va="top", fontsize=8.0, bbox=dict(boxstyle="round,pad=0.3", fc="#FFF7ED", ec="#FDBA74"))
        ax.legend(loc="lower right", framealpha=0.92)
        add_dual_pct_frac_note(ax)
        save_fig(fig, gid, pd.DataFrame({
            "time_h": t, "stage_d_ramp_decay_off_pct": fa_dro, "stage_d_isothermal_pct": fa_di,
            "stage_a_ref_pct": fa_a, "stage_d_combined_pct": fa_dc
        }))


def plot_group_b():
    for idx, tc in enumerate(TEMPS, start=1):
        gid = f"B{idx:02d}"
        df_a = load_curve(tc, "stage_a")
        df_b = load_curve(tc, "stage_b")
        meta_a = load_meta(tc, "stage_a")
        t = df_a["time_h"]
        fa_a = 100.0 * df_a["f_method_a"]
        fb_a = 100.0 * df_a["f_method_b"]
        fa_b = 100.0 * df_b["f_method_a"]
        fb_b = 100.0 * df_b["f_method_b"]

        diff_a = np.abs(fa_a - fb_a)
        vf = meta_a["numerics"].get("b_valid_from_h") or 0.0
        fl = meta_a["numerics"].get("b_accuracy_floor") or 1e-12

        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(7.8, 6.2), sharex=True, gridspec_kw={"height_ratios": [1.5, 1.0]})
        ax1.plot(t, fa_a, color=RED_A, lw=2.0, label="Method A: Stage A (F_init)")
        vb_a = fb_a.notna()
        if vb_a.any():
            ax1.plot(t[vb_a], fb_a[vb_a], color=BLUE_B, lw=1.7, ls="--", label="Method B: Stage A (Sturm–Liouville)")
        ax1.plot(t, fa_b, color="#991B1B", lw=1.3, ls="-.", label="Method A: Stage B (+G, F_gen)")
        vb_b = fb_b.notna()
        if vb_b.any():
            ax1.plot(t[vb_b], fb_b[vb_b], color="#1D4ED8", lw=1.3, ls=":", label="Method B: Stage B (+G, F_gen)")
        if vf > 0:
            ax1.axvspan(0, min(vf, t.iloc[-1]), color="#E2E8F0", alpha=0.55, label=f"Method B withheld mask (t < {vf:.2f} h)")
        ax1.set_ylabel("Cumulative release fraction (%)")
        ax1.set_title(f"{gid}: Analytical Sturm–Liouville vs Method A \u2014 {tc} \u00b0C (Cs-137)")
        ax1.legend(loc="best", fontsize=8.0, framealpha=0.92)

        ax2.semilogy(t[vb_a], np.maximum(diff_a[vb_a], 1e-16), color=BLUE_B, lw=1.6, label="|Method A \u2212 Method B| (Stage A, pp)")
        ax2.axhline(fl * 100.0, color="#0F172A", ls=":", lw=1.1, label=f"Method B accuracy floor ({fl*100:.2e} pp)")
        ax2.set_xlabel("Time (hours)")
        ax2.set_ylabel("|F_A \u2212 F_B| (pp)")
        ax2.legend(loc="upper right", fontsize=8.0, framealpha=0.92)
        ax2.text(0.02, 0.08, "Note: Stage D ramp T(t) excluded by applicability mask (TIME_VARYING_TEMPERATURE).",
                 transform=ax2.transAxes, fontsize=8.0, bbox=dict(boxstyle="round,pad=0.25", fc="#FFF7ED", ec="#FDBA74"))
        fig.tight_layout()
        save_fig(fig, gid, pd.DataFrame({
            "time_h": t, "stage_a_method_a_pct": fa_a, "stage_a_method_b_pct": fb_a,
            "stage_b_method_a_pct": fa_b, "stage_b_method_b_pct": fb_b, "abs_diff_stage_a_pp": diff_a
        }))


def plot_group_c():
    # C01: BTCS Numerical Convergence Study
    with open(DATA / "diagnostics" / "c01_convergence.json", encoding="utf-8") as f:
        c01 = json.load(f)
    mb = pd.read_csv(DATA / "diagnostics" / "mass_balance_audit.csv")
    sp = pd.DataFrame(c01["spatial"])
    tp = pd.DataFrame(c01["temporal"])

    fig, axes = plt.subplots(1, 3, figsize=(13.5, 4.4))
    # Spatial
    axes[0].loglog(sp["step"], sp["error"], "o-", color=RED_A, lw=1.8, ms=6, label="Observed |F_BTCS \u2212 F_exact|")
    h_ref = sp["step"].to_numpy()
    axes[0].loglog(h_ref, sp["error"].iloc[-1] * (h_ref / h_ref[-1]) ** 2, "--", color="#111111", lw=1.1, label=f"O(\u0394r\u00b2) slope (obs = {c01['last_spatial_order']:.3f})")
    axes[0].set_xlabel("Radial spacing \u0394r (\u03bcm / scaled)")
    axes[0].set_ylabel("Absolute error |F_solver \u2212 F_exact|")
    axes[0].set_title("Spatial Convergence (6 levels)")
    axes[0].legend(fontsize=8.0)

    # Temporal
    axes[1].loglog(tp["step"], tp["error"], "s-", color=BLUE_B, lw=1.8, ms=5, label="Observed |F_BTCS \u2212 F_exact|")
    dt_ref = tp["step"].to_numpy()
    axes[1].loglog(dt_ref, tp["error"].iloc[-1] * (dt_ref / dt_ref[-1]) ** 1, "--", color="#111111", lw=1.1, label=f"O(\u0394t) slope (obs = {c01['last_temporal_order']:.3f})")
    axes[1].set_xlabel("Time step \u0394t (s)")
    axes[1].set_ylabel("Absolute error |F_solver \u2212 F_exact|")
    axes[1].set_title("Temporal Convergence (9 levels)")
    axes[1].legend(fontsize=8.0)

    # Mass conservation across 48 runs
    axes[2].semilogy(range(len(mb)), mb["max_residual_over_bound"], "o", color=GREEN_D, ms=4.5, label="Run max |\u03b5| / bound (48 runs)")
    axes[2].axhline(1.0, color=RED_A, ls="--", lw=1.3, label="Derived rounding bound (|\u03b5|/bound = 1)")
    axes[2].set_xlabel("Simulation run index (1200\u20131800 \u00b0C, Stages A\u2013D)")
    axes[2].set_ylabel("Max |\u03b5| / rounding bound")
    axes[2].set_title(f"Mass Balance Check (max = {mb['max_residual_over_bound'].max():.3f})")
    axes[2].legend(fontsize=8.0)

    fig.suptitle("C01: BTCS Numerical Convergence & Mass Conservation Audit", fontsize=12.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.94])
    c01_csv = pd.concat([
        sp.assign(kind="spatial")[["kind", "level", "step", "count", "error", "order"]],
        tp.assign(kind="temporal")[["kind", "level", "step", "count", "error", "order"]],
    ], ignore_index=True)
    save_fig(fig, "C01", c01_csv)

    # C02: Method A versus Method B
    with open(DATA / "diagnostics" / "c02_method_a_vs_b.json", encoding="utf-8") as f:
        c02 = pd.DataFrame(json.load(f)["comparisons"])
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(11.2, 4.6))
    x = np.arange(len(c02))
    w = 0.36
    ax1.bar(x - w / 2, c02["final_f_a"] * 100.0, width=w, color=RED_A, label="Method A: FDM-BTCS")
    ax1.bar(x + w / 2, c02["final_f_b"] * 100.0, width=w, color=BLUE_B, label="Method B: Sturm–Liouville")
    ax1.set_xticks(x)
    ax1.set_xticklabels([f"{int(t)} \u00b0C" for t in c02["temperature_c"]])
    ax1.set_yscale("log")
    ax1.set_ylabel("Final Stage A release fraction (%)")
    ax1.set_title("End-of-Test Release Agreement (Stage A)")
    ax1.legend(fontsize=8.5)

    ax2.semilogy(x, c02["max_abs_diff_pp"], "o-", color=BLUE_B, lw=1.8, ms=6, label="Max |F_A \u2212 F_B| over valid mask (pp)")
    ax2.semilogy(x, c02["accuracy_floor"] * 100.0, "s--", color="#111111", lw=1.2, ms=5, label="Method B accuracy floor (pp)")
    ax2.set_xticks(x)
    ax2.set_xticklabels([f"{int(t)} \u00b0C" for t in c02["temperature_c"]])
    ax2.set_ylabel("Maximum solver difference (pp)")
    ax2.set_title("Numerical vs Analytical Discrepancy & Floor")
    ax2.legend(fontsize=8.2)

    fig.suptitle("C02: Method A (BTCS) vs Method B (Sturm–Liouville) Numerical Agreement", fontsize=12, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.94])
    save_fig(fig, "C02", c02)

    # C03: STRATO Experimental Deviation Summary
    sum_df = pd.read_csv(DATA / "deviations_exact_summary.csv")
    bnd_df = pd.read_csv(DATA / "deviations_bounds.csv")

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12.2, 5.0))
    stage_order = ["stage_a", "stage_b", "stage_c", "stage_c_decay_only", "stage_d_ramp_decay_off"]
    stage_names = ["Stage A\n(Base)", "Stage B*\n(+G)", "Stage C*\n(+G,+\u03bb)", "Stage C-dec\n(G=0,+\u03bb)", "Stage D\n(Ramp T(t))"]
    x = np.arange(len(stage_order))
    w = 0.36
    act3 = sum_df[(sum_df["case_id"] == "D04_1700C_ACT3") & (sum_df["method"] == "Method A")].set_index("stage_key")
    act4 = sum_df[(sum_df["case_id"] == "D05_1800C_ACT4_N11") & (sum_df["method"] == "Method A")].set_index("stage_key")
    r3 = [act3.loc[s, "rmse_pp"] for s in stage_order]
    r4 = [act4.loc[s, "rmse_pp"] for s in stage_order]

    ax1.bar(x - w / 2, r3, width=w, color=RED_A, label="1700 \u00b0C (Minato ACT3, N=8)")
    ax1.bar(x + w / 2, r4, width=w, color=BLUE_B, label="1800 \u00b0C (Minato ACT4, N=11)")
    for i in range(len(x)):
        ax1.text(x[i] - w / 2, r3[i] + 0.25, f"{r3[i]:.2f}", ha="center", va="bottom", fontsize=7.5)
        ax1.text(x[i] + w / 2, r4[i] + 0.25, f"{r4[i]:.2f}", ha="center", va="bottom", fontsize=7.5)
    ax1.set_xticks(x)
    ax1.set_xticklabels(stage_names, fontsize=8.5)
    ax1.set_ylim(0, max(max(r3), max(r4)) * 1.24)
    ax1.set_ylabel("RMSE vs experiment (percentage points, pp)")
    ax1.set_title("Exact Time-Series RMSE Across Stages (*F_gen != F_init)")
    ax1.legend(fontsize=8.0, loc="upper center", ncol=2)

    # Right panel: Upper bound ratios F_model / U_exp for Stage A & Stage D-ramp
    b_sub = bnd_df[(bnd_df["method"] == "Method A") & (bnd_df["stage_key"].isin(["stage_a", "stage_d_ramp_decay_off"]))]
    b_targets = [
        (1200, "exp:1200-Cs-UCO", "1200 \u00b0C UCO (5%)"),
        (1200, "exp:1200-Cs-UCO (alt_0.01pct_collector)", "1200 \u00b0C UCO (0.01%)"),
        (1350, "exp:1350-Cs-UCO", "1350 \u00b0C UCO (5%)"),
        (1350, "exp:1350-Cs-UCO (alt_0.01pct_collector)", "1350 \u00b0C UCO (0.01%)"),
        (1500, "exp:1500-Cs-UCO", "1500 \u00b0C UCO (5%)"),
        (1500, "exp:1500-Cs-UO2*(1)", "1500 \u00b0C UO2*(1) (0.01%)"),
        (1600, "IAEA97-K31-CS137 (HFR-K3/1 sphere, t<=400h, N=17)", "1600 \u00b0C HFR-K3/1"),
    ]
    xb = np.arange(len(b_targets))
    ratios_a = [b_sub[(b_sub["temperature_c"] == tc) & (b_sub["series"] == s) & (b_sub["stage_key"] == "stage_a")]["ratio_model_over_bound"].iloc[0] for tc, s, _ in b_targets]
    ratios_d = [b_sub[(b_sub["temperature_c"] == tc) & (b_sub["series"] == s) & (b_sub["stage_key"] == "stage_d_ramp_decay_off")]["ratio_model_over_bound"].iloc[0] for tc, s, _ in b_targets]
    ax2.bar(xb - 0.18, ratios_a, width=0.35, color=RED_A, label="Stage A / Bound")
    ax2.bar(xb + 0.18, ratios_d, width=0.35, color=GREEN_D, label="Stage D-ramp / Bound")
    ax2.axhline(1.0, color="#111111", ls="--", lw=1.2, label="Bound limit (ratio = 1)")
    ax2.set_yscale("log")
    ax2.set_xticks(xb)
    ax2.set_xticklabels([lbl for _, _, lbl in b_targets], rotation=28, ha="right", fontsize=8.0)
    ax2.set_ylabel("Model / Upper Bound Ratio F_model / U_exp")
    ax2.set_title("Upper-Bound Compliance Check (1200\u20131600 \u00b0C)")
    ax2.legend(fontsize=8.0)

    fig.suptitle("C03: STRATO Experimental Deviation & Bound Summary (Stages A\u2013D)", fontsize=12.5, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.94])
    save_fig(fig, "C03", sum_df[sum_df["method"] == "Method A"])


def plot_group_d():
    bnd_df = pd.read_csv(DATA / "deviations_bounds.csv")
    sum_df = pd.read_csv(DATA / "deviations_exact_summary.csv")

    # D01 (1200 C), D02 (1350 C), D03 (1500 C)
    d_bounds = [
        ("D01", 1200, "Bullock (1984) / TE-2090 Table 22 (10,138 h; upper bounds only)"),
        ("D02", 1350, "Bullock (1984) / TE-2090 Table 22 (10,685 h; upper bounds only)"),
        ("D03", 1500, "Bullock (1984) / TE-2090 Fig. 124a & Table 22 (11,866 h; NO AGR-1 1500 \u00b0C Cs data)"),
    ]
    for gid, tc, subtitle in d_bounds:
        df_a = load_curve(tc, "stage_a")
        df_c = load_curve(tc, "stage_c")
        df_cd = load_curve(tc, "stage_c_decay_only")
        df_dr = load_curve(tc, "stage_d_ramp_decay_off")
        exp = pd.read_csv(EXP / f"exp_{tc}c_cs.csv")

        fig, ax = plt.subplots(figsize=(8.0, 5.0))
        t = df_a["time_h"]
        ax.plot(t, 100 * df_a["f_method_a"], color=RED_A, lw=2.0, label=f"Stage A Method A (end = {100*df_a['f_method_a'].iloc[-1]:.4g}%)")
        vb = df_a["f_method_b"].notna()
        if vb.any():
            ax.plot(t[vb], 100 * df_a.loc[vb, "f_method_b"], color=BLUE_B, lw=1.5, ls="--", label="Stage A Method B")
        ax.plot(t, 100 * df_cd["f_method_a"], color=ORANGE_C, lw=1.4, ls="-.", label=f"Stage C decay-only (G=0,+\u03bb; end = {100*df_cd['f_method_a'].iloc[-1]:.4g}%)")
        ax.plot(t, 100 * df_c["f_method_a"], color=PURPLE_D, lw=1.3, ls=":", label=f"Stage C (+G,+\u03bb, F_gen; end = {100*df_c['f_method_a'].iloc[-1]:.4g}%)")
        ax.plot(t, 100 * df_dr["f_method_a"], color=GREEN_D, lw=1.4, label=f"Stage D ramp (D(T(t)), end = {100*df_dr['f_method_a'].iloc[-1]:.4g}%)")

        # Upper bounds
        ub = exp[exp["series_kind"] == "upper_bound"]
        for _, r in ub.iterrows():
            ax.scatter([r["time_h"]], [100 * r["release_fraction"]], marker="v", s=65, color=BLACK_EXP, zorder=6,
                       label=f"Upper bound {r['series']} (<{100*r['release_fraction']:.4g}%)")
            if pd.notna(r.get("alt_bound_fraction")):
                ax.scatter([r["time_h"]], [100 * r["alt_bound_fraction"]], marker="v", s=65, facecolors="none", edgecolors=BLACK_EXP, lw=1.5, zorder=6,
                           label=f"Alt. collector bound {r['series']} (<{100*r['alt_bound_fraction']:.4g}%)")
        # Non-comparable degraded curves at 1500 C
        nc = exp[exp["series_kind"] == "not_comparable_plotted"]
        if not nc.empty:
            for ser, grp in nc.groupby("series"):
                ax.plot(grp["time_h"], 100 * grp["release_fraction"], marker="o", ms=3.5, mfc="none", mec=GREY_NC, color=GREY_NC, lw=1.0, ls=":",
                        label=f"Non-comparable degraded ({ser})")

        ax.set_title(f"{gid}: Cs-137 Release Comparison \u2014 {tc} \u00b0C\n{subtitle}", fontsize=10.5)
        ax.legend(loc="best", fontsize=7.5, framealpha=0.92)
        add_dual_pct_frac_note(ax)
        save_fig(fig, gid, exp)

    # D04 (1700 C Minato ACT3) & D05 (1800 C Minato ACT4)
    for gid, tc, exp_ser, case_id, title_sub in [
        ("D04", 1700, "exp:F125-Cs", "D04_1700C_ACT3", "Minato et al. (2000) HRB-22 ACT3 (NOT AGR-1; N=8)"),
        ("D05", 1800, "exp:F126-Cs", "D05_1800C_ACT4_N11", "Minato et al. (2000) HRB-22 ACT4 (w=1/25, N=11; NOT AGR-1)"),
    ]:
        df_a = load_curve(tc, "stage_a")
        df_c = load_curve(tc, "stage_c")
        df_cd = load_curve(tc, "stage_c_decay_only")
        df_dr = load_curve(tc, "stage_d_ramp_decay_off")
        exp = pd.read_csv(EXP / f"exp_{tc}c_cs.csv")
        sub_exp = exp[exp["series"] == exp_ser].sort_values("time_h")

        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(8.2, 6.4), sharex=True, gridspec_kw={"height_ratios": [1.65, 1.0]})
        t = df_a["time_h"]
        ax1.plot(t, 100 * df_a["f_method_a"], color=RED_A, lw=2.0, label="Stage A Method A (const D, G=0, \u03bb=0)")
        vb = df_a["f_method_b"].notna()
        if vb.any():
            ax1.plot(t[vb], 100 * df_a.loc[vb, "f_method_b"], color=BLUE_B, lw=1.5, ls="--", label="Stage A Method B (Sturm–Liouville)")
        ax1.plot(t, 100 * df_cd["f_method_a"], color=ORANGE_C, lw=1.4, ls="-.", label="Stage C decay-only (G=0, +\u03bb)")
        ax1.plot(t, 100 * df_dr["f_method_a"], color=GREEN_D, lw=1.6, label="Stage D Method A (AGR1-ST-C16 ramp+hold)")
        ax1.plot(t, 100 * df_c["f_method_a"], color=PURPLE_D, lw=1.3, ls=":", label="Stage C (+G,+\u03bb, F_gen — diff. def.)")
        ax1.scatter(sub_exp["time_h"], 100 * sub_exp["release_fraction"], color=BLACK_EXP, s=36, zorder=6, label=f"Experiment {exp_ser} (circles)")

        ax1.set_title(f"{gid}: Cs-137 Release Comparison \u2014 {tc} \u00b0C ({title_sub})", fontsize=10.5)
        ax1.set_ylabel("Cumulative release fraction (%)")
        ax1.legend(loc="upper left", fontsize=7.6, framealpha=0.92)

        # Residual panel Delta_s(t_i)
        pts = pd.read_csv(DATA / "deviations_exact_points.csv")
        ps = pts[(pts["case_id"] == case_id) & (pts["method"] == "Method A")]
        for sk, col, lbl in [
            ("stage_a", RED_A, "Stage A"),
            ("stage_c_decay_only", ORANGE_C, "Stage C-dec"),
            ("stage_d_ramp_decay_off", GREEN_D, "Stage D-ramp"),
            ("stage_c", PURPLE_D, "Stage C (+G)"),
        ]:
            pss = ps[ps["stage_key"] == sk]
            srow = sum_df[(sum_df["case_id"] == case_id) & (sum_df["stage_key"] == sk) & (sum_df["method"] == "Method A")].iloc[0]
            ax2.plot(pss["time_h"], pss["delta_pp"], "o-", ms=4, lw=1.3, color=col,
                     label=f"{lbl}: RMSE={srow['rmse_pp']:.2f} pp, final \u0394={srow['final_delta_pp']:+.2f} pp")
        ax2.axhline(0.0, color="#111111", lw=1.0, ls="--")
        ax2.set_xlabel("Time (hours)")
        ax2.set_ylabel("\u0394_s = 100(F_s \u2212 F_exp) (pp)")
        ax2.legend(loc="upper left", fontsize=7.4, ncol=2, framealpha=0.92)
        if gid == "D05":
            ax2.set_ylim(-6.2, 18.2)
            ax2.text(0.01, 0.04, "Note: AGR-1 4-4-2 (HUNN16-442, 5 failed SiC) & JAERI-A excluded from fit (incompatible failure/definition).",
                     transform=ax2.transAxes, fontsize=7.4, bbox=dict(boxstyle="round,pad=0.2", fc="#FFF7ED", ec="#FDBA74"))
        fig.tight_layout()
        save_fig(fig, gid, ps)

    # D06: AGR-1 Irradiation 955-1136 C
    with open(DATA / "diagnostics" / "d06_irradiation.json", encoding="utf-8") as f:
        d06 = json.load(f)
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12.2, 4.8))
    cols = ["#2563EB", "#D97706", "#DC2626"]
    csv_rows = []
    for c, item in zip(cols, d06["temperatures"]):
        tc = int(item["temperature_c"])
        th = item["time_h"]
        fa = [100.0 * v for v in item["stage_a_f_method_a"]]
        fc = [100.0 * v for v in item["stage_c_f_gen_method_a"]]
        ax1.semilogy(th, np.maximum(fa, 1e-22), color=c, lw=1.8, label=f"{tc} \u00b0C Stage A (end = {fa[-1]:.2e}%)")
        ax1.semilogy(th, np.maximum(fc, 1e-22), color=c, lw=1.4, ls="--", label=f"{tc} \u00b0C Stage C/D F_gen (end = {fc[-1]:.2e}%)")
        for xh, a, cv in zip(th, fa, fc):
            csv_rows.append({"temperature_c": tc, "time_h": xh, "stage_a_pct": a, "stage_c_fgen_pct": cv})

    ax1.set_xlabel("Irradiation time (hours; 620.2 EFPD = 14,884.8 h)")
    ax1.set_ylabel("Predicted Cs-137 release fraction (%)")
    ax1.set_title("AGR-1 Irradiation Exploration (C15 Cs Laws, 955\u20131136 \u00b0C)")
    ax1.legend(loc="lower right", fontsize=7.6, framealpha=0.92)

    ax2.axis("off")
    txt = (
        "D06 Non-Comparability Assessment (AGR-1 Irradiation, 620.2 EFPD)\n"
        "------------------------------------------------------------------\n"
        "1. Cs-137 NOT reported quantitatively in AGR-1/AGR-2 irradiation\n"
        "   inventories (no time-resolved intact-particle Cs-137 series).\n\n"
        "2. Published AGR-1 end-of-irradiation compact/capsule fractional\n"
        "   values (Ag-110m, Cs-134, Sr-90) are integrated over\n"
        "   time-varying power/temperature and include matrix & failed-SiC\n"
        "   contributions.\n\n"
        "3. Model exploration (left panel) uses constant calculated TAVA\n"
        "   temperatures (955, 1028, 1136 \u00b0C) with C15 laws: predicted intact\n"
        "   Cs-137 release remains < 10^-4 % throughout irradiation.\n\n"
        "4. Verdict: NO quantitative \u0394 or RMSE computed; strictly\n"
        "   exploration + non-comparability documentation (Decision 71)."
    )
    ax2.text(0.04, 0.94, txt, va="top", family="monospace", fontsize=8.5,
             bbox=dict(boxstyle="round,pad=0.6", fc="#FFF7ED", ec="#FB923C"))
    fig.suptitle("D06: AGR-1 Irradiation-Temperature Exploration (~955\u20131136 \u00b0C, 620.2 EFPD)", fontsize=12, fontweight="bold")
    fig.tight_layout(rect=[0, 0, 1, 0.94])
    save_fig(fig, "D06", pd.DataFrame(csv_rows))


def plot_supporting():
    # S01 & S02: 6-temperature Stage A & Stage D-ramp log-time overviews
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
        ax.set_ylabel("Cumulative release fraction (%)")
        ax.set_title(title)
        ax.legend(loc="upper left", fontsize=8.2)
        save_fig(fig, sid, pd.concat(rows, ignore_index=True))

    # S03: 1600 C non-comparable context (HFR-K3/1 + JAERI-A + AGR-1 unavailable note)
    df16a = load_curve(1600, "stage_a")
    df16d = load_curve(1600, "stage_d_ramp_decay_off")
    hfr = pd.read_csv(EXP / "exp_1600c_hfr_k31_cs137.csv")
    jaeri = pd.read_csv(EXP / "exp_jaeri_sample_a_cs137.csv")
    j16 = jaeri[jaeri["T_C"].astype(str) == "1600"]
    fig, ax = plt.subplots(figsize=(7.8, 4.9))
    ax.semilogy(df16a["time_h"], np.maximum(100 * df16a["f_method_a"], 1e-12), color=RED_A, lw=2.0, label="Stage A Method A (1600 \u00b0C)")
    ax.semilogy(df16d["time_h"], np.maximum(100 * df16d["f_method_a"], 1e-12), color=GREEN_D, lw=1.6, ls="--", label="Stage D ramp Method A (1600 \u00b0C)")
    ax.scatter(hfr["time_h"], 100 * hfr["value"], marker="v", color=BLACK_EXP, s=36, label="IAEA97-K31 HFR-K3/1 sphere (upper bound, UO2)")
    ax.scatter(j16["time_h"], 100 * j16["value"], marker="o", facecolors="none", edgecolors=GREY_NC, s=48, lw=1.4, label="IAEA97-JAERI-A (PLOT ONLY, incompatible def.)")
    ax.set_title("S03: 1600 \u00b0C Cs-137 Context (No AGR-1 Cs-137 Curve; Sphere Upper Bound Only)")
    ax.set_xlabel("Time (hours)")
    ax.set_ylabel("Cumulative release fraction (%, log scale)")
    ax.legend(loc="best", fontsize=8.0)
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
        for rx, rlbl in [(175.0, "Kernel|Buf"), (278.5, "Buf|IPyC"), (317.9, "IPyC|SiC"), (353.2, "SiC|OPyC")]:
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

    # S07: Ramp temperature histories comparison
    fig, ax = plt.subplots(figsize=(7.8, 4.6))
    t_rows = []
    for tc in [1200, 1350, 1500, 1600, 1700, 1800]:
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

    # S08: AGR-1 Operational Scenarios (S1-S5 Cs-137 from data/results/curves)
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
    ax.set_ylabel("Predicted Cs-137 release fraction (%)")
    ax.set_title("S08: Prescribed-Input Reactor Scenarios S1\u2013S5 (Cs-137, 1028 \u00b0C Normal Ref.)")
    ax.legend(loc="best", fontsize=8.0)
    save_fig(fig, "S08", pd.DataFrame(s_rows) if s_rows else pd.DataFrame({"empty": []}))

    # S09 & S10: Pointwise residual profiles at 1700 C and 1800 C + Stage B vs Fresh check
    pts = pd.read_csv(DATA / "deviations_exact_points.csv")
    for sid, cid, title in [
        ("S09", "D04_1700C_ACT3", "S09: Pointwise Absolute Deviation |\u0394_s(t_i)| \u2014 1700 \u00b0C (ACT3)"),
        ("S10", "D05_1800C_ACT4_N11", "S10: Pointwise Absolute Deviation |\u0394_s(t_i)| \u2014 1800 \u00b0C (ACT4)"),
    ]:
        sub = pts[(pts["case_id"] == cid) & (pts["method"] == "Method A")]
        fig, ax = plt.subplots(figsize=(7.8, 4.7))
        for sk, col, lbl in [
            ("stage_a", RED_A, "Stage A"),
            ("stage_c_decay_only", ORANGE_C, "Stage C-decay (G=0)"),
            ("stage_d_ramp_decay_off", GREEN_D, "Stage D-ramp (G=0)"),
            ("stage_b", BLUE_B, "Stage B (+G, F_gen)"),
            ("stage_c", PURPLE_D, "Stage C (+G,+\u03bb, F_gen)"),
        ]:
            ss = sub[sub["stage_key"] == sk]
            ax.plot(ss["time_h"], ss["abs_delta_pp"], "o-", ms=4.5, lw=1.6, color=col, label=lbl)
        ax.set_xlabel("Time (hours)")
        ax.set_ylabel("Absolute deviation |\u0394_s(t_i)| (pp)")
        ax.set_title(title)
        ax.legend(loc="best", fontsize=8.0)
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
