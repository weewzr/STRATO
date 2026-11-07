"""Render STRATO_Parameter_Register.md from a template plus the data CSVs.

usage: python render_register.py <STRATO root> <template.md> <out.md>
Template lines of the form  @@kind arg1 arg2 ...@@  are replaced by Markdown tables.
"""
import csv
import pathlib
import re
import sys

ROOT = pathlib.Path(sys.argv[1])
TPL = pathlib.Path(sys.argv[2])
OUT = pathlib.Path(sys.argv[3])
D = ROOT / "data"


def load(rel):
    with open(D / rel, newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


geo = load("geometry/geometry.csv")
arr = load("diffusivity/arrhenius.csv")
dec = load("nuclear/decay.csv")
fy = load("nuclear/fission_yields.csv")
th = load("temperature/temperature_histories.csv")
rel = load("experimental/release_measurements.csv")

UNIT = {
    "um": r"$\mathrm{\mu m}$", "mm": r"$\mathrm{mm}$", "%": r"$\%$", "1": "1",
    "Mg m^-3": r"$\mathrm{Mg\ m^{-3}}$", "g": r"$\mathrm{g}$", "wt%": r"$\mathrm{wt\%}$",
    "particles": "particles", "h^-1": r"$\mathrm{h^{-1}}$",
}
SYM = {"d_k": r"$d_k = 2R_1$", "t_2": r"$t_2$", "t_3": r"$t_3$", "t_4": r"$t_4$", "t_5": r"$t_5$",
       "rho_1": r"$\rho_1$", "rho_2": r"$\rho_2$", "rho_3": r"$\rho_3$", "rho_4": r"$\rho_4$", "rho_5": r"$\rho_5$",
       "e_U5": r"$\varepsilon_5$", "N_p": r"$N_p$", "f_pack": r"$f_{\mathrm{pack}}$", "f_U": r"$f_U$",
       "L_c": r"$L_c$", "D_c": r"$D_c$", "g_c": r"$g_c$", "w_mat": r"$w_{\mathrm{mat}}$", "m_U": r"$m_U$",
       "f_dSiC": r"$f_{\mathrm{dSiC}}$", "f_dOPyC": r"$f_{\mathrm{dOPyC}}$", "2R": r"$2R$", "m_U5": r"$m_{U5}$",
       "f_U2": r"$f_{U,2}$", "f_U3": r"$f_{U,3}$", "f_U4": r"$f_{U,4}$", "f_U5": r"$f_{U,5}$", "f_Um": r"$f_{U,m}$"}


def sci(s):
    """'5.6e-8' -> $5.6\\times10^{-8}$ ; plain numbers unchanged."""
    s = s.strip()
    m = re.fullmatch(r"([-+]?\d*\.?\d+)[eE]([-+]?\d+)", s)
    if not m:
        return s
    mant, ex = m.group(1), int(m.group(2))
    if ex == 0:
        return mant
    if mant in ("1", "1.0"):
        return rf"$10^{{{ex}}}$"
    return rf"${mant}\times10^{{{ex}}}$"


def link(u):
    if u.startswith("http"):
        return f"[doi]({u})" if "doi.org" in u else f"[link]({u})"
    return u.replace("file:", "local ")


def esc(s):
    return s.replace("|", "/")


def table(header, rows):
    out = ["| " + " | ".join(header) + " |", "|" + "---|" * len(header)]
    out += ["| " + " | ".join(esc(c) for c in r) + " |" for r in rows]
    return "\n".join(out)


def geom(args):
    ids = set(args)
    rows = []
    for r in geo:
        if r["design_id"] not in ids:
            continue
        if r["value"]:
            v = sci(r["value"])
        elif r["value_min"] and r["value_max"]:
            v = f"{sci(r['value_min'])} to {sci(r['value_max'])}"
        else:
            v = f"< {sci(r['value_max'])}"
        var = f"{r['layer']} {r['quantity']}" if r["layer_index"] != "0" else r["quantity"]
        note = f"; {r['notes']}" if r["notes"] else ""
        rows.append([r["id"], f"{var} ({r['design']})", SYM.get(r["symbol"], r["symbol"]), v,
                     UNIT.get(r["unit"], r["unit"]), r["material"], "n/a",
                     r["uncertainty"] + f"; {r['data_class']}" + note, r["source"], r["source_location"],
                     link(r["link"]), r["evidence_class"], r["status"]])
    return table(["No.", "Variable", "Symbol", "Value", "Unit", "Material / Nuclide", "Temperature range",
                  "Uncertainty / class / note", "Source / APA citation", "Location", "Link", "Evidence class",
                  "Status"], rows)


def radii(args):
    rows = []
    for did in args:
        lay = {int(r["layer_index"]): r for r in geo if r["design_id"] == did and r["outer_radius_um_calc"]}
        d = lay[1]["design"]
        rows.append([did, d] + [lay[i]["outer_radius_um_calc"] for i in range(1, 6)] +
                    [", ".join(lay[i]["id"] for i in range(1, 6))])
    return table(["Design id", "Design", r"$R_1$ ($\mathrm{\mu m}$)", r"$R_2$", r"$R_3$", r"$R_4$",
                  r"$R_5 = R$", "Input rows"], rows)


def trange(r):
    a, b, note = r["T_min_C"], r["T_max_C"], r["T_range_note"]
    if a and b and a == b:
        return f"{a} °C only"
    if a and b:
        return f"{a} to {b} °C"
    if b:
        return rf"$\le {b}$ °C"
    return note


def arrh(args):
    sets = set(args)
    rows = []
    for r in arr:
        if r["set_id"] not in sets:
            continue
        d0 = sci(r["D0_m2_s"]) if r["D0_m2_s"] else r"$\infty$"
        if r["id"] in ("A3-01", "E109-01"):
            d0 = r"$5.5\times10^{-14}\,e^{\Gamma/5}$"
        if r["id"] == "A3-13":
            d0 = r"$\le 6.8\times10^{-12}$"
        q = r["Q_kJ_mol"] if r["Q_kJ_mol"] else "none"
        note = f" {r['D0_expression']}." if r["D0_expression"] and r["id"] not in ("A3-01", "E109-01", "A3-13") else ""
        form = r["form"] + (f"; {r['notes']}" if r["notes"] else "") + note
        rows.append([r["id"], f"{r['nuclide']} ({r['origin']})", f"{r['material']} [{r['layer']}]", r["term"], d0, q,
                     r"$\mathrm{m^2\ s^{-1}}$; $\mathrm{kJ\ mol^{-1}}$", trange(r), form,
                     f"{r['source']}; ref. {r['ref_in_source']}", r["source_location"], link(r["link"]),
                     f"{r['evidence_class']}; $R_g$ = {r['R_J_mol_K']}", r["status"]])
    return table(["No.", "Nuclide (origin)", "Layer/material [STRATO layer]", "Term", "D0", "Q", "Units",
                  "Temperature range", "Form / note", "APA citation", "Location", "Link",
                  "Evidence class; gas constant", "Status"], rows)


def decay(_):
    rows = []
    for i, r in enumerate(dec, 1):
        if r["stable"] == "yes":
            hl, lam, ul, mode = "stable", "0", "", "stable"
        else:
            hl = f"{r['half_life']} {r['unit']} (ENSDF last digit {r['unc_half_life'].split()[0]})"
            lam, ul = sci(r["lambda_s-1"]), sci(r["unc_lambda_s-1"])
            mode = f"{r['decay_mode_1']} {r['branch_1_pct']} %" + (
                f"; {r['decay_mode_2']} {r['branch_2_pct']} %" if r["decay_mode_2"] else "")
        rows.append([f"N-{i:02d}", "decay constant", r"$\lambda_k$", lam, r"$\mathrm{s^{-1}}$", r["nuclide"],
                     "n/a", ul, hl, mode, r["daughters"], r["product_class"],
                     f"{r['ENSDF_evaluation']}; via IAEA LiveChart", r["source_location"].split(";")[0],
                     "[link](https://www-nds.iaea.org/relnsd/v1/data)", r["evidence_class"], r["status"]])
    return table(["No.", "Variable", "Symbol", "Value", "Unit", "Material / Nuclide", "Temperature range",
                  r"Uncertainty $u(\lambda_k)$", r"$t_{1/2}$", "Decay mode", "Daughter(s)",
                  "Direct fission product or activation product", "Source / APA citation", "Location", "Link",
                  "Evidence class", "Status"], rows)


def fyt(args):
    keep = set(args)
    rows = []
    i = 0
    for r in fy:
        if r["product"] not in keep:
            continue
        i += 1
        par = "U-235" if r["parent"] == "235U" else "Pu-239"

        def fmt(v):
            x = float(v)
            if x == 0:
                return "0"
            m, e = f"{x:.5e}".split("e")
            m = m.rstrip("0").rstrip(".")
            return sci(f"{m}e{int(e)}")

        def pm(v, u):
            if "BLANK" in v:
                return "blank in API"
            return rf"{fmt(v)} $\pm$ {fmt(u)}" if u else fmt(v)
        if r["CFY_ENDFB_NuDat3"]:
            endf = pm(r["CFY_ENDFB_NuDat3"], r["unc_CFY_ENDFB_NuDat3"]) + f" ({r['status_ENDFB_crosscheck']})"
        else:
            endf = "not given"
        rows.append([f"Y-{i:02d}", f"cumulative yield, {par} thermal fission", r"$y^C_k$",
                     pm(r["CFY_JEFF311"], r["unc_CFY_JEFF311"]), "per fission", r["product"], "thermal",
                     pm(r["IFY_JEFF311"], r["unc_IFY_JEFF311"]), endf,
                     "JEFF-3.1.1", f"{r['product_class']}; {r['STRATO_use_note']}",
                     "Kellett et al. (2009) via IAEA LiveChart",
                     r["source_location"].split(";")[0], "[link](https://www-nds.iaea.org/relnsd/v1/data)",
                     r["evidence_class"], r["status"]])
    return table(["No.", "Variable", "Symbol", r"Value (CFY $\pm 1\sigma$)", "Unit", "Material / Nuclide",
                  "Neutron energy", r"Independent yield $y^I_k \pm 1\sigma$", "ENDF/B CFY via NuDat 3 (status)",
                  "Library", "Direct fission product or activation product; STRATO use", "Source / APA citation",
                  "Location", "Link", "Evidence class", "Status"], rows)


def temp(args):
    pref = tuple(args)
    rows = []
    for r in th:
        if not r["history_id"].startswith(pref):
            continue
        rows.append([r["id"], r["history"], r["step"], r["segment"], r["T_start_C"] or "n/s", r["T_end_C"] or "n/s",
                     r["rate_C_per_h"], r["ramp_h"], r["hold_h"], r["cumulative_end_h_calc"], r["history_class"],
                     r["notes"], r["source"], r["source_location"], link(r["link"]), r["evidence_class"],
                     r["status"]])
    return table(["No.", "History", "Step", "Segment", "T start (°C)", "T end (°C)",
                  r"Rate ($\mathrm{°C\ h^{-1}}$)", "Ramp (h)", "Hold (h)", "Cumulative end (h, calc.)",
                  "Class (measured / reconstructed / specified)", "Note", "Source / APA citation", "Location",
                  "Link", "Evidence class", "Status"], rows)


def relt(args):
    prog, kind = args[0], args[1]
    rows = []
    for r in rel:
        if r["program"] != prog:
            continue
        if kind == "counts" and "failed" not in r["quantity"]:
            continue
        if kind != "counts" and r["unit"] == "particles" and "failed" in r["quantity"]:
            continue
        if kind not in ("all", "counts") and r["nuclide"] != kind:
            continue
        if r["value"]:
            v = f"{r['qualifier'] if r['qualifier'] not in ('=',) else ''} {sci(r['value'])}".strip()
            if r["value_max"] and kind == "counts":
                v += f" of {r['value_max']}"
        else:
            v = f"{sci(r['value_min'])} to {sci(r['value_max'])}"
        rows.append([r["id"], f"{r['quantity']} ({r['group']})", r"$f_{\mathrm{rel}}$" if r["unit"] == "1" else "",
                     v, UNIT.get(r["unit"], r["unit"]), f"{r['nuclide']}; {r['compact_or_sample']} ({r['fuel_type']})",
                     f"{r['T_C']} °C" if r["T_C"] else "n/a",
                     f"{r['value_class']}; t = {r['time_h'] or 'n/s'} h ({r['time_basis']})",
                     sci(r["parfume_value_same_source"]) if r["parfume_value_same_source"] else "",
                     r["notes"], r["particle_applicability"], r["source"], r["source_location"], link(r["link"]),
                     r["evidence_class"], r["status"]])
    return table(["No.", "Variable", "Symbol", "Value", "Unit", "Material / Nuclide", "Temperature range",
                  "Uncertainty / data class / time", "PARFUME, same source (calculated)", "Note",
                  "Applicability to one intact particle", "Source / APA citation", "Location", "Link",
                  "Evidence class", "Status"], rows)


def checkout(_):
    lines = pathlib.Path(sys.argv[4]).read_text(encoding="utf-8").splitlines()
    s = [x for x in lines if x.startswith(("PASS [S]", "FAIL [S]"))]
    ok = all(x.startswith("PASS") for x in s)
    head = f"{'PASS' if ok else 'FAIL'} [S] {len(s)} schema checks on the six CSV files (header, source and status columns, status vocabulary, unique ids, rectangular)"
    return "\n".join([head] + [x for x in lines if x not in s])


KINDS = {"geom": geom, "radii": radii, "arr": arrh, "decay": decay, "fy": fyt, "temp": temp, "rel": relt,
         "CHECKOUT": checkout}
out = []
for line in TPL.read_text(encoding="utf-8").splitlines():
    m = re.fullmatch(r"@@(\w+)\s*(.*)@@", line.strip())
    if m:
        out.append(KINDS[m.group(1)](m.group(2).split()))
    else:
        out.append(line)
OUT.write_text("\n".join(out) + "\n", encoding="utf-8")
print("rendered", OUT, "lines", len(out))
