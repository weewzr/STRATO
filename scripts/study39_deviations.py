#!/usr/bin/env python3
"""
Step 3: Quantitative experimental deviation and Stage A-D improvement analysis
for STRATO Phase 2 (39 principal graphs + supporting diagnostics).

Reads canonical Rust simulation outputs in `data/study39/` and extracts
verified experimental Cs datasets from `data/experimental/release_measurements.csv`
and the local archive CSVs into `data/study39/experimental/`.

Produces:
  - data/study39/deviations_exact_points.csv
  - data/study39/deviations_exact_summary.csv
  - data/study39/deviations_bounds.csv
  - data/study39/stage_improvement.csv
  - data/study39/study39_manifest.json
"""

import csv
import json
import math
import os
import subprocess
from pathlib import Path

import numpy as np
import pandas as pd

REPO = Path(__file__).resolve().parent.parent
ARCHIVE = Path(r"C:\Users\Alf\Downloads\STRATO_Archive\STRATO_Archive")
GD_ARCH = ARCHIVE / "deliverables" / "powerpoint" / "graph-data"
EXP_OUT = REPO / "data" / "study39" / "experimental"
OUT = REPO / "data" / "study39"

TEMPS = [1200, 1350, 1500, 1600, 1700, 1800]
DURATIONS_H = {
    1200: 10138.0,
    1350: 10685.0,
    1500: 11866.0,
    1600: 400.0,
    1700: 272.0,
    1800: 222.0,
}

STAGES_ORDER = [
    ("stage_a", "Stage A (Baseline: const D, G=0, lambda=0)"),
    ("stage_b", "Stage B (+G, lambda=0, F_gen)"),
    ("stage_c", "Stage C (+G, +lambda, F_gen)"),
    ("stage_c_decay_only", "Stage C-decay (G=0, +lambda, F_init)"),
    ("stage_d_isothermal", "Stage D-iso (D(T) dynamic, G=0, lambda=0)"),
    ("stage_d_ramp_decay_off", "Stage D-ramp (D(T(t)), G=0, lambda=0)"),
    ("stage_d_ramp_decay_on", "Stage D-ramp (D(T(t)), G=0, +lambda)"),
    ("stage_d_combined", "Stage D-comb (+G, +lambda, D(T(t)), F_gen)"),
]


def load_curve(t_c: int, stage_key: str) -> pd.DataFrame:
    p = OUT / "curves" / f"{t_c}c_{stage_key}.csv"
    return pd.read_csv(p)


def interp_series(df: pd.DataFrame, col: str, t_h: float):
    t = df["time_h"].to_numpy(dtype=float)
    y = df[col].to_numpy(dtype=float)
    mask = ~np.isnan(y)
    if not np.any(mask):
        return np.nan
    t = t[mask]
    y = y[mask]
    if t_h < t[0] - 1e-9 or t_h > t[-1] + 1e-9:
        return np.nan
    t_clamped = float(np.clip(t_h, t[0], t[-1]))
    return float(np.interp(t_clamped, t, y))


