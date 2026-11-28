"""Export complete Phase 3 Cs-137 study data and authentic s5 operating scenario curves
to web/src/data/study39.json for interactive Option B exploration and Scientific Documentation.
"""
import json
import pathlib

import numpy as np
import pandas as pd

REPO = pathlib.Path(__file__).resolve().parent.parent
DATA = REPO / "data" / "study39"
EXP = DATA / "experimental"
S5_DIR = REPO / "data" / "results" / "curves"
OUT = REPO / "web" / "src" / "data" / "study39.json"

TEMPS = [1200, 1350, 1500, 1600, 1700, 1800]

CONFIG_META = [
    {"config_id": "P01", "stage_alias": "Stage A", "label": "Baseline diffusion", "curve_suffix": "stage_a", "generation": False, "decay": False, "temperature": False, "controlled_id": None},
    {"config_id": "P02", "stage_alias": "Stage B", "label": "Baseline + generation", "curve_suffix": "stage_b", "generation": True, "decay": False, "temperature": False, "controlled_id": "CTRL_G"},
    {"config_id": "P03", "stage_alias": "Stage C (decay-only; G=0)", "label": "Baseline + decay", "curve_suffix": "stage_c_decay_only", "generation": False, "decay": True, "temperature": False, "controlled_id": None},
    {"config_id": "P04", "stage_alias": "Stage D (heating ramp; G=0)", "label": "Baseline + temperature-dependent D(T)", "curve_suffix": "stage_d_ramp_decay_off", "generation": False, "decay": False, "temperature": True, "controlled_id": None},
    {"config_id": "P05", "stage_alias": "Stage C", "label": "Baseline + generation + decay", "curve_suffix": "stage_c", "generation": True, "decay": True, "temperature": False, "controlled_id": "CTRL_GL"},
    {"config_id": "P06", "stage_alias": "P06 (+G, T(t))", "label": "Baseline + generation + temperature-dependent D(T)", "curve_suffix": "p06_gen_temp", "generation": True, "decay": False, "temperature": True, "controlled_id": None},
    {"config_id": "P07", "stage_alias": "Stage D (+decay, T(t))", "label": "Baseline + decay + temperature-dependent D(T)", "curve_suffix": "stage_d_ramp_decay_on", "generation": False, "decay": True, "temperature": True, "controlled_id": None},
    {"config_id": "P08", "stage_alias": "Stage D (combined)", "label": "Baseline + generation + decay + temperature-dependent D(T)", "curve_suffix": "stage_d_combined", "generation": True, "decay": True, "temperature": True, "controlled_id": None},
]

CONTROL_CONFIGS = [
    {"config_id": "CTRL_G", "stage_alias": "Controlled (+G matched C_init)", "label": "Controlled +G (matched C_init = G * 620.2 d)", "curve_suffix": "ctrl_gen_on_decay_off", "base_config": "P02"},
    {"config_id": "CTRL_GL", "stage_alias": "Controlled (+G +lambda matched C_init)", "label": "Controlled +G +decay (matched C_init = G * 620.2 d)", "curve_suffix": "ctrl_gen_on_decay_on", "base_config": "P05"},
]


def decimate_curve_df(df: pd.DataFrame, n_max: int = 70) -> dict:
    """Downsample a curve DataFrame while keeping the initial and exact final points."""
    n = len(df)
    if n <= n_max:
        sub = df
    else:
        idx = np.unique(np.concatenate(([0], np.linspace(0, n - 1, n_max - 2, dtype=int), [n - 1])))
        sub = df.iloc[idx]
    fb = []
    if "f_method_b" in sub.columns:
        for v in sub["f_method_b"]:
            fb.append(None if pd.isna(v) else float(v))
    else:
        fb = [None] * len(sub)
    return {
        "time_h": [float(x) for x in sub["time_h"]],
        "f_a": [float(x) for x in sub["f_method_a"]],
        "f_b": fb,
    }


