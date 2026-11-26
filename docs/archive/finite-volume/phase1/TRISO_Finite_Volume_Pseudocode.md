# TRISO Finite-Volume Pseudocode

Equations match `TRISO_Finite_Volume_Derivation.md` (same equation IDs).

## Governing equation (TRISO-FV-100)

```math
\frac{\partial c_i}{\partial t}=\frac{1}{r^2}\frac{\partial}{\partial r}\left(r^2D_i\frac{\partial c_i}{\partial r}\right)+S_i
```

## Conservative FV cell equation (TRISO-FV-155) and face conductance (TRISO-FV-140)

```math
V_j\frac{dC_j}{dt}=G_W\,(C_{j-1}-C_j)+G_E\,(C_{j+1}-C_j)+S_jV_j,\qquad G_f=\frac{A_f}{\delta_P/D_P+\delta_E/D_E}
```

## Backward Euler (TRISO-DEL-040)

```math
-G_WC_{j-1}^{n+1}+\left(\frac{V_j}{\Delta t}+G_W+G_E\right)C_j^{n+1}-G_EC_{j+1}^{n+1}=\frac{V_j}{\Delta t}C_j^n+S_j^{n+1}V_j
```

## Forward Euler bound, verification only (TRISO-FV-300)

```math
\Delta t\le\min_j\frac{V_j}{G_W+G_E}
```

This is a sufficient positivity and stability condition for the explicit verification scheme. Production uses Backward Euler.

## Release and conservation

```math
M^n=\sum_jV_jC_j^n,\qquad M_{\mathrm{rel}}^{n+1}=M_{\mathrm{rel}}^n+\Delta t\,G_RC_N^{n+1},\qquad F^n=\frac{M_{\mathrm{rel}}^n}{M^0},\qquad M^n+M_{\mathrm{rel}}^n-M^0\approx0
```

## Exact reference series (TRISO-DEL-021), same inputs as the FV run

```math
F(t)=1-\sum_nw_n\,e^{-\Lambda_nt},\qquad w_n=\frac{3\,I^{(1)}_nI_n}{R_1^3N_n},\qquad g(\Lambda_n)=0
```

## Calculation sequence

```text
READ five radii, five diffusivities, per-region cell counts, dt, t_end
READ initial kernel concentration (the supported case: S = 0 after t = 0, absorbing outer surface)
REJECT any value still equal to INSERT_VALUE_HERE and list each one

BUILD five-region interface-aligned spherical mesh
FOR each cell:
    compute exact shell volume
    compute spherical volume-centroid radius
    assign physical material and diffusivity

FOR each internal face:
    compute face area
    compute left/right half-cell distances
    compute series-resistance conductance G_f

SET centre west conductance = 0
SET absorbing outer-boundary conductance G_R

INITIALISE kernel concentration; zero in the coatings

FOR each timestep:
    solve the Backward-Euler tridiagonal system (subtraction-free elimination)
    compute boundary release rate G_R * C_N from the new state
    accumulate released inventory with the same implicit boundary flux
    compute remaining inventory and mass-balance residual
    check concentration >= 0 and 0 <= release fraction <= 1
    write requested time-series rows

WRITE timeseries.csv and final_profile.csv

EXACT SERIES (separate analytical code, same radii and diffusivities):
    FOR Lambda in (0, 45 / t_first_output]:
        count eigenvalues with the Sturm oscillation count of the regular solution
    ISOLATE every root of g(Lambda) = u(R; Lambda) by bisection on that count
    FOR each root:
        propagate (u, q) through the five regions, with q jumping at each
        interface by (D_next - D_prev) * u / R_i
        integrate I1, I, N region by region and form the weight w_n
    EVALUATE F(t) for every output time at or beyond 40 / Lambda_max
    REPORT the accuracy floor (cancellation estimate) of the series

COMPARE FV and exact F(t); write comparison CSV and figure

REPEAT FV with refined cell counts (time error removed by Richardson extrapolation)
REPORT observed spatial order against the exact answer
REPEAT FV with refined dt on the configured mesh
REPORT observed temporal order against the exact answer

REPORT runtime, positivity and maximum conservation residual
```