def extract_experimental_data():
    EXP_OUT.mkdir(parents=True, exist_ok=True)
    extracted = {}

    # 1. From verified archive g1_expcomp_*_Cs.csv (1200, 1350, 1500, 1700, 1800 C)
    for tc in [1200, 1350, 1500, 1700, 1800]:
        src = GD_ARCH / f"g1_expcomp_{tc}c_Cs.csv"
        df = pd.read_csv(src)
        exp = df[~df["series"].isin(["method_a", "method_b", "method_a_decay_on", "method_b_decay_on"])].copy()
        exp.to_csv(EXP_OUT / f"exp_{tc}c_cs.csv", index=False)
        extracted[tc] = exp

    # 2. From data/experimental/release_measurements.csv for 1600 C & 1800 C context
    rm = pd.read_csv(REPO / "data" / "experimental" / "release_measurements.csv")
    hfr_k31 = rm[(rm["program"] == "FRG") & (rm["nuclide"] == "Cs-137") & (rm["T_C"].astype(str) == "1600")].copy()
    hfr_k31["time_h"] = hfr_k31["time_h"].astype(float)
    hfr_k31["value"] = hfr_k31["value"].astype(float)
    hfr_k31 = hfr_k31.sort_values("time_h")
    hfr_k31.to_csv(EXP_OUT / "exp_1600c_hfr_k31_cs137.csv", index=False)

    jaeri = rm[(rm["program"] == "JAERI") & (rm["nuclide"] == "Cs-137") & (rm["compact_or_sample"] == "sample A")].copy()
    jaeri.to_csv(EXP_OUT / "exp_jaeri_sample_a_cs137.csv", index=False)

    hunn = rm[(rm["program"] == "AGR-1") & (rm["nuclide"] == "Cs-137") & (rm["compact_or_sample"].str.contains("4-4-2", na=False))].copy()
    hunn.to_csv(EXP_OUT / "exp_1800c_hunn16_442_cs137.csv", index=False)

    # 3. Copy D06 sixth group conditions summary from archive
    with open(GD_ARCH / "g5_agr1_sixth_group.json", encoding="utf-8") as f:
        g5 = json.load(f)
    with open(EXP_OUT / "d06_agr1_irradiation_summary.json", "w", encoding="utf-8") as f:
        json.dump(
            {
                "conditions_table": g5.get("conditions_table"),
                "measurements_not_overlaid": g5.get("measurements_not_overlaid"),
                "verdict": g5.get("verdict"),
                "reasons": g5.get("reasons"),
            },
            f,
            indent=2,
        )

    return extracted, hfr_k31, jaeri, hunn


def compute_exact_series_metrics(extracted):
    point_rows = []
    summary_rows = []

    exact_cases = [
        {
            "case_id": "D04_1700C_ACT3",
            "t_c": 1700,
            "series": "exp:F125-Cs",
            "label": "Minato et al. (2000) HRB-22 ACT3 (1700 C, N=8)",
            "drop_cluster": False,
            "w_fail": 0.0,
        },
        {
            "case_id": "D05_1800C_ACT4_N11",
            "t_c": 1800,
            "series": "exp:F126-Cs",
            "label": "Minato et al. (2000) HRB-22 ACT4 incl. 0-10h cluster (1800 C, N=11)",
            "drop_cluster": False,
            "w_fail": 1.0 / 25.0,
        },
        {
            "case_id": "D05_1800C_ACT4_N10_resolved",
            "t_c": 1800,
            "series": "exp:F126-Cs",
            "label": "Minato et al. (2000) HRB-22 ACT4 resolved markers only (1800 C, N=10)",
            "drop_cluster": True,
            "w_fail": 1.0 / 25.0,
        },
    ]

    for ec in exact_cases:
        tc = ec["t_c"]
        exp_df = extracted[tc]
        sub = exp_df[exp_df["series"] == ec["series"]].sort_values("time_h").copy()
        if ec["drop_cluster"]:
            sub = sub[sub["time_h"] > 6.0].copy()

        t_pts = sub["time_h"].to_numpy(dtype=float)
        f_exp = sub["release_fraction"].to_numpy(dtype=float)

        for st_key, st_label in STAGES_ORDER:
            cdf = load_curve(tc, st_key)
            for method_col, method_name in [("f_method_a", "Method A"), ("f_method_b", "Method B")]:
                f_mod = np.array([interp_series(cdf, method_col, t) for t in t_pts], dtype=float)
                valid = ~np.isnan(f_mod)
                if not np.any(valid):
                    continue
                tv = t_pts[valid]
                ev = f_exp[valid]
                mv = f_mod[valid]

                delta_pp = 100.0 * (mv - ev)
                abs_delta_pp = np.abs(delta_pp)
                rel_err_pct = np.where(ev > 0.0, 100.0 * (mv - ev) / ev, np.nan)

                for i in range(len(tv)):
                    # Intact bracket if bounded w_fail > 0
                    lo_pp = hi_pp = np.nan
                    if ec["w_fail"] > 0.0:
                        f_int_hi = min(1.0, ev[i] / (1.0 - ec["w_fail"]))
                        f_int_lo = max(0.0, (ev[i] - ec["w_fail"]) / (1.0 - ec["w_fail"]))
                        lo_pp = 100.0 * (mv[i] - f_int_hi)
                        hi_pp = 100.0 * (mv[i] - f_int_lo)

                    point_rows.append(
                        {
                            "case_id": ec["case_id"],
                            "temperature_c": tc,
                            "stage_key": st_key,
                            "stage_label": st_label,
                            "method": method_name,
                            "time_h": tv[i],
                            "f_exp": ev[i],
                            "f_model": mv[i],
                            "delta_pp": delta_pp[i],
                            "abs_delta_pp": abs_delta_pp[i],
                            "rel_err_pct": rel_err_pct[i],
                            "intact_bracket_lo_pp": lo_pp,
                            "intact_bracket_hi_pp": hi_pp,
                        }
                    )

                mae_pp = float(np.mean(abs_delta_pp))
                rmse_pp = float(np.sqrt(np.mean(delta_pp**2)))
                max_abs_pp = float(np.max(abs_delta_pp))
                if len(tv) >= 2 and tv[-1] > tv[0]:
                    iae_pp = float(np.trapz(abs_delta_pp, tv) / (tv[-1] - tv[0]))
                else:
                    iae_pp = float(mae_pp)
                final_delta_pp = float(delta_pp[-1])
                ratio_final = float(mv[-1] / ev[-1]) if ev[-1] > 0 else np.nan
                abs_rel_mean = float(np.nanmean(np.abs(rel_err_pct)))

                summary_rows.append(
                    {
                        "case_id": ec["case_id"],
                        "case_label": ec["label"],
                        "temperature_c": tc,
                        "stage_key": st_key,
                        "stage_label": st_label,
                        "method": method_name,
                        "n_points": int(len(tv)),
                        "t_min_h": float(tv[0]),
                        "t_max_h": float(tv[-1]),
                        "f_exp_final": float(ev[-1]),
                        "f_model_final": float(mv[-1]),
                        "final_delta_pp": final_delta_pp,
                        "min_delta_pp": float(np.min(delta_pp)),
                        "max_delta_pp": float(np.max(delta_pp)),
                        "mae_pp": mae_pp,
                        "rmse_pp": rmse_pp,
                        "max_abs_delta_pp": max_abs_pp,
                        "iae_time_avg_pp": iae_pp,
                        "mean_abs_rel_err_pct": abs_rel_mean,
                        "ratio_final_model_over_exp": ratio_final,
                        "comparable_definition": (
                            "heating_test_G0"
                            if st_key in ("stage_a", "stage_c_decay_only", "stage_d_isothermal", "stage_d_ramp_decay_off", "stage_d_ramp_decay_on")
                            else "source_inclusive_F_gen (different definition vs G=0 anneal)"
                        ),
                    }
                )

    return pd.DataFrame(point_rows), pd.DataFrame(summary_rows)


