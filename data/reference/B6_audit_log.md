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

## 8. Data integration: re-read of suspected errors (decision 45, 2026-11-25)

Task `btcs-data-integration`. Each row named by the research reports as suspect was re-read in its cited source by the integrating agent; a row is changed only where the source shows the error. Three documents were opened as files (Collin et al., 2016, accepted manuscript OSTI 1246116; Schappel et al., 2020, OSTI 1606816; INL/EXT-16-39548) and the bars of Figs. 11 and 12 were measured on the embedded bitmaps against the axis (32.2 px per decade in Fig. 11, 40.75 px per decade in Fig. 12). The Table 14 values of Stempien et al. (2023) quoted in the rows below are those already held in rows E-159, E-160 and E-171 of the release file; Table 14 itself was not re-read in this task. No value other than the range of C16-20 changed, and the fitted range 1600 to 1800 C of S20-01 to S20-03 was entered in T_min_C and T_max_C with the new column range_kind set to fitted (see the last rows of the table). Correction after verification: the reading uncertainty of every digitised Collin et al. (2016) row (39 rows) was then determined from the pixel pitch of the embedded bitmap of its figure (4.9 percent per pixel in Fig. 3 at 48.25 px per decade, 5.8 in Fig. 4 and Fig. 12 at 40.75, 7.4 in Fig. 6 and Fig. 11 at 32.2, 4.3 in Fig. 15 at 54.33) and recorded as rel_value_pct in the release file; this is a reading resolution of one pixel, not an experimental uncertainty. The rounding of the recorded value (two significant figures, half-width up to 5 percent) was then compared with the pixel resolution and the larger of the two is recorded; it raised E-091 from 4.3 to 4.6 percent, and the note of each of the 39 rows states the rounding. The 34 visually read rows of Morris et al. (2016) (E-015 to E-026, E-046 to E-059, E-080 to E-087; 32 of them had no reading uncertainty) were re-read in the last block of rows of this section: their figures are vector drawings, and the stored vertices of each curve give the value with a resolution of 0.06 pt (0.4 percent of value on the logarithmic axes), so the earlier visual values were replaced by the vector values and the class changed from R-vis to R-dig. No row keeps reading_uncertainty not recorded. Section 9 (task btcs-data-integration-minors) supersedes the values of E-054, E-055, E-082 and E-083 and the reading uncertainties stated in the 34 Morris entries below.

