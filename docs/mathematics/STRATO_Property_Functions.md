# STRATO property functions and closed-form references

Companion to the Rust module `src/physics/` (directive section 17, preparation). It states what each function computes, derives each closed-form reference, and gives the limits the unit tests check. The physics of decay, generation and temperature-dependent diffusivity is the one of `STRATO_Phase2_Derivations.md` (equations P21.x and P22.x), which stays valid; its finite-volume discretisation does not, and nothing here depends on a discretisation. No function is coupled into a solver yet, and no parameter value is stored in the code: D0, Ea, half-lives, yields and fission-rate densities are always arguments.

All quantities are SI: lengths in m, time in s, concentration in mol m^-3 (or any consistent amount per volume), `D` in m^2 s^-1, `lambda` in s^-1, `T` in K, `Ea` and `Q` in J mol^-1, `R` in J mol^-1 K^-1.

## 1 Arrhenius diffusivity (`physics::arrhenius`)

One term, with `D0 > 0`, `Ea >= 0`:

$$
D(T) = D_0 \exp\!\left(-\frac{E_a}{R\,T}\right).
$$

This is the general law (decision 13). Where the parameter register documents a row as a sum of parallel mechanisms (TECDOC-978 Eq. A-1, P22.1), the code evaluates exactly that sum and nothing else:

$$
D(T) = \sum_{m} D_{0,m}\exp\!\left(-\frac{E_{a,m}}{R\,T}\right).
$$

Parallel mechanisms driven by one concentration gradient add their fluxes, `J = -(D_a + D_b) grad c` (P22.1 and its discussion), which gives the sum. The caller chooses which terms form a sum; the module never merges rows. The fluence factor of the FRG caesium-in-SiC row (P22.9) is not implemented here.

**Evaluation.** `ln D_m = ln D0_m - Ea_m / (R T)` is formed per term and the sum by log-sum-exp, `ln D = M + ln sum_m exp(ln D_m - M)` with `M = max_m ln D_m`. Then `exp(-Ea/(R T))` underflowing for one term cannot disturb the others, and `ln D` stays finite when `D` itself underflows to 0. The result carries both `D` and `ln D`.

**Units written out.**

| Quantity | Input | Conversion |
|---|---|---|
| `Ea` | tag `JoulePerMole` or `KilojoulePerMole` | `Ea[J/mol] = 1000 Ea[kJ/mol]` |
| `T` | tag `Kelvin` or `Celsius` | `T[K] = T[degC] + 273.15` |
| `D0` | m^2 s^-1 | none |
| `R` | `GasConstant` | see below |

The exponent `Ea/(R T)` is dimensionless: `(J mol^-1)/((J mol^-1 K^-1)(K))`. A temperature is refused unless it is finite and strictly positive in kelvin; a missing tag is not possible because the tag is part of the type.

**Gas constant.** `GasConstant::SI` is `N_A k_B = 6.02214076e23 x 1.380649e-23 = 8.314462618...` J mol^-1 K^-1 (both exact since 2019) and is the default. `GasConstant::custom(r)` takes the value a source printed, so that a source can be reproduced; the register prints 8.3143 for some sets and 8.3145 for others, and the module carries neither. For one term, `D(R')/D(R) = exp[(Ea/T)(1/R - 1/R')]` (P22.8); the test checks it.

**Validity range.** A term may carry the inclusive range `[T_min, T_max]` its source printed. `evaluate` reports for each term `Within`, `BelowMin`, `AboveMax` or `NotStated` and always returns the value of the formula at the requested temperature. It never clamps, floors or substitutes; what to do with an extrapolation is a decision for the caller and the site.

**Also provided.** The apparent activation energy `Q_eff = sum_m Ea_m D_m / sum_m D_m` (P22.6), and `reduced_diffusivity(D, r) = D / r^2` in s^-1, because some register rows state a reduced coefficient and the model must use one form consistently.

**Limits tested.** `Ea = 0` gives `D = D0` at every `T`; `T -> infinity` gives `D0`; `Ea/(R T) = 1` (with `R = 8`, `Ea = 8000 J/mol`, `T = 1000 K`, equal to 726.85 degC) gives `D0 e^-1`; two equal terms give `2 D`; two unequal terms give the plain sum; `Q_eff` of one term is its `Ea`.

## 2 Decay constant and half-life (`physics::decay`)

For `N` atoms each decaying with constant probability `lambda` per unit time, `dN/dt = -lambda N`, so `N = N0 exp(-lambda t)` (P21.5, P21.6). The half-life satisfies `exp(-lambda t_half) = 1/2`:

$$
\lambda = \frac{\ln 2}{t_{1/2}},\qquad t_{1/2} = \frac{\ln 2}{\lambda},\qquad \bar t = \frac1\lambda .
$$

