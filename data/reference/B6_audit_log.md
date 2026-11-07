# B6 audit log: parameter register and data CSVs

Audit date 2026-11-06. Scope: `data/diffusivity/arrhenius.csv`, `data/nuclear/decay.csv`, `data/nuclear/fission_yields.csv`, `data/experimental/release_measurements.csv`, the evidence classes of all six CSVs, and the generated register `data/reference/STRATO_Parameter_Register.md` with its generator `scripts/data_register/render_register.py` and template `scripts/data_register/register_template.md`. No Rust file was read for editing or changed.

## 1. Method

- Primary sources were read from page images, not from the OCR layer: IAEA (1997) Tables A-1 to A-5 (2.pdf idx 443, 446, 450, 455, 457, rendered at 2x with PyMuPDF), Collin et al. (2016) Table 2, Table 3, Table 5, Fig. 6 and Fig. 12 (6.pdf idx 3, 4, 8, 6, 8, rendered at 2.5x to 4x), IAEA (2012) Eq. (10.9) (3.pdf idx 561, 3x). Hales et al. (2021) Table 1 (8.pdf idx 2) is born-digital, so its text layer was used. Morris et al. (2016) statements were checked by text search of 5.pdf.
- Nuclear data were downloaded again from the IAEA LiveChart API v1 on 2026-11-06 (20 responses, server `Extraction_date` 2026-11-06) and compared cell by cell with the CSVs by `scripts/data_register/b6_nuclear_crosscheck.py`. The API guide was fetched to confirm that its fission yields are "Source: JEFF 3.1.1".
- DOIs of Collin et al. (2015), Collin et al. (2016), Hales et al. (2021) and Morris et al. (2016) were resolved through the Crossref API.
- Corrections were applied by the idempotent script `scripts/data_register/b6_audit_patch.py`; the register was regenerated from the patched CSVs and template. A cell-by-cell diff against a backup of `data/` shows 33 changed text cells in pre-existing columns and 0 changed numerical values.

## 2. Diffusivity rows (Ag, Cs, Sr)

| Item | Check | Result | Correction |
|---|---|---|---|
| A1-01 to A1-12 (UO2 kernel) | $D_0$, $Q$, term, 1T/2T form, origin, T range, reference against the Table A-1 render | all equal; Japan Cs range 1400 to 1720 °C confirmed; "with respect to kernel radius r = 250 $\mathrm{\mu m}$" confirmed in the title | none |
| A2-01 to A2-14 (LTI PyC) | same, Table A-2 render | all equal; USA Cs 1200 to 1800 °C and Sr 1000 to 1650 °C, Japan Cs two branches (1250 to 1385 °C; 1600 to 2300 °C), Russian Ag 1400 to 1700 °C confirmed | none |
| A3-01 to A3-21 (SiC) | same, Table A-3 render, including footnote (1) | all equal; FRG Cs term 1 printed as $5.5\times10^{-14}\,e^{\Gamma/5}$ with $\Gamma$ in $10^{25}\ \mathrm{m^{-2}}$, E > 16 fJ, "reactor-specific"; Japan Cs printed as an upper bound $\le 6.8\times10^{-12}$; Russian Cs is a single value at 1000 °C with no $Q$ | none to values; evidence class of A3-13 set to "literature correlation (upper bound)" and of A3-20 to "literature value (single temperature)" |
| A4-01 to A4-13 (matrix graphite, Ag/Cs/Sr) | same, Table A-4 render; graphite grade from text p. 448 (idx 452) | all equal; text confirms that the KFA metallic data of Table A-4 refer to irradiated A3-3 | A4-02 note now states the grade source; the assignment of the three FRG Ag rows A4-03 to A4-05 to grades stays NOT VERIFIED (GAP-06) |
| A5-01 to A5-13 (structural graphite) | same, Table A-5 render | all equal | none |
| C16-01 to C16-15 | Collin et al. (2016) Table 2 render and its note | all equal, including Sr matrix $D_0$ printed as "$10^{-2}$" and the dashes for absent second terms; note confirms UO2 values used for UCO, buffer $10^{-8}\ \mathrm{m^2\ s^{-1}}$, fluence-dependent Cs-in-SiC term not used | none |
| H21-01 to H21-09 | Hales et al. (2021) Table 1 text | all equal; Table 1 prints only $D_{0,1}$ and $Q_1$ and is cited to Collin et al. (2015), ref. [3]; p. 6 (idx 5) says it is based on the German models of IAEA (1997); Fig. 11 caption uses $5.5\times10^{25}\ \mathrm{n\ m^{-2}}$ | notes of H21-04, H21-06 and H21-09 now say the second terms are absent in the source; Collin et al. (2015) added to the source list (DOI resolved; paper not opened, so NOT VERIFIED as a data source) |
| Gas constant | Eq. (A-1) context, Collin Eq. (3), full-text search of 8.pdf | 8.3143 (IAEA 1997) and 8.3145 (Collin 2016; IAEA 2012) confirmed; Hales prints no value (0 hits for "8.314") | register RG-6 changed from NOT VERIFIED to "verified (absence)"; the value inside BISON stays NOT VERIFIED (GAP-08) |
| Units | column heads of Tables A-1 to A-5 and Table 2 | $D_0$ in $\mathrm{m^2\ s^{-1}}$, $Q$ in $\mathrm{kJ\ mol^{-1}}$ in all tables; text definitions say $\mathrm{J\ mol^{-1}}$; second $Q$ column of Tables A-1 to A-5 is misprinted "Q1" | none (already documented) |
| Cs-in-SiC fluence factor | IAEA (2012) Eq. (10.9) render and a recomputation | Eq. (10.9) prints $\Gamma = 2$ ($10^{25}$ n/m2, E > 16 fJ) and $D = 1.011\times10^{-16}\ \mathrm{m^2\ s^{-1}}$ at 1600 °C; recomputed $1.0107\times10^{-16}$ with $R_g = 8.3143$ and $1.0113\times10^{-16}$ with 8.3145 (relative difference about $3\times10^{-4}$ either way) | none; AGR-1 fluences use E > 0.18 MeV, so the factor cannot be applied without a cut-off conversion (GAP-04, NOT VERIFIED) |
| Transcription regression | new check family V in `D5_parameter_register.py`: 81 Ag/Cs/Sr rows re-entered from the renders | 81 of 81 rows identical in $D_0$, $Q$, $T_{\min}$, $T_{\max}$; mutation test (A3-16 $D_0$ altered in a scratch copy) gives FAIL | check family V added |