| Row | Document, page, table or figure | Old value | New value | Reason |
|---|---|---|---|---|
| E-067 (Sr-90, 6-4-3, 1600 C) | Collin et al. (2016) Fig. 11, printed p. 386, idx 8 | 8.4e-6 | confirmed | The 6-4-3 bar reads about 8e-6. INL/RPT-23-74651 Table 14 prints 1.10E-5 for the end of the same test; two sources, both recorded, no misreading. |
| E-069 (Sr-90, 3-3-2, 1600 C) | Collin et al. (2016) Fig. 12, printed p. 386, idx 8 | 3.3e-4 | confirmed | The 3-3-2 bar top is 20 px above the 1e-4 axis line, about 3.2e-4. It is not 1.33e-3: that value is the later tabulated total of INL/RPT-23-74651 Table 14 (and its Fig. 12), which is a different source and is held in E-157 to E-231. The cited source shows no error, so the row is not corrected; the note records the disagreement. |
| E-072 (Sr-90, 3-3-1, 1700 C) | Collin et al. (2016) Fig. 12, printed p. 386, idx 8 | 1.1e-3 | confirmed | The 3-3-1 bar reads about 1.1e-3 (1.06e-3 measured); Table 14 of the later report prints 1.33E-3. Within the reading precision of a log bar chart. |
| E-077 (Sr-90, label 4-4-2, 1800 C) | Collin et al. (2016) Fig. 12, printed p. 386, idx 8 | label 4-4-2, 5.3e-2, NOT VERIFIED | confirmed as printed, status kept | Fig. 12 prints the label 4-4-2 under the ninth bar (about 5.3e-2). INL/RPT-23-74651 Table 14 has no compact 4-4-2 among the safety tests (4-4-2 was the loose-particle compact), so the label may be a mislabel in the source; the source shows no other label, so none is substituted. |
| C16-20 (Kr in SiC, 3.7e1, 657) | Collin et al. (2016) Table 2 and note, printed p. 381; INL/EXT-16-39548 Table 13, printed p. 7; TECDOC-978 Table A-3 (row A3-17) | range empty, note "accident conditions" | T_min 1400, T_max 1750 C, note "printed range" | Collin Table 2 prints 3.7e1 and 657 without a range and says the Kr coefficients of the kernel and SiC layer are specific to accident conditions. The origin (Xe in SiC, Fukuda) is printed with 1400 to 1750 C in INL/EXT-16-39548 Table 13 and A3-17. The range belongs to the coefficient, so it is added. The 1800 C heating tests lie above it. |
| S20-04 (Ag in PyC, 3.78e-5, 200) | Schappel et al. (2020) section 4.6, manuscript PDF p. 20 (printed p. 19) | form "D fitted at one temperature, Q assumed"; note "Q = 200 assumed as plausible" | form "1T (hypothesis)"; note rewritten; numbers unchanged | The source takes D = 3.5e-13 m2/s at 1030 C from the German sources and requires about 1e-10 m2/s at 1600 C; Q = 200 kJ/mol follows from the two points (calculated 201) and is judged plausible against the 193 to 222 of TECDOC-978 Table A-2. TECDOC-978 row A2-03 gives 3.6e-15 m2/s at 1030 C (calculated), so the 3.5e-13 is not reproduced by that row. A hypothesis, not a fit. |
| S20-01, S20-02, S20-03 (Ag, Cs, Sr in SiC; 1.61e-10 and 215, 2.5e-12 and 180, 9.6e-12 and 180) | Schappel et al. (2020) section 3, printed p. 13-14 (heating tests at 1600 and 1800 C), Table 6, printed p. 26 | T_min_C and T_max_C empty | T_min_C 1600, T_max_C 1800, range_kind fitted | The source prints no temperature range with these coefficients. The coefficients were fitted to the 1600 and 1800 C post-irradiation heating tests, so those two temperatures bound what they were fitted to. The range is a fitted range and not a validity range printed by the source; the new column range_kind (printed or fitted) says so, and the typed refusal names it a fitted range (not printed by the source). D0 and Q are unchanged. This entry was missing from the first version of this section. |
| E-015 (Cs-134, 3-2-2, 1600 C) | Morris et al. (2016) Fig. 5, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 2.7e-7, class R-vis, reading uncertainty factor=1.5 (about a factor 1.5) | 2.79e-7, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.0 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 3.1 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-016 (Cs-134, 4-3-3, 1600 C) | Morris et al. (2016) Fig. 5, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 7e-7, class R-vis, reading uncertainty not recorded | 6.75e-7, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 300.1 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 3.6 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-017 (Cs-134, 5-3-3, 1600 C) | Morris et al. (2016) Fig. 5, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 4e-6, class R-vis, reading uncertainty not recorded | 3.80e-6, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.0 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 5.3 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-018 (Cs-134, 6-2-1, 1600 C) | Morris et al. (2016) Fig. 5, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 5e-6, class R-vis, reading uncertainty not recorded | 4.76e-6, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 301.8 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 5.1 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-019 (Cs-134, 6-4-3, 1600 C) | Morris et al. (2016) Fig. 5, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1e-6, class R-vis, reading uncertainty not recorded | 9.43e-7, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 399.0 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 6.0 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-020 (Cs-134, 4-4-3, 1700 C) | Morris et al. (2016) Fig. 11, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 8e-8, class R-vis, reading uncertainty not recorded | 8.08e-8, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 255.2 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 1.0 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-021 (Cs-134, 4-4-3, 1700 C) | Morris et al. (2016) Fig. 11, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 7.5e-7, class R-vis, reading uncertainty not recorded | 7.28e-7, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.0 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 3.1 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-022 (Cs-134, 3-3-1, 1700 C) | Morris et al. (2016) Fig. 11, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 3.6e-4, class R-vis, reading uncertainty not recorded | 3.59e-4, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 330.5 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 0.3 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-023 (Cs-134, 3-2-3, 1800 C) | Morris et al. (2016) Fig. 14, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1.6e-3, class R-vis, reading uncertainty rel_value_pct=10..15 (about 10-15 %) | 1.60e-3, class R-dig, reading uncertainty rel_value_pct=0.1 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.1 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 0.1 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-024 (Cs-134, 4-3-2, 1800 C) | Morris et al. (2016) Fig. 14, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1.2e-3, class R-vis, reading uncertainty not recorded | 1.21e-3, class R-dig, reading uncertainty rel_value_pct=0.1 | The curve of the compact was identified by its legend entry and its last vertex (t = 277.2 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 1.1 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-025 (Cs-134, 4-4-1, 1800 C) | Morris et al. (2016) Fig. 14, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 3.6e-4, class R-vis, reading uncertainty not recorded | 3.62e-4, class R-dig, reading uncertainty rel_value_pct=0.2 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.1 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 0.5 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-026 (Cs-134, 5-1-3, 1800 C) | Morris et al. (2016) Fig. 14, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1.17e-3, class R-vis, reading uncertainty not recorded | 1.17e-3, class R-dig, reading uncertainty rel_value_pct=0.1 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.3 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 0.2 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-046 (Ag-110m, 4-3-3, 1600 C) | Morris et al. (2016) Fig. 6, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 3.3e-1, class R-vis, reading uncertainty not recorded | 3.38e-1, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 300.1 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 2.5 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-047 (Ag-110m, 3-3-2, 1600 C) | Morris et al. (2016) Fig. 6, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1e-1, class R-vis, reading uncertainty not recorded | 1.00e-1, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 384.8 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 0.1 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-048 (Ag-110m, 3-2-2, 1600 C) | Morris et al. (2016) Fig. 6, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 6e-2, class R-vis, reading uncertainty not recorded | 6.33e-2, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 302.9 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 5.3 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-049 (Ag-110m, 4-1-2, 1600 C) | Morris et al. (2016) Fig. 6, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 6e-2, class R-vis, reading uncertainty not recorded | 5.80e-2, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.0 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 3.5 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-050 (Ag-110m, 6-4-1, 1600 C) | Morris et al. (2016) Fig. 6, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 3.3e-2, class R-vis, reading uncertainty not recorded | 3.24e-2, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.4 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 2.0 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-051 (Ag-110m, 5-3-3, 1600 C) | Morris et al. (2016) Fig. 6, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 2e-2, class R-vis, reading uncertainty not recorded | 2.29e-2, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.0 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 12.6 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-052 (Ag-110m, 6-4-3, 1600 C) | Morris et al. (2016) Fig. 6, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 2e-2, class R-vis, reading uncertainty not recorded | 1.92e-2, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 399.0 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 4.4 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-053 (Ag-110m, 6-2-1, 1600 C) | Morris et al. (2016) Fig. 6, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 3.5e-3, class R-vis, reading uncertainty not recorded | 3.51e-3, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 301.8 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 0.3 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-054 (Ag-110m, 3-3-1, 1700 C) | Morris et al. (2016) Fig. 12, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 5e-2, class R-vis, reading uncertainty not recorded | 5.10e-2, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 330.4 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 2.0 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-055 (Ag-110m, 4-4-3, 1700 C) | Morris et al. (2016) Fig. 12, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1.7e-1, class R-vis, reading uncertainty not recorded | 1.65e-1, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.0 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 3.1 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-056 (Ag-110m, 4-4-1, 1800 C) | Morris et al. (2016) Fig. 18, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 2.3e-1, class R-vis, reading uncertainty not recorded | 2.34e-1, class R-dig, reading uncertainty rel_value_pct=0.1 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.2 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 1.8 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-057 (Ag-110m, 4-3-2, 1800 C) | Morris et al. (2016) Fig. 18, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1.4e-1, class R-vis, reading uncertainty not recorded | 1.40e-1, class R-dig, reading uncertainty rel_value_pct=0.1 | The curve of the compact was identified by its legend entry and its last vertex (t = 277.3 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 0.1 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-058 (Ag-110m, 3-2-3, 1800 C) | Morris et al. (2016) Fig. 18, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 9.8e-2, class R-vis, reading uncertainty not recorded | 9.93e-2, class R-dig, reading uncertainty rel_value_pct=0.1 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.2 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 1.3 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-059 (Ag-110m, 5-1-3, 1800 C) | Morris et al. (2016) Fig. 18, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 4.3e-2, class R-vis, reading uncertainty not recorded | 4.34e-2, class R-dig, reading uncertainty rel_value_pct=0.2 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.4 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 1.0 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-080 (Eu-154, 4-1-2, 1600 C) | Morris et al. (2016) Fig. 7, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 3e-4, class R-vis, reading uncertainty not recorded | 3.38e-4, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.1 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 11.2 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-081 (Eu-154, 6-2-1, 1600 C) | Morris et al. (2016) Fig. 7, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 3e-3, class R-vis, reading uncertainty not recorded | 2.88e-3, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 301.9 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 4.1 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-082 (Eu-154, 3-3-1, 1700 C) | Morris et al. (2016) Fig. 12, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 2.3e-3, class R-vis, reading uncertainty not recorded | 2.33e-3, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 330.4 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 1.3 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-083 (Eu-154, 4-4-3, 1700 C) | Morris et al. (2016) Fig. 12, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 3e-3, class R-vis, reading uncertainty not recorded | 2.90e-3, class R-dig, reading uncertainty rel_value_pct=0.4 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.0 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 3.3 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-084 (Eu-154, 4-3-2, 1800 C) | Morris et al. (2016) Fig. 16, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1.8e-2, class R-vis, reading uncertainty not recorded | 1.78e-2, class R-dig, reading uncertainty rel_value_pct=0.1 | The curve of the compact was identified by its legend entry and its last vertex (t = 277.2 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 0.9 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-085 (Eu-154, 4-4-1, 1800 C) | Morris et al. (2016) Fig. 16, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1.5e-2, class R-vis, reading uncertainty not recorded | 1.50e-2, class R-dig, reading uncertainty rel_value_pct=0.1 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.2 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 0.3 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-086 (Eu-154, 5-1-3, 1800 C) | Morris et al. (2016) Fig. 16, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1.4e-2, class R-vis, reading uncertainty not recorded | 1.43e-2, class R-dig, reading uncertainty rel_value_pct=0.1 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.4 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 2.0 % below it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |
| E-087 (Eu-154, 3-2-3, 1800 C) | Morris et al. (2016) Fig. 16, accepted manuscript OSTI 1325505 (vector drawing; printed p. 29 to 33) | 1.4e-2, class R-vis, reading uncertainty not recorded | 1.36e-2, class R-dig, reading uncertainty rel_value_pct=0.1 | The curve of the compact was identified by its legend entry and its last vertex (t = 303.2 h on the figure axis) read in the vector path with the axes calibrated on the tick labels and gridlines; the earlier visual reading was 3.1 % above it. Value replaced by the vector reading (three significant figures); the uncertainty is the position resolution of the vector (0.06 pt), not an experimental uncertainty. |

## 9. Data integration, minor corrections: Morris et al. (2016) re-read and reading uncertainty (decision 50, 2026-11-26)

Task `btcs-data-integration-minors`. The independent verification of the data integration found four Fig. 12 values 0.5 to 0.6 percent below the curve vertex and a reading uncertainty that covered the drawing resolution only. All 34 digitised rows of Morris et al. (2016) were therefore read again from the source, and every figure that carries them is recorded here so that the reading can be repeated without the scratch tool that was used (it is not committed).

### 9.1 Source and how to repeat the reading

- Source identifier: Morris et al. (2016), accepted manuscript, OSTI 1325505 (downloaded from `https://www.osti.gov/servlets/purl/1325505`, 26 pages, 508 437 bytes).
- SHA-256 of the file read: `D35F95DF366D5AC56BA19008FF71B0663EF5B5D3A676AE04D68525C6F2CB3059`.
- The figures are vector drawings. For the page named in the table of 9.3 (page number in the file, counted from 1 in page-tree order), decompress the Flate stream of the `/Contents` entry of the page object. In it, `w` sets the line width, `RG` and `G` the stroke colour, `m` and `l` build a polyline and `S` strokes it; coordinates are in points (1/72 inch) from the lower-left corner of the page. The coordinates of the curves and gridlines have a granularity of 0.12 pt, so each is known to +/-0.06 pt.
- Horizontal gridlines: 0.24 pt wide, grey (0.498), together with the bottom axis line. Their coordinates and the labels of the y axis (read from the text of the page, top to bottom) are in 9.3. The time axis has nine equally spaced major ticks, labelled 0 to 400 h in steps of 50 h; the first and last tick are the x coordinates in 9.3, and t = 400 h * (x - x(0 h)) / (x(400 h) - x(0 h)).
- Curves: polylines 1.44 to 2.16 pt wide with 10 or more vertices. A curve is identified by the colour of the 2-point legend line (19 to 25 pt long, left of the legend text) of the compact. Where the colour is not unique (Fig. 12: Ag-110m and Sr-90 of compact 3-3-1 share one colour, and so do Eu-154 and Sr-90 of compact 4-4-3) the legend marker settles it: square for Ag-110m, diamond for Eu-154, triangle for Sr-90. The legend text rows (`AGR-1-nnn`, in Fig. 12 with the nuclide) are in the same order as the legend lines.
- The value is the vertex of the curve at the stated time (the last vertex of the curve where the row states no time, as the vertex time in the table of 9.5 shows). With y the vertex coordinate and y_hi, y_lo the coordinates of the two gridlines that enclose it, with labelled values v_hi and v_lo: logarithmic axis, value = 10^(log10 v_lo + (y - y_lo) / (y_hi - y_lo) * (log10 v_hi - log10 v_lo)); linear axis, value = v_lo + (y - y_lo) / (y_hi - y_lo) * (v_hi - v_lo). Local gridline pairs are used so that uneven rounding of the gridline coordinates (adjacent decades of Fig. 6 differ by 0.12 pt) does not accumulate over several decades.

### 9.2 Reading uncertainty

The reading uncertainty u is a half-width in percent of the value, computed as a worst case (all errors added, none combined in quadrature):

1. the vertex coordinate, +/-0.06 pt;
2. the two gridlines that enclose the vertex, each +/-0.06 pt. With the weights of the linear interpolation, (1 - f) and f, their contribution plus that of the vertex is at most 0.12 pt along the gridline spacing D of the bracket (D is the decade length on a logarithmic axis). This is the calibration quantisation; the earlier statement used the vertex term alone;
3. the half-unit of the last recorded digit of the three-figure value (for example 0.005e-2 of 5.13e-2).

Terms 1 and 2 give u_read = ln(10) * 0.12 / D on a logarithmic axis (0.73 percent for D = 37.68 pt) and u_read = (v_hi - v_lo) / D * 0.12 / v on a linear axis (0.04 to 0.4 percent here). The recorded `rel_value_pct` is u_read plus the half-unit of term 3, rounded up to 0.1. It is a reading resolution of the drawing, not an experimental uncertainty. The time of a vertex is read from its x coordinate (+/-0.06 pt, 0.07 h); because the stated time is the time of the vertex itself, this does not add to the value. Each recorded value was compared with the vertex value: every recorded value lies within its recorded uncertainty of the vertex value (column deviation of 9.5).

### 9.3 Axis calibration of the eight figures

Coordinates in pt (granularity 0.12 pt, so each is known to +/-0.06 pt). The nine major ticks of the time axis are equally spaced within the granularity in every figure.

| Figure | Page in the file | Axis | Horizontal gridlines, y coordinate (labelled value), top to bottom | Time axis: x of 0 h and of 400 h |
|---|---|---|---|---|
| Fig. 5 | 10 | logarithmic | 741.48 (1E+0), 703.44 (1E-1), 665.40 (1E-2), 627.36 (1E-3), 589.32 (1E-4), 551.16 (1E-5), 513.12 (1E-6), 475.08 (1E-7), 437.04 (1E-8) | 146.28 and 491.76 |
| Fig. 6 | 11 | logarithmic | 741.48 (1E+0), 704.16 (1E-1), 666.72 (1E-2), 629.28 (1E-3), 591.84 (1E-4), 554.40 (1E-5), 516.96 (1E-6), 479.52 (1E-7), 442.08 (1E-8) | 140.28 and 488.76 |
| Fig. 7 | 12 | logarithmic | 660.96 (1E+0), 623.04 (1E-1), 585.00 (1E-2), 546.96 (1E-3), 508.92 (1E-4), 471.00 (1E-5), 432.96 (1E-6), 394.92 (1E-7), 356.88 (1E-8) | 143.64 and 487.56 |
| Fig. 11 | 16 | logarithmic | 741.48 (1E+0), 703.44 (1E-1), 665.40 (1E-2), 627.24 (1E-3), 589.20 (1E-4), 551.04 (1E-5), 513.00 (1E-6), 474.84 (1E-7), 436.80 (1E-8) | 144.24 and 486.60 |
| Fig. 12 | 17 | logarithmic | 741.48 (1E+0), 703.80 (1E-1), 666.12 (1E-2), 628.44 (1E-3), 590.76 (1E-4), 553.08 (1E-5), 515.40 (1E-6), 477.72 (1E-7), 440.04 (1E-8) | 146.28 and 483.60 |
| Fig. 14 | 19 | linear | 741.48 (2.0E-3), 666.24 (1.5E-3), 591.00 (1.0E-3), 515.64 (5.0E-4), 440.40 (0.0E+0) | 153.60 and 486.60 |
| Fig. 16 | 21 | linear | 741.48 (2.0E-2), 666.36 (1.5E-2), 591.12 (1.0E-2), 516.00 (5.0E-3), 440.76 (0.0E+0) | 160.20 and 486.60 |
| Fig. 18 | 23 | linear | 741.48 (4.0E-1), 666.12 (3.0E-1), 590.64 (2.0E-1), 515.16 (1.0E-1), 439.80 (0.0E+0) | 156.24 and 487.56 |

### 9.4 Rows whose value is corrected (decision 45 procedure)

| Row | Document, page, figure | Old value | New value | Reason |
|---|---|---|---|---|
| E-054 (Ag-110m, 3-3-1, 1700 C) | Morris et al. (2016) Fig. 12, accepted manuscript OSTI 1325505, page 17 of the file (SHA-256 above) | 5.10e-2, class R-dig, rel_value_pct=0.4 | 5.13e-2, class R-dig, rel_value_pct=0.9 | The vertex of the curve at t = 330.4 h lies at y = 692.88 pt (legend line colour (RGB) 0 0.439 0.753, marker square), between the gridlines 703.80 pt (0.1) and 666.12 pt (0.01); the value there is 0.05131. The value of the first vector reading, 5.10e-2, is 0.6 % below it, outside the reading uncertainty recorded for it (0.4 %). Corrected to the vertex value, three significant figures. The new reading uncertainty is that of 9.2. |
| E-055 (Ag-110m, 4-4-3, 1700 C) | Morris et al. (2016) Fig. 12, accepted manuscript OSTI 1325505, page 17 of the file (SHA-256 above) | 1.65e-1, class R-dig, rel_value_pct=0.4 | 1.66e-1, class R-dig, rel_value_pct=1.1 | The vertex of the curve at t = 303.0 h lies at y = 712.08 pt (legend line colour (RGB) 0.855 0.506 0.216, marker square), between the gridlines 741.48 pt (1) and 703.80 pt (0.1); the value there is 0.1659. The value of the first vector reading, 1.65e-1, is 0.5 % below it, outside the reading uncertainty recorded for it (0.4 %). Corrected to the vertex value, three significant figures. The new reading uncertainty is that of 9.2. |
| E-082 (Eu-154, 3-3-1, 1700 C) | Morris et al. (2016) Fig. 12, accepted manuscript OSTI 1325505, page 17 of the file (SHA-256 above) | 2.33e-3, class R-dig, rel_value_pct=0.4 | 2.34e-3, class R-dig, rel_value_pct=1.0 | The vertex of the curve at t = 330.4 h lies at y = 642.36 pt (legend line colour (RGB) 0.255 0.435 0.651, marker diamond), between the gridlines 666.12 pt (0.01) and 628.44 pt (0.001); the value there is 0.002341. The value of the first vector reading, 2.33e-3, is 0.5 % below it, outside the reading uncertainty recorded for it (0.4 %). Corrected to the vertex value, three significant figures. The new reading uncertainty is that of 9.2. |
| E-083 (Eu-154, 4-4-3, 1700 C) | Morris et al. (2016) Fig. 12, accepted manuscript OSTI 1325505, page 17 of the file (SHA-256 above) | 2.90e-3, class R-dig, rel_value_pct=0.4 | 2.92e-3, class R-dig, rel_value_pct=1.0 | The vertex of the curve at t = 303.0 h lies at y = 645.96 pt (legend line colour (RGB) 0.796 0.471 0.2, marker diamond), between the gridlines 666.12 pt (0.01) and 628.44 pt (0.001); the value there is 0.002917. The value of the first vector reading, 2.90e-3, is 0.6 % below it, outside the reading uncertainty recorded for it (0.4 %). Corrected to the vertex value, three significant figures. The new reading uncertainty is that of 9.2. |

The other 30 rows keep their value: each recorded value lies within its new reading uncertainty of the vertex value (largest deviation 0.37 percent, E-046). Their only change is the reading uncertainty (9.5).

### 9.5 All 34 rows: vertex, deviation and reading uncertainty

The vertex time and y coordinate are those of the vertex read; `bracket` gives the enclosing gridline coordinates. Deviation = (recorded value - vertex value) / vertex value. `u_read` is term 1 plus 2 and `half-unit` term 3 of 9.2; u is their sum rounded up to 0.1 and is what `reading_uncertainty` records as `rel_value_pct`.

| Row | Figure | Curve (legend line colour RGB) | Vertex t (h) | Vertex y (pt) | Bracket y_hi / y_lo (pt) | Vertex value | Recorded value | Deviation (%) | u_read (%) | half-unit (%) | Old rel_value_pct | New rel_value_pct |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| E-015 | Fig. 5 | 0.29 0.271 0.165 | 303.0 | 492.00 | 513.12 / 475.08 | 2.785e-07 | 2.79e-7 | +0.19 | 0.73 | 0.18 | 0.4 | 1.0 |
| E-016 | Fig. 5 | 0.275 0.667 0.773 | 300.1 | 506.64 | 513.12 / 475.08 | 6.755e-07 | 6.75e-7 | -0.08 | 0.73 | 0.07 | 0.4 | 0.9 |
| E-017 | Fig. 5 | 0.573 0.816 0.314 | 303.0 | 535.20 | 551.16 / 513.12 | 3.806e-06 | 3.80e-6 | -0.15 | 0.73 | 0.13 | 0.4 | 0.9 |
| E-018 | Fig. 5 | 0 0.69 0.314 | 301.8 | 538.92 | 551.16 / 513.12 | 4.767e-06 | 4.76e-6 | -0.14 | 0.73 | 0.11 | 0.4 | 0.9 |
| E-019 | Fig. 5 | 0.439 0.188 0.627 | 399.0 | 512.16 | 513.12 / 475.08 | 9.435e-07 | 9.43e-7 | -0.06 | 0.73 | 0.05 | 0.4 | 0.8 |
| E-020 | Fig. 11 | 0.855 0.506 0.216 | 255.2 | 471.36 | 474.84 / 436.80 | 8.101e-08 | 8.08e-8 | -0.25 | 0.73 | 0.06 | 0.4 | 0.8 |
| E-021 | Fig. 11 | 0.855 0.506 0.216 | 303.0 | 507.72 | 513.00 / 474.84 | 7.272e-07 | 7.28e-7 | +0.11 | 0.72 | 0.07 | 0.4 | 0.8 |
| E-022 | Fig. 11 | 0 0.439 0.753 | 330.5 | 610.32 | 627.24 / 589.20 | 0.0003591 | 3.59e-4 | -0.03 | 0.73 | 0.14 | 0.4 | 0.9 |
| E-023 | Fig. 14 | 0.439 0.188 0.627 | 303.1 | 681.00 | 741.48 / 666.24 | 0.001598 | 1.60e-3 | +0.12 | 0.05 | 0.31 | 0.1 | 0.4 |
| E-024 | Fig. 14 | 0.965 0.573 0.251 | 277.2 | 623.04 | 666.24 / 591.00 | 0.001213 | 1.21e-3 | -0.24 | 0.07 | 0.41 | 0.1 | 0.5 |
| E-025 | Fig. 14 | 0 0.69 0.941 | 303.1 | 494.88 | 515.64 / 440.40 | 0.000362 | 3.62e-4 | -0.01 | 0.22 | 0.14 | 0.2 | 0.4 |
| E-026 | Fig. 14 | 0.122 0.286 0.49 | 303.3 | 616.92 | 666.24 / 591.00 | 0.001172 | 1.17e-3 | -0.19 | 0.07 | 0.43 | 0.1 | 0.5 |
| E-046 | Fig. 6 | 0.275 0.667 0.773 | 300.1 | 723.96 | 741.48 / 704.16 | 0.3393 | 3.38e-1 | -0.37 | 0.74 | 0.15 | 0.4 | 0.9 |
| E-047 | Fig. 6 | 1 0 0 | 384.8 | 704.16 | 741.48 / 704.16 | 0.1 | 1.00e-1 | +0.00 | 0.74 | 0.50 | 0.4 | 1.3 |
| E-048 | Fig. 6 | 0.29 0.271 0.165 | 302.9 | 696.72 | 704.16 / 666.72 | 0.06328 | 6.33e-2 | +0.03 | 0.74 | 0.08 | 0.4 | 0.9 |
| E-049 | Fig. 6 | 0.753 0 0 | 303.0 | 695.28 | 704.16 / 666.72 | 0.05792 | 5.80e-2 | +0.14 | 0.74 | 0.09 | 0.4 | 0.9 |
| E-050 | Fig. 6 | 0.192 0.522 0.612 | 303.4 | 685.80 | 704.16 / 666.72 | 0.03233 | 3.24e-2 | +0.21 | 0.74 | 0.15 | 0.4 | 0.9 |
| E-051 | Fig. 6 | 0.573 0.816 0.314 | 303.0 | 680.16 | 704.16 / 666.72 | 0.02285 | 2.29e-2 | +0.20 | 0.74 | 0.22 | 0.4 | 1.0 |
| E-052 | Fig. 6 | 0.439 0.188 0.627 | 399.0 | 677.28 | 704.16 / 666.72 | 0.01914 | 1.92e-2 | +0.29 | 0.74 | 0.26 | 0.4 | 1.0 |
| E-053 | Fig. 6 | 0 0.69 0.314 | 301.8 | 649.68 | 666.72 / 629.28 | 0.003506 | 3.51e-3 | +0.10 | 0.74 | 0.14 | 0.4 | 0.9 |
| E-054 | Fig. 12 | 0 0.439 0.753, marker square | 330.4 | 692.88 | 703.80 / 666.12 | 0.05131 | 5.13e-2 | -0.02 | 0.73 | 0.10 | 0.4 | 0.9 |
| E-055 | Fig. 12 | 0.855 0.506 0.216, marker square | 303.0 | 712.08 | 741.48 / 703.80 | 0.1659 | 1.66e-1 | +0.08 | 0.73 | 0.30 | 0.4 | 1.1 |
| E-056 | Fig. 18 | 0 0.69 0.941 | 303.2 | 616.56 | 666.12 / 590.64 | 0.2343 | 2.34e-1 | -0.15 | 0.07 | 0.21 | 0.1 | 0.3 |
| E-057 | Fig. 18 | 0.965 0.573 0.251 | 277.3 | 545.28 | 590.64 / 515.16 | 0.1399 | 1.40e-1 | +0.07 | 0.11 | 0.36 | 0.1 | 0.5 |
| E-058 | Fig. 18 | 0.439 0.188 0.627 | 303.2 | 514.68 | 515.16 / 439.80 | 0.09936 | 9.93e-2 | -0.06 | 0.16 | 0.05 | 0.1 | 0.3 |
| E-059 | Fig. 18 | 0.122 0.286 0.49 | 303.4 | 472.56 | 515.16 / 439.80 | 0.04347 | 4.34e-2 | -0.16 | 0.37 | 0.12 | 0.2 | 0.5 |
| E-080 | Fig. 7 | 0.753 0 0 | 303.1 | 529.08 | 546.96 / 508.92 | 0.0003388 | 3.38e-4 | -0.24 | 0.73 | 0.15 | 0.4 | 0.9 |
| E-081 | Fig. 7 | 0 0.69 0.314 | 301.9 | 564.48 | 585.00 / 546.96 | 0.002888 | 2.88e-3 | -0.27 | 0.73 | 0.17 | 0.4 | 0.9 |
| E-082 | Fig. 12 | 0.255 0.435 0.651, marker diamond | 330.4 | 642.36 | 666.12 / 628.44 | 0.002341 | 2.34e-3 | -0.05 | 0.73 | 0.21 | 0.4 | 1.0 |
| E-083 | Fig. 12 | 0.796 0.471 0.2, marker diamond | 303.0 | 645.96 | 666.12 / 628.44 | 0.002917 | 2.92e-3 | +0.10 | 0.73 | 0.17 | 0.4 | 1.0 |
| E-084 | Fig. 16 | 0.965 0.573 0.251 | 277.2 | 709.08 | 741.48 / 666.36 | 0.01784 | 1.78e-2 | -0.24 | 0.04 | 0.28 | 0.1 | 0.4 |
| E-085 | Fig. 16 | 0 0.69 0.941 | 303.2 | 667.08 | 741.48 / 666.36 | 0.01505 | 1.50e-2 | -0.32 | 0.05 | 0.33 | 0.1 | 0.4 |
| E-086 | Fig. 16 | 0.122 0.286 0.49 | 303.4 | 655.68 | 666.36 / 591.12 | 0.01429 | 1.43e-2 | +0.07 | 0.06 | 0.35 | 0.1 | 0.5 |
| E-087 | Fig. 16 | 0.439 0.188 0.627 | 303.2 | 645.00 | 666.36 / 591.12 | 0.01358 | 1.36e-2 | +0.14 | 0.06 | 0.37 | 0.1 | 0.5 |

### 9.6 Corrections to section 8

- The entries of E-015 and E-023 in section 8 gave the old reading uncertainty as not recorded. The file at the commit before the Morris re-read held `factor=1.5` (E-015, note "about a factor 1.5") and `rel_value_pct=10..15` (E-023, note "about 10-15 %"). The entries are corrected, and the superseded visual-reading statements in the notes of the two rows are marked as superseded.
- The reading uncertainties and, for E-054, E-055, E-082 and E-083, the values stated in the 34 Morris entries of section 8 are superseded by 9.4 and 9.5. The other columns of those entries stand.

## 10. Comparison denominators: matrix-corrected rows (decision 45 procedure, task data-comparison-denominators, 2026-11-26)

Source: Collin et al. (2016), accepted manuscript INL/JOU-15-34147 (OSTI 1246116), read from the PDF (manuscript pp. 3, 5, 6, 8 and the page images of Figs. 3, 4 and 6 on manuscript pp. 16 to 18); the journal's printed pages 382 to 384 cited by the register could not be checked in this file. Supporting: Demkowicz et al. (2015), INL/EXT-15-36407, printed p. 58 ("The released fraction is the cumulative inventory on the cups/plates divided by the predicted compact inventory at the end of the irradiation"); Stempien et al. (2023), INL/RPT-23-74651, printed p. 10 (the same normalisation, Mx/Ccompact).

What changed in the eighteen rows below: the column `denominator` (old value "not established", new value "calculated compact inventory"); the column `notes` (one sentence appended: denominator confirmed and what the value counts); and, for the ten intact rows, the column `particle_applicability` (the sentence that the compact fraction "equals the mean particle release fraction" is replaced by a statement that it equals the mean intact-particle release after about 24 h only if no release from outside the SiC occurs after about 24 h, an assumption of the source). Changed numerical values: 0. Class, qualifier and reading uncertainty: unchanged.

Reason for the denominator: the source normalises the released amount to a predicted (as-run, calculated) compact source term (manuscript p. 3). The p. 5 definition "relative to the total inventory produced during irradiation" belongs to the model (PARFUME) release fraction, not to the experimental one (corrected in 10.1); the end-of-irradiation date is stated by Demkowicz et al. (2015) p. 58 and Stempien et al. (2023) p. 10, not by Collin et al. The registered matrix-corrected values are below the Table 14 totals of the same compacts in all ten intact cases, and the implied prompt release (Table 14 total minus the corrected value) is 1.9e-7 to 3.2e-6 for Cs-134 (Collin p. 8 states 2e-7 to 3e-6) and 2.9e-3 (compact 6-2-1) to 0.338 (compact 4-3-3) for Ag-110m (Collin p. 6 states 2.9e-3 to 3.4e-1).

Reason for the notes and the applicability sentence: Collin p. 6 calls the correction "an attempt" and states that release from the particles cannot be distinguished from silver "slowly driven out of the matrix" later. The value is therefore at most the release through intact coatings in the window plus any later matrix release; it is not a point measurement.

| Row | Nuclide, compact, T (C) | Source location | Denominator old | Denominator new | Other cells |
|---|---|---|---|---|---|
| E-010 | Cs-134, 3-2-2, 1600 | Collin et al. (2016) Fig. 6, manuscript p. 18; p. 3 (denominator), p. 6 (correction) | not established | calculated compact inventory | notes appended; applicability corrected |
| E-011 | Cs-134, 4-3-3, 1600 | Collin et al. (2016) Fig. 6, manuscript p. 18; p. 3, p. 6 | not established | calculated compact inventory | notes appended; applicability corrected |
| E-012 | Cs-134, 5-3-3, 1600 | Collin et al. (2016) Fig. 6, manuscript p. 18; p. 3, p. 6 | not established | calculated compact inventory | notes appended; applicability corrected |
| E-013 | Cs-134, 6-2-1, 1600 | Collin et al. (2016) Fig. 6, manuscript p. 18; p. 3, p. 6 | not established | calculated compact inventory | notes appended; applicability corrected |
| E-014 | Cs-134, 6-4-3, 1600 | Collin et al. (2016) Fig. 6, manuscript p. 18; p. 3, p. 6 | not established | calculated compact inventory | notes appended; applicability corrected |
| E-033 | Ag-110m, 4-3-3, 1600 | Collin et al. (2016) Fig. 3, manuscript p. 16; p. 3, p. 6 | not established | calculated compact inventory | notes appended; applicability corrected |
| E-034 | Ag-110m, 5-3-3, 1600 | Collin et al. (2016) Fig. 3, manuscript p. 16; p. 3, p. 6 | not established | calculated compact inventory | notes appended; applicability corrected |
| E-035 | Ag-110m, 6-2-1, 1600 | Collin et al. (2016) Fig. 3, manuscript p. 16; p. 3, p. 6 | not established | calculated compact inventory | notes appended; applicability corrected |
| E-036 | Ag-110m, 6-4-3, 1600 | Collin et al. (2016) Fig. 3, manuscript p. 16; p. 3, p. 6 | not established | calculated compact inventory | notes appended; applicability corrected |
| E-037 | Ag-110m, 4-4-3, 1700 | Collin et al. (2016) Fig. 3, manuscript p. 16; p. 3, p. 6 | not established | calculated compact inventory | notes appended; applicability corrected |
| E-038 | Ag-110m, 4-1-2, 1600, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | not established | calculated compact inventory | notes appended (failed SiC, not comparable) |
| E-039 | Ag-110m, 6-4-1, 1600, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | not established | calculated compact inventory | notes appended (failed SiC, not comparable) |
| E-040 | Ag-110m, 3-3-1, 1700, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | not established | calculated compact inventory | notes appended (failed SiC, not comparable) |
| E-041 | Ag-110m, 5-1-1, 1700, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | not established | calculated compact inventory | notes appended (failed SiC, not comparable) |
| E-042 | Ag-110m, 3-2-3, 1800, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | not established | calculated compact inventory | notes appended (failed SiC, not comparable) |
| E-043 | Ag-110m, 4-3-2, 1800, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | not established | calculated compact inventory | notes appended (failed SiC, not comparable) |
| E-044 | Ag-110m, 4-4-1, 1800, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | not established | calculated compact inventory | notes appended (failed SiC, not comparable) |
| E-045 | Ag-110m, 5-1-3, 1800, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | not established | calculated compact inventory | notes appended (failed SiC, not comparable) |

Two register prose passages (R.7 and the applicability sentence on matrix-corrected values) were extended in the register document and in its template with the same finding. The data check `DI.release.denominator` now requires the eighteen rows to be compact-inventory rows and expects 43 rows (was 61) that say "not established" (JAERI E-117..E-149, CRP-6 E-150..E-156 and the remaining text rows); a failure test was added. Not changed: E-117..E-149 (denominator, decay correction and test sequence are stated neither on printed p. 164 of IAEA-TECDOC-978 nor on the pages around it).

## 10.1 Corrections to section 10 after independent verification (decision 45 procedure, task data-comparison-denominators-fix, 2026-11-26)

Three findings of the verification are applied; no measured value, class, qualifier or reading uncertainty changed, and the column denominator stays as set in section 10.

1. Window end (major). Collin et al. (2016), accepted manuscript p. 4: the experimental release fractions "were determined at these times", that is 334 h of test time at 1600 C and 336 h at 1700 C (end of the 300 h plateaus). Section 10 and the notes said the value runs "to the end of the test". Compact 6-4-3 ran about 400 h (TH-032) and compact 4-4-3 about 303 h (TH-036), so the window ends at the plateau for E-014, E-036 and E-037. The register holds no as-run duration for compacts 3-2-2, 4-3-3, 5-3-3 and 6-2-1; whether they ran past 300 h is not established here. New wording: from about 24 h at temperature to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter.

2. Citations (minor). Collin p. 5 defines the PARFUME (model) release fraction; it is no longer cited for the experimental denominator, which is Collin p. 3. The end-of-irradiation date is cited to Demkowicz et al. (2015) p. 58 and Stempien et al. (2023) p. 10, which state it; Collin et al. do not.

3. Class (minor). Science design Section 1.1 defines a bound as a total release fraction that is not matrix-corrected, so the ten intact matrix-corrected rows are neither point nor bound comparisons and are context until the coordinator extends the bound definition (Sections 1.1 and 2.1). "Window upper bound" is kept only as a statement under premise BP1, and the notes now say so. The class column is not changed.

| Row | Nuclide, compact, T (C) | Source location | Cell | Old value (changed part) | New value (changed part) | Reason |
|---|---|---|---|---|---|---|
| E-010 | Cs-134, 3-2-2, 1600 | Collin et al. (2016) Fig. 6, manuscript p. 18; p. 3, p. 4, p. 6 | notes | "accepted manuscript pp. 3, 5 and 6"; "to the end of the test"; "a window upper bound ... not a point comparison" | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10; "to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter"; class under 1.1: context (premise BP1) | window end (p. 4), citations, class |
| E-011 | Cs-134, 4-3-3, 1600 | Collin et al. (2016) Fig. 6, manuscript p. 18; p. 3, p. 4, p. 6 | notes | "accepted manuscript pp. 3, 5 and 6"; "to the end of the test"; "a window upper bound ... not a point comparison" | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10; "to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter"; class under 1.1: context (premise BP1) | window end (p. 4), citations, class |
| E-012 | Cs-134, 5-3-3, 1600 | Collin et al. (2016) Fig. 6, manuscript p. 18; p. 3, p. 4, p. 6 | notes | "accepted manuscript pp. 3, 5 and 6"; "to the end of the test"; "a window upper bound ... not a point comparison" | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10; "to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter"; class under 1.1: context (premise BP1) | window end (p. 4), citations, class |
| E-013 | Cs-134, 6-2-1, 1600 | Collin et al. (2016) Fig. 6, manuscript p. 18; p. 3, p. 4, p. 6 | notes | "accepted manuscript pp. 3, 5 and 6"; "to the end of the test"; "a window upper bound ... not a point comparison" | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10; "to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter"; class under 1.1: context (premise BP1) | window end (p. 4), citations, class |
| E-014 | Cs-134, 6-4-3, 1600 | Collin et al. (2016) Fig. 6, manuscript p. 18; p. 3, p. 4, p. 6 | notes | "accepted manuscript pp. 3, 5 and 6"; "to the end of the test"; "a window upper bound ... not a point comparison" | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10; "to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter"; class under 1.1: context (premise BP1) | window end (p. 4), citations, class |
| E-033 | Ag-110m, 4-3-3, 1600 | Collin et al. (2016) Fig. 3, manuscript p. 16; p. 3, p. 4, p. 6 | notes | "accepted manuscript pp. 3, 5 and 6"; "to the end of the test"; "a window upper bound ... not a point comparison" | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10; "to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter"; class under 1.1: context (premise BP1) | window end (p. 4), citations, class |
| E-034 | Ag-110m, 5-3-3, 1600 | Collin et al. (2016) Fig. 3, manuscript p. 16; p. 3, p. 4, p. 6 | notes | "accepted manuscript pp. 3, 5 and 6"; "to the end of the test"; "a window upper bound ... not a point comparison" | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10; "to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter"; class under 1.1: context (premise BP1) | window end (p. 4), citations, class |
| E-035 | Ag-110m, 6-2-1, 1600 | Collin et al. (2016) Fig. 3, manuscript p. 16; p. 3, p. 4, p. 6 | notes | "accepted manuscript pp. 3, 5 and 6"; "to the end of the test"; "a window upper bound ... not a point comparison" | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10; "to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter"; class under 1.1: context (premise BP1) | window end (p. 4), citations, class |
| E-036 | Ag-110m, 6-4-3, 1600 | Collin et al. (2016) Fig. 3, manuscript p. 16; p. 3, p. 4, p. 6 | notes | "accepted manuscript pp. 3, 5 and 6"; "to the end of the test"; "a window upper bound ... not a point comparison" | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10; "to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter"; class under 1.1: context (premise BP1) | window end (p. 4), citations, class |
| E-037 | Ag-110m, 4-4-3, 1700 | Collin et al. (2016) Fig. 3, manuscript p. 16; p. 3, p. 4, p. 6 | notes | "accepted manuscript pp. 3, 5 and 6"; "to the end of the test"; "a window upper bound ... not a point comparison" | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10; "to the end of the 300 h plateau (p. 4), or to the end of the test where it was shorter"; class under 1.1: context (premise BP1) | window end (p. 4), citations, class |
| E-038 | Ag-110m, 4-1-2, 1600, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | notes | "accepted manuscript pp. 3 and 6"; "at the end of irradiation" with no source for the date | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10 | citation only (no window statement for failed-SiC rows; not comparable) |
| E-039 | Ag-110m, 6-4-1, 1600, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | notes | "accepted manuscript pp. 3 and 6"; "at the end of irradiation" with no source for the date | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10 | citation only (no window statement for failed-SiC rows; not comparable) |
| E-040 | Ag-110m, 3-3-1, 1700, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | notes | "accepted manuscript pp. 3 and 6"; "at the end of irradiation" with no source for the date | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10 | citation only (no window statement for failed-SiC rows; not comparable) |
| E-041 | Ag-110m, 5-1-1, 1700, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | notes | "accepted manuscript pp. 3 and 6"; "at the end of irradiation" with no source for the date | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10 | citation only (no window statement for failed-SiC rows; not comparable) |
| E-042 | Ag-110m, 3-2-3, 1800, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | notes | "accepted manuscript pp. 3 and 6"; "at the end of irradiation" with no source for the date | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10 | citation only (no window statement for failed-SiC rows; not comparable) |
| E-043 | Ag-110m, 4-3-2, 1800, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | notes | "accepted manuscript pp. 3 and 6"; "at the end of irradiation" with no source for the date | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10 | citation only (no window statement for failed-SiC rows; not comparable) |
| E-044 | Ag-110m, 4-4-1, 1800, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | notes | "accepted manuscript pp. 3 and 6"; "at the end of irradiation" with no source for the date | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10 | citation only (no window statement for failed-SiC rows; not comparable) |
| E-045 | Ag-110m, 5-1-3, 1800, failed SiC | Collin et al. (2016) Fig. 4, manuscript p. 17; p. 3, p. 6 | notes | "accepted manuscript pp. 3 and 6"; "at the end of irradiation" with no source for the date | "p. 3 for the denominator and p. 6 for the correction"; end of irradiation cited to Demkowicz p. 58 and Stempien p. 10 | citation only (no window statement for failed-SiC rows; not comparable) |

The R.7 paragraph and the applicability paragraph on matrix-corrected values were corrected in the register document and its template in the same way (window end, citations, class). The tolerance and the expected count of the check DI.release.denominator are unchanged.

## 11. Window label and applicability of the ten intact matrix-corrected rows (decision 45 procedure, decisions 67 and 68(b), task comparison-decision-67, 2026-11-26)

Source: Collin et al. (2016), accepted manuscript INL/JOU-15-34147 (OSTI 1246116), manuscript p. 4 (the experimental fractions were determined at the end of the 300 h plateaus), p. 3 (denominator) and p. 6 (the correction is an attempt); as already recorded in sections 10 and 10.1. No page was re-read for this section: it applies decision 68(b) to what sections 10 and 10.1 recorded. No measured value, qualifier, reading uncertainty, denominator, status or evidence class changed.

What changed in each of the ten rows, in `release_measurements.csv` and in the parameter register: (1) the column `time_basis` (the time window label): old value "after about 24 h at T to end (matrix-corrected)", new value "after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected)"; (2) the column `particle_applicability`: the text "; under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)" is appended; (3) the last sentence of the column `notes`: old "class under science design 1.1: context until the coordinator extends the bound definition", new "class under science design 1.1 as extended by decision 67: window upper bound under premise BP1 (the premise is the source's assumption, not an established fact)".

Reason: decision 67 rules that these rows are window upper bounds under premise BP1 and decision 68(b) words the label and the applicability addition; the label said "to end" although the window ends at the 300 h plateau or the shorter test end, and the notes sentence of (3) stated a class that decision 68(a) has since fixed. The failed-SiC rows E-038..E-045 keep their label, because decision 68(b) names only the ten intact rows. The data checks do not read the label and are unchanged.

| Row | Nuclide, compact, T (C) | Source page | Old time window label | New time window label | Other cells |
|---|---|---|---|---|---|
| E-010 | Cs-134, 3-2-2, 1600 | Collin et al. (2016) manuscript p. 4 (window end); Fig. 6, p. 3, p. 6 as in section 10 | after about 24 h at T to end (matrix-corrected) | after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected) | applicability: appended "under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)"; notes: class sentence updated |
| E-011 | Cs-134, 4-3-3, 1600 | Collin et al. (2016) manuscript p. 4 (window end); Fig. 6, p. 3, p. 6 as in section 10 | after about 24 h at T to end (matrix-corrected) | after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected) | applicability: appended "under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)"; notes: class sentence updated |
| E-012 | Cs-134, 5-3-3, 1600 | Collin et al. (2016) manuscript p. 4 (window end); Fig. 6, p. 3, p. 6 as in section 10 | after about 24 h at T to end (matrix-corrected) | after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected) | applicability: appended "under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)"; notes: class sentence updated |
| E-013 | Cs-134, 6-2-1, 1600 | Collin et al. (2016) manuscript p. 4 (window end); Fig. 6, p. 3, p. 6 as in section 10 | after about 24 h at T to end (matrix-corrected) | after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected) | applicability: appended "under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)"; notes: class sentence updated |
| E-014 | Cs-134, 6-4-3, 1600 | Collin et al. (2016) manuscript p. 4 (window end); Fig. 6, p. 3, p. 6 as in section 10 | after about 24 h at T to end (matrix-corrected) | after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected) | applicability: appended "under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)"; notes: class sentence updated |
| E-033 | Ag-110m, 4-3-3, 1600 | Collin et al. (2016) manuscript p. 4 (window end); Fig. 3, p. 3, p. 6 as in section 10 | after about 24 h at T to end (matrix-corrected) | after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected) | applicability: appended "under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)"; notes: class sentence updated |
| E-034 | Ag-110m, 5-3-3, 1600 | Collin et al. (2016) manuscript p. 4 (window end); Fig. 3, p. 3, p. 6 as in section 10 | after about 24 h at T to end (matrix-corrected) | after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected) | applicability: appended "under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)"; notes: class sentence updated |
| E-035 | Ag-110m, 6-2-1, 1600 | Collin et al. (2016) manuscript p. 4 (window end); Fig. 3, p. 3, p. 6 as in section 10 | after about 24 h at T to end (matrix-corrected) | after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected) | applicability: appended "under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)"; notes: class sentence updated |
| E-036 | Ag-110m, 6-4-3, 1600 | Collin et al. (2016) manuscript p. 4 (window end); Fig. 3, p. 3, p. 6 as in section 10 | after about 24 h at T to end (matrix-corrected) | after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected) | applicability: appended "under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)"; notes: class sentence updated |
| E-037 | Ag-110m, 4-4-3, 1700 | Collin et al. (2016) manuscript p. 4 (window end); Fig. 3, p. 3, p. 6 as in section 10 | after about 24 h at T to end (matrix-corrected) | after about 24 h at T to end of 300 h plateau (or test end if shorter) (matrix-corrected) | applicability: appended "under premise BP1; window to the end of the 300 h plateau or the shorter test end (Collin p. 4)"; notes: class sentence updated |