Units: the half-life is converted to seconds first, so `lambda` is in s^-1. Minute, hour and day are exact (60, 3600, 86400 s). A year has no single length, so `TimeUnit::Years(seconds_per_year)` takes it as an argument; `JULIAN_YEAR_S = 31 557 600 s` (365.25 d) is offered for convenience, and a source that uses another year length (the register's nuclear data say which one) must pass that. A half-life must be finite and strictly positive; a stable nuclide has `lambda = 0` and no half-life, and is handled by the caller. Decay is a loss of atoms; the module has no notion of release, and decay loss is reported separately from physical release (Section 5.3).

Tests: `t_half = 1 s` gives `exp(-lambda) = 1/2`; 1440 min, 24 h, 1 d and 86400 s give the same `lambda`; `t_half = 10 s` gives `lambda = 0.0693147... s^-1`; round trip in both directions.

## 3 Volumetric source (`physics::source`)

A fission-rate density `F` (fissions m^-3 s^-1) times a yield `y` (atoms per fission) is the production rate of atoms per volume, and dividing by `N_A` converts to moles (P21.8):

$$
S = \frac{F\,y}{N_A}\quad[\mathrm{mol\,m^{-3}\,s^{-1}}],\qquad
\frac{\mathrm{m^{-3}\,s^{-1}}\cdot 1}{\mathrm{mol^{-1}}}=\mathrm{mol\,m^{-3}\,s^{-1}}.
$$

The yield is typed as independent or cumulative (`YieldKind`) and the product as direct fission product or activation product (`ProductKind`). Which yield a model must use is the modelling rule of P21.11 and P21.12: the cumulative yield when the precursors are not separate species, the independent yield when they are. The module checks no data and only carries the label with the number.

**Activation products are refused.** Ag-110m, Cs-134 and Eu-154 are made by neutron capture on Ag-109, Cs-133 and Eu-153 (P21.13 and P21.14), not by fission chains; their tiny fission yields must not be used as a source (decision 15, directive section 7). `source_*` returns `PhysicsError::DirectYieldForActivationProduct` for `ProductKind::ActivationProduct` whatever the yield kind or value, so no number can leak. A capture production model (cross sections, spectrum, isomeric fraction, flux history) is not approved and is not implemented.

The fission-rate density is not in the register (GAP-28). The function takes it as an argument and no default exists.

Tests: `F = 1e19`, `y = 0.05` gives `5e17` atoms m^-3 s^-1 and `5e17/N_A = 8.3027e-7` mol m^-3 s^-1; `F = N_A`, `y = 0.5` gives exactly 0.5; zero rate or zero yield gives zero; the source is linear in both factors; invalid input (negative, NaN, infinite) is refused.

## 4 Closed-form references (`physics::reference`)

These verify the coupled solver later. Each is the exact solution of the problem stated, and shares no code with a discretisation.

### 4.1 Decay only, well mixed

`dC/dt = -lambda C`, `C(0) = C0`. Separating variables, `dC/C = -lambda dt`, so

$$
C(t) = C_0\,e^{-\lambda t}.
$$

Limits: `lambda = 0` gives `C0`; `t = t_half` gives `C0/2`, two half-lives `C0/4`; `C0 = 0` stays 0.

### 4.2 Source only, well mixed

`dC/dt = G`, `C(0) = C0` with constant `G` (same amount unit per volume per second):

$$
C(t) = C_0 + G\,t .
$$

Limits: `G = 0` gives `C0`; `t = 0` gives `C0`.

### 4.3 Source with decay, well mixed

`dC/dt = G - lambda C`. Multiply by the integrating factor `e^{lambda t}`: `d(C e^{lambda t})/dt = G e^{lambda t}`, integrate from 0 to `t`:

$$
C(t) = C_0 e^{-\lambda t} + \frac{G}{\lambda}\left(1-e^{-\lambda t}\right)
\;\xrightarrow{\,t\to\infty\,}\; \frac G\lambda .
$$

The code forms `(1 - e^{-lambda t})/lambda` with `expm1`, so it is accurate for small `lambda t` and equals `t` exactly at `lambda = 0`, where the formula reduces to 4.2 (`(1-e^{-x})/lambda -> t - lambda t^2/2 + ...`). With `G = 0` it reduces to 4.1. Starting at `C0 = G/lambda` the concentration stays there. The half-life of the approach to `G/lambda` is `t_half` of the nuclide: at `t = t_half` and `C0 = 0`, `C = G/(2 lambda)`.

### 4.4 Decay with diffusion in a homogeneous sphere

Problem: sphere of radius `R`, constant `D`, decay `lambda`, no source, uniform `C(r,0) = C0`, absorbing surface `C(R,t) = 0`:

$$
\frac{\partial C}{\partial t} = \frac1{r^2}\frac{\partial}{\partial r}\!\left(r^2 D\frac{\partial C}{\partial r}\right) - \lambda C .
$$

Write `C = e^{-lambda t} W`. Then `dC/dt = e^{-lambda t}(W_t - lambda W)`, so `W_t = (1/r^2)(r^2 D W_r)_r` with the same boundary and initial conditions: pure diffusion. With `u = r W`, `u_t = D u_rr`, `u(0,t) = 0` (finite `W`), `u(R,t) = 0`, `u(r,0) = C0 r`. The sine series `u = sum_n b_n sin(n pi r/R) e^{-a_n t}`, `a_n = n^2 pi^2 D / R^2`, has

$$
b_n = \frac2R\int_0^R C_0\,r\sin\frac{n\pi r}R\,dr = \frac{2R\,C_0\,(-1)^{n+1}}{n\pi},
$$

so, with `sinc(x) = sin x / x` (`sinc(0) = 1`, which removes the singularity at the centre),

$$
\boxed{\;\frac{C(r,t)}{C_0}
= 2\,e^{-\lambda t}\sum_{n\ge1}(-1)^{n+1}\,\mathrm{sinc}\!\left(\frac{n\pi r}{R}\right)e^{-a_n t}\;}
$$

(the centre value is `2 e^{-lambda t} sum (-1)^{n+1} e^{-a_n t}`). Integrating over the volume, with `int_0^R r sin(n pi r/R) dr = (-1)^{n+1} R^2/(n pi)`, the inventory still inside as a fraction of the initial one is the diffusion series times `exp(-lambda t)`:

$$
\boxed{\;f_\mathrm{ret}(t) = \frac6{\pi^2}\,e^{-\lambda t}\sum_{n\ge1}\frac{e^{-a_n t}}{n^2}\;}
$$

**Release and decay loss, kept apart.** The flux through the surface of `W` is the pure-diffusion flux `J_W`, so the physical release rate is `J = e^{-lambda t} J_W` and `J_W = -d f_W/dt` times the initial inventory, with `f_W = (6/pi^2) sum e^{-a_n t}/n^2`; hence the rate per mode is `(6/pi^2) a_n e^{-(a_n+lambda) t}/n^2`. Integrating from 0 to `t`, and likewise for the decay loss rate `lambda f_ret`:

$$
f_\mathrm{rel}(t) = \frac6{\pi^2}\sum_n\frac{a_n}{a_n+\lambda}\,\frac{1-e^{-(a_n+\lambda)t}}{n^2},\qquad
f_\mathrm{dec}(t) = \frac6{\pi^2}\sum_n\frac{\lambda}{a_n+\lambda}\,\frac{1-e^{-(a_n+\lambda)t}}{n^2}.
$$

Per mode, `e^{-x} + (a/(a+lambda))(1-e^{-x}) + (lambda/(a+lambda))(1-e^{-x}) = 1` with `x = (a+lambda)t`, so `f_ret + f_rel + f_dec = (6/pi^2) sum_n 1/n^2 = 1` term by term, with the truncated sums adding to the truncated `(6/pi^2) sum_{n<=N} 1/n^2`. Here `f_rel` counts atoms that left the sphere before decaying; it is not the decay-corrected quantity `F_B` of Section 3.7.3 of the Phase 2 derivations (which divides by the surviving production).

**Limits tested.** `lambda = 0` gives the Booth series and `f_dec = 0`; at long time only `n = 1` survives, `f_ret -> (6/pi^2) e^{-(pi^2 D/R^2 + lambda) t}` and the centre ratio `2 e^{-(pi^2 D/R^2 + lambda)t}`; `D = 0` gives pure decay, `f_ret = e^{-lambda t}`, `f_rel = 0`, `f_dec = 1 - e^{-lambda t}`; `t = 0` gives `f_ret = 1` up to the truncation; the small-time law `f_rel = 6 sqrt(D t/(pi R^2)) - 3 D t/R^2` (lambda = 0); the concentration profile integrates (by Simpson quadrature, independently of the formulas above) to `f_ret`; the surface value is 0; `d f_dec/dt = lambda f_ret`; `C(r,t;lambda) = e^{-lambda t} C(r,t;0)`.

**Truncation.** Every inventory fraction above is a sum of terms bounded by `(6/pi^2)/n^2`, and `sum_{n>N} 1/n^2 < 1/N`, so keeping `N = ceil(6/(pi^2 tol))` terms leaves a tail below `tol` (absolute, on a fraction of the initial inventory). `series_terms_for_tolerance` returns it. The bound is slow at `t -> 0` (it is `10^6` terms for `10^-6`); for later times far fewer terms suffice because `e^{-a_n t}` cuts the series off. The bound does not cover the pointwise profile, whose series alternates and shows Gibbs oscillation at early times; use many terms there.

## 5 What this does not do

1. No coupling into the BTCS solver; nothing here chooses a time step, a grid or an interface treatment.
2. No parameter values: the register's rows are not read and none is embedded.
3. Decay loss and physical release are separate quantities in 4.4; a solver that tracks both can be checked against them.
4. Parent-daughter chains, activation-product production, the fluence factor, and any other mechanism of section 17's "stop" list are not implemented.
5. Numerical agreement of these references with a solver is numerical verification, not experimental validation.
