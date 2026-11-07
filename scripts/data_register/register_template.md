# STRATO Parameter Register (Deliverable 5)

Project: STRATO (Spherical TRISO Analysis and Transport Operations). Register compiled 2026-11-06 from the verified Stage 1 audit notes (`01` to `08`, including each file's Verification log, whose corrections supersede the original text). No value in this register was invented, estimated or taken from memory. Every number traces to a printed page of a source PDF or to an evaluated nuclear data library queried during the audit.

Tables in this file are rendered from the machine-readable CSV files listed in Section 0.3, so the row identifiers (G-, A-, N-, Y-, TH-, E-) are the same in both places.

## 0. How to read this register

### 0.1 Rule on synthetic values

Synthetic, notional or "demonstration" values never appear here. The v1.0 synthetic case (`deliverables/synthetic_five_layer.toml`: radii 200 to 450 $\mathrm{\mu m}$, piecewise-constant $D$) and the VTB example inputs found in the repository review (kernel 425 $\mathrm{\mu m}$, 15.5 wt% enrichment, 700 °C) are deliberately excluded. Benchmark definitions that are synthetic by construction (CRP-6 sensitivity cases 1 to 5 and case 11) are included because they are printed specifications with published reference answers; they carry the class "specified (synthetic benchmark)".

### 0.2 Status and data-class vocabulary

| Status (CSV column `status`) | Meaning |
|---|---|
| verified | value read from the printed page (or library) by the audit and re-checked by the independent verifier |
| digitised | value read from a raster figure, either by pixel digitisation (R-dig, about $\pm 5$ % of value, validated against a printed table) or by visual reading (R-vis, about $\pm 10$ to 15 % on linear axes, about a factor 1.5 on log axes) |
| NOT VERIFIED | value, label or interpretation that could not be confirmed against a primary source; see Section 11 |

Data classes: **measured** (characterisation or release measurement), **M-UB** (measured upper bound or detection-limit statement), **design specification**, **benchmark specification**, **recommended/fitted correlation** (Arrhenius fits to irradiation and heating data), **model input set** (a code's chosen parameter set), **calculated** (computed in this register by the check script; never a source value), **evaluated** (nuclear data library).

Evidence classes (CSV column `evidence_class`, added by the B6 audit on 2026-11-06). The column `status` says whether a value was checked against its source; the column `evidence_class` says what kind of value it is. Every row of every CSV carries exactly one of the following.

| Evidence class | Meaning | May be used as |
|---|---|---|
| measured | experimental value printed in a table, in the text or in a vector figure | validation target |
| measured, digitised | experimental value read from a raster figure (status `digitised`) | validation target with the digitisation uncertainty of Section 0.2 |
| measured bound | upper bound or detection-limit statement ("<") | inequality constraint only, never a point value |
| literature correlation | Arrhenius fit from the literature, including the same fit reproduced as a code input set (Collin et al., 2016; Hales et al., 2021) | model input |
| literature correlation (upper bound) | correlation whose $D_0$ is printed as an upper bound (row A3-13) | bounding model input |
| literature value (single temperature) | single diffusivity at one temperature, no activation energy (row A3-20) | spot comparison only |
| modelling convention | value chosen by a modeller (buffer $D$, failed-layer $D$, instantaneous compact release, PARFUME test schedule) | reproduction of that model only |
| specification | design, nominal, test-procedure or benchmark-of-a-real-test specification | model input |
| synthetic benchmark | value defined only for a code-comparison case (CRP-6 cases 1 to 5 and 11) | code verification only, never a physical parameter |
| calculated | computed by a source (as-run neutronics, thermal analysis) or by this register | model input with the source's model uncertainty |
| estimate | author's inference or assumption (PIE failure counts, assumed 30 °C start, interpretive thresholds) | with caution; see the row note |
| evaluated nuclear data | ENSDF or JEFF-3.1.1 library value | model input |
| basis not stated | source does not say whether the value is measured, specified or calculated | context only until resolved |

### 0.3 Files

| File | Content | Rows |
|---|---|---|
| `data/geometry/geometry.csv` | particle and compact geometry, densities, defect fractions, derived outer radii | 149 |
| `data/diffusivity/arrhenius.csv` | Arrhenius coefficients, buffer and failed-layer conventions | 118 |
| `data/nuclear/decay.csv` | half-lives, decay constants, modes, daughters (ENSDF) | 16 |
| `data/nuclear/fission_yields.csv` | independent and cumulative thermal yields, U-235 and Pu-239 (JEFF-3.1.1, ENDF/B cross-check) | 56 |
| `data/temperature/temperature_histories.csv` | heating schedules, as-run deviations, irradiation temperatures | 122 |
| `data/experimental/release_measurements.csv` | AGR-1, JAERI and CRP-6 release data, upper bounds, failure counts | 156 |
| `scripts/derivation_checks/D5_parameter_register.py` | consistency and derivation checks (Section 10), including the B6 family V | 91 checks |
| `scripts/data_register/b6_audit_patch.py` | idempotent B6 patch: adds `evidence_class` and `particle_applicability`, applies the location and note corrections | 33 cells changed, 0 numerical values |
| `scripts/data_register/b6_nuclear_crosscheck.py` | B6 fresh LiveChart comparison of `decay.csv` and `fission_yields.csv` (needs network) | 204 checks |
| `data/reference/B6_audit_log.md` | B6 audit log (Item, Check, Result, Correction) | |

### 0.4 Location convention

"p." is the printed page number on the page. "idx" is the 0-based PyMuPDF page index, so the PDF viewer page number is idx + 1. The page maps confirmed by the audits are: 2.pdf (TECDOC-978) offset not constant, read from each footer; 3.pdf (TECDOC-1674) idx = p. + 13; 5.pdf (Morris 2016) p. = 24 + idx; 6.pdf and 7.pdf (Collin 2016, identical content) p. = 378 + idx; 8.pdf (Hales 2021) p. = idx + 1; 4.pdf (Collin 2021) p. = idx.

### 0.5 Equation labels

No equation block was assigned to this deliverable, so the defining relations below carry register labels (R.1) to (R.8). They restate source equations or unit conversions; none is a new model.

## 1. Notation

| Symbol | Meaning | SI unit |
|---|---|---|
| $i = 1..5$ | layer index: kernel, buffer, IPyC, SiC, OPyC | 1 |
| $R_i$ | outer radius of layer $i$; $R_0 = 0$, $R_5 = R$ | $\mathrm{m}$ (tables in $\mathrm{\mu m}$) |
| $d_k = 2R_1$ | kernel diameter | $\mathrm{m}$ |
| $t_i = R_i - R_{i-1}$ | thickness of coating layer $i$ ($i \ge 2$) | $\mathrm{m}$ |
| $\rho_i$ | density of layer $i$ | $\mathrm{kg\ m^{-3}}$ (tables in $\mathrm{Mg\ m^{-3}}$) |
| $D_i(T)$ | effective diffusivity in layer $i$ | $\mathrm{m^2\ s^{-1}}$ |
| $D_{0,i,m}$, $Q_{i,m}$ | pre-exponential factor and activation energy of Arrhenius term $m$ | $\mathrm{m^2\ s^{-1}}$; $\mathrm{J\ mol^{-1}}$ (tables in $\mathrm{kJ\ mol^{-1}}$) |
| $D' = D/r_k^2$, $D'_0$ | reduced diffusivity referred to the kernel radius $r_k$ | $\mathrm{s^{-1}}$ |
| $R_g$ | molar gas constant | $\mathrm{J\ mol^{-1}\ K^{-1}}$ |
| $T$ | absolute temperature ($T = \theta + 273.15$ with $\theta$ in °C) | $\mathrm{K}$ |
| $\Gamma$ | fast neutron fluence in units of $10^{25}\ \mathrm{m^{-2}}$ (cut-off stated per row) | 1 |
| $c(r,t)$ | fission-product concentration | $\mathrm{mol\ m^{-3}}$ |
| $\lambda_k$, $t_{1/2}$ | decay constant and half-life of nuclide $k$ | $\mathrm{s^{-1}}$; $\mathrm{s}$ |
| $y^I_k$, $y^C_k$ | independent and cumulative fission yield | atoms per fission |
| $\dot F$ | fission rate density | $\mathrm{fissions\ m^{-3}\ s^{-1}}$ |
| $N_A$ | Avogadro constant, $6.02214076\times10^{23}$ (exact, SI definition) | $\mathrm{mol^{-1}}$ |
| $S_k(r,t)$ | volumetric source of nuclide $k$ | $\mathrm{mol\ m^{-3}\ s^{-1}}$ |
| $M_{\mathrm{rel}}(t)$ | cumulative amount released through $r = R$ | $\mathrm{mol}$ |
| $f_{\mathrm{rel}}$ | release fraction as defined by each experiment (Eq. R.7) | 1 |
| $N_p$ | particles per compact or fuel element | 1 |

## 2. Defining relations

**(R.1) Arrhenius form.** IAEA (1997) Eq. (A-1), p. 435, idx 439:

$$
D_i(T) = \sum_{m} D_{0,i,m}\,\exp\!\left(-\frac{Q_{i,m}}{R_g\,T}\right) \tag{R.1}
$$

The tables print $Q$ in $\mathrm{kJ\ mol^{-1}}$ although the text of Eq. (A-1) and of Collin et al. (2016) Eq. (3) defines $Q$ in $\mathrm{J\ mol^{-1}}$, so $Q[\mathrm{J\ mol^{-1}}] = 10^3\,Q[\mathrm{kJ\ mol^{-1}}]$ before use. Class: empirical correlation, exact as a definition. Limits: one term ("1T") when $m = 1$; $D \to \sum_m D_{0,i,m}$ as $T \to \infty$; each term vanishes as $T \to 0$. Two-term rows ("2T") are summed, which IAEA (2012, p. 571, idx 584) confirms is the intended reading for Cs in SiC.

**(R.2) Fluence-dependent Cs-in-SiC term.** IAEA (1997) Table A-3 footnote (1), p. 446, idx 450, and IAEA (2012) Eq. (10.9), p. 548, idx 561:

$$
D_{0,4,1}^{\mathrm{Cs}}(\Gamma) = 5.5\times10^{-14}\,e^{\Gamma/5}\ \mathrm{m^2\ s^{-1}},\qquad \Gamma \text{ in } 10^{25}\ \mathrm{m^{-2}},\ E > 16\ \mathrm{fJ} \tag{R.2}
$$

Exact as printed. Limit $\Gamma \to 0$ gives the zero-fluence value used by Collin et al. (2016) and Hales et al. (2021).

**(R.3) Reduced coefficient conversion.** IAEA (2012) Eq. (7.4), p. 318, idx 331: $D' = D/r_k^2$ with $r_k = 250\ \mathrm{\mu m}$, hence $D_0 = D'_0\,r_k^2$. Exact unit conversion. The kernel coefficients of IAEA (1997) Table A-1 are effective values "with respect to kernel radius r = 250 $\mathrm{\mu m}$"; applying them to a 350 $\mathrm{\mu m}$ AGR-1 kernel is an extrapolation.

**(R.4) Radii.** $R_1 = d_k/2$ and $R_i = R_{i-1} + t_i$ for $i = 2..5$. Exact.

**(R.5) Decay constant.** $\lambda_k = \ln 2 / t_{1/2}$ (exact) and $u(\lambda_k) = \lambda_k\,u(t_{1/2})/t_{1/2}$ (first-order propagation, approximate).

**(R.6) Fission source.** For a nuclide whose precursors are short-lived compared with the time step and the diffusion time, audit 07 Sec. 5.2:

$$
S_k(r,t) = \frac{y^C_k\,\dot F(r,t)}{N_A}\quad (0 \le r < R_1),\qquad S_k = 0 \text{ in the coatings} \tag{R.6a}
$$

Eq. (R.6a) applies only to rows whose `product_class` starts with "fission product". Ag-110m, Cs-134 and Eu-154 are fission-shielded activation products (`product_class` ends with "NO fission-yield source in STRATO"); their sources are capture terms on Ag-109, Cs-133 and Eu-153, whose cross sections are NOT VERIFIED (GAP-26). Their tabulated yields are listed for completeness only.

If a daughter $d$ of parent $p$ is tracked as a separate species, its source is

$$
S_d = \frac{y^I_d\,\dot F}{N_A} + b_{p\to d}\,\lambda_p\,c_p \tag{R.6b}
$$

never $y^C_d\dot F/N_A + \lambda_p c_p$, which counts the parent twice. Class: modelling rule (approximate, validity condition above). Precursor delays ($t_{99} = \log_2(100)\,t_{1/2}$ of the longest precursor) are 0.42 h (Cs-137), 0.48 h (Sr-90), 29.8 h (Kr-85), 36.5 h (Ag-111) and 221 h (I-131) (calculated in audit 07).

**(R.7) Release fraction definitions used by the data.** AGR-1: $f_{\mathrm{rel}}(t)$ is the cumulative measured release divided by the **calculated** compact inventory from as-run neutronics (Morris et al., 2016, p. 28, idx 4; Collin et al., 2016, p. 379, idx 1). "Matrix-corrected" values count release only after about the first 24 h at test temperature (Collin et al., 2016, p. 382, idx 4). CRP-6: reference inventory is the inventory at the start of the heating test (IAEA, 2012, p. 552, idx 565). JAERI Table 4-9: fractional release per test, normalisation not restated in the audit notes.

**(R.8) Booth bare-sphere release (used only as a register check).** IAEA (2012) Eq. (10.7), p. 544, idx 557: $F = 1 - \frac{6}{\pi^2}\sum_{n=1}^{\infty} e^{-n^2\pi^2 D' t}/n^2$. Series, truncated at $2\times10^4$ terms in the check.

## 3. Sources (APA 7)

| Key | Reference | Local file | Link |
|---|---|---|---|
| IAEA (1997) | International Atomic Energy Agency. (1997). *Fuel performance and fission product behaviour in gas cooled reactors* (IAEA-TECDOC-978; K. Verfondern, Ed.). IAEA. | 2.pdf | no DOI; candidate URL https://www-pub.iaea.org/MTCD/Publications/PDF/te_978_prn.pdf (NOT VERIFIED, not opened) |
| IAEA (2012) | International Atomic Energy Agency. (2012). *Advances in high temperature gas cooled reactor fuel technology* (IAEA-TECDOC-CD-1674) [CD-ROM]. IAEA. ISBN 978-92-0-186810-7. | 3.pdf | no DOI; candidate URL https://www-pub.iaea.org/MTCD/Publications/PDF/TE_1674_CD_web.pdf (NOT VERIFIED) |
| Morris et al. (2016) | Morris, R. N., Baldwin, C. A., Demkowicz, P. A., Hunn, J. D., & Reber, E. L. (2016). Performance of AGR-1 high-temperature reactor fuel during post-irradiation heating tests. *Nuclear Engineering and Design, 306*, 24–35. | 5.pdf | https://doi.org/10.1016/j.nucengdes.2016.04.031 |
| Collin et al. (2016) | Collin, B. P., Petti, D. A., Demkowicz, P. A., & Maki, J. T. (2016). Comparison of fission product release predictions using PARFUME with results from the AGR-1 safety tests. *Nuclear Engineering and Design, 301*, 378–390. | 6.pdf (7.pdf identical) | https://doi.org/10.1016/j.nucengdes.2016.03.023 |
| Hales et al. (2021) | Hales, J. D., Jiang, W., Toptan, A., & Gamble, K. A. (2021). Modeling fission product diffusion in TRISO fuel particles with BISON. *Journal of Nuclear Materials, 548*, 152840. | 8.pdf | https://doi.org/10.1016/j.jnucmat.2021.152840 |
| Collin et al. (2015) | Collin, B. P., Petti, D. A., Demkowicz, P. A., & Maki, J. T. (2015). Comparison of silver, cesium, and strontium release predictions using PARFUME with results from the AGR-1 irradiation experiment. *Journal of Nuclear Materials, 466*, 426–442. Cited by Hales et al. (2021) as ref. [3] for Table 1; metadata confirmed through the Crossref API on 2026-11-06; the paper itself is not in the paper set and was NOT opened (NOT VERIFIED as a data source) | none | https://doi.org/10.1016/j.jnucmat.2015.08.033 |
| Collin et al. (2021) | Collin, B., Jiang, W., Gamble, K., Gardner, R., Hales, J., Haugh, B., Novascone, S., Roche, J., Spencer, B., & Toptan, A. (2021). TRISO fuel performance modelling with BISON. *Journal of Physics: Conference Series, 2048*(1), 012012. | 4.pdf | https://doi.org/10.1088/1742-6596/2048/1/012012 |
| ENSDF evaluations | Evaluations listed per nuclide in Table 20 (full references in Section 12) | none | via IAEA LiveChart API |
| Kellett et al. (2009) | JEFF-3.1/-3.1.1 decay data and fission yields sub-libraries (JEFF Report 20), served by the IAEA LiveChart API | none | https://www.oecd-nea.org/jcms/pl_14322 |

## 4. Geometry

### Table 1. AGR-1 Baseline particle and compact (Morris et al., 2016; Collin et al., 2016)

@@geom AGR1-B-M16 AGR1-NOM-C16 AGR1-NOM-M16@@

The Baseline set G-01 to G-05 (Morris et al., 2016, Table 2) is the recommended STRATO AGR-1 reference geometry. Collin et al. (2016) give only rounded nominal values (G-10 to G-14).

### Table 2. AGR-1 characterisation data by fuel variant (IAEA, 2012, Table 7.14)

@@geom AGR1-T714 AGR1-T714-B AGR1-T714-V1 AGR1-T714-V2 AGR1-T714-V3 AGR1-SPEC AGR1-NOM-T1674@@

Table 7.14 closes the AGR-1 kernel-density gap left open by 5.pdf and 6.pdf. The per-layer standard deviations of the coating thicknesses were not transcribed in the audit notes (Section 11).

### Table 3. TECDOC-978 reference TRISO designs (IAEA, 1997, Table 2-1)

@@geom T978-FRG T978-USFIS T978-USFER T978-JP T978-USSR T978-CN@@

### Table 4. TECDOC-978 Table 2-1 operating and element data (design specification; p. 8 to 9, idx 15 to 16; status verified)

| No. | Variable | FRG | US fissile | US fertile | Japan | USSR | China | Unit |
|---|---|---|---|---|---|---|---|---|
| G-D1 | Fuel element and graphite | sphere, A3-3 | block, H-451 | block, H-451 | block, IG-110 | sphere, 30PG / MPG-6 | sphere, Chinese graphite | n/a |
| G-D2 | Protective overcoating | about 200 | none | none | about 200 | blank | about 200 | $\mathrm{\mu m}$ |
| G-D3 | Volume packing fraction | 10 | 51 (US combined) | (combined) | 30 | 12 to 15 (fn. 2) | 4 | $\%$ |
| G-D4 | Design temperature limit | 1250 | 700 to 1250 (US combined) | (combined) | 1495 | 1400 (fn. 2) | 1000 | °C |
| G-D5 | Burnup | 8 to 10 | $\le 26$ | $\le 7$ | 3.6 | 8.8 (fn. 2) | 11.0 | % FIMA |
| G-D6 | Fast fluence | 2 to 3 (E > 16 fJ) | 4.5 (E > 29 fJ, US combined) | (combined) | 1.3 (E > 29 fJ) | 3.0 (fn. 2) | 1.0 | $10^{25}\ \mathrm{m^{-2}}$ |
| G-D7 | Heavy metal per element | 7 | blank | blank | 13.5 | 7.0 (fn. 2) | 5 | $\mathrm{g}$ |

Footnote (2): specification value for the VGM reactor. Footnote (1) on the Japanese SiC thickness: 25 $\mathrm{\mu m}$ is the first HTTR core specification, actual 25 to 35 $\mathrm{\mu m}$.

### Table 5. HTR-10 particle (IAEA, 1997, Table 2-3) and JAERI heating-test particle 80FPC1 (IAEA, 1997, Table 4-7)

@@geom HTR10-DES HTR10-MEAS JAERI-80FPC1@@

The JAERI 80FPC1 particle (G-109 to G-113) is the particle of the measured release data in Table 39, which makes it the only fully specified measured-geometry plus measured-release pair in the paper set.

### Table 6. CRP-6 benchmark particles (IAEA, 2012, Tables 10.1 to 10.3)

@@geom CRP6-SENS CRP6-UOUT CRP6-P4K3 CRP6-HRB22 CRP6-K6 CRP6-EU1BIS CRP6-HTRPM@@

### Table 7. Outer radii $R_1..R_5$ in $\mathrm{\mu m}$ (calculated with Eq. R.4 from the rows listed; checked by script, Section 10)

@@radii AGR1-B-M16 AGR1-NOM-C16 T978-FRG T978-USFIS T978-USFER T978-JP T978-CN HTR10-DES HTR10-MEAS JAERI-80FPC1 CRP6-SENS CRP6-P4K3 CRP6-HRB22 CRP6-K6 CRP6-EU1BIS CRP6-HTRPM@@

The AGR-1 Baseline gives $2R = 788.4\ \mathrm{\mu m}$ against the nominal 780 $\mathrm{\mu m}$ of IAEA (2012, p. 395, idx 408). The USSR design is omitted because its buffer and IPyC are printed as ranges.

## 5. Arrhenius register

All rows use Eq. (R.1). "2T" rows are the two terms of one summed correlation; separate "1T" rows with different temperature ranges (for example A2-11 and A2-12) are alternative branches and must not be summed. "none printed" means the table cell is blank. FRG, USA, Japan, UK and Russian Federation rows are kept separate even where the numbers coincide.

### 5.1 Gas constant used by each source

| No. | Source | $R_g$ ($\mathrm{J\ mol^{-1}\ K^{-1}}$) | Location | Status |
|---|---|---|---|---|
| RG-1 | IAEA (1997), Appendix A.1, A.2 and Eq. (4-10) | 8.3143 | Eq. (A-1), p. 435, idx 439; p. 165, idx 171; Table A-7 notes p. 470, idx 474 | verified |
| RG-2 | IAEA (1997), chapters 3 and 5, Table A-11 region | 8.314 | PDF pages 60, 230, 256, 413, 485 (audit 03) | verified |
| RG-3 | IAEA (2012) | 8.3145 | Eq. (10.5a), p. 532, idx 545 | verified |
| RG-4 | Collin et al. (2016) | 8.3145 | Eq. (3), p. 381, idx 3 | verified |
| RG-5 | Collin et al. (2021) | 8.3145 | Eq. (71), p. 10, idx 10 | verified |
| RG-6 | Hales et al. (2021) | not printed ("R is the universal gas constant"; full-text search of 8.pdf for "8.314" returns 0 hits, B6 audit) | Eq. (5), p. 2, idx 1; text p. 3, idx 2 | verified (absence); value used by BISON NOT VERIFIED |

Using 8.3142 instead of 8.3143 changes $D$ at 1600 °C by $-1.2\times10^{-4}$ to $-3.8\times10^{-4}$ relative (calculated, check A). This is negligible against the order-of-magnitude uncertainty of the coefficients. Use the $R_g$ of the set being reproduced.

### Table 8. Diffusion coefficients in UO2 kernel (IAEA, 1997, Table A-1)

@@arr T978-A1@@

### Table 9. Diffusion coefficients in LTI pyrocarbon (IAEA, 1997, Table A-2)

@@arr T978-A2@@

### Table 10. Diffusion coefficients in SiC (IAEA, 1997, Table A-3 and Eq. 4-10)

@@arr T978-A3 T978-EQ410@@

### Table 11. Buffer and failed-layer conventions

@@arr T978-BUF C16-BUF C16-FAIL H21-FAIL@@

### Table 12. Diffusion coefficients in matrix graphite (IAEA, 1997, Table A-4)

@@arr T978-A4@@

### Table 13. Diffusion coefficients in structural graphite (IAEA, 1997, Table A-5; outside the particle, recorded for completeness)

@@arr T978-A5@@

### Table 14. Collin et al. (2016) Table 2 set (PARFUME AGR-1 safety-test analysis)

@@arr C16-T2@@

Table 2 note (Collin et al., 2016, p. 381, idx 3): UO2 values are used for the UCO kernel for lack of UCO data; the fluence-dependent Cs-in-SiC term is not used; the Kr kernel and SiC coefficients are accident-condition values. The Kr matrix row equals the TECDOC-978 "I in pores" row and the Kr SiC row equals the Japanese Xe-in-SiC row (check X).

### Table 15. Hales et al. (2021) Table 1 set (BISON)

@@arr H21-T1@@

Hales et al. (2021) cite Table 1 to Collin et al. (2015) (ref. [3], p. 3, idx 2) and state on p. 6 (idx 5) that "Table 1 is based on" the German models of IAEA (1997). Table 1 prints only $D_{0,1}$ and $Q_1$ and adds that "additional terms targeting high-temperature behavior are often used" (p. 3, idx 2). Check X confirms each row equals the FRG first term; the second Cs kernel term (A1-02), the second Cs SiC term (A3-02) and the second Sr SiC term (A3-04) are absent from the printed table. This resolves the audit-06 open item "cross-check of H21 Table 1 against TECDOC-978" (calculated comparison of two verified transcriptions).

### Table 16. IAEA (2012) Table 7.1 reduced kernel coefficients and Eq. (10.9)

@@arr T1674-T71 T1674-EQ109@@

### Table 17. Cs-in-SiC fluence factor $e^{\Gamma/5}$ of Eq. (R.2) (calculated; check F)

| No. | $\Gamma$ ($10^{25}\ \mathrm{m^{-2}}$, E > 16 fJ) | $e^{\Gamma/5}$ | Note |
|---|---|---|---|
| FF-1 | 0 | 1.000 | value used by Collin et al. (2016) and Hales et al. (2021) Table 1 |
| FF-2 | 1 | 1.221 | |
| FF-3 | 2 | 1.492 | IAEA (2012) Eq. (10.9) check value $1.011\times10^{-16}\ \mathrm{m^2\ s^{-1}}$ at 1600 °C |
| FF-4 | 3 | 1.822 | |
| FF-5 | 4.5 | 2.460 | |

Caveat: AGR-1 fluences (2.43 to 4.28 $\times10^{25}\ \mathrm{m^{-2}}$) are quoted for E > 0.18 MeV (29 fJ), a different cut-off from the 16 fJ of Eq. (R.2). A conversion factor between the two cut-offs is NOT VERIFIED in the paper set, so the factor cannot be applied to AGR-1 without an external spectrum-specific ratio.

### 5.2 UCO kernel coefficients: NOT VERIFIED

No UCO kernel diffusion coefficients were found in any source of the paper set. IAEA (1997, p. 437, idx 441) states that the US uses "a separate set of constants used for UCO kernels which are slightly higher than those for UO2" but does not tabulate them; IAEA (2012) contains no UCO diffusion data (keyword search, audit 04); Collin et al. (2016) and Collin et al. (2021) use UO2 values "due to lack of data for UCO kernels". Every AGR-1 kernel coefficient in this register is therefore a UO2 surrogate.

### 5.3 Interface, validity and uncertainty statements (IAEA, 1997; verified)

| No. | Statement | Location |
|---|---|---|
| V-1 | Partition coefficient at layer interfaces assumed 1 (concentration continuity); measured values range from about 0.3 to 3 and may change with temperature and fluence | p. 436, idx 440 |
| V-2 | Rapid diffusive transport (buffer-like) recommended for metallic fission products in LTI-PyC above 1900 °C | p. 437, idx 441 |
| V-3 | Matrix graphite uncertainty at least one order of magnitude, Ag up to three orders | p. 447, idx 451 |
| V-4 | US Integrated Failure and Release Model assumes no Cs or Sr diffusion through intact SiC | p. 447, idx 451 |
| V-5 | FRG combined Sr-in-SiC fit "significantly underestimates the experimental retention quality of SiC for strontium at 1600 °C" | p. 443, idx 447 |
| V-6 | High-quality SiC release below 1600 °C so low that sometimes only an upper limit can be given (see A3-13) | p. 447, idx 451 |
| V-7 | Log-normal distribution of $D$ reproduced R2-K13/1 Cs-137 release at 1600 °C (about 2000 particles) | p. 447, idx 451 |
| V-8 | No IPyC/OPyC distinction; PyC data are "LTI pyrocarbon" | Tables A-2, p. 442, idx 446 |

### Table 18. Collin et al. (2016) diffusivity correction factors, $D_c = F\,D_m$ (Eq. 4, p. 383, idx 5; status verified)

| No. | Variable | Nuclide / layer | Phase and temperature | Value of $F$ | Location |
|---|---|---|---|---|---|
| CF-1 | irradiation-phase factor | Cs, kernel | irradiation | $1.25\times10^{-2}$ | p. 384, idx 6 |
| CF-2 | irradiation-phase factor | Cs, SiC | irradiation | 0.6 | p. 384, idx 6 |
| CF-3 | safety-test factor (matrix-corrected) | Ag, SiC | 1600 / 1700 / 1800 °C | $2\times10^{-4}$ to $4\times10^{-3}$ / $8\times10^{-4}$ to $2\times10^{-3}$ / $2\times10^{-3}$ to $4\times10^{-2}$ | Table 4, p. 384, idx 6 |
| CF-4 | safety-test factor (no matrix correction) | Ag, SiC | 1600 / 1700 / 1800 °C | $6\times10^{-3}$ to $7\times10^{-1}$ / $3\times10^{-2}$ to $10^{-1}$ / $10^{-2}$ to $8\times10^{-2}$ | p. 383, idx 5 |
| CF-5 | safety-test factor | Cs, kernel | 1600 / 1700 / 1800 °C | $4\times10^{-3}$ to $4\times10^{-2}$ / $7\times10^{-4}$ (5-1-1 only; text says $10^{-4}$ to $7\times10^{-4}$) / $10^{-3}$ to $3\times10^{-3}$ | Table 5, p. 386, idx 8 |
| CF-6 | safety-test factor (1600 °C only) | Cs, SiC | 1600 °C | $5\times10^{-3}$ to $6\times10^{-2}$ (matrix-corrected); $7\times10^{-3}$ to $10^{-1}$ (uncorrected) | Table 6, p. 386, idx 8 |
| CF-7 | rate match, compact 4-3-2 | Kr, PyC | 1800 °C | reduce by about 20 % | p. 388, idx 10 |
| CF-8 | none derivable | Sr | all | none | p. 387, idx 9 |

Per-compact values (Tables 4 to 6 of the source) are carried in the notes of `release_measurements.csv` for Ag, and here for Cs: Table 5 kernel $F$: 3-3-2 $4\times10^{-2}$, 4-1-2 $2\times10^{-2}$, 6-4-1 $4\times10^{-3}$, 3-3-1 n/c, 5-1-1 $7\times10^{-4}$, 3-2-3 $10^{-3}$, 4-3-2 n/c, 4-4-1 $3\times10^{-3}$, 5-1-3 $3\times10^{-3}$. Table 6 SiC $F$: 3-2-2 $3\times10^{-2}$, 4-3-3 $9\times10^{-3}$, 5-3-3 $6\times10^{-2}$, 6-2-1 $8\times10^{-3}$, 6-4-3 $5\times10^{-3}$. These factors are compact-specific and depend on PARFUME's irradiation-phase corrections; they must not be used as universal constants.

### Table 19. FRG set evaluated with Eq. (R.1), $R_g = 8.3143$, $\Gamma = 0$ (calculated; reproduced by check A to 4 significant figures)

| No. | Set (rows) | $D$ at 1000 °C ($\mathrm{m^2\ s^{-1}}$) | $D$ at 1600 °C | $D$ at 1800 °C |
|---|---|---|---|---|
| DC-1 | Cs, UO2 (A1-01 + A1-02) | $1.498\times10^{-16}$ | $1.250\times10^{-13}$ | $6.972\times10^{-13}$ |
| DC-2 | Sr, UO2 (A1-03) | $2.093\times10^{-23}$ | $5.421\times10^{-17}$ | $1.114\times10^{-15}$ |
| DC-3 | Ag, UO2 (A1-04) | $1.139\times10^{-15}$ | $1.678\times10^{-13}$ | $4.664\times10^{-13}$ |
| DC-4 | Cs, PyC (A2-01) | $4.911\times10^{-17}$ | $4.061\times10^{-14}$ | $1.606\times10^{-13}$ |
| DC-5 | Sr, PyC (A2-02) | $1.902\times10^{-14}$ | $7.382\times10^{-12}$ | $2.501\times10^{-11}$ |
| DC-6 | Ag, PyC (A2-03) | $2.547\times10^{-15}$ | $2.691\times10^{-13}$ | $6.984\times10^{-13}$ |
| DC-7 | Cs, SiC (A3-01 + A3-02) | $4.092\times10^{-19}$ | $9.223\times10^{-17}$ | $1.832\times10^{-15}$ |
| DC-8 | Sr, SiC (A3-03 + A3-04) | $4.661\times10^{-18}$ | $2.462\times10^{-15}$ | $2.936\times10^{-14}$ |
| DC-9 | Ag, SiC (A3-05) | $5.436\times10^{-18}$ | $3.638\times10^{-15}$ | $1.378\times10^{-14}$ |

## 6. Radioactive decay data

### Table 20. ENSDF adopted decay data (evaluated; via IAEA LiveChart API, extraction 2026-11-05)

@@decay@@

Year conversion used by LiveChart: 1 y = 31 556 925.97 s (365.2422 d). The NuDat 3 chart carries slightly different half-lives for several nuclides (for example Cs-137 30.007(23) y); the CSV column `crosscheck_NuDat3_chart_half_life` keeps them as a sensitivity case only.

### Table 21. Half-lives printed in the IAEA documents (cross-check only; status verified as printed)

| No. | Nuclide | Printed $t_{1/2}$ | Source and location | ENSDF value (Table 20) |
|---|---|---|---|---|
| HL-1 | Cs-137 | 30.07 y | IAEA (2012) Table 8.1, p. 435, idx 448 | 30.08(9) y |
| HL-2 | Sr-90 | 28.78 y | same | 28.91(3) y |
| HL-3 | Cs-134 | 2.066 y | same | 2.0652(4) y |
| HL-4 | Ag-110m | 249.8 d | same | 249.83(4) d |
| HL-5 | Ag-111 | 7.454 d | same | 7.45(1) d |
| HL-6 | Kr-85 | 10.783 y | same | 10.739(14) y |
| HL-7 | I-131 | 8.021 d | same | 8.0252(6) d |
| HL-8 | Ag-110m | 250.4 d | IAEA (1997), p. 165, idx 171 | 249.83(4) d |
| HL-9 | Kr-88 | $\lambda = 6.88\times10^{-5}\ \mathrm{s^{-1}}$ (2.80 h, calculated) | IAEA (2012) Eq. (5.4), p. 217, idx 230 | not retrieved |

## 7. Fission yields

### Table 22. Thermal fission yields for STRATO nuclides (JEFF-3.1.1 primary, ENDF/B via NuDat 3 as cross-check)

@@fy Ag-110m Ag-111 Ag-109 Cs-134 Cs-133 Cs-137 Sr-89 Sr-90 Y-90 Ba-137m Eu-153 Eu-154 Kr-85 Kr-85m I-131@@

Precursor rows (Kr-90, Rb-90, Rb-90m, Kr-89, Rb-89, I-137, Xe-137, Ag-111m, Cs-134m, Y-90m, Ba-137, Eu-154m) are in `fission_yields.csv`. Rules for use (audit 07): use the ground-state CFY for Kr-85 (2.8575e-3 for U-235), not the Kr-85m CFY; do not combine the Y-90 or Ba-137m CFY with a parent-decay source (Eq. R.6b); Ag-110m, Cs-134 and Eu-154 need activation sources, whose capture cross sections and isomeric ratios are NOT VERIFIED (Section 11).

### Table 23. PARFUME "effective" yields printed by Hales et al. (2021) and Collin et al. (2021) (status verified as printed; yield type NOT VERIFIED)

| No. | Species | $^{235}$U enrichment $\varepsilon$ < 17.5 % | $\varepsilon \ge 17.5$ % | Unit | Location |
|---|---|---|---|---|---|
| PY-1 | Ag | $1.31625\times10^{-3}\,b^{0.55734}$, $b = \max(1.0, Bu)$ | $8.24492\times10^{-4}\,b^{0.53853}$ | atoms per fission | Hales et al. (2021) Eq. (6), p. 3, idx 2; Collin et al. (2021) Eqs. (51), (55), p. 8 to 9, idx 8 to 9 |
| PY-2 | Cs | 0.14 | 0.16 | atoms per fission | Hales Eq. (7); Collin (2021) Eqs. (52), (56) |
| PY-3 | Sr | $0.11754\,b^{-0.21762}$, $b = \max(0.6, Bu)$ | $0.11819\,b^{-0.15778}$ | atoms per fission | Hales Eq. (8); Collin (2021) Eqs. (53), (57) |
| PY-4 | Kr + Xe | 0.297 | 0.297 | atoms per fission | Collin (2021) Eqs. (54), (58) only |

$Bu$ in % FIMA. AGR-1 (19.74 % enrichment) falls in the right-hand column. Neither paper states whether these are independent or cumulative yields, or the basis of the burnup dependence; the PARFUME Theory and Model Basis report (INL/EXT-08-14497) is needed. Hales et al. (2021) apply them with decay switched off.

## 8. Temperature histories

### Table 24. AGR-1 safety-test procedure (Morris et al., 2016) and schedule as modelled by PARFUME (Collin et al., 2016)

@@temp AGR1-ST@@

Ramp durations and cumulative times are calculated from the printed rates; they reproduce the printed totals of 334, 336 and 338 h (check T). The 30 °C start temperature is an assumption, hence NOT VERIFIED on the first ramp. Time origin warning: 5.pdf figures use "elapsed time at temperature", while 6.pdf event times appear to include the ramps (about 31 to 35 h); this offset is an audit inference.

### Table 25. AGR-1 as-run deviations from the plan

@@temp AGR1-ASRUN@@

### Table 26. AGR-1 irradiation temperatures (compact TAVA from as-run thermal analysis; capsule values preliminary)

@@temp AGR1-IRR AGR1-TAVA AGR1-CAP@@

Daily compact temperatures (Hawkes et al., 2012, used by PARFUME at the OPyC surface) are not in the paper set. Within a compact the temperature spreads "several hundred degrees" (Collin et al., 2016, p. 380, idx 2).

### Table 27. CRP-6 post-calculation and prediction heating schedules (IAEA, 2012, Tables 10.11 and 10.12)

@@temp CRP6-6a CRP6-7 CRP6-8 CRP6-9 CRP6-10 CRP6-11@@

Each printed triple (T, time to reach T, time at T) is one row: "Ramp" is the time to reach T and "Hold" the time at T. Cumulative times reproduce all printed totals (check T).

### Table 28. CRP-6 sensitivity cases 1 to 5 (IAEA, 2012, Table 10.5; synthetic benchmarks)

@@temp CRP6-1a CRP6-1b CRP6-2 CRP6-3 CRP6-4 CRP6-5 CRP6-CH9@@

### Table 29. JAERI irradiation and heating conditions (IAEA, 1997, Tables 4-8 and 4-9)

@@temp JAERI@@

## 9. Experimental release data

### Table 30. Measurement basis and uncertainties (AGR-1 unless stated; status verified)

| No. | Variable | Value | Location |
|---|---|---|---|
| MU-1 | Release fraction normalisation | cumulative release / calculated compact inventory (as-run neutronics, Sterbentz et al., 2015) | Morris et al. (2016) p. 28, idx 4; Collin et al. (2016) p. 379, idx 1 |
| MU-2 | Inventory uncertainty | 7 % to 15 % depending on isotope | Morris et al. (2016) p. 28, idx 4 |
| MU-3 | Gamma counting uncertainty | 10 % | p. 27, idx 3 |
| MU-4 | Beta/gamma counting after leaching | 25 % | p. 27, idx 3 |
| MU-5 | CCCTF cup collection efficiency at 1600 °C (typical; determined per test) | Ag about 98 %, Cs about 84 %, Eu about 8 %, Sr about 25 % | p. 27, idx 3 |
| MU-6 | FACS plate collection efficiency (1600 °C calibration) | Ag 62 ($\pm 4.4$) %, Cs 66 ($\pm 21$) %, Eu 51 ($\pm 16$) %, Sr 55 ($\pm 10$) % | p. 28, idx 4 |
| MU-7 | Furnace thermocouple uncertainty | $\pm 1$ % of reading | p. 25, idx 1 |
| MU-8 | Kr-85 trap counting period | typically 2 to 4 h | p. 25, idx 1 |
| MU-9 | Sr-90 recovery | leaching efficiency cannot be estimated (no gamma signature) | p. 27, idx 3; p. 31, idx 7 |
| MU-10 | Error bars on release curves; numerical detection limits for Ag, Cs, Sr, Eu | not given (NOT VERIFIED) | both papers |
| MU-11 | One particle as compact fraction | $2.4\times10^{-4}$ (printed); $1/4145 = 2.41\times10^{-4}$ (calculated) | Morris et al. (2016) p. 29, idx 5 |
| MU-12 | JAERI tests | about 100 particles per test; no coating failure detected by Kr-85 monitoring | IAEA (1997) p. 162, idx 168 |

Printed-table values (class "M printed table") and text statements ("M text", "M-UB") are kept apart from digitised values (R-dig, R-vis) in every table below through the Uncertainty / data class column and the Status column. The PARFUME column is model output printed or plotted in the same source; it is a calculated value, carried only for code-to-code comparison.

Applicability to a single intact particle (CSV column `particle_applicability`, B6 audit). Every AGR-1 value is a compact-level quantity: release from one compact of about 4145 particles divided by the calculated compact inventory, so one particle's full inventory is $2.4\times10^{-4}$ of the compact (MU-11). For STRATO, which models one particle, this means:

- Compacts with particles that have failed SiC (Tables 31 to 36, group "failed SiC") measure the failed particles, not diffusion through intact coatings. They apply to a STRATO failed-layer case only, and only after dividing by the number of failed particles of Table 38.
- For compacts with intact particles only, the compact release fraction equals the mean particle release fraction if all particles release alike. The matrix-corrected values of Collin et al. (2016) (release after the first 24 h) are the nearest AGR-1 analogue of a single intact particle. Values without that correction include in-pile inventory already outside the SiC; Morris et al. (2016) state that Ag, Eu and Sr release was dominated by release during irradiation. The "<" values (evidence class "measured bound") bound the mean intact-particle release, include background and matrix contamination, and are inequality constraints, never point targets.

JAERI Table 4-9 values come from batches of about 100 loose particles without matrix (MU-12), which is closer to the single-particle setting, but the end-of-irradiation profile needed to simulate them is not given (GAP-24). CRP-6 values are fuel-sphere quantities.

### Table 31. AGR-1 Cs-134 compact release

@@rel AGR-1 Cs-134@@

### Table 32. AGR-1 Cs upper bounds (all Cs isotopes)

@@rel AGR-1 Cs@@

### Table 33. AGR-1 Ag-110m compact release

@@rel AGR-1 Ag-110m@@

### Table 34. AGR-1 Sr-90 compact release (not matrix-corrected; Collin et al., 2016, Fig. 12 labels the measured bars "PIE")

@@rel AGR-1 Sr-90@@

### Table 35. AGR-1 Eu-154 compact release (Morris et al., 2016 only; not modelled by Collin et al., 2016)

@@rel AGR-1 Eu-154@@

### Table 36. AGR-1 Kr-85 release, upper bounds and detection limit

@@rel AGR-1 Kr-85@@

### Table 37. AGR-1 Ce-144

@@rel AGR-1 Ce-144@@

### Table 38. AGR-1 particles with failed SiC after heating

@@rel AGR-1 counts@@

### Table 39. JAERI accident-simulation tests on irradiated UO2 TRISO particles (IAEA, 1997, Table 4-9; measured)

@@rel JAERI all@@

Blank cells of the printed table are omitted (33 values transcribed, check E). The end-of-irradiation concentration profile needed to simulate these tests is not given in IAEA (1997).

### Table 40. CRP-6 measured outcomes quoted in text (IAEA, 2012; measured curves are graphical only)

@@rel CRP-6 all@@

## 10. Verification

Script: `scripts/derivation_checks/D5_parameter_register.py`, run with `.venv/Scripts/python.exe -I -X utf8 D5_parameter_register.py`. It reads the six CSV files and writes nothing. Output (2026-11-06), abridged to one line per check family plus every non-schema line:

```text
@@CHECKOUT@@
```

What the checks establish:

- Schema (S): every CSV has one header row, a source column, a status column restricted to the three status values, unique identifiers and no ragged rows.
- Geometry (G): Eq. (R.4) recovers every thickness symbolically; all 80 tabulated outer radii are recomputed exactly.
- Decay (L): Eq. (R.5) holds for all 14 radioactive rows to $4\times10^{-7}$ relative; the uncertainty propagation is the exact first derivative.
- Fluence factor and Eq. (10.9) (F): the register's coefficients reproduce the printed check value $1.011\times10^{-16}\ \mathrm{m^2\ s^{-1}}$.
- Cross-set equality (X): the 27 "equals" claims linking Collin et al. (2016) and Hales et al. (2021) rows to IAEA (1997) rows hold exactly.
- Transcription (A): the 27 FRG reference values of audit 03 are reproduced to 4 significant figures, so no exponent or activation-energy slip entered the CSV.
- Benchmark (B): the two-branch Cs kernel row pair A1-01 + A1-02 reproduces both printed CRP-6 case 1 analytical values (0.4674 vs 0.4673; 0.99999959 exactly), while the single Table 7.1 branch gives 0.4616. STRATO must therefore use both Cs kernel terms for CRP-6 case 1.
- Source rule (Y): Eq. (R.6b) gives the cumulative yield at steady state; the alternative over-counts by exactly the parent yield.
- Histories (T) and release data (E): ramp durations, cumulative sums, printed totals, value ranges, failed-particle fractions and the Table 4-9 cell count all agree.
- B6 audit (V): an independent re-reading of all 81 Ag, Cs and Sr rows of IAEA (1997) Tables A-1 to A-5, Collin et al. (2016) Table 2 and Hales et al. (2021) Table 1 from page renders gives identical $D_0$, $Q$ and temperature limits; the gas constant per set is as printed; every CSV row carries an evidence class from the Section 0.2 vocabulary; every "<" release value is a measured bound; the 12 yield rows of Ag-110m, Cs-134, Eu-154 and their isomer partners are flagged as no fission-yield source. A mutation test (A3-16 $D_0$ changed from $6.8\times10^{-11}$ to $6.8\times10^{-12}$ in a scratch copy) makes the V transcription check FAIL, so it is not vacuous. ENSDF half-lives, decay modes, cut-off dates and all JEFF-3.1.1 yields were also compared cell by cell with a fresh LiveChart API download on 2026-11-06 (script `scripts/data_register/b6_nuclear_crosscheck.py`: 189 value checks and 15 activation-product class checks, 204 of 204 PASS; server `Extraction_date` 2026-11-06 on all 20 responses); see `data/reference/B6_audit_log.md`.

Limiting cases verified: $\Gamma \to 0$ in Eq. (R.2); Booth $F(D't = 0) = 0$ exactly; stable nuclides have $\lambda = 0$.

## 11. Gaps (NOT VERIFIED) and how to close them

| No. | Gap | Affects | How to obtain |
|---|---|---|---|
| GAP-01 | UCO kernel diffusion coefficients (US constants "slightly higher than UO2") not tabulated anywhere in the paper set | AGR-1 kernel D (A1, C16, H21 rows) | PARFUME Theory and Model Basis report (INL/EXT-08-14497); later INL AGR-1/AGR-2 kernel analyses |
| GAP-02 | No separate IPyC and OPyC coefficients; no HTI-PyC table | layers 3 and 5 | IAEA (1997) ref. [4] (German HBK data set); AGR PIE reports |
| GAP-03 | Temperature validity ranges absent for FRG and USA rows (except USA Cs and Sr in PyC) | extrapolation limits for all FRG rows | original references [4], [25], [28] of IAEA (1997) |
| GAP-04 | Fluence cut-off conversion between E > 16 fJ (Eq. R.2) and E > 29 fJ (AGR-1 values) | Cs-in-SiC term for AGR-1 | spectrum-specific ratio from the AGR-1 as-run physics report (Collin, 2015, INL/EXT-10-18097 Rev. 3; Sterbentz et al., 2015) |
| GAP-05 | Nuclide label of rows A3-18 and A3-19 (printed only on the A3-17 line) | Japanese Xe in SiC | IAEA (1997) refs. [35], [36] |
| GAP-06 | Assignment of the three FRG Ag matrix rows A4-03 to A4-05 to graphite grades | Ag in matrix | IAEA (1997) ref. [40] |
| GAP-07 | Source of IAEA (2012) Table 7.1 gas row T71-05 (cited [228], absent from TECDOC-978) | Kr/Xe kernel D | IAEA (2012) ref. [228] |
| GAP-08 | $R_g$ used by Hales et al. (2021): the paper prints no value (B6 full-text search), so the value inside BISON is unknown | reproduction of BISON results | BISON documentation (mooseframework.org/bison) |
| GAP-09 | AGR-1 coating-thickness standard deviations and per-variant SiC density | geometry uncertainty | re-read IAEA (2012) Table 7.14, p. 397, idx 410; Hunn et al. (2007); Maki (2009) |
| GAP-10 | Normalisation of the AGR-1 U contamination $< 5\times10^{-7}$ | contamination source term | Hunn et al. (2007) characterisation report |
| GAP-11 | AGR-1 daily compact irradiation temperatures and TAVA of compact 5-1-1; basis of the preliminary capsule values of Table 7.15 | irradiation-phase T(t) | Hawkes et al. (2012); Collin (2015) as-run report |
| GAP-12 | Fuel type and laboratory of compact 5-1-1 | grouping of results | Demkowicz et al. (2015a) |
| GAP-13 | Full furnace temperature traces of each AGR-1 safety test (only plan, deviations and visual end times exist) | as-run T(t) | Morris et al. (2014) HTR2014-31135; ORNL/INL test reports |
| GAP-14 | Time origin of the 6.pdf event times (inferred offset of about 31 to 35 h) | timing of failures | Collin et al. (2015) or the authors' data files |
| GAP-15 | RESOLVED by B6: 6.pdf Fig. 6 is on p. 384, idx 6 (caption located by text search, figure rendered; the five digitised values re-read on the render agree) | location field | none |
| GAP-16 | Compact "4-4-2" in 6.pdf Fig. 12 (row E-077): label confirmed as printed (B6 render at 4x) but absent from every table of both papers | Sr data set | authors or Morris et al. (2014) data |
| GAP-17 | 6-4-3 Cs: matrix-corrected $1.6\times10^{-8}$ implies about 98 % prompt release, against "30 % to 50 %" in the text | E-014, E-019 | compare with the HTR2014 data set (Morris et al., 2014) |
| GAP-18 | Error bars, numerical detection limits (Ag, Cs, Sr, Eu), Eu-155 values, capsule-level data, in-pile failed-particle counts, IMGA per-particle retention | uncertainty of all AGR-1 rows | Demkowicz et al. (2015b, 2016); Hunn et al. (2016) |
| GAP-19 | Time-resolved AGR-1 and CRP-6 release curves exist only as raster figures (5.pdf Figs. 3 to 18; 6.pdf Figs. 8, 9, 13, 14; 3.pdf Figs. 10.8 to 10.28) | transient validation | dedicated digitisation with the audit's validated pixel method |
| GAP-20 | Applicability of IAEA (2012) Table 10.3 uranium contamination to CRP-6 cases 1 to 5; case 5 cycle shape; heat-up ramps of cases 1 to 5; transition time of the 3c step | CRP-6 sensitivity inputs | IAEA (2012) Ch. 10 authors' input decks; Hales et al. (2021) case set-up |
| GAP-21 | Decay constants and source normalisation used by each CRP-6 participant | CRP-6 cases 4 and 5 | IAEA (2012) participant chapters |
| GAP-22 | Minato value printed "$2.1\times10^{4}$" (probably $2.1\times10^{-4}$, ZrC-particle study) | CRP-6 case 3a context | Minato et al. (1995), J. Nucl. Mater. 224, 85 |
| GAP-23 | Basis (per capsule or per compact) of the AGR-1 one-failure R/B $3.5\times10^{-6}$ | in-pile gas release | IAEA (2012) p. 402 source report |
| GAP-24 | End-of-irradiation inventory profile of the JAERI particles | simulation of Table 39 | JAERI reports cited as [63], [64] in IAEA (1997) |
| GAP-25 | Yield type (independent or cumulative) and burnup basis of the PARFUME effective yields (Table 23) | PARFUME-consistent source term | PARFUME Theory and Model Basis report |
| GAP-26 | Capture cross sections and isomeric ratios for Ag-109$(n,\gamma)$Ag-110m, Cs-133$(n,\gamma)$Cs-134, Eu-153$(n,\gamma)$Eu-154 in an HTGR spectrum (NuDat 3 thermal values are unit-less in the JSON and not spectrum-averaged) | activation sources | evaluated library (ENDF/B-VIII.0 or JEFF-3.3) processed with an HTGR spectrum |
| GAP-27 | ENDF/B-VIII.0 nfy values bit-identical to NuDat 3; JEFF-3.3 yields; blank U-235 Ag-110 yields in the API; meaning of `daughter_level_idx = -2`; incident energy of "thermal" yields; provenance of NuDat 3 half-lives; page of the JEFF Report 20 $10^3$ y statement | nuclear data provenance | download the library files directly (NNDC, NEA) |
| GAP-28 | Fission rate density and fission fractions (U-235, Pu-239, U-238) for AGR-1 compacts | $\dot F$ in Eq. (R.6a) | Sterbentz et al. (2015) as-run physics |
| GAP-29 | IAEA URLs for TECDOC-978 and TECDOC-1674 not opened; catalogue year 2013 vs printed 2012 | citation links | open the IAEA publication pages |
| GAP-30 | HRB-15B values quoted from Bullock (1984) by Morris et al. (2016) | historical comparison only | Bullock (1984) |

Excluded from this register by scope (verified in audit 03 but not particle diffusion data): concentration-dependent graphite diffusivity Eqs. (A-2) to (A-7), sorption isotherms (Tables A-6, A-7, including the probable misprint "52.14" for 5214), and Tables A-8 to A-19 of IAEA (1997).

## 12. References

Basu, S. K., & McCutchan, E. A. (2020). Nuclear data sheets for A = 90. *Nuclear Data Sheets, 165*, 1–329. https://doi.org/10.1016/j.nds.2020.04.001

Blachot, J. (2009). Nuclear data sheets for A = 111. *Nuclear Data Sheets, 110*, 1239–1407. https://doi.org/10.1016/j.nds.2009.04.002

Browne, E., & Tuli, J. K. (2007). Nuclear data sheets for A = 137. *Nuclear Data Sheets, 108*, 2173–2318. https://doi.org/10.1016/j.nds.2007.09.002

Collin, B., Jiang, W., Gamble, K., Gardner, R., Hales, J., Haugh, B., Novascone, S., Roche, J., Spencer, B., & Toptan, A. (2021). TRISO fuel performance modelling with BISON. *Journal of Physics: Conference Series, 2048*(1), 012012. https://doi.org/10.1088/1742-6596/2048/1/012012

Collin, B. P., Petti, D. A., Demkowicz, P. A., & Maki, J. T. (2015). Comparison of silver, cesium, and strontium release predictions using PARFUME with results from the AGR-1 irradiation experiment. *Journal of Nuclear Materials, 466*, 426–442. https://doi.org/10.1016/j.jnucmat.2015.08.033

Collin, B. P., Petti, D. A., Demkowicz, P. A., & Maki, J. T. (2016). Comparison of fission product release predictions using PARFUME with results from the AGR-1 safety tests. *Nuclear Engineering and Design, 301*, 378–390. https://doi.org/10.1016/j.nucengdes.2016.03.023

Demkowicz, P. A., Hunn, J. D., Ploger, S. A., Morris, R. N., Baldwin, C. A., Harp, J. M., Winston, P. L., Gerczak, T. J., van Rooyen, I. J., Montgomery, F. C., & Silva, C. M. (2016). Irradiation performance of AGR-1 high temperature reactor fuel. *Nuclear Engineering and Design, 306*, 2–13. https://doi.org/10.1016/j.nucengdes.2015.09.011

Gürdal, G., & Kondev, F. G. (2012). Nuclear data sheets for A = 110. *Nuclear Data Sheets, 113*, 1315–1561. https://doi.org/10.1016/j.nds.2012.05.002

Hales, J. D., Jiang, W., Toptan, A., & Gamble, K. A. (2021). Modeling fission product diffusion in TRISO fuel particles with BISON. *Journal of Nuclear Materials, 548*, 152840. https://doi.org/10.1016/j.jnucmat.2021.152840

International Atomic Energy Agency. (1997). *Fuel performance and fission product behaviour in gas cooled reactors* (IAEA-TECDOC-978; K. Verfondern, Ed.). IAEA.

International Atomic Energy Agency. (2012). *Advances in high temperature gas cooled reactor fuel technology* (IAEA-TECDOC-CD-1674) [CD-ROM]. IAEA.

International Atomic Energy Agency, Nuclear Data Section. (n.d.). *LiveChart of Nuclides, data download API v1* [Data set]. Retrieved November 5, 2026, from https://www-nds.iaea.org/relnsd/vcharthtml/api_v0_guide.html

Kellett, M., Bersillon, O., & Mills, R. (2009). *The JEFF-3.1/-3.1.1 radioactive decay data and fission yields sub-libraries* (JEFF Report 20, NEA No. 6287). OECD Nuclear Energy Agency. https://www.oecd-nea.org/jcms/pl_14322

Khazov, Yu., Mitropolsky, I., & Rodionov, A. (2006). Nuclear data sheets for A = 131. *Nuclear Data Sheets, 107*, 2715–2930. https://doi.org/10.1016/j.nds.2006.10.001

Morris, R. N., Baldwin, C. A., Demkowicz, P. A., Hunn, J. D., & Reber, E. L. (2016). Performance of AGR-1 high-temperature reactor fuel during post-irradiation heating tests. *Nuclear Engineering and Design, 306*, 24–35. https://doi.org/10.1016/j.nucengdes.2016.04.031

National Nuclear Data Center. (n.d.). *NuDat 3* [Data set]. Brookhaven National Laboratory. Retrieved November 6, 2026, from https://www.nndc.bnl.gov/nudat3/

Nica, N. (2025). Nuclear data sheets for A = 154. *Nuclear Data Sheets, 200*, 2–524. https://doi.org/10.1016/j.nds.2025.02.002

Singh, B. (2013). Nuclear data sheets for A = 89. *Nuclear Data Sheets, 114*, 1–208. https://doi.org/10.1016/j.nds.2013.01.001

Singh, B., & Chen, J. (2014). Nuclear data sheets for A = 85. *Nuclear Data Sheets, 116*, 1–162. https://doi.org/10.1016/j.nds.2014.01.001

Sonzogni, A. A. (2004). Nuclear data sheets for A = 134. *Nuclear Data Sheets, 103*, 1–182. https://doi.org/10.1016/j.nds.2004.11.001