## 12. Experiment data integration: curves for the whole-curve comparison (decision 45 procedure, decision 69(d), task experiment-data-integration, 2026-11-26)

Scope. Task `experiment-data-integration` integrates the datasets that `experiment-comparability.md` (task `experiment-comparability` with its correction pass) names in its ranked list, with the class that it gives. Added: 96 rows E-541..E-636 in `release_measurements.csv` (six curves), 7 rows TH-158..TH-164 in `temperature_histories.csv` (one history), and the new file `data/experimental/curve_assessment.csv` (14 curve assessments, six new curves and eight earlier curves). No existing row, id, value, class or audit entry was changed. No arrhenius, decay, yield or geometry row was added: the assessment needs only registered rows (geometry G-01..G-05, CRP6-P4K3, JAERI-80FPC1; diffusivity sets C16; histories AGR1-LP-4-4-2, AGR1-ST-C16-1600, AGR1-ASRUN-6-4-3, AGR1-ASRUN-4-2-2-MULTI, CRP6-8a, JAERI-HEAT). No curve is R2 SUITABLE; all 14 are PLOT ONLY with the reason code of `experiment-comparability.md`.

### 12.1 Sources read and how their links were checked

| Source | Used for | Link checked on 2026-11-26 | File read (SHA-256) |
|---|---|---|---|
| Hunn, J. D., Gerczak, T. J., Morris, R. N., Baldwin, C. A., & Montgomery, F. C. (2016). *PIE on safety-tested loose particles from AGR-1 compact 4-4-2* (ORNL/TM-2015/161). Oak Ridge National Laboratory. https://doi.org/10.2172/1252138 | Fig. 2 (printed p. 4): Cs-134 and Cs-137 of the 75 loose particles; text pp. 3 to 7 | The DOI resolves (HTTP 200) to https://www.osti.gov/biblio/1252138; the OSTI record gives the five authors and the report number ORNL/TM-2015/161, and titles the report "PIE on Safety-Tested Loose Particles from Irradiated Compact 4-4-2" (the cover of the PDF says "from AGR-1 Compact 4-4-2"; the cover title is used) | `CEF23E85F731210E2164A797BEF500D56F165065B0D395D475E712E489E58787` (63 pages, from OSTI, kept outside every repository) |
| Demkowicz, P. A., Reber, E. L., Scates, D. M., Scott, L., & Collin, B. P. (2015). First high temperature safety tests of AGR-1 TRISO fuel with the Fuel Accident Condition Simulator (FACS) furnace. *Journal of Nuclear Materials, 464*, 320-330. https://doi.org/10.1016/j.jnucmat.2015.05.006 | Fig. 3 (printed p. 323): Cs-134 and Sr-90 of compact 4-3-3; Sec. 2.3 and Eq. (1) (printed p. 322); Table 2 | The DOI resolves (HTTP 200, publisher page); title, volume, pages, year and the five authors agree with the Crossref record (fetched 2026-11-26) and with the first page of the PDF | `56158F8498B61BF010F25FCF6972690662F33AC762950EDF473F1E3077D5429B` (the owner's private copy of the journal article: read in place, never copied, not in any repository) |
| International Atomic Energy Agency. (1997). *Fuel performance and fission product behaviour in gas cooled reactors* (IAEA-TECDOC-978; K. Verfondern, Ed.). IAEA. https://www-pub.iaea.org/MTCD/Publications/PDF/te_978_prn.pdf | Fig. 4-7 left (printed p. 137, PDF p. 144) and the text of printed pp. 136 to 137 | The URL answers HTTP 200 with a PDF of 33,825,723 bytes, which is the size of the file read | `33535574FDED0B51D90B503DB571A058F32C7CE029BCDD1342873B15E9E0386E` |