Table: 2.1 Diffusivity checks against IAEA (1997), Collin et al. (2016) and Hales et al. (2021).

## 3. Nuclear data conventions

| Item | Check | Result | Correction |
|---|---|---|---|
| Half-lives, uncertainties, units (16 nuclides) | fresh LiveChart ground-state and level downloads | all equal, including ENSDF last-digit uncertainties of the isomers Ag-110m, Cs-134m, Ba-137m, Kr-85m | none |
| Decay constants | $\lambda = \ln 2 / t_{1/2}$ against the API `half_life_sec` | all 14 radioactive rows within $10^{-6}$ relative; year convention of the API is 31 556 925.97 s (365.2422 d) | none |
| Decay modes, branchings, ENSDF cut-off dates | fresh API | all equal | none |
| JEFF-3.1.1 independent and cumulative thermal yields with uncertainties (U-235, Pu-239) | fresh API, matched on element, mass number and level index | all non-blank cells equal; blank U-235 Ag-110 and Ag-110m cells are still blank; independent-yield sums 2.00168 (U-235) and 1.99993 (Pu-239) | none |
| Yield library | API guide fetched 2026-11-06 | "Fission Yields (Cumulative or Independent) Source: JEFF 3.1.1" confirmed | none |
| Activation products Ag-110m, Cs-134, Eu-154 | `product_class` and `STRATO_use_note` | already classed as fission-shielded activation products, but the yield notes said only "FY negligible" or "FY not appropriate" | `decay.csv` product class now ends "NO fission-yield source in STRATO"; the 12 yield rows of Ag-110m, Ag-110, Cs-134, Cs-134m, Eu-154, Eu-154m now start "do NOT use as a fission-yield source" and name the capture route; Eq. (R.6a) in the register now states the restriction |
| Capture cross sections and isomeric ratios for the activation routes | search of the paper set and the data folder | not available | none; stays NOT VERIFIED (GAP-26) |

Table: 3.1 Nuclear data checks against a fresh LiveChart download.

## 4. AGR-1 provenance and applicability

