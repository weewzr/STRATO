#!/usr/bin/env python3
"""Export compact canonical Phase 2 Cs-137 study data into web/src/data/study39.json."""
import json
from pathlib import Path
import numpy as np
import pandas as pd

REPO = Path(__file__).resolve().parent.parent
DATA = REPO / "data" / "study39"
EXP = DATA / "experimental"
OUT = REPO / "web" / "src" / "data" / "study39.json"
TEMPS = [1200, 1350, 1500, 1600, 1700, 1800]
STAGES = ["stage_a", "stage_b", "stage_c", "stage_c_decay_only", "stage_d_ramp_decay_off"]


def decimate(df: pd.DataFrame, max_pts: int = 65):
    n = len(df)
    if n <= max_pts:
        sub = df
    else:
        idx = sorted(set(np.linspace(0, n - 1, max_pts, dtype=int).tolist()))
        sub = df.iloc[idx]
    return {
        "time_h": [round(float(x), 4) for x in sub["time_h"]],
        "f_a": [float(x) for x in sub["f_method_a"]],
        "f_b": [None if pd.isna(x) else float(x) for x in sub["f_method_b"]],
    }


def main():
    c01 = json.load(open(DATA / "diagnostics" / "c01_convergence.json", encoding="utf-8"))
    c02 = json.load(open(DATA / "diagnostics" / "c02_method_a_vs_b.json", encoding="utf-8"))
    mb = pd.read_csv(DATA / "diagnostics" / "mass_balance_audit.csv")
    sum_df = pd.read_csv(DATA / "deviations_exact_summary.csv")
    bnd_df = pd.read_csv(DATA / "deviations_bounds.csv")
    imp_df = pd.read_csv(DATA / "stage_improvement.csv")
    manifest = json.load(open(DATA / "study39_manifest.json", encoding="utf-8"))

    curves = {}
    experimental = {}
    for tc in TEMPS:
        curves[str(tc)] = {}
        for st in STAGES:
            df = pd.read_csv(DATA / "curves" / f"{tc}c_{st}.csv")
            curves[str(tc)][st] = decimate(df, 65)

        if tc == 1600:
            hfr = pd.read_csv(EXP / "exp_1600c_hfr_k31_cs137.csv")
            hfr = hfr[hfr["time_h"] <= 400.0]
            experimental[str(tc)] = [
                {
                    "series": "IAEA97-K31-CS137 (HFR-K3/1 sphere)",
                    "kind": "upper_bound",
                    "time_h": [float(x) for x in hfr["time_h"]],
                    "f": [float(x) for x in hfr["value"]],
                }
            ]
        else:
            edf = pd.read_csv(EXP / f"exp_{tc}c_cs.csv")
            slist = []
            for ser, grp in edf.groupby("series"):
                grp = grp.sort_values("time_h")
                slist.append(
                    {
                        "series": ser,
                        "kind": str(grp["series_kind"].iloc[0]),
                        "time_h": [float(x) for x in grp["time_h"]],
                        "f": [float(x) for x in grp["release_fraction"]],
                    }
                )
            experimental[str(tc)] = slist

    exact_rows = sum_df[
        (sum_df["method"] == "Method A")
        & (sum_df["case_id"].isin(["D04_1700C_ACT3", "D05_1800C_ACT4_N11"]))
        & (sum_df["stage_key"].isin(STAGES))
    ].to_dict(orient="records")

    imp_rows = imp_df[
        (imp_df["method"] == "Method A")
        & (imp_df["case_id"].isin(["D04_1700C_ACT3", "D05_1800C_ACT4_N11"]))
        & (imp_df["stage_key"].isin(STAGES))
    ].to_dict(orient="records")

    bnd_rows = bnd_df[
        (bnd_df["method"] == "Method A")
        & (bnd_df["stage_key"].isin(["stage_a", "stage_c_decay_only", "stage_c", "stage_d_ramp_decay_off"]))
    ][["temperature_c", "series", "stage_key", "upper_bound_pct", "f_model_pct", "delta_bound_pp", "violation_pp", "ratio_model_over_bound"]].to_dict(orient="records")

    out = {
        "species": "Cs-137",
        "temperatures": TEMPS,
        "stages": STAGES,
        "verification": {
            "spatial_order": c01["last_spatial_order"],
            "temporal_order": c01["last_temporal_order"],
            "max_residual_over_bound": float(mb["max_residual_over_bound"].max()),
            "run_count": int(len(mb)),
            "max_ab_diff_pp": float(max(r["max_abs_diff_pp"] for r in c02["comparisons"])),
        },
        "curves": curves,
        "experimental": experimental,
        "exact_summary": exact_rows,
        "stage_improvement": imp_rows,
        "bounds": bnd_rows,
        "graph_count": manifest["principal_graph_count"],
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(out, indent=2), encoding="utf-8")
    print(f"Exported {OUT} ({OUT.stat().st_size} B)")


if __name__ == "__main__":
    main()