### 12.2 Procedure A: vector figures (Hunn et al. 2016 Fig. 2; Demkowicz et al. 2015 Fig. 3)

Both figures are vector drawings. With pdf.js (the copy bundled with a VS Code extension, run in Edge through a scratch Playwright script on 127.0.0.1) the operator list of the page was read, the current transformation matrix was tracked through `save`, `restore` and `transform`, and every path was recorded with its page coordinates (pt, origin at the lower left), its paint operator and its colours. A marker is a closed path of marker size (Hunn: 5.76 pt diamonds with black stroke; Demkowicz: 4.2 pt squares and triangles). The data point is the vertex of the series polyline (Demkowicz) or the centre of the marker path (Hunn, where the Cs-134 line vertices coincide with the marker centres). Axes were calibrated on the printed tick marks and gridlines, which are paths too. The digitising scripts are scratch tools (decision 50) and are not committed.

Independent checks (done after the extraction, by other means than the extraction):
- A 4x render of the figure was drawn with a cross at every extracted point and looked at: every point sits on a printed marker, and no printed marker of the series is without a point (Hunn Cs-134: 16 of 16; Demkowicz Cs-134 and Sr-90: 11 of 11 each).
- Hunn Cs-134: the centroid of the dark pixels of each marker in the 4x render (window 3.2 pt) was compared with the extracted centre. For 12 of the 16 markers the difference is below 0.15 pt (largest 0.145 pt, a bias that the red line passing through the marker causes); the other four (the first three and the marker at 167 h) are overlapped by black Kr-85 dots or the axis and cannot be checked this way.
- Demkowicz: the last points agree with Table 2 of the article (Cs-134 6.785e-7 read against 6.8e-7 printed; Sr-90 7.589e-4 read against 7.6e-4 printed), and the first points with the text.
- Hunn Cs-137: the 16 vertices of the thick line have the abscissae of the Cs-134 markers within 0.14 h.