def compute_bound_metrics(extracted, hfr_k31):
    rows = []

    # Upper bounds from Bullock / TE-2090 Table 22 at 1200, 1350, 1500 C
    for tc in [1200, 1350, 1500]:
        exp_df = extracted[tc]
        ub = exp_df[exp_df["series_kind"] == "upper_bound"].copy()
        for _, r in ub.iterrows():
            series_name = r["series"]
            t_h = float(r["time_h"])
            u_exp = float(r["release_fraction"])
            # Primary reading + optional alternative collector reading
            readings = [("primary", u_exp)]
            if pd.notna(r.get("alt_bound_fraction")):
                readings.append(("alt_0.01pct_collector", float(r["alt_bound_fraction"])))

            for reading_kind, u_val in readings:
                for st_key, st_label in STAGES_ORDER:
                    cdf = load_curve(tc, st_key)
                    for m_col, m_name in [("f_method_a", "Method A"), ("f_method_b", "Method B")]:
                        f_mod = interp_series(cdf, m_col, t_h)
                        if np.isnan(f_mod):
                            continue
                        delta_b_pp = 100.0 * (f_mod - u_val)
                        viol_pp = max(0.0, delta_b_pp)
                        rows.append(
                            {
                                "temperature_c": tc,
                                "series": f"{series_name} ({reading_kind})" if reading_kind != "primary" else series_name,
                                "reading_kind": reading_kind,
                                "stage_key": st_key,
                                "stage_label": st_label,
                                "method": m_name,
                                "time_h": t_h,
                                "upper_bound_fraction": u_val,
                                "upper_bound_pct": 100.0 * u_val,
                                "f_model": f_mod,
                                "f_model_pct": 100.0 * f_mod,
                                "delta_bound_pp": delta_b_pp,
                                "violation_pp": viol_pp,
                                "exceeds_bound": bool(f_mod > u_val),
                                "ratio_model_over_bound": float(f_mod / u_val) if u_val > 0 else np.nan,
                            }
                        )

    # 1600 C HFR-K3/1 sphere upper bound (points up to 400 h)
    sub_hfr = hfr_k31[hfr_k31["time_h"] <= 400.0].copy()
    t_last = float(sub_hfr["time_h"].max())
    u_last = float(sub_hfr[sub_hfr["time_h"] == t_last]["value"].iloc[0])
    for st_key, st_label in STAGES_ORDER:
        cdf = load_curve(1600, st_key)
        for m_col, m_name in [("f_method_a", "Method A"), ("f_method_b", "Method B")]:
            f_arr = np.array([interp_series(cdf, m_col, t) for t in sub_hfr["time_h"]], dtype=float)
            valid = ~np.isnan(f_arr)
            if not np.any(valid):
                continue
            u_arr = sub_hfr["value"].to_numpy(dtype=float)[valid]
            f_v = f_arr[valid]
            viol = np.maximum(0.0, 100.0 * (f_v - u_arr))
            f_at_last = interp_series(cdf, m_col, t_last)
            rows.append(
                {
                    "temperature_c": 1600,
                    "series": f"IAEA97-K31-CS137 (HFR-K3/1 sphere, t<=400h, N={int(np.sum(valid))})",
                    "reading_kind": "sphere_upper_bound",
                    "stage_key": st_key,
                    "stage_label": st_label,
                    "method": m_name,
                    "time_h": t_last,
                    "upper_bound_fraction": u_last,
                    "upper_bound_pct": 100.0 * u_last,
                    "f_model": f_at_last,
                    "f_model_pct": 100.0 * f_at_last,
                    "delta_bound_pp": 100.0 * (f_at_last - u_last),
                    "violation_pp": float(np.max(viol)),
                    "exceeds_bound": bool(np.any(f_v > u_arr)),
                    "ratio_model_over_bound": float(f_at_last / u_last) if u_last > 0 else np.nan,
                }
            )

    return pd.DataFrame(rows)