def decimate_s5_series(path: pathlib.Path, n_max: int = 90) -> dict:
    """Export one authentic Stage 5 operating scenario JSON curve with exact endpoint preservation."""
    d = json.loads(path.read_text(encoding="utf-8"))
    ma = d["method_a"]
    th = ma["time_h"]
    rf = ma["release_fraction"]
    rfg = ma["release_fraction_of_generated"]
    n = len(th)

    # Always preserve post-event transition index + start + end.
    keep = {0, n - 1}
    first_post = next((i for i, v in enumerate(rf) if v is not None), None)
    if first_post is not None:
        keep.add(max(0, first_post - 1))
        keep.add(first_post)
        if first_post + 1 < n:
            keep.add(first_post + 1)
    if n <= n_max:
        idx = list(range(n))
    else:
        grid = set(int(x) for x in np.linspace(0, n - 1, n_max - len(keep), dtype=int))
        idx = sorted(keep | grid)

    ev_s = d["selection"].get("event_time_s")
    return {
        "label": ma.get("label", d["selection"]["scenario"]),
        "scenario": d["selection"]["scenario"],
        "variant": d["selection"].get("variant", ""),
        "context": d["selection"].get("context", ""),
        "nuclide": d["selection"]["nuclide"],
        "normal_temperature_c": float(d["selection"]["normal_operation_temperature_c"]),
        "event_time_h": None if ev_s is None else float(ev_s) / 3600.0,
        "plotted_quantity": ma.get("plotted_quantity", ""),
        "final_rfg": None if rfg[-1] is None else float(rfg[-1]),
        "final_post_event": None if rf[-1] is None else float(rf[-1]),
        "time_h": [float(th[i]) for i in idx],
        "rf_gen": [None if rfg[i] is None else float(rfg[i]) for i in idx],
        "post_event": [None if rf[i] is None else float(rf[i]) for i in idx],
    }