Calibration (granularity of the stored positions: 0.06 pt for markers, 0.24 pt for the Hunn time ticks, 0.001 pt in Demkowicz).

| Figure | Page in the PDF | Time axis | Value axis | Residual of the calibration |
|---|---|---|---|---|
| Hunn et al. (2016) Fig. 2 | 16 | linear, 29 ticks every 25 h from 0 to 700 h at x = 137.76 pt .. 474.48 pt; t = -286.2147 + 2.078908 x (h, x in pt) | linear, gridlines 0 to 6 at y = 204.48 + 32.40 k pt; v = -6.311111 + 0.030864 y (particle equivalents) | time: largest tick residual 0.25 h; value: none |
| Demkowicz et al. (2015) Fig. 3 | 4 | linear, 8 ticks at 0, 50, ..., 350 h at x = 353.722 .. 516.748 pt; t = -759.5852 + 2.147289 x | logarithmic, 10 ticks at 1e-9 .. 1e0 at y = 97.460 .. 272.769 pt (19.479 pt per decade); log10 F = -14.005518 + 0.05134942 y | time: largest residual 0.069 h; value: 0.0015 decade = 0.34 % |

Reading uncertainty. Hunn: +/-0.01 particle equivalents (the positions have a granularity of 0.12 pt = 0.004 equivalents, and the first marker lies 0.004 above the zero line) and +/-0.3 h (tick granularity 0.24 pt gives +/-0.25 h, plus the marker granularity 0.06 pt = 0.13 h, rounded up). Demkowicz: +/-0.5 % of the value (the calibration residual of 0.34 % plus the path resolution) and +/-0.1 h (0.069 h plus the path resolution). These are reading resolutions of the drawing; they are not experimental uncertainties. A calibration error common to the whole figure is not covered.