def compute_stage_improvements(summary_df: pd.DataFrame) -> pd.DataFrame:
    rows = []
    for (case_id, method), grp in summary_df.groupby(["case_id", "method"]):
        base = grp[grp["stage_key"] == "stage_a"]
        if base.empty:
            continue
        b = base.iloc[0]
        for _, r in grp.iterrows():
            imp_rmse = 100.0 * (b["rmse_pp"] - r["rmse_pp"]) / b["rmse_pp"] if b["rmse_pp"] > 0 else np.nan
            imp_mae = 100.0 * (b["mae_pp"] - r["mae_pp"]) / b["mae_pp"] if b["mae_pp"] > 0 else np.nan
            imp_max = 100.0 * (b["max_abs_delta_pp"] - r["max_abs_delta_pp"]) / b["max_abs_delta_pp"] if b["max_abs_delta_pp"] > 0 else np.nan
            imp_iae = 100.0 * (b["iae_time_avg_pp"] - r["iae_time_avg_pp"]) / b["iae_time_avg_pp"] if b["iae_time_avg_pp"] > 0 else np.nan
            rows.append(
                {
                    "case_id": case_id,
                    "temperature_c": int(r["temperature_c"]),
                    "method": method,
                    "stage_key": r["stage_key"],
                    "stage_label": r["stage_label"],
                    "comparable_definition": r["comparable_definition"],
                    "rmse_pp": r["rmse_pp"],
                    "mae_pp": r["mae_pp"],
                    "max_abs_delta_pp": r["max_abs_delta_pp"],
                    "iae_time_avg_pp": r["iae_time_avg_pp"],
                    "final_delta_pp": r["final_delta_pp"],
                    "rmse_improvement_vs_A_pct": imp_rmse,
                    "mae_improvement_vs_A_pct": imp_mae,
                    "max_abs_improvement_vs_A_pct": imp_max,
                    "iae_improvement_vs_A_pct": imp_iae,
                }
            )
    return pd.DataFrame(rows)


def build_manifest(summary_df, bounds_df, imp_df):
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()
    c01 = json.load(open(OUT / "diagnostics" / "c01_convergence.json", encoding="utf-8"))
    c02 = json.load(open(OUT / "diagnostics" / "c02_method_a_vs_b.json", encoding="utf-8"))

    graphs = []
    # A01-A24
    stage_defs = [
        ("Stage A", "stage_a", "Constant D(T_nom), G=0, lambda=0, KernelUniform C_init=1"),
        ("Stage B", "stage_b", "Constant D(T_nom), +G_ref (AGR-1 cap3), lambda=0, Fresh particle, F_gen"),
        ("Stage C", "stage_c", "Constant D(T_nom), +G_ref, +lambda(Cs-137), Fresh particle, F_gen (+ G=0 decay bracket)"),
        ("Stage D", "stage_d_ramp_decay_off", "Temperature-dependent D(r, T(t)) over heating ramp+hold (+ isothermal & combined)"),
    ]
    idx = 1
    for st_name, st_key, st_desc in stage_defs:
        for tc in TEMPS:
            gid = f"A{idx:02d}"
            idx += 1
            graphs.append(
                {
                    "graph_id": gid,
                    "group": "A",
                    "title": f"{st_name} — {tc} °C",
                    "temperature_c": tc,
                    "duration_h": DURATIONS_H[tc],
                    "stage_key": st_key,
                    "physics_summary": st_desc,
                    "nuclide": "Cs-137",
                    "solver": "strato v2.1.0 (Method A BTCS + Method B S-L where applicable)",
                    "commit": commit,
                    "curve_csv": f"data/study39/curves/{tc}c_{st_key}.csv",
                    "output_files": [f"graphs/study39/{gid}.png", f"graphs/study39/{gid}.svg", f"graphs/study39/{gid}.csv"],
                    "validation_status": "verified_simulation_generated",
                }
            )

    # B01-B06
    for k, tc in enumerate(TEMPS, start=1):
        gid = f"B{k:02d}"
        c02_row = next((r for r in c02["comparisons"] if int(r["temperature_c"]) == tc), {})
        graphs.append(
            {
                "graph_id": gid,
                "group": "B",
                "title": f"Analytical Sturm–Liouville — {tc} °C",
                "temperature_c": tc,
                "duration_h": DURATIONS_H[tc],
                "physics_summary": "Method B (Sturm-Liouville) vs equivalent Method A BTCS under Stages A, B, C; ramp Stage D withheld (TIME_VARYING_TEMPERATURE)",
                "method_a_vs_b_stage_a_max_pp": c02_row.get("max_abs_diff_pp"),
                "valid_from_h": c02_row.get("valid_from_h"),
                "accuracy_floor": c02_row.get("accuracy_floor"),
                "output_files": [f"graphs/study39/{gid}.png", f"graphs/study39/{gid}.svg", f"graphs/study39/{gid}.csv"],
                "validation_status": "verified_analytical_comparison",
            }
        )

    # C01-C03
    graphs.extend(
        [
            {
                "graph_id": "C01",
                "group": "C",
                "title": "BTCS Numerical Convergence Study",
                "spatial_order": c01.get("last_spatial_order"),
                "temporal_order": c01.get("last_temporal_order"),
                "output_files": ["graphs/study39/C01.png", "graphs/study39/C01.svg", "graphs/study39/C01.csv"],
                "validation_status": "verified_convergence_order_2_space_1_time",
            },
            {
                "graph_id": "C02",
                "group": "C",
                "title": "Method A versus Method B",
                "temperatures_c": TEMPS,
                "output_files": ["graphs/study39/C02.png", "graphs/study39/C02.svg", "graphs/study39/C02.csv"],
                "validation_status": "verified_numerical_analytical_agreement",
            },
            {
                "graph_id": "C03",
                "group": "C",
                "title": "STRATO Experimental Deviation Summary",
                "output_files": ["graphs/study39/C03.png", "graphs/study39/C03.svg", "graphs/study39/C03.csv"],
                "validation_status": "verified_experimental_deviation_summary",
            },
        ]
    )

    # D01-D06
    d_specs = [
        ("D01", 1200, "Cs-137 release comparison — 1200 °C", "Bullock (1984) / IAEA-TECDOC-2090 Table 22 UCO/UO2*(2) 5% & <0.01% upper bounds", "upper_bound_only"),
        ("D02", 1350, "Cs-137 release comparison — 1350 °C", "Bullock (1984) / IAEA-TECDOC-2090 Table 22 UCO/UO2*(2) 5% & <0.01% upper bounds", "upper_bound_only"),
        ("D03", 1500, "Cs-137 release comparison — 1500 °C", "Bullock (1984) Fig. 1 / IAEA-TECDOC-2090 Fig. 124a & Table 22 (no AGR-1 1500 C Cs dataset exists)", "upper_bound_plus_noncomparable_degraded_context"),
        ("D04", 1700, "Cs-137 release comparison — 1700 °C", "Minato et al. (2000) HRB-22 ACT3 / TE-2090 Fig. 125 (not AGR-1; N=8 intact particles)", "comparable_exact_time_series"),
        ("D05", 1800, "Cs-137 release comparison — 1800 °C", "Minato et al. (2000) HRB-22 ACT4 / TE-2090 Fig. 126 (bounded w=1/25, N=11 incl. 4.6h cluster) + AGR-1 4-4-2 HUNN16-442 & JAERI-A non-comparable context", "comparable_bounded_time_series"),
        ("D06", "955-1136", "AGR-1 irradiation-temperature comparison — approximately 955–1136 °C", "AGR-1 irradiation C15 Cs-137 exploration (620.2 EFPD, 955/1028/1136 C); no direct Cs-137 irradiation validation possible", "non_comparable_exploration_only"),
    ]
    for gid, tc, title, exp_ref, status in d_specs:
        graphs.append(
            {
                "graph_id": gid,
                "group": "D",
                "title": title,
                "temperature_c": tc,
                "experimental_reference": exp_ref,
                "validation_status": status,
                "output_files": [f"graphs/study39/{gid}.png", f"graphs/study39/{gid}.svg", f"graphs/study39/{gid}.csv"],
            }
        )

    manifest = {
        "schema_version": 1,
        "commit": commit,
        "principal_graph_count": len(graphs),
        "graphs": graphs,
        "exact_series_summary_count": int(len(summary_df)),
        "upper_bound_rows_count": int(len(bounds_df)),
        "stage_improvement_rows_count": int(len(imp_df)),
    }
    return manifest