def main():
    c01 = json.loads((DATA / "diagnostics" / "c01_convergence.json").read_text(encoding="utf-8"))
    c02 = json.loads((DATA / "diagnostics" / "c02_method_a_vs_b.json").read_text(encoding="utf-8"))
    mb = pd.read_csv(DATA / "diagnostics" / "mass_balance_audit.csv")
    sum_df = pd.read_csv(DATA / "deviations_exact_summary.csv")
    bnd_df = pd.read_csv(DATA / "deviations_bounds.csv")
    imp_df = pd.read_csv(DATA / "stage_improvement.csv")
    ep_df = pd.read_csv(DATA / "endpoint_deviations.csv")
    manifest = json.loads((DATA / "study39_manifest.json").read_text(encoding="utf-8"))
    web81 = json.loads((DATA / "web81_manifest.json").read_text(encoding="utf-8"))

    all_curve_specs = CONFIG_META + CONTROL_CONFIGS
    curves = {}
    experimental = {}

    for tc in TEMPS:
        t_key = str(tc)
        curves[t_key] = {}
        for spec in all_curve_specs:
            cid = spec["config_id"]
            suf = spec["curve_suffix"]
            csv_path = DATA / "curves" / f"{tc}c_{suf}.csv"
            json_path = DATA / "curves" / f"{tc}c_{suf}.json"
            df = pd.read_csv(csv_path)
            entry = decimate_curve_df(df, 70)
            raw_meta = json.loads(json_path.read_text(encoding="utf-8"))
            mb_meta = raw_meta.get("method_b", {})
            entry["method_b_meta"] = {
                "available": bool(mb_meta.get("available", False)),
                "reason_code": mb_meta.get("reason_code"),
                "reason": mb_meta.get("reason"),
                "valid_from_h": mb_meta.get("valid_from_h"),
                "accuracy_floor": mb_meta.get("accuracy_floor"),
            }
            curves[t_key][cid] = entry

        if tc == 1600:
            hfr = pd.read_csv(EXP / "exp_1600c_hfr_k31_cs137.csv")
            hfr = hfr[hfr["time_h"] <= 400.0]
            experimental[t_key] = [
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
                        "series": str(ser),
                        "kind": str(grp["series_kind"].iloc[0]),
                        "time_h": [float(x) for x in grp["time_h"]],
                        "f": [float(x) for x in grp["release_fraction"]],
                    }
                )
            experimental[t_key] = slist

    ep_cols = [
        "temperature_c", "config_id", "stage_alias", "curve_key", "method",
        "t_final_h", "exp_dataset", "exp_kind", "f_exp_or_u_pct",
        "f_model_pct", "delta_pp", "violation_pp", "ratio_model_over_exp",
        "label_text", "comparison_class",
    ]
    endpoint_deviations = []
    for rec in ep_df[ep_cols].to_dict(orient="records"):
        clean = {}
        for k, v in rec.items():
            clean[k] = None if pd.isna(v) else v
        endpoint_deviations.append(clean)

    exact_rows = sum_df[
        (sum_df["method"] == "Method A")
        & (sum_df["case_id"].isin(["D04_1700C_ACT3", "D05_1800C_ACT4_N11"]))
    ].to_dict(orient="records")

    imp_rows = imp_df[
        (imp_df["method"] == "Method A")
        & (imp_df["case_id"].isin(["D04_1700C_ACT3", "D05_1800C_ACT4_N11"]))
    ].to_dict(orient="records")

    bnd_rows = bnd_df[bnd_df["method"] == "Method A"][
        ["temperature_c", "series", "stage_key", "upper_bound_pct", "f_model_pct", "delta_pp" if "delta_pp" in bnd_df.columns else "delta_bound_pp", "violation_pp", "ratio_model_over_bound"]
    ].to_dict(orient="records")

    # Summarize web81 entries cleanly for the website (preserving all 81 entries without large path clutter).
    web81_entries = []
    for e in web81["entries"]:
        item = {
            "entry_id": e["entry_id"],
            "category": e["category"],
            "status": e.get("status"),
        }
        for k in ("method", "config_id", "stage_alias", "temperature_c", "applicable", "reason_code", "label_text", "comparison_class", "dataset_id", "description", "verification_id", "title"):
            if k in e and e[k] is not None:
                item[k] = e[k]
        web81_entries.append(item)

    # Authentic Stage 5 operating scenario curves for Cs-137 & Sr-90 at 1028 °C (plus refuelling heating 1600/1700/1800 °C).
    s5_map = {
        "steady": "steady-state",
        "shutdown": "shutdown-cooldown",
        "startup": "startup",
        "control-rod": "control-rod-insertion",
        "refuelling": "refuelling",
        "refuelling-1600": "refuelling.heating1600",
        "refuelling-1700": "refuelling.heating1700",
        "refuelling-1800": "refuelling.heating1800",
    }
    scenarios = {}
    for nuc in ("cs137", "sr90"):
        scenarios[nuc] = {}
        for skey, fname in s5_map.items():
            p = S5_DIR / f"s5.{fname}.{nuc}.1028c.json"
            if p.exists():
                scenarios[nuc][skey] = decimate_s5_series(p, 90)

    out = {
        "species": "Cs-137",
        "temperatures": TEMPS,
        "configs": CONFIG_META,
        "control_configs": CONTROL_CONFIGS,
        "config_ids": [c["config_id"] for c in CONFIG_META],
        "verification": {
            "spatial_order": float(c01["last_spatial_order"]),
            "temporal_order": float(c01["last_temporal_order"]),
            "max_residual_over_bound": float(mb["max_residual_over_bound"].max()),
            "run_count": int(len(mb)),
            "max_ab_diff_pp": float(max(r["max_abs_diff_pp"] for r in c02["comparisons"])),
        },
        "curves": curves,
        "experimental": experimental,
        "endpoint_deviations": endpoint_deviations,
        "web81": {
            "counts": web81["counts"],
            "stage_to_p_map": web81["stage_to_p_map"],
            "entries": web81_entries,
        },
        "exact_summary": exact_rows,
        "stage_improvement": imp_rows,
        "bounds": bnd_rows,
        "scenarios": scenarios,
        "graph_count": manifest["principal_graph_count"],
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(out, separators=(",", ":")), encoding="utf-8")
    print(f"Exported {OUT} ({OUT.stat().st_size} B)")


if __name__ == "__main__":
    main()