Image identity (SHA-256 of the PNG of the figure region rendered by pdf.js at scale 4, which identifies the render and not the figure): Hunn Fig. 2, region x 130 to 485 pt, y 195 to 415 pt of page 16: `8b0684dc91e1913b59877c774e121b1229c9d34bc3386a866728d67bd6f2aa79`. Demkowicz Fig. 3, region x 345 to 525 pt, y 90 to 280 pt of page 4: `49b004babbbf14d881d6d8b3087090b7ac78d8512d2d20d2fd9ffc25c176bb40`. The images are not stored in the repository.

### 12.3 Procedure B: the raster scan of TECDOC-978 Fig. 4-7 (IAEA, 1997)

The page (PDF p. 144) holds one image of 2480 x 3458 pixels, 1 bit per pixel (300 dpi): there is no vector data. The page was rendered at exactly one pixel per image pixel and binarised (dark = ink). In the left panel (region x 385 to 1125, y 270 to 1250 px): (1) the horizontal gridlines (decades) and vertical gridlines (every 100 h) were located and fitted with their tilt (Theil-Sen); (2) the markers of the Ag-110m (open triangles) and Cs-137 (open circles and squares) curves were found as enclosed background components of 5 to 30 pixels (the interior of an open marker), inside the band of each curve; the centroid of the component is the marker centre; (3) each centre was converted to (time, value) by piecewise interpolation between the neighbouring gridlines at its own position: linear in time between the vertical lines at its row, and logarithmic in value between the horizontal lines at its column. The scan is tilted by about 0.3 degree and the decade spacing varies (118.3, 119.0, 120.0, 119.4, 116.2, 120.3, 122.5, 120.4 px), so a single linear axis would be wrong by up to 4 px.

Gridline fits (pixel coordinates of the page image; y at x = 500 px and x at y = 300 px):

| Decade line | 1e0 | 1e-1 | 1e-2 | 1e-3 | 1e-4 | 1e-5 | 1e-6 | 1e-7 | 1e-8 |
|---|---|---|---|---|---|---|---|---|---|
| y at x = 500 px | 280.35 | 398.68 | 517.68 | 637.69 | 757.06 | 873.23 | 993.50 | 1115.96 | 1236.34 |
| slope (px per px) | 0.00207 | 0.00186 | 0.00143 | 0.00037 | 0.00087 | 0.00046 | 0.00082 | -0.00014 | 0.00082 |

| Time line | 0 h | 100 h | 200 h | 300 h | 400 h | 500 h |
|---|---|---|---|---|---|---|
| x at y = 300 px | 394.80 | 535.18 | 681.43 | 823.02 | 966.28 | 1111.93 |
| slope (px per px) | -0.00571 | -0.00443 | -0.00574 | -0.00383 | -0.00564 | -0.00605 |

Reading uncertainty: +/-6 % of the value (3 px on a logarithmic axis of about 120 px per decade: 3 px = 0.025 decade = 5.8 %) and +/-2 h (3 px of about 142 px per 100 h = 2.1 h). Time is rounded to 1 h and the value to two figures. Printed markers per curve: 22 on the Cs-137 curve and 22 on the Ag-110m curve (the first count, "about 30 printed markers", was wrong; recounted in 12.8). In this first pass 21 markers per curve had a resolved interior and were recorded; the 22nd Cs-137 marker (18 h) was missed and is E-637 (12.8); the 22nd Ag-110m marker lies on the 500 h gridline at the edge of the axis (hole of 3 px) and was not recorded in the first pass; it was added as E-658 in 12.9. No marker was added by interpolation. Independent check: the marker centres were overlaid on the rendered region and looked at (every circle sits on a printed marker, none was found off the curve); the last Cs-137 marker (499 h, 1.1e-4) agrees with the CRP-6 text value E-150 (1e-4 at 500 h). Image identity: SHA-256 of the ink bitmap (one byte per pixel, 740 x 980 = 725,200 bytes, 69,726 ink pixels, row by row) of the region: `3ab8c46c4d08c08a57e13a8fc0f7e493096f569c05d66f602e649f993435921a`.

### 12.4 Curves that were not digitised, and why