def main():
    extracted, hfr_k31, jaeri, hunn = extract_experimental_data()
    pts_df, sum_df = compute_exact_series_metrics(extracted)
    bnd_df = compute_bound_metrics(extracted, hfr_k31)
    imp_df = compute_stage_improvements(sum_df)

    pts_df.to_csv(OUT / "deviations_exact_points.csv", index=False)
    sum_df.to_csv(OUT / "deviations_exact_summary.csv", index=False)
    bnd_df.to_csv(OUT / "deviations_bounds.csv", index=False)
    imp_df.to_csv(OUT / "stage_improvement.csv", index=False)

    manifest = build_manifest(sum_df, bnd_df, imp_df)
    with open(OUT / "study39_manifest.json", "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)

    print("=== EXACT SERIES SUMMARY (Method A) ===")
    m_a = sum_df[sum_df["method"] == "Method A"]
    print(m_a[["case_id", "stage_key", "n_points", "f_exp_final", "f_model_final", "final_delta_pp", "mae_pp", "rmse_pp", "max_abs_delta_pp"]].to_string(index=False))

    print("\n=== STAGE A->D IMPROVEMENT (ACT3 & ACT4 N=11, Method A) ===")
    mi = imp_df[(imp_df["method"] == "Method A") & (imp_df["case_id"].isin(["D04_1700C_ACT3", "D05_1800C_ACT4_N11"]))]
    print(mi[["case_id", "stage_key", "rmse_pp", "mae_pp", "rmse_improvement_vs_A_pct", "mae_improvement_vs_A_pct"]].to_string(index=False))

    print("\n=== UPPER BOUNDS SUMMARY (Stage A vs Stage D-ramp vs Stage C, Method A) ===")
    ba = bnd_df[(bnd_df["method"] == "Method A") & (bnd_df["stage_key"].isin(["stage_a", "stage_c_decay_only", "stage_c", "stage_d_ramp_decay_off"]))]
    print(ba[["temperature_c", "series", "stage_key", "upper_bound_pct", "f_model_pct", "delta_bound_pp", "violation_pp", "exceeds_bound"]].to_string(index=False))
    print(f"\nWrote manifest ({manifest['principal_graph_count']} graphs) and all deviation CSVs to {OUT}")


if __name__ == "__main__":
    main()