| Item | Check | Result | Correction |
|---|---|---|---|
| E-001 to E-009 (Cs-134, compacts with failed SiC) | Collin et al. (2016) Table 5 render | 9 of 9 experimental and PARFUME values, target fractions and correction factors equal | none |
| E-099 to E-113 (failed-particle counts) | Collin et al. (2016) Table 3 render | 15 of 15 equal, including the asterisk on 4-3-2 and its footnote | evidence class "estimate" (PIE inference; partly inferred from DLBL, see notes) |
| E-010 to E-014 (Cs-134, intact, matrix-corrected) | location and values on the Fig. 6 render | figure is on p. 384, idx 6; the five measured and five PARFUME bars agree with the digitised values to the reading precision of the render | `source_location` changed from "page not recorded" to "6.pdf Fig. 6; printed p. 384; idx 6" (GAP-15 resolved) |
| E-033 to E-037 (Ag-110m, intact, matrix-corrected) | Fig. 3 render | bars agree with the digitised values to the reading precision of the render | none |
| E-069 to E-078 (Sr-90, failed SiC) | Fig. 12 render | values agree; the legend labels the measured bars "PIE"; label "4-4-2" is printed but appears in no table of either paper | notes record the legend; E-077 stays NOT VERIFIED with the label confirmed as printed (GAP-16) |
| Upper bounds E-027 to E-029, E-032, E-088, E-089 | text search of 5.pdf | "<5e-6 after 300 h at 1600 C or 100 h at 1800 C" (abstract), 6-2-1 Cs-134 below $5\times10^{-6}$ (p. 28), Kr-85 near or below detection, release fraction $<5\times10^{-6}$ (p. 29 and p. 32) confirmed | evidence class "measured bound" for every "<" value |
| One particle as compact fraction | text p. 29 (idx 5) and $1/4145$ | printed $2.4\times10^{-4}$; calculated $2.413\times10^{-4}$ | none |
| Printed vs digitised | `value_class` and `status` | 58 measured (printed table, text or vector figure), 74 measured digitised (R-dig or R-vis), 8 measured bounds, 16 estimates | new column `evidence_class` |
| Compact vs particle level | every AGR-1 row | all AGR-1 values are compact-level (about 4145 particles); Morris et al. (2016) state that Ag, Eu and Sr release was dominated by release during irradiation; JAERI rows are batches of about 100 loose particles; CRP-6 rows are sphere level | new column `particle_applicability`; register Section 9 explains which rows can constrain a single intact particle (matrix-corrected intact-compact values, and bounds as inequalities only) |
| Detection limits | both papers | numerical detection limits for Ag, Cs, Sr and Eu are not printed; Kr-85 is "near or below detection levels" at $<5\times10^{-6}$ | none; stays NOT VERIFIED (MU-10, GAP-18) |

Table: 4.1 AGR-1 data provenance checks against Morris et al. (2016) and Collin et al. (2016).

## 5. Register, generator and formatting

| Item | Check | Result | Correction |
|---|---|---|---|
| Register in sync with its generator | old template, old CSVs and old check output rendered again | byte-identical to the old register | none |
| Status vs evidence | Section 0.2 of the register | `status` described checking only; there was no uniform column for the kind of value | 13-term `evidence_class` vocabulary added to Section 0.2 and to every rendered table |
| Display math | lines holding a double-dollar delimiter | four display equations had both delimiters and the equation on one line, two of them without a tag | Eqs. (R.1), (R.2), (R.6a), (R.6b) now put each delimiter on its own line and carry a `\tag{}` |
| Unicode math symbols in prose | template and generator | plus-minus, approximately, less-or-equal, greater-or-equal, gamma, times, micro and infinity signs present | replaced by LaTeX in the template and in the generator (degree sign, en dashes in page ranges and the u-umlaut in an author name remain) |
| Inline math padding | regex over the rendered register | one case, the definition of $f_{\mathrm{rel}}(t)$ in Eq. (R.7), whose inline math ended in a space | rewritten |
| Code fences | template and `data/reference/README.md` | three fences without a language tag | tagged `text` or `bash` |
| Em dashes | all edited Markdown and Python files | 0 | none |

Table: 5.1 Register and formatting checks.

## 6. Items that remain NOT VERIFIED

UCO kernel coefficients (GAP-01); IPyC vs OPyC data (GAP-02); temperature ranges of the FRG and USA rows (GAP-03); fluence cut-off conversion between E > 16 fJ and E > 0.18 MeV (GAP-04); labels of A3-18 and A3-19 (GAP-05); grades of A4-03 to A4-05 (GAP-06); source of T71-05 (GAP-07); gas constant inside BISON (GAP-08); compact 4-4-2 (GAP-16); AGR-1 detection limits and error bars (GAP-18); activation cross sections and isomeric ratios (GAP-26); the content of Collin et al. (2015), which is cited by Hales et al. (2021) but is not in the paper set. The IAEA URLs for TECDOC-978 and TECDOC-1674 were not opened (GAP-29).

## 7. Reproduction

```bash
cd STRATO
.venv/Scripts/python.exe -I -X utf8 scripts/data_register/b6_audit_patch.py .
.venv/Scripts/python.exe -I -X utf8 scripts/derivation_checks/D5_parameter_register.py > <scratch dir>/d5_out.txt
.venv/Scripts/python.exe -I -X utf8 scripts/data_register/render_register.py . scripts/data_register/register_template.md data/reference/STRATO_Parameter_Register.md <scratch dir>/d5_out.txt
.venv/Scripts/python.exe -I -X utf8 scripts/data_register/b6_nuclear_crosscheck.py . <download dir outside the repository>
```

Results on 2026-11-06: patch idempotent (identical file hashes after a second run); `D5_parameter_register.py` 91 checks, all PASS (78 before the audit plus 13 in family V); `b6_nuclear_crosscheck.py` 204 of 204 PASS.