- **Fig. 4-7, Sr-90 of HFR-K3/1**: digitised in 12.8 (E-638..E-657) after the independent verification showed the filled dots to be separate. The first-pass reason that follows was too strict and is kept for the record. Assessment rank 7 names Ag-110m, Cs-137 and Sr-90. The Sr-90 markers are small filled dots about as thick as the line itself (distance-transform radii of 3.6 to 6 px against a line half-width of about 2.5 px). Candidate maxima were found at spacings of 10 px that do not follow the plotting interval, so line-thickness variations cannot be told from markers. Not recorded. A cleaner copy of the figure, or the original paper (Schenk & Nabielek, 1991, Nuclear Technology, 96(3), 323-336, https://doi.org/10.13182/NT96-3-323), is needed.
- **Fig. 4-7, Kr-85 of HFR-K3/1**: drawn as a solid line without markers; not asked for by the assessment.
- **Eu-154 curves** (Demkowicz Fig. 3, Hunn 2014 E-525..E-540): the assessment rejects them for a model curve (no Eu diffusivity row); the registered Hunn 2014 rows stay as they are.
- **Ranks 10 and 11** (AGR-2 compacts, Morris et al. 2018, Hunn et al. 2017 and 2019): the assessment makes them depend on AGR-2 geometry rows that do not exist and marks them low priority; not done.
- **Rank 12** (Bullock 1984, held): not integrated, as the assessment orders.

### 12.5 One entry per new row

Each row below was written by the data task from the coordinates in the last columns; the values in the table are the cell texts of the CSV file. The reading uncertainty of every row is that of its dataset (12.2 and 12.3).

**HUNN16-442-CS134**: E-541..E-556, 16 rows.

| Row | t (h) | value | source coordinates |
|---|---|---|---|
| E-541 | 23.3 | 3.82e-3 | marker centre (148.86, 204.60) pt |
| E-542 | 35.7 | 1.37e0 | marker centre (154.86, 248.76) pt |
| E-543 | 47.2 | 1.6e0 | marker centre (160.38, 256.20) pt |
| E-544 | 70.7 | 1.62e0 | marker centre (171.66, 256.92) pt |
| E-545 | 95.1 | 1.62e0 | marker centre (183.42, 256.92) pt |
| E-546 | 119 | 1.62e0 | marker centre (194.94, 256.92) pt |
| E-547 | 143 | 1.63e0 | marker centre (206.46, 257.16) pt |
| E-548 | 167.4 | 2.01e0 | marker centre (218.22, 269.64) pt |
| E-549 | 191.4 | 2.11e0 | marker centre (229.74, 273.00) pt |
| E-550 | 215.3 | 2.13e0 | marker centre (241.26, 273.48) pt |
| E-551 | 287.2 | 2.13e0 | marker centre (275.82, 273.48) pt |
| E-552 | 365.5 | 2.13e0 | marker centre (313.50, 273.48) pt |
| E-553 | 455.3 | 2.14e0 | marker centre (356.70, 273.96) pt |
| E-554 | 532.7 | 2.33e0 | marker centre (393.90, 279.96) pt |
| E-555 | 628 | 2.47e0 | marker centre (439.74, 284.52) pt |
| E-556 | 691.3 | 2.47e0 | marker centre (470.22, 284.52) pt |

**HUNN16-442-CS137**: E-557..E-572, 16 rows.

| Row | t (h) | value | source coordinates |
|---|---|---|---|
| E-557 | 23.4 | 1.11e-2 | line vertex (148.92, 204.84) pt |
| E-558 | 35.8 | 1.49e0 | line vertex (154.92, 252.60) pt |
| E-559 | 47.3 | 1.74e0 | line vertex (160.44, 260.76) pt |
| E-560 | 70.8 | 1.77e0 | line vertex (171.72, 261.72) pt |
| E-561 | 95.2 | 1.77e0 | line vertex (183.48, 261.72) pt |
| E-562 | 119.2 | 1.77e0 | line vertex (195.00, 261.72) pt |
| E-563 | 143.1 | 1.78e0 | line vertex (206.52, 262.20) pt |
| E-564 | 167.6 | 2.19e0 | line vertex (218.28, 275.40) pt |
| E-565 | 191.5 | 2.3e0 | line vertex (229.80, 279.00) pt |
| E-566 | 215.5 | 2.31e0 | line vertex (241.32, 279.48) pt |
| E-567 | 287.3 | 2.33e0 | line vertex (275.88, 279.96) pt |
| E-568 | 365.6 | 2.34e0 | line vertex (313.56, 280.20) pt |
| E-569 | 455.5 | 2.35e0 | line vertex (356.76, 280.68) pt |
| E-570 | 532.8 | 2.57e0 | line vertex (393.96, 287.64) pt |
| E-571 | 628.1 | 2.72e0 | line vertex (439.80, 292.68) pt |
| E-572 | 691.5 | 2.73e0 | line vertex (470.28, 292.92) pt |

**DEM15-433-CS134**: E-573..E-583, 11 rows.

| Row | t (h) | value | source coordinates |
|---|---|---|---|
| E-573 | 24.2 | 7.55e-8 | polyline vertex (364.995, 134.049) pt |
| E-574 | 31.6 | 1.28e-7 | polyline vertex (368.473, 138.532) pt |
| E-575 | 46.1 | 2.83e-7 | polyline vertex (375.190, 145.216) pt |
| E-576 | 55.6 | 3.15e-7 | polyline vertex (379.627, 146.139) pt |
| E-577 | 70.2 | 3.42e-7 | polyline vertex (386.422, 146.819) pt |
| E-578 | 94 | 3.68e-7 | polyline vertex (397.536, 147.459) pt |
| E-579 | 118 | 3.99e-7 | polyline vertex (408.690, 148.139) pt |
| E-580 | 142.2 | 4.31e-7 | polyline vertex (419.963, 148.780) pt |
| E-581 | 166.1 | 4.58e-7 | polyline vertex (431.078, 149.300) pt |
| E-582 | 190.2 | 4.89e-7 | polyline vertex (442.311, 149.860) pt |
| E-583 | 331.7 | 6.78e-7 | polyline vertex (508.193, 152.622) pt |

**DEM15-433-SR90**: E-584..E-594, 11 rows.

| Row | t (h) | value | source coordinates |
|---|---|---|---|
| E-584 | 24.2 | 5.08e-9 | polyline vertex (364.995, 111.225) pt |
| E-585 | 31.6 | 3.3e-8 | polyline vertex (368.473, 127.040) pt |
| E-586 | 46.1 | 1.25e-5 | polyline vertex (375.190, 177.243) pt |
| E-587 | 55.6 | 3.76e-5 | polyline vertex (379.627, 186.571) pt |
| E-588 | 70.2 | 7.9e-5 | polyline vertex (386.422, 192.856) pt |
| E-589 | 94 | 1.49e-4 | polyline vertex (397.536, 198.221) pt |
| E-590 | 118 | 2.25e-4 | polyline vertex (408.690, 201.704) pt |
| E-591 | 142.2 | 2.93e-4 | polyline vertex (419.963, 203.945) pt |
| E-592 | 166.1 | 3.68e-4 | polyline vertex (431.078, 205.868) pt |
| E-593 | 190.2 | 4.42e-4 | polyline vertex (442.311, 207.429) pt |
| E-594 | 331.7 | 7.59e-4 | polyline vertex (508.193, 211.993) pt |

**IAEA97-K31-AG110M**: E-595..E-615, 21 rows.

| Row | t (h) | value | source coordinates |
|---|---|---|---|
| E-595 | 0 | 1.7e-2 | marker interior centroid (394.1, 490.8) px, hole 9 px |
| E-596 | 19 | 2e-2 | marker interior centroid (420.5, 482.5) px, hole 9 px |
| E-597 | 44 | 2e-2 | marker interior centroid (456.0, 482.5) px, hole 10 px |
| E-598 | 78 | 2e-2 | marker interior centroid (502.9, 482.8) px, hole 8 px |
| E-599 | 90 | 2e-2 | marker interior centroid (521.0, 481.7) px, hole 10 px |
| E-600 | 98 | 2e-2 | marker interior centroid (531.2, 481.6) px, hole 7 px |
| E-601 | 140 | 2e-2 | marker interior centroid (592.8, 482.4) px, hole 9 px |
| E-602 | 164 | 2e-2 | marker interior centroid (627.4, 482.5) px, hole 8 px |
| E-603 | 184 | 2e-2 | marker interior centroid (657.3, 482.0) px, hole 11 px |
| E-604 | 211 | 2e-2 | marker interior centroid (696.5, 481.5) px, hole 9 px |
| E-605 | 234 | 2e-2 | marker interior centroid (729.3, 482.6) px, hole 12 px |
| E-606 | 258 | 2e-2 | marker interior centroid (763.0, 481.5) px, hole 12 px |
| E-607 | 283 | 2e-2 | marker interior centroid (798.6, 481.6) px, hole 8 px |
| E-608 | 308 | 2.1e-2 | marker interior centroid (833.8, 480.5) px, hole 7 px |
| E-609 | 331 | 2.1e-2 | marker interior centroid (866.3, 479.4) px, hole 10 px |
| E-610 | 353 | 2.1e-2 | marker interior centroid (897.6, 479.6) px, hole 8 px |
| E-611 | 380 | 2.1e-2 | marker interior centroid (936.0, 479.5) px, hole 8 px |
| E-612 | 405 | 2.2e-2 | marker interior centroid (972.0, 477.7) px, hole 10 px |
| E-613 | 428 | 2.3e-2 | marker interior centroid (1005.6, 475.6) px, hole 8 px |
| E-614 | 452 | 2.3e-2 | marker interior centroid (1041.6, 474.6) px, hole 8 px |
| E-615 | 475 | 2.5e-2 | marker interior centroid (1074.7, 472.0) px, hole 10 px |

**IAEA97-K31-CS137**: E-616..E-637, 22 rows (E-637 added in 12.8; the first 21 rows are E-616..E-636).

| Row | t (h) | value | source coordinates |
|---|---|---|---|
| E-616 | 1 | 4.1e-7 | marker interior centroid (391.6, 1041.4) px, hole 8 px |
| E-617 | 43 | 8.2e-7 | marker interior centroid (451.2, 1003.9) px, hole 7 px |
| E-618 | 76 | 9.8e-7 | marker interior centroid (498.3, 994.7) px, hole 11 px |
| E-619 | 91 | 1.1e-6 | marker interior centroid (519.7, 990.0) px, hole 10 px |
| E-620 | 97 | 1.1e-6 | marker interior centroid (527.9, 989.3) px, hole 8 px |
| E-621 | 139 | 1.2e-6 | marker interior centroid (588.6, 984.6) px, hole 8 px |
| E-622 | 164 | 1.3e-6 | marker interior centroid (624.6, 981.6) px, hole 8 px |
| E-623 | 185 | 1.3e-6 | marker interior centroid (655.2, 981.6) px, hole 9 px |
| E-624 | 211 | 1.4e-6 | marker interior centroid (693.3, 977.0) px, hole 11 px |
| E-625 | 234 | 1.8e-6 | marker interior centroid (726.7, 962.1) px, hole 5 px |
| E-626 | 257 | 2.3e-6 | marker interior centroid (759.7, 950.1) px, hole 5 px |
| E-627 | 283 | 2.9e-6 | marker interior centroid (795.7, 938.1) px, hole 5 px |
| E-628 | 306 | 4.1e-6 | marker interior centroid (829.0, 920.2) px, hole 6 px |
| E-629 | 331 | 6.4e-6 | marker interior centroid (864.2, 896.6) px, hole 9 px |
| E-630 | 352 | 9.5e-6 | marker interior centroid (894.4, 876.2) px, hole 12 px |
| E-631 | 378 | 1.6e-5 | marker interior centroid (931.6, 850.6) px, hole 8 px |
| E-632 | 404 | 2.4e-5 | marker interior centroid (968.7, 829.1) px, hole 5 px |
| E-633 | 428 | 3.7e-5 | marker interior centroid (1003.7, 808.1) px, hole 5 px |
| E-634 | 451 | 5.4e-5 | marker interior centroid (1038.4, 788.4) px, hole 9 px |
| E-635 | 476 | 7.5e-5 | marker interior centroid (1073.5, 771.8) px, hole 7 px |
| E-636 | 499 | 1.1e-4 | marker interior centroid (1108.1, 752.7) px, hole 5 px |
| E-637 | 18 | 6.0e-7 | marker interior centroid (415.0, 1024.0) px in the frame of 12.8, hole 5 px |

### 12.6 Temperature history rows TH-158..TH-164

History AGR1-FACS-4-3-3 (Demkowicz et al. 2015, Fig. 3 and Sec. 2.3). The grey Temperature trace is a path of 16,643 vertices in 732 segments (stroke colour 0.5 grey). The right axis (ticks every 200 C at y = 97.447 .. 267.435 pt) and the time axis (Procedure A) were used. Knots: 395 C first reached at 2.57 h; the 400 C hold ends at 4.80 h (last vertex below 415 C); 1245 C first reached at 11.93 h (1250 C reached at 11.84 h); the 1250 C hold ends at 24.55 h; 1590 C first reached at 31.41 h; the last vertex above 1580 C at 331.72 h; the trace is below 100 C at 334.46 h. The plateau levels read 400 C (397 to 404), 1250 C (1246 to 1251) and 1593 to 1597 C. The printed procedure (printed p. 322) gives the ramp rates and nominal holds; the trace agrees with it to the reading resolution, with holds of 2.2 h and 12.6 h against the printed 2 h and 12 h. One entry per row: TH-158 ramp to 400 C, 2.6 h; TH-159 hold 400 C, 2.2 h; TH-160 ramp to 1250 C, 7.1 h; TH-161 hold 1250 C, 12.6 h; TH-162 ramp to 1600 C, 6.9 h; TH-163 hold 1600 C, 300.3 h; TH-164 cool-down, 3.0 h; cumulative end times 2.6, 4.8, 11.9, 24.5, 31.4, 331.7 and 334.7 h. Class "reconstructed", evidence class "measured, digitised", status "digitised", like the other histories read from a thermocouple trace (TH-123 and TH-150 onwards). The history is that of DEM15-433-CS134 and DEM15-433-SR90; the registered series E-425..E-434 is the same test (the end value 3.40e-1 of its note equals Table 2 of the article).

### 12.7 Curve assessments (new file) and changes to the checks

`curve_assessment.csv` holds one record per curve (25 columns: the fields of `metric-r2.md` Section 1.3 plus the registered history, geometry and model configuration of `experiment-comparability.md` Section 5, the source with page and link, the value kind and the verification status). The 14 records are DEM15-433-CS134, DEM15-433-SR90, HUNN16-442-CS134, HUNN16-442-CS137, IAEA97-K31-AG110M, IAEA97-K31-CS137 (new rows) and HUNN14-442-AG110M, HUNN14-442-SR90, STEMPIEN23-433-AG110M, STEMPIEN23-643-AG110M, STEMPIEN23-643-SR90, HUNN14-422-AG110M, IAEA97-JAERI-A-CS137-1600, IAEA97-JAERI-A-CS137-1800 (earlier rows: assessment only, their rows and values are unchanged). Every record is not independently verified, so the check C0 of the metric fails closed; none sets `r2_class` to R2 SUITABLE.

Changes to the Rust checks of `strato-data`: the two checks `DI.release.reading-uncertainty` and `DI.release.denominator` count digitised rows and rows without a denominator; their counts (219 and 43) are the counts of the earlier register and are now taken over E-001..E-540, so the checks keep their meaning; the new rows are covered by the family EX (units, monotone times, source resolution, uncertainty and audit entry of every digitised row). No tolerance was loosened. The row counts of the tests of the crate were raised by the new rows (release 482 to 578, temperature 157 to 164).

### 12.8 Corrections after independent verification (decision 45 procedure, task experiment-data-integration-fix, 2026-11-26)

An independent verifier of the range 7de5bdb..764723e found six defects in this section's data integration. What was done, in the order of the findings:

1. **Missing Cs-137 marker (blocker).** The first pass missed an open marker on the Cs-137 curve between the markers at 1 h and 43 h. It is added as E-637 (18 h, 6.0e-7). It is placed after E-616 in the order of the curve (the record lists `E-616,E-637,E-617..E-636`), so the ids E-617..E-636 are unchanged.
2. **Count of printed markers (major).** The statement "21 of about 30 printed markers" is withdrawn from the 42 row notes, the two curve records, 12.3 and the handback. Recount on the render and on the hole list of the embedded image: Cs-137 22 printed markers (21 first pass plus E-637), all recorded; Ag-110m 22 printed triangles, 21 recorded and the 22nd on the 500 h gridline at the edge of the axis (hole of 3 px, 2 x 2 px), which the 5 px interior criterion excludes and which is therefore not recorded. The first pass found no further unresolved marker on either curve.
3. **Comparison grouping (major).** Not a data defect: the comparison program grouped rows by programme, nuclide, group and quantity only, so the Sr-90 rows of compact 4-3-3 (Demkowicz et al. 2015, E-584..E-594) were pooled with the Sr-90 rows of compact 6-4-3 (Stempien et al. 2023) under the name of the first row's source. The pooling of rows (programme, nuclide, group, quantity) is unchanged, because splitting it would change the aggregates of the verified earlier groups; instead every dataset now names all the sources of its rows (joined by " | "), so no row appears under another paper's name. The Sr-90 group of the intact AGR-1 compacts (22 rows) names Stempien et al. (2023) and Demkowicz et al. (2015; FACS); the earlier groups that held rows of two papers (Morris et al. 2016 with Stempien et al. 2023, Demkowicz et al. 2013 with Stempien et al. 2023) now name both. The comparison outputs were regenerated with `strato-compare run` (data/comparison, the report and the figures).
4. **Source record (minor).** The register's source record IAEA (1997) is left as it was; a dated note beside it records that the URL was opened and checked on 2026-11-26 (12.1).
5. **Sr-90 curve (minor).** Digitised as E-638..E-657 (20 filled dots), dataset IAEA97-K31-SR90, see below.
6. **Specimen level (minor).** The vocabulary of `specimen_level` gains `sphere`; the records of the HFR-K3/1 curves use it. The bound class stays `compact_total` as the assessment gives it.

**Image and frame.** The page image was taken from the PDF itself (pdf.js, the single image of PDF p. 144: 2480 x 3458 px, 1 bit per pixel, 1,071,980 bytes; SHA-256 of the packed bytes `040BB49A604E6A9023DFDAA04E5F12AA1B1E1231654E5974C5487210B9ED9683`), not from a render. Ink = bit 0. The ink bitmap of the region x 385 to 1124, y 270 to 1249 (740 x 980 px, one byte per pixel, row by row) has 70,051 ink pixels and SHA-256 `a211002fc5c7b4befd090ae505e24f4b88bf1582ce5a5238ade5eed7ddd0089e`. The first pass binarised a render and counted 69,726 ink pixels, and its pixel frame lies about 3 px higher in y; the pixel coordinates below are therefore in this frame, not in that of 12.5. Enclosed background components (4-connected, not touching the region border) of 3 to 60 px were listed; markers are those of 5 px or more with an extent of at least 3 x 3 px in the band of a curve.

**Calibration in this frame** (Theil-Sen fits of the gridline centres over windows of 20 px; y of a horizontal line at x = 500 px, x of a vertical line at y = 300 px):

| Decade line | 1e0 | 1e-1 | 1e-2 | 1e-3 | 1e-4 | 1e-5 | 1e-6 | 1e-7 | 1e-8 |
|---|---|---|---|---|---|---|---|---|---|
| y at x = 500 px | 284.41 | 402.85 | 521.76 | 641.50 | 761.06 | 877.36 | 996.74 | 1119.02 | 1239.47 |
| slope (px per px) | 0.00205 | 0.00146 | 0.00118 | 0.00000 | 0.00111 | 0.00017 | 0.00157 | -0.00018 | 0.00093 |

| Time line | 0 h | 100 h | 200 h | 300 h | 400 h | 500 h |
|---|---|---|---|---|---|---|
| x at y = 300 px | 394.95 | 535.52 | 681.86 | 823.38 | 966.23 | 1112.47 |
| slope (px per px) | -0.00648 | -0.00607 | -0.00703 | -0.00564 | -0.00655 | -0.00668 |

A marker centre is converted by linear interpolation in time between the vertical lines at its own row and by logarithmic interpolation in value between the horizontal lines at its own column. **Check of the frame:** the 41 markers of E-595..E-636 that are not at the edge of the axis were converted again in this frame; they agree with the registered values within 1.1 h and 3.3 % (the reading uncertainties are 2 h and 6 %), so the two frames give the same curves.

**E-637.** Enclosed component of 5 px (3 x 3 px) at (415.0, 1024.0) px; converted: 17.6 h and 5.97e-7. The verifier's own calibration gave 17.6 h and 6.07e-7 (1.7 % apart). Recorded: 18 h and 6.0e-7, reading uncertainty 6 % and 2 h as for the other open markers.

**Sr-90 (IAEA97-K31-SR90), E-638..E-657.** The filled dots are not enclosed components, so they were found with a distance transform: the ink of the region x 385 to 1124, y 1060 to 1249 got its Euclidean distance to the nearest background pixel (search radius 9 px); a local maximum of the distance (within 3 px) of 4.4 px or more is a candidate (the line itself reaches 3.6 to 4.0 px; dots reach 4.5 to 5.4 px), candidates less than 6 px apart are merged. The candidate list was compared with a 4-fold enlargement of the figure: 20 candidates on the Sr-90 curve are separate dots. Not taken: the candidates of the Kr-85 curve above y = 1075 px, the label Sr90, the junction of the Sr-90 line with the 1e-7 gridline at (570.6, 1121.3) px (distance 5.0 px, but at a gridline crossing and not a dot in the enlargement) and a thickening at (606, 1114) px. The Sr-90 reading uncertainty is larger than for open markers: +/-8 % of the value (4 px: the peak of a dot on a line of 8 px width is located to about 2 px in y and 3 px in x, plus the calibration residual) and +/-3 h. Observation, not used to place any dot: the times of the 20 dots agree with the times of the Cs-137 markers within 1.2 h, so both curves share their sampling times. A dot at 0 h is not recorded: no dot was found there.

**IAEA97-K31-SR90**: E-638..E-657, 20 rows.

| Row | t (h) | value | source coordinates |
|---|---|---|---|
| E-638 | 19 | 1.2e-8 | dot centre (415.3, 1228.3) px, inscribed radius 5.0 px |
| E-639 | 42 | 2.0e-8 | dot centre (449.0, 1202.0) px, inscribed radius 5.0 px |
| E-640 | 76 | 4.0e-8 | dot centre (497.0, 1167.0) px, inscribed radius 4.5 px |
| E-641 | 97 | 6.0e-8 | dot centre (525.5, 1146.0) px, inscribed radius 4.5 px |
| E-642 | 138 | 1.0e-7 | dot centre (586.0, 1117.0) px, inscribed radius 5.4 px |
| E-643 | 162 | 1.1e-7 | dot centre (621.0, 1112.0) px, inscribed radius 5.0 px |
| E-644 | 185 | 1.2e-7 | dot centre (653.7, 1107.7) px, inscribed radius 5.0 px |
| E-645 | 212 | 1.3e-7 | dot centre (693.0, 1104.0) px, inscribed radius 5.0 px |
| E-646 | 235 | 1.4e-7 | dot centre (726.0, 1103.0) px, inscribed radius 5.4 px |
| E-647 | 257 | 1.4e-7 | dot centre (758.0, 1099.5) px, inscribed radius 5.0 px |
| E-648 | 283 | 1.5e-7 | dot centre (794.5, 1099.0) px, inscribed radius 5.0 px |
| E-649 | 306 | 1.6e-7 | dot centre (828.0, 1095.0) px, inscribed radius 5.0 px |
| E-650 | 330 | 1.5e-7 | dot centre (862.0, 1096.0) px, inscribed radius 5.0 px |
| E-651 | 352 | 1.6e-7 | dot centre (893.5, 1095.0) px, inscribed radius 5.0 px |
| E-652 | 380 | 1.7e-7 | dot centre (932.5, 1092.0) px, inscribed radius 5.0 px |
| E-653 | 403 | 1.7e-7 | dot centre (966.0, 1091.0) px, inscribed radius 5.1 px |
| E-654 | 428 | 1.7e-7 | dot centre (1001.7, 1090.3) px, inscribed radius 5.0 px |
| E-655 | 452 | 1.7e-7 | dot centre (1037.0, 1090.5) px, inscribed radius 5.0 px |
| E-656 | 475 | 1.8e-7 | dot centre (1070.0, 1087.0) px, inscribed radius 5.0 px |
| E-657 | 498 | 1.8e-7 | dot centre (1105.0, 1087.0) px, inscribed radius 4.5 px |

**Totals after 12.8.** `release_measurements.csv` holds E-541..E-657 as the experiment-data integration's rows (117 rows: 96 of 12.5 plus E-637 and E-638..E-657); `curve_assessment.csv` holds 15 curve assessments. No earlier value, id or tolerance was changed; the cell texts of the 42 rows E-595..E-636 changed only in the marker count and the marker numbers of the Cs-137 rows after E-616 (the marker at 18 h is number 2, so every later Cs-137 marker number rose by one).


### 12.9 Corrections after the second independent verification (decision 45 procedure, task experiment-data-integration-fix, 2026-11-27)

An independent verifier of the range 7de5bdb..8ce463f found one blocker and two minor defects. What was done:

1. **Missing Ag-110m marker (blocker).** The 22nd printed marker of the Ag-110m curve, an open triangle on the 500 h gridline at the right edge of the axis of TECDOC-978 Fig. 4-7 left, was left out because its interior (3 px) is below the 5 px criterion. That is a limit of the detection method, not of the source, so the marker is now recorded from its outline. In the page image of 12.8 (same image, same frame, same calibration): the enclosed background component is an L-shaped group of 3 px with its centroid at (1110.7, 470.3) px; the ink outline of the triangle has its apex at row 464, its base at row 474 (columns 1104 to 1118) and its area centroid (apex row plus two thirds of the height) at about (1111, 470.7) px, so the two readings agree within 0.5 px. Converted with the calibration of 12.8: 499.5 h and 2.75e-2 from the interior, 499.8 h and 2.73e-2 from the outline. Recorded: 500 h and 2.7e-2. The reading uncertainty is stated larger than for the resolved markers because the interior is not resolved: +/-8 % of the value and +/-3 h (4 px each way). The marker is the highest value of the Ag-110m curve (the previous marker, E-615, is 2.5e-2 at 475 h) and lies on the last gridline, so 500 h is the end of the 500 h hold. It is added as **E-658** (next free id; no id was renumbered, no earlier value changed) and appended to the row ids of IAEA97-K31-AG110M, which now holds 22 of 22 printed markers (`E-595..E-615,E-658`). The 21 earlier Ag-110m rows changed only in the marker count (`Marker n of 22 recorded`) and in the sentence that accounted for the unrecorded marker.
2. **Gridline name (minor).** The gridline that the Sr-90 line crosses at about 128 h is the 1e-7 gridline (1e-7 is at y = 1119.02 px and 1e-5 at y = 877.36 px in the calibration table of 12.8), not the 1e-5 gridline. Corrected in the notes of E-638..E-657, in the register and in 12.8. The junction at (570.6, 1121.3) px was and is not taken as a dot.
3. **Provenance of the comparison outputs (minor).** The headers of data/comparison named commit 388bacc, which does not contain the source-naming code of 8ce463f. The outputs were regenerated after the data commit of this section with `strato-compare run --commit <that commit>`, which contains the code and the data, so the header names the commit that holds both.

| Row | dataset | t (h) | value | source coordinates |
|---|---|---|---|---|
| E-658 | IAEA97-K31-AG110M | 500 | 2.7e-2 | interior centroid (1110.7, 470.3) px, 3 px; outline centroid (1111, 470.7) px; reading uncertainty 8 % and 3 h |

**Totals after 12.9.** `release_measurements.csv` holds E-541..E-658 as the experiment-data integration's rows (118 rows); `curve_assessment.csv` still holds 15 curve assessments, of which the Ag-110m record now counts 22 points. No earlier value, id or tolerance was changed.

## 13. Register field fitted_to: which diffusivity rows were fitted to a dataset (decision 45 procedure, decisions 72(f) and 76(d), task integration-2a, 2026-11-27)

The independence rule of the R2 records (`metric-r2.md` Section 6) read the sentence "fitted to <table> data" from the free-text notes of a diffusivity row. It now reads a register field. `data/diffusivity/arrhenius.csv` gains the column `fitted_to` (last column, empty on every row except the two below). An entry reads `<source> | <location prefix>`; a curve is read from that dataset when its source equals the entry's source and its location starts with the entry's prefix. The comparability assessment (`experiment-comparability.md`, row R.6 and item 9 of Section 6) establishes that the Cs-in-SiC coefficients A3-14 and E410-01 were fitted to the JAERI heating tests of IAEA-TECDOC-978 Table 4-9 (the note of E410-01 says "fitted to Table 4-9 data"; the note of A3-14 reads "identical to Eq. (4-10)", the fit of E410-01); no other row is established as fitted to a registered dataset. The AGR-1 rows of Schappel et al. (2020) are fitted to the AGR-1 heating tests, but those tests are not a registered curve of `curve_assessment.csv` and the assessment makes no statement on them, so they stay without an entry. No measured value, no coefficient and no other cell changed; the check `DI.arrhenius.fitted-to` requires exactly these two entries and one audit entry per row.

| Row | Column | Before | After | Basis |
|---|---|---|---|---|
| A3-14 | fitted_to | (column absent; the note reads "identical to Eq. (4-10)") | IAEA (1997) &#124; Table 4-9 | assessment R.6 and item 9: fitted to the JAERI tests of Table 4-9 |
| E410-01 | fitted_to | (column absent; the note said "fitted to Table 4-9 data") | IAEA (1997) &#124; Table 4-9 | assessment R.6 and item 9: fitted to the JAERI tests of Table 4-9 |