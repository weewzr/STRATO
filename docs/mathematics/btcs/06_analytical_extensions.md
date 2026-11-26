# STRATO: Method B (Sturm–Liouville) with decay and a kernel source

Document 6 of the BTCS series. It extends the analytical series of `src/analytical/sturm_liouville.rs` (Method B) to radioactive decay (stage 3) and to a piecewise-constant kernel source (stages 2, 3 and 5 at constant temperature). It uses the notation of Documents 1, 2 and 5 and continues the equation numbering of Document 5, which ends at (41). The science design (`science-design-v3.md`, Sections 4.1 to 4.3 and open point O10) calls the four formulas (B2) to (B5); they are (43) to (46) here, and (B1) is (42).

Every property carries a status: **PROVED** (derived here, every step shown), **OBSERVED** (a computed result of the test suite, with its number), **NOT PROVEN** (stated, not established), or **CONVENTION** (a labelled choice, not derived). Nothing in this document is an experimental validation: every comparison is a numerical verification against a closed form or an independent scheme.

## 0. Symbols and units

| Symbol | Meaning | Unit |
|---|---|---|
| $r$, $t$ | radius, time | m, s |
| $R_1,\dots,R_5$ | outer radii of kernel, buffer, IPyC, SiC, OPyC | m |
| $D_{\mathrm{kernel}}, D_{\mathrm{buffer}}, D_{\mathrm{IPyC}}, D_{\mathrm{SiC}}, D_{\mathrm{OPyC}}$ | diffusivities, constant in time | m² s⁻¹ |
| $C(r,t)$ | concentration | mol m⁻³ |
| $\lambda$ | decay constant, the same in every layer | s⁻¹ |
| $G_k$ | kernel source on $[s_k, s_{k+1})$, $s_0 = 0$ | mol m⁻³ s⁻¹ |
| $V_{\mathrm K} = \frac{4\pi}{3}\left(R_1\right)^3$ | kernel volume | m³ |
| $M^0 = V_{\mathrm K}C_{\mathrm{init}}$ | initial kernel inventory | mol |
| $\mu$, $\alpha_\mu$, $w_\mu$ | mode number, mode decay rate, weight (Section 4.1 of the science design; $\alpha_\mu$ is the module's `eigenvalues`) | –, s⁻¹, – |
| $a_\mu = \alpha_\mu + \lambda$ | rate of mode $\mu$ with decay | s⁻¹ |
| $m_\mu(t)$ | inventory of mode $\mu$ | mol |
| $M$, $M_{\mathrm{rel}}$, $M_{\mathrm{dec}}$, $M_{\mathrm{gen}}$ | inside, released through the surface, lost by decay inside, generated | mol |
| $\Phi$ | release rate through the surface | mol s⁻¹ |
| $F_0(\theta)$, $R(\theta) = 1 - F_0(\theta)$ | release and remaining fraction of the existing series (no source, no decay) | – |
| $\alpha_{\max}$ | the series' `lambda_covered` | s⁻¹ |
| $t_v = 40/\alpha_{\max}$ | the series' `min_valid_time()` | s |
| $\langle f,g\rangle$ | weighted inner product (below) | m³ times the unit of $fg$ |
| $c_\mu$, $c_\mu(t)$ | $C = \sum_\mu c_\mu(t)\varphi_\mu$ with $\varphi_\mu$ dimensionless: the coefficient (Section 1.1); in Section 3 the abbreviation $c_\mu = w_\mu e^{-a_\mu t_v}$ | mol m⁻³; – |
| $\hat R$, $\hat F_\lambda$, $\hat F_\lambda'$ | model remaining fraction, model release fraction with decay, its time derivative (Section 3) | –, –, s⁻¹ |
| $\hat E$, $\hat Q$, $\hat P$ | time integrals of $e^{-\lambda\theta}\hat R$, of $\hat E$ and of $\hat F_\lambda$ (Section 3) | s, s², s |
| $\ell$, $\ell_2$ | $\int_0^xe^{-\lambda s}ds$ and its integral (Section 3) | s, s² |
| $\Delta$ | computed bound on the release fraction of the kernel step at $t_v$ (49) | – |
| $f$ | factor of the floor inside $\Delta$ (Section 2.3, `EARLY_RELEASE_FLOOR_FACTOR`) | – |
| $f_{\mathrm{series}}$ | factor of item 3 of the mask for the existing series (Section 4, `SERIES_FLOOR_FACTOR`) | – |
| $A(t)$ | $M^0 + V_{\mathrm K}\sum_{k:\ s_k<t}G_k\left(t - s_k\right)$: the input weighted by its age (Section 8) | mol |
| $S_\Phi(t)$ | $V_{\mathrm K}\sum_{k:\ s_k<t}G_k + M^0/t$: the scale of the release rate (Section 8) | mol s⁻¹ |
| $\varepsilon_{\mathrm{floor}}$ | the series' `accuracy_floor()` | – |
| $c_M, c_R, c_\Phi$ | factors of the floor terms of the estimate (Section 8, `EXT_FACTOR_*`) | – |
| $e_M, e_R, e_\Phi$ | estimated absolute errors of $M$, $M_{\mathrm{rel}}$, $\Phi$ (Section 8) | mol, mol, mol s⁻¹ |

The inner product is $\langle f,g\rangle = \int_0^{R_5}\left(r\right)^2fg\,dr$ (unit: m³ times the unit of $fg$, because $dr$ is in m and $\left(r\right)^2$ in m²) and $\varphi_\mu$ are the eigenfunctions of the layered operator (regular centre, perfect contact, $\varphi_\mu(R_5)=0$). The weights are $w_\mu = \dfrac{\langle\chi,\varphi_\mu\rangle\langle 1,\varphi_\mu\rangle}{\langle\varphi_\mu,\varphi_\mu\rangle\langle\chi,1\rangle}$ (dimensionless, independent of the normalisation of $\varphi_\mu$) with $\chi$ the kernel indicator; this is the formula of Section 4.1 of the science design ($\langle\chi,1\rangle = \left(R_1\right)^3/3$).

**Premise used throughout (NOT PROVEN in this repository, standard theory):** the eigenfunctions $\varphi_\mu$ are complete in the space of functions that are square-integrable with the weight $\left(r\right)^2$ (the weighted Lebesgue space of the layered operator) (regular Sturm–Liouville problem with piecewise-constant coefficients).

## 1. The formulas

### 1.1 The mode equation (B1)

Write $C(r,t) = \sum_\mu c_\mu(t)\varphi_\mu(r)$. The decay-free, source-free operator has $\varphi_\mu$ as eigenfunctions with rate $\alpha_\mu$. Decay adds $-\lambda C$ with the same $\lambda$ in every layer, so each mode gets the rate $a_\mu = \alpha_\mu + \lambda$. The source $G(t)\chi$ has the kernel indicator as its spatial shape, so its projection on mode $\mu$ is $G(t)\langle\chi,\varphi_\mu\rangle/\langle\varphi_\mu,\varphi_\mu\rangle$. With $m_\mu = 4\pi c_\mu\langle1,\varphi_\mu\rangle$ (the inventory in mode $\mu$, because the inventory of $c_\mu\varphi_\mu$ is $4\pi\int\left(r\right)^2c_\mu\varphi_\mu\,dr$), projection and multiplication by $4\pi\langle1,\varphi_\mu\rangle$ give, since $4\pi\langle\chi,1\rangle = V_{\mathrm K}$,

$$
\frac{dm_\mu}{dt} = -a_\mu\,m_\mu + w_\mu V_{\mathrm K}\,G(t), \qquad m_\mu(0) = w_\mu M^0. \tag{42}
$$

The initial value is the same projection of the kernel step. The surface release of mode $\mu$ is $\alpha_\mu m_\mu$: without source and decay the mode inventory falls only by release, $dm_\mu/dt = -\alpha_\mu m_\mu$, so its release rate is $\alpha_\mu m_\mu$; decay and source add terms to the balance but do not change the surface flux of the mode. Hence $\Phi = \sum_\mu\alpha_\mu m_\mu$. **PROVED** given completeness; termwise differentiation of the series for $t>0$ is used for the flux.

### 1.2 Decay, no source (B2)

$m_\mu = w_\mu M^0e^{-a_\mu t}$ solves (42) with $G=0$. Hence

$$
\frac{M(t)}{M^0} = \sum_\mu w_\mu e^{-a_\mu t}, \quad F(t) = \frac{M_{\mathrm{rel}}(t)}{M^0} = \sum_\mu w_\mu\alpha_\mu\frac{1 - e^{-a_\mu t}}{a_\mu}, \quad \frac{M_{\mathrm{dec}}(t)}{M^0} = \sum_\mu w_\mu\lambda\frac{1 - e^{-a_\mu t}}{a_\mu}. \tag{43}
$$

($M_{\mathrm{rel}} = \sum_\mu\alpha_\mu\int_0^tm_\mu$ and $M_{\mathrm{dec}} = \lambda\sum_\mu\int_0^tm_\mu$.) Conditions, all checked by the code with an error and never a silent result: the same $\lambda$ in every layer (`Decay::PerLayer` with unequal entries gives `UnequalDecayConstants`); constant diffusivities (those of the series); kernel-step start ($M^0$ is the kernel inventory); $\lambda$, $M^0$, $t$ finite and not negative. The release is **not** the decay-free $F_0$ times $e^{-\lambda t}$; only the inventory is: $M/M^0 = e^{-\lambda t}R(t)$ (Section 1.5). **PROVED.**

### 1.3 Piecewise-constant source (B3) to (B5)

On $[s_k,s_{k+1})$ the source is the constant $G_k$ ($G = 0$ after $s_K$). Variation of constants on (42): $m_\mu(t) = w_\mu M^0e^{-a_\mu t} + w_\mu V_{\mathrm K}\int_0^te^{-a_\mu(t-u)}G(u)\,du$. Over one piece with $e_k = \min(t,s_{k+1})$, $\int_{s_k}^{e_k}e^{-a(t-u)}du = \dfrac{e^{-a(t-e_k)} - e^{-a(t-s_k)}}{a}$, so

$$
m_\mu(t) = w_\mu\left[M^0e^{-a_\mu t} + V_{\mathrm K}\sum_{k:\ s_k < t}G_k\,\frac{e^{-a_\mu\left(t - \min(t, s_{k+1})\right)} - e^{-a_\mu\left(t - s_k\right)}}{a_\mu}\right]. \tag{44}
$$

Integrating (42) from $0$ to $t$ gives $m_\mu(t) - w_\mu M^0 = -a_\mu\int_0^tm_\mu + w_\mu M_{\mathrm{gen}}$, with

$$
M_{\mathrm{gen}}(t) = V_{\mathrm K}\sum_{k:\ s_k < t}G_k\left(\min(t, s_{k+1}) - s_k\right), \qquad \int_0^t m_\mu\,ds = \frac{w_\mu\left(M^0 + M_{\mathrm{gen}}(t)\right) - m_\mu(t)}{a_\mu}. \tag{45}
$$

$$
M = \sum_\mu m_\mu, \quad M_{\mathrm{rel}} = \sum_\mu\alpha_\mu\int_0^t m_\mu\,ds, \quad M_{\mathrm{dec}} = \lambda\sum_\mu\int_0^t m_\mu\,ds, \quad F_{\mathrm{gen}} = \frac{M_{\mathrm{rel}}}{M^0 + M_{\mathrm{gen}}}, \quad \Phi = \sum_\mu\alpha_\mu m_\mu. \tag{46}
$$

Adding the three sums mode by mode, $M + M_{\mathrm{rel}} + M_{\mathrm{dec}} = \left(\sum_\mu w_\mu\right)\left(M^0 + M_{\mathrm{gen}}\right)$ (the balance of the science design). With $\beta = \Phi/(V_{\mathrm K}G)$ for $G(t) > 0$. $M_{\mathrm{rel}}$ is the sum of releases, not a difference of inventories. **PROVED.** The code takes $F$ (existing series) when there is neither source nor decay, and defines $F_{\mathrm{post}}(t) = \left(M_{\mathrm{rel}}(t) - M_{\mathrm{rel}}(t_e)\right)/M(t_e)$ for $t \ge t_e$ (Section 5.1 of the science design).

## 2. Evaluation: why the modal sums are not truncated

### 2.1 Two facts about the weights

**(a) $\sum_\mu\left|w_\mu\right| < \infty$ and $\sum_\mu w_\mu = 1$ (PROVED given completeness).** By Bessel's inequality $\sum_\mu\left(\langle\chi,\varphi_\mu\rangle\right)^2/\langle\varphi_\mu,\varphi_\mu\rangle \le \langle\chi,\chi\rangle$ and the same for $1$, and Cauchy–Schwarz gives $\sum_\mu|w_\mu| \le \sqrt{\langle\chi,\chi\rangle\langle1,1\rangle}/\langle\chi,1\rangle < \infty$. Parseval's identity for the complete system gives $\langle\chi,1\rangle = \sum_\mu\langle\chi,\varphi_\mu\rangle\langle1,\varphi_\mu\rangle/\langle\varphi_\mu,\varphi_\mu\rangle$, that is $\sum_\mu w_\mu = 1$. The weights change sign, so a partial sum can be near 1 by cancellation, and the **missing weight** $W_{\mathrm{miss}} = 1 - \sum_{\mu\le\mu_{\max}}w_\mu$ is reported as a diagnostic only.

**(b) The existing series is the case $\lambda = 0$ (PROVED).** $R(\theta) = \sum_\mu w_\mu e^{-\alpha_\mu\theta}$, hence

$$
\sum_\mu w_\mu e^{-a_\mu\theta} = e^{-\lambda\theta}R(\theta). \tag{47}
$$

### 2.2 The problem with a truncated (43)

Sum (43) over the held modes only. The terms of the omitted modes are $w_\mu\alpha_\mu(1 - e^{-a_\mu t})/a_\mu \approx w_\mu$ (they have decayed), so the truncated sum is short by about $W_{\mathrm{miss}}$ at all times, which is 1e-3 or larger for the series of the tests and far above a release fraction of 1e-5. The existing $F_0 = 1 - \sum_\mu w_\mu e^{-\alpha_\mu t}$ avoids it by (a). Completing the omitted modes with $W_{\mathrm{miss}}$ is exact for $\lambda = 0$, but for $\lambda > 0$ it misses $\lambda\sum_{\mu>\mu_{\max}}w_\mu/a_\mu$, of size $\lambda W_{\mathrm{miss}}/\alpha_{\max}$. **Scratch values, not reproduced by the committed test suite** ($D = 10^{-10}$ m² s⁻¹, $\lambda = 5\times10^{-3}$ s⁻¹; they illustrate the size of the effect and carry no further weight): at $F_{\mathrm{rel}} = 3.19\times10^{-5}$ the completed truncation gave $3.46\times10^{-5}$ with 21 modes, $3.25\times10^{-5}$ with 42 and $3.195\times10^{-5}$ with 96, the error scaling as $1/\alpha_{\max}$, and the same for the source sums. That error is far above `accuracy_floor()` and not bounded a priori.

### 2.3 The evaluation used

Integrate (47) and use (a). For a unit kernel-step inventory released with decay,

$$
F_\lambda(\theta) = \sum_\mu w_\mu\alpha_\mu\frac{1 - e^{-a_\mu\theta}}{a_\mu} = \sum_\mu w_\mu\left(1 - e^{-a_\mu\theta}\right) - \lambda\sum_\mu w_\mu\frac{1 - e^{-a_\mu\theta}}{a_\mu} = 1 - e^{-\lambda\theta}R(\theta) - \lambda E(\theta), \tag{48}
$$

with $E(\theta) = \int_0^\theta e^{-\lambda s}R(s)\,ds = \sum_\mu w_\mu(1 - e^{-a_\mu\theta})/a_\mu$. The exchange of sum and integral is justified by $\sum_\mu|w_\mu|/a_\mu \le \sum_\mu|w_\mu|/\alpha_1 < \infty$. **PROVED.** Equation (48) uses the complete set only through $R$.

Because $R$ is the remaining fraction of the existing series, (48) needs no tail of the modal sum. The series holds the modes $\mu \le \mu_{\max}$ and is exact for $\theta \ge t_v$ up to the content $e^{-40}$ of the omitted modes (the module's convention behind `min_valid_time`). For $\theta < t_v$ the series is not used: $R$ is replaced by $1$. The model remaining fraction is

$$
\hat R(\theta) = \begin{cases}1, & \theta < t_v,\\ \sum_{\mu\le\mu_{\max}}w_\mu e^{-\alpha_\mu\theta}, & \theta \ge t_v,\end{cases}\qquad \Delta = 1 - \sum_{\mu\le\mu_{\max}}w_\mu e^{-\alpha_\mu t_v} + f\cdot\text{(the series' accuracy floor)}. \tag{49}
$$

($\Delta$ is dimensionless. The factor $f = 4$ (`EARLY_RELEASE_FLOOR_FACTOR`) is the ratio of check (v), 3.34, rounded up: the shooting error of the weights enters $1 - \sum_\mu w_\mu e^{-\alpha_\mu t_v}$ with at most that many floors. **OBSERVED** in Section 8: the sum of the absolute weight errors of the homogeneous particle against the closed-form weights is 3 floors at 64 modes and 5 floors at 1024 modes, so $f = 4$ is a convention of the right size, not a bound; the floor term of the estimate (Section 8) covers the rest. A unit test pins the term: for a series with $F_0(t_v) < 10^{-15}$, $\Delta$ equals $f\,\varepsilon_{\mathrm{floor}}$.) $F_0$ is not decreasing, so $0 \le 1 - R(\theta) = F_0(\theta) \le F_0(t_v) \le \Delta$ for $\theta < t_v$. **PROVED** (monotonicity: $C \ge 0$ by the parabolic maximum principle for non-negative data and zero surface value, so the outward flux at the absorbing surface is $\ge 0$ and the cumulative release does not decrease). $\Delta$ is computed from the series itself; it is large if the cover is too small, and then the mask rejects the value.

## 3. Closed forms of the evaluation

With $\hat R$ in place of $R$, define for $\theta \ge 0$ (units of $\theta$: s), with $\ell(x) = \int_0^xe^{-\lambda s}ds = \left(1 - e^{-\lambda x}\right)/\lambda$ ($=x$ for $\lambda=0$) and $\ell_2(x) = \int_0^x\ell = \left(\lambda x - 1 + e^{-\lambda x}\right)/\left(\lambda\right)^2$ ($= \left(x\right)^2/2$ for $\lambda = 0$; units s and s²), and $c_\mu = w_\mu e^{-a_\mu t_v}$ (dimensionless), $L = \theta - t_v$ (s); the units are s for $\hat E$ and $\hat P$, s² for $\hat Q$, and none for $\hat F_\lambda$:

$$
\hat E(\theta) = \begin{cases}\ell(\theta), & \theta<t_v\\ \ell(t_v) + \sum_\mu c_\mu\dfrac{1 - e^{-a_\mu L}}{a_\mu}, & \theta\ge t_v\end{cases}\qquad \hat Q(\theta) = \int_0^\theta\hat E = \begin{cases}\ell_2(\theta), & \theta<t_v\\ \ell_2(t_v) + \ell(t_v)L + \sum_\mu\dfrac{c_\mu}{a_\mu}\left(L - \dfrac{1 - e^{-a_\mu L}}{a_\mu}\right), & \theta\ge t_v\end{cases}
$$

$$
\hat F_\lambda(\theta) = \begin{cases}0,&\theta<t_v\\ 1 - e^{-\lambda\theta}\hat R(\theta) - \lambda\hat E(\theta),&\theta\ge t_v\end{cases}\qquad \hat P(\theta) = \int_0^\theta\hat F_\lambda = \begin{cases}0,&\theta<t_v\\ \left(\theta - t_v\right) - \left(\hat E(\theta) - \hat E(t_v)\right) - \lambda\left(\hat Q(\theta) - \hat Q(t_v)\right),&\theta\ge t_v\end{cases}
$$

$$
\hat F_\lambda'(\theta) = e^{-\lambda\theta}\sum_\mu w_\mu\alpha_\mu e^{-\alpha_\mu\theta}\ \ (\theta \ge t_v),\qquad 0\ \ (\theta<t_v). \tag{50}
$$

Each is derived by integrating the previous one ($\hat E' = e^{-\lambda\theta}\hat R$, $\hat Q' = \hat E$). The identity $\hat F_\lambda + e^{-\lambda\theta}\hat R + \lambda\hat E = 1$ holds for every $\theta$ (for $\theta<t_v$: $0 + e^{-\lambda\theta} + (1 - e^{-\lambda\theta}) = 1$), and so does $\hat P + \hat E + \lambda\hat Q = \theta$ (for $\theta\ge t_v$: $\left(\theta - t_v\right) + \ell(t_v) + \lambda\ell_2(t_v) = \theta$, since $\ell + \lambda\ell_2 = \mathrm{id}$). **PROVED.**

The quantities are superpositions (Duhamel): each source increment $V_{\mathrm K}G_k\,du$ at time $u$ behaves as a kernel-step inventory released over $\theta = t - u$. With $\theta_{\mathrm{hi},k} = t - s_k$, $\theta_{\mathrm{lo},k} = t - e_k$, $e_k = \min(t, s_{k+1})$, summing (44) over modes and using (47):

$$
\begin{aligned}
M &= M^0e^{-\lambda t}\hat R(t) + V_{\mathrm K}\sum_kG_k\left[\hat E(\theta_{\mathrm{hi},k}) - \hat E(\theta_{\mathrm{lo},k})\right],\\
M_{\mathrm{dec}} &= \lambda\left\{M^0\hat E(t) + V_{\mathrm K}\sum_kG_k\left[\hat Q(\theta_{\mathrm{hi},k}) - \hat Q(\theta_{\mathrm{lo},k})\right]\right\},\\
M_{\mathrm{rel}} &= M^0\hat F_\lambda(t) + V_{\mathrm K}\sum_kG_k\left[\hat P(\theta_{\mathrm{hi},k}) - \hat P(\theta_{\mathrm{lo},k})\right],\\
\Phi &= M^0\hat F_\lambda'(t) + V_{\mathrm K}\sum_kG_k\left[\hat F_\lambda(\theta_{\mathrm{hi},k}) - \hat F_\lambda(\theta_{\mathrm{lo},k})\right].
\end{aligned}\tag{51}
$$

The first line is (44) summed ($\sum_\mu w_\mu e^{-a_\mu x}(1 - e^{-a_\mu d})/a_\mu = \int_x^{x+d}\sum_\mu w_\mu e^{-a_\mu\theta}\,d\theta$). The second is $\lambda\int_0^tM\,ds$ with the source part $\int_{s_k}^{e_k}\hat E(t - u)\,du = \hat Q(\theta_{\mathrm{hi}}) - \hat Q(\theta_{\mathrm{lo}})$. The third is $\sum_\mu\alpha_\mu\int_0^tm_\mu$ with (48) applied to each increment. The fourth is the time derivative of the third with $\hat F_\lambda(0) = 0$. Cost per time point: $O(\mu_{\max}K)$ (the cost stated in the science design); the mask adds one evaluation with the reduced cover. **PROVED** (algebra), with $R$ replaced by $\hat R$.

Consequences, all **PROVED** by (50) and (51):

* **Balance of the evaluation.** $M + M_{\mathrm{rel}} + M_{\mathrm{dec}} = M^0 + M_{\mathrm{gen}}$ to rounding, because the three lines are built from the one $\hat R$ (the identities after (50), integrated over each increment). **OBSERVED:** below $10^{-12}\left(M^0 + M_{\mathrm{gen}}\right)$ in the tests. This identity cannot show that $\hat R$ is right; the comparisons of Section 5 do.
* **Limit $\lambda\to0$.** (51) reduces to the Duhamel row of Document 5 Section 4.5, and (48) to $F_0 = 1 - \hat R$ for $\theta \ge t_v$.
* **Limit $G\to0$.** (51) reduces to $M^0\hat F_\lambda(t)$ etc., which is (43) with the tail resolved.
* **Superposition.** (51) is linear in $(M^0, G_0, \dots, G_{K-1})$, so two source segments equal the sum of the two single-segment runs, and two adjacent equal segments equal one segment.
* **Bit for bit.** With $\lambda = 0$ and every $G_k = 0$ the code returns the existing `release_fraction` (it calls the same expression); this is by construction, not a numerical coincidence.

### 3.1 Computed error bounds

Let $e(\theta) = \hat R(\theta) - R(\theta)$. For $\theta<t_v$, $e = F_0(\theta)\in[0,\Delta]$. For $\theta \ge t_v$ the omitted modes contribute $|e| \le e^{-40}\sum_\mu|w_\mu|$ (**CONVENTION:** neglected; the module's own convention, and $\sum_\mu|w_\mu|$ is finite by Section 2.1 but its value is not bounded in this document). Define $\ell_k = \left|[\theta_{\mathrm{lo},k},\theta_{\mathrm{hi},k}]\cap[0,t_v)\right|$ (s) and $\mathrm{len}_k = e_k - s_k$ (s). Then, from (51) and (48):

$$
\begin{aligned}
0 \le \hat M - M &\le \Delta\left(M^0\,\mathbf 1_{t<t_v} + V_{\mathrm K}\sum_kG_k\ell_k\right),\\
\left|\hat M_{\mathrm{rel}} - M_{\mathrm{rel}}\right| &\le \Delta\left[M^0\left(\mathbf 1_{t<t_v} + \lambda\min(t,t_v)\right) + V_{\mathrm K}\sum_kG_k\left(\ell_k + \lambda t_v\,\mathrm{len}_k\right)\right],
\end{aligned}\tag{52}
$$

and $\left|\hat M_{\mathrm{dec}} - M_{\mathrm{dec}}\right|$ is bounded by the sum of the first two. With $a(\theta) = 0$ for $\theta \le 0$ and $a(\theta) = \mathbf 1_{\theta<t_v} + \lambda\min(\theta,t_v)$ for $\theta > 0$ (dimensionless), the release rate satisfies, for $t \ge t_v$,

$$
\left|\hat\Phi - \Phi\right| \le \Delta\,V_{\mathrm K}\sum_{k:\ s_k<t}G_k\,\max\!\left(a\!\left(\theta_{\mathrm{hi},k}\right),\ a\!\left(\theta_{\mathrm{lo},k}\right)\right).\tag{53}
$$

Derivation: $\hat F_\lambda - F_\lambda = -e^{-\lambda\theta}e(\theta) - \lambda\int_0^\theta e^{-\lambda s}e(s)\,ds$, so $-\Delta\,a(\theta) \le \hat F_\lambda - F_\lambda \le 0$ (with $e(0) = F_0(0) = 0$, hence $0$ at $\theta = 0$); integrating over each increment gives the second line of (52), and $\hat M - M$ is the same with $e^{-\lambda\theta}e(\theta)\ge 0$. For (53): by (51) the error of the rate is $V_{\mathrm K}\sum_kG_k\left[\left(\hat F_\lambda - F_\lambda\right)\!\left(\theta_{\mathrm{hi},k}\right) - \left(\hat F_\lambda - F_\lambda\right)\!\left(\theta_{\mathrm{lo},k}\right)\right]$ (plus the initial-inventory term, below), and two numbers in $[-\Delta\,a_1, 0]$ and $[-\Delta\,a_2, 0]$ differ by at most $\Delta\max(a_1, a_2)$. This replaces the first version of the bound, $2\Delta V_{\mathrm K}\sum_kG_k\left(1 + \lambda t_v\right)$, which does not decay: for a source on since the start without decay (53) is zero, with decay $\Delta\lambda t_vV_{\mathrm K}G$, and it is $\Delta\left(1 + \lambda\theta\right)V_{\mathrm K}G$ only within $t_v$ after a switch-on (a unit test checks the three cases). The initial-inventory term $M^0\hat F_\lambda'(t)$ has no model error beyond the convention for $\theta\ge t_v$. **PROVED** apart from the convention for $\theta\ge t_v$ and the mode truncation of $\Delta$. They are computed and returned by the code (`ModalState::inventory_error_bound`, `release_error_bound`, `flux_error_bound`); the flux bound is valid for $t\ge t_v$ only. They are bounds on the model error, not on the shooting error of the eigenvalues and weights or the rounding of the evaluation (that is `accuracy_floor()` and Section 8).

**OBSERVED** (decay, no source, $D = 10^{-10}$ m² s⁻¹, $\lambda = 5\times10^{-3}$ s⁻¹, 21 modes, $t_v = 17.8$ s): at $t = 20$ s the error against the independent closed form is $7.8\times10^{-8}$, the bound $1.0\times10^{-6}$, the release fraction $3.2\times10^{-5}$.

## 4. The drawing mask (science design Section 4.2)

No a-priori tail bound for the *modal sums* is claimed. A value $V(t)$ of $F$, $F_{\mathrm{gen}}$, $F_{\mathrm{post}}$, $M/M^0$ or $\beta$ is drawn only if all hold; otherwise no value is returned, with the reason (`Withheld`):

1. $t \ge t_v$ (for $F_{\mathrm{post}}$ also $t_e$);
2. $\left|V_{\alpha_{\max}} - V_{\alpha_{\max}/4}\right| \le 0.05\left|V_{\alpha_{\max}}\right|$, where the second value uses only the modes with $\alpha_\mu \le \alpha_{\max}/4$ (and so $t_v' = 4t_v$ in (49)) (**CONVENTION:** the labelled 5 % of the science design, not a derived bound);
3. for the existing series (no source, no decay): $f_{\mathrm{series}}\cdot$`accuracy_floor()` $\le 0.05\,V$ with $V$ as a fraction of $M^0$ ($f_{\mathrm{series}} = 4$, below). For a run with decay or a source: the estimated relative error of the value, the sum of the estimated absolute errors of the quantities that enter it (Section 8) over the value, is at most $0.05$, and the run lies in the regime in which the estimate was measured (Section 8.5); otherwise the value is withheld with the reason `ExtensionErrorTooLarge` or `OutsideEstimatedRegime`.

Item 2 is stricter than the science design may have intended near $t_v$: for the extension quantities the reduced-cover value has no release before $4t_v$ (Section 2.3), so a release that is not already negligible at $t_v$ fails item 2 for $t_v \le t < 4t_v$. That is conservative, not wrong. **The factors $f_{\mathrm{series}}$ and $f$ (FINDING).** The science design assumed that `accuracy_floor()`, the estimate for the decay-free `release_fraction`, also holds for (43) to (46) (check (v)), and asks that item 3 use the observed error if it does not. **OBSERVED:** at values of 1 to 100 times the floor (five times, one homogeneous layer, 67 modes, floor $5.30\times10^{-16}$) the difference between the shooting series and the closed-form modes is $1.55\times10^{-15}$, 3.34 times the floor (worst case). The decay-free series differs from the closed-form modes by the same $1.55\times10^{-15}$ ($2.55\times10^{-15}$ against $1.0\times10^{-15}$ at the first point): the offset belongs to the existing series (its weights differ from the closed-form weights by up to $3.8\times10^{-15}$), and the extension adds none. The ratio rounded up, 4, is the factor $f_{\mathrm{series}}$ (`SERIES_FLOOR_FACTOR`) of item 3 for the existing series, and the factor $f$ of $\Delta$ (49) (`EARLY_RELEASE_FLOOR_FACTOR`): a **CONVENTION** from one geometry. The literal check (v) criterion (difference $\le$ floor) is **not met**. For a run with decay or a source the factor $f_{\mathrm{series}}$ is not used: item 3 uses the estimate of Section 8, which replaces it (decision 68(c)).

**The full fallback rule, applied.** The science design (Section 4.2, item 3) says: "if check (v) fails, item 3 uses the observed error of check (v) instead of the floor, **and the extension is not drawn until the floor is re-estimated**." Both halves were applied in the first version of this document: the first half as one factor $f = 4$ from one geometry, the second half as a gate in `KernelRun::drawn`, which returned `Withheld::FloorNotReestimated` for every quantity of a run with decay ($\lambda > 0$) or a non-zero source while the constant `EXTENSION_FLOOR_REESTIMATED` was `false`, because the factor 3.34 was observed in one geometry only. **Decision 68(c) of the coordinator** asks for the re-estimate over several geometries and mode counts and authorises setting the constant once the independent verifier passes it. Section 8 is that re-estimate, the factor of item 3 for the extension quantities is replaced by its rule, and the constant is `true`. The gate stays in the code as `KernelRun::drawn_gated` (a test closes it and checks the reason), and `KernelRun::mask` is the mask without the gate. A run with neither decay nor source (the existing series) is drawn as before, through items 1 to 3. The raw functions (`state`, `value`, `with_decay`, `with_source_decay`) are labelled "unmasked, not for display" in their documentation: before $t_v$ they hold the convention $\hat R = 1$ and near $t_v$ they can be wrong by up to 100 % (reported by the independent verification for a source at $t_v$ in a scratch run that the committed suite does not reproduce), so a page must show only what `drawn` returns.
## 5. The five verifier checks

All results are numerical verifications (cargo tests in `src/analytical/extensions.rs` and `tests/analytical_extensions_btcs.rs`), not experimental validations.

| Check | Statement | Result |
|---|---|---|
| (i) | $G = 0$, $\lambda = 0$ reproduces `release_fraction` bit for bit; $\lambda\to0$ and $G\to0$ within the series' `accuracy_floor()` | OBSERVED: bit for bit (equality asserted); $\lambda = 10^{-30}$ to $10^{-20}$ s⁻¹ and $G = 10^{-30}$ to $10^{-20}$ mol m⁻³ s⁻¹ within the floor |
| (ii) | $\lambda = 0$, constant $G$, $t\to\infty$: $\Phi\to V_{\mathrm K}G$ and $M_\infty = 4\pi\int_0^{R_5}C_{\mathrm{ss}}\left(r\right)^2dr$ with D5 (14) integrated layer by layer in closed form | OBSERVED: five different diffusivities ($10^{-12}$, $4\times10^{-10}$, $8\times10^{-13}$, $2\times10^{-14}$, $7\times10^{-13}$ m² s⁻¹), 64 modes: $M$ against D5 (14) $5.6\times10^{-13}$ relative; $\Phi/(V_{\mathrm K}G)$ equals 1 to $10^{-9}$; $\beta\to1$. The truncated modal sum $V_{\mathrm K}G\sum w/\alpha$ differs from D5 (14) by $1.5\times10^{-9}$ (tail of order $N^{-3}$, OBSERVED not proved). With decay (one material, closed form with $\sinh$ matching at $R_1$): $2.0\times10^{-15}$ relative |
| (iii) | one homogeneous layer reproduces D5 (21) and the Duhamel row | OBSERVED: shooting modes against the closed-form modes of `homogeneous_sphere` agree to $10^{-9}$ in $M$, $M_{\mathrm{rel}}$, $M_{\mathrm{dec}}$ (decay and two source segments); decay against the closed form $F_\lambda = e^{-\lambda\theta}F_0 + \lambda\int_0^\theta e^{-\lambda s}F_0$ (rigorous series of `homogeneous_sphere`) and against the physics module's uniform-start sphere within its stated tail bound $6/\left(\left(\pi\right)^2N\right)$; source against the superposition of that closed form |
| (iv) | BTCS with the D5 rows converges to (B5) at order 2 in space, 1 in time | OBSERVED, run as specified with the production solver (`run_physics`, merged from the verified `feat/btcs-physics`), test `tests/analytical_extensions_btcs.rs`: five different diffusivities (D_kernel $10^{-12}$, D_buffer $4\times10^{-10}$, D_IPyC $8\times10^{-13}$, D_SiC $2\times10^{-14}$, D_OPyC $7\times10^{-13}$ m² s⁻¹), $\lambda = 2\times10^{-6}$ s⁻¹, constant kernel source $G = 5\times10^{-6}$ mol m⁻³ s⁻¹, $t = 3\times10^5$ s, extension on 20 000 modes; fractions of $M^0 + M_{\mathrm{gen}}$: $M$ 0.415826454, $M_{\mathrm{rel}}$ 0.331767200, $M_{\mathrm{dec}}$ 0.252406346. After Richardson extrapolation in time the errors at 10, 20, 40 and 80 intervals per material are for $M_{\mathrm{rel}}$ $6.92\times10^{-5}$, $1.73\times10^{-5}$, $4.33\times10^{-6}$, $1.08\times10^{-6}$ (space orders 1.998, 2.000, 2.001), for $M$ $6.83\times10^{-6}$ to $1.06\times10^{-7}$ (orders 1.999 to 2.013) and for $M_{\mathrm{dec}}$ $6.24\times10^{-5}$ to $9.77\times10^{-7}$ (orders 1.998 to 1.999); time orders of the three fractions 0.995, 0.998, 0.999 (100 to 1600 steps, 40 intervals per material); the remaining time error at 80 intervals is below $1.7\times10^{-9}$. The test asserts that the orders round to 2 and 1 (the repository's convention, decision 49). The earlier test-local scalar scheme (backward Euler, centred differences on $u = rC$) stays in `src/analytical/extensions.rs` as an additional independent check |
| (v) | floor test at 1 to 100 times `accuracy_floor()` | **FINDING, not met literally**: worst ratio 3.34 (Section 4). Consequence: the floor is re-estimated over several geometries, mode counts, decay constants, sources and times in Section 8, and the extension quantities are drawn by the rule derived there (decision 68(c)) |

Refusals (each tested): unequal decay constants; a per-layer decay list that does not hold exactly five entries; negative or non-finite $\lambda$, $M^0$, $t$, source rates, breakpoints; breakpoints not starting at 0 or not strictly increasing; rate and breakpoint counts that do not match; non-positive kernel volume; an empty or inconsistent series; a quantity that is undefined ($F$ with $M^0 = 0$, $\beta$ with $G = 0$, $F_{\mathrm{post}}$ before $t_e$).

## 6. The applicability function (science design Section 4.3)

`applicability(stage, source, decay, temperature)` returns `Exact`, `Extension(Decay | Source | SourceWithDecay)` or `NotApplicable(reason)`. Order of the checks: invalid input; a temperature that varies in time (a series of breakpoints, whose times must be strictly increasing, is constant only when every value is exactly equal); a mechanism the stage does not contain (stage 1: no source, no decay; stage 2: no decay); a source that is not piecewise constant, or not uniform in the kernel; unequal decay constants. Stage 4 refuses neither mechanism: it keeps every term of stage 3 (science design Section 6), because $G = 0$ and $\lambda = 0$ are default inputs and not a different equation, so at a constant temperature stage 4 is answered from its inputs exactly as stages 3 and 5 are, and a varying temperature is refused at stage 4 as at every stage. A source whose rates are all zero counts as no source, and $\lambda = 0$ as no decay. Then:

| Source | Decay | Answer | Formulas |
|---|---|---|---|
| none | none | exact | existing series |
| none | $\lambda>0$ | extension E-decay | (43) |
| piecewise constant | none | extension E-source | (44) to (46), $\lambda = 0$ |
| piecewise constant | $\lambda>0$ | both | (44) to (46) |

Whether a page shows the result also depends on the verification status (science design 4.3, "yes, after verification"), which this function does not know. **Why a varying temperature is never applicable (PROVED in outline, science design Section 4.2):** with $T(t)$ the five diffusivities change by different factors, so the eigenfunctions $\varphi_\mu$ change in time and (42) no longer decouples.

## 7. Summary of properties

| Property | Status |
|---|---|
| Mode equation (42), $\Phi = \sum_\mu\alpha_\mu m_\mu$ | PROVED (given completeness of the eigenfunctions, NOT PROVEN here) |
| $\sum_\mu\left|w_\mu\right| < \infty$, $\sum_\mu w_\mu = 1$ | PROVED (same premise) |
| (43) to (46) | PROVED |
| (48), (51) (tail-free evaluation) | PROVED (algebra) |
| Balance, superposition, limits $\lambda\to0$, $G\to0$ | PROVED; balance OBSERVED below $10^{-12}$ |
| Error bounds (52), (53) | PROVED, except the neglected $e^{-40}\sum_\mu\left|w_\mu\right|$ (CONVENTION) |
| Mask items 1 and 2 | CONVENTION (5 %); no a-priori bound of the modal sums |
| Mask item 3 for the existing series | CONVENTION ($f_{\mathrm{series}} = 4$, observed in one geometry) |
| Mask item 3 for decay or a source: estimate (54) to (56), factors, regime | The perturbation structure PROVED; the factors OBSERVED over the designed set with a margin of 2.8 to 4 (Section 8); outside the regime and for the contrasted layered sets NOT PROVEN, withheld or resting on `accuracy_floor()` (Section 8.6) |
| Cover/4 sensitivity (item 2) is a bound on the truncation error | NOT PROVEN |
| `accuracy_floor()` is the error of (43) to (46) | NOT MET literally (3.34 times in check (v)); re-estimated in Section 8: the error exceeds the computed bounds by at most 3.5 (M), 8.4 ($M_{\mathrm{rel}}$) and 2.6 ($\Phi$) times `accuracy_floor()` times the scale $A$ or $S_\Phi$ |
| Check (iv) with the production BTCS | OBSERVED: orders 2 in space and 1 in time (Section 5) |

## 8. Re-estimate of the accuracy floor for (B2) to (B5) (decision 68(c))

Every number of this section is a numerical verification against a closed form or against a series with many more modes, never an experimental validation. Reproduce it with `cargo run --release --features extension-estimate --example extension_floor_table -- full` (the full set, tens of minutes: the references of the layered geometries have 16 384 modes), without the argument for the smaller set that the committed tests assert, and with `-- full setup` for Table 8.1.

### 8.1 What is estimated

Check (v) (Section 5) found that `accuracy_floor()`, the estimate of the error of the decay-free `release_fraction`, is exceeded 3.34 times in one geometry, and the science design then withholds the extension "until the floor is re-estimated" (decision 68(c) of the coordinator). The task of this section: an estimated absolute error $e$ for each extension quantity, $e_M$ for $M$, $e_R$ for $M_{\mathrm{rel}}$ and $e_\Phi$ for $\Phi$, that holds over a designed set of geometries, mode counts, decay constants, sources and times with a stated margin; the drawing rule that follows from it; and a statement of what is not covered.

### 8.2 The form of the estimate

An error of an extension value has three parts.

1. **Model error:** replacing $R$ by $\hat R$ below $t_v$ and neglecting the $e^{-40}$ content of the omitted modes. It is bounded by (52) and (53), computed by the code: **PROVED** (Section 3.1), apart from the convention for $\theta \ge t_v$.
2. **Error of the series:** the weights $w_\mu$ and rates $\alpha_\mu$ come from the shooting solution with rounding error. Let $\varepsilon_{\mathrm{series}}$ bound the error of the remaining fraction, $\left|\delta\hat R(\theta)\right| \le \varepsilon_{\mathrm{series}}$ for $\theta \ge t_v$ (dimensionless); then $\varepsilon_{\mathrm{series}} \le \sum_\mu\left|\delta w_\mu\right| + \left(\delta\alpha/\alpha\right)_{\max}\sum_\mu\left|w_\mu\right|/e$, because $\alpha\theta\,e^{-\alpha\theta} \le 1/e$. It passes through (51) linearly. From $\hat F_\lambda = 1 - e^{-\lambda\theta}\hat R - \lambda\hat E$ and $\lambda\int_0^\theta e^{-\lambda s}ds \le 1 - e^{-\lambda\theta}$ follows $\left|\delta\hat F_\lambda\right| \le \varepsilon_{\mathrm{series}}$, $\left|\delta\hat E\right| \le \varepsilon_{\mathrm{series}}\,\theta$ and $\left|\delta\hat P(\theta_{\mathrm{hi}}) - \delta\hat P(\theta_{\mathrm{lo}})\right| \le \varepsilon_{\mathrm{series}}\left(\theta_{\mathrm{hi}} - \theta_{\mathrm{lo}}\right)$. With (51) and $\sum_kG_kV_{\mathrm K}\left(\theta_{\mathrm{hi},k} - \theta_{\mathrm{lo},k}\right) = M_{\mathrm{gen}}$ the errors of $M$, $M_{\mathrm{rel}}$ and $M_{\mathrm{dec}}$ are at most $\varepsilon_{\mathrm{series}}\left(M^0 + M_{\mathrm{gen}}\right)$, and the error of $\Phi$ at most $\varepsilon_{\mathrm{series}}\left(2V_{\mathrm K}\sum_kG_k + M^0/\left(e\,t\right)\right)$ (for the initial-inventory part of $\Phi$ use $\alpha e^{-\alpha t} \le 1/(e\,t)$). **PROVED** as a perturbation argument; the size of $\varepsilon_{\mathrm{series}}$ is not proved, it is observed (Section 8.4).
3. **Rounding of the evaluation:** $\hat E(\theta_{\mathrm{hi}}) - \hat E(\theta_{\mathrm{lo}})$, $\hat P(\theta_{\mathrm{hi}}) - \hat P(\theta_{\mathrm{lo}})$ and the like subtract numbers of the size of the age $\theta_{\mathrm{hi}}$ of the piece (the sums of (51) hold terms up to $V_{\mathrm K}G_k\theta_{\mathrm{hi},k}$), so their rounding error grows with the age-weighted input

$$
A(t) = M^0 + V_{\mathrm K}\sum_{k:\ s_k<t}G_k\left(t - s_k\right) \ \ (\mathrm{mol}) \ge M^0 + M_{\mathrm{gen}}(t),\qquad S_\Phi(t) = V_{\mathrm K}\sum_{k:\ s_k<t}G_k + \frac{M^0}{t}\ \ (\mathrm{mol\ s^{-1}}).
$$

Both 2 and 3 are proportional to $A$ (or $S_\Phi$) times a number of the size of the series' `accuracy_floor()` $\varepsilon_{\mathrm{floor}}$ (at least $2.2\times10^{-16}$, rounding of one operation, and larger where the shooting solution loses digits). The estimate is therefore

$$
e_M = B_M + c_M\,\varepsilon_{\mathrm{floor}}\,A,\qquad e_R = B_R + c_R\,\varepsilon_{\mathrm{floor}}\,A,\qquad e_\Phi = B_\Phi + c_\Phi\,\varepsilon_{\mathrm{floor}}\,S_\Phi,\tag{54}
$$

with $B_M$, $B_R$ the bounds (52) of $\hat M - M$ and $\hat M_{\mathrm{rel}} - M_{\mathrm{rel}}$ and $B_\Phi$ the bound (53), all computed with $\Delta$ of (49) ($f = 4$), and $c_M, c_R, c_\Phi$ dimensionless factors that the data of Section 8.4 fix. The unit of every term is the unit of its quantity (mol, mol, mol s⁻¹).

The relative error of a drawn value is the sum of the estimated absolute errors of the quantities that enter it over the value:

$$
\frac{e_M}{M}\ \text{for } M/M^0,\qquad \frac{e_R}{M_{\mathrm{rel}}}\ \text{for } F \text{ and } F_{\mathrm{gen}},\qquad \frac{e_\Phi}{\Phi}\ \text{for } \beta,\qquad \frac{e_R(t) + e_R(t_e)}{M_{\mathrm{rel}}(t) - M_{\mathrm{rel}}(t_e)} + \frac{e_M(t_e)}{M(t_e)}\ \text{for } F_{\mathrm{post}}.\tag{55}
$$

(The denominators $M^0$, $M^0 + M_{\mathrm{gen}}$ and $V_{\mathrm K}G$ are exact inputs, so they do not enter the relative error.) The mask draws a value only where (55) is at most $0.05$, the drawing threshold of the science design (the labelled 5 % of items 2 and 3; **CONVENTION**).

### 8.3 The designed set

Table 8.1 lists the geometries. The diffusivities of the two reference-particle sets are the registered Arrhenius laws at 1000 and 1600 °C (silver in UO₂, silver in low-temperature isotropic pyrocarbon, silver in SiC; `data/diffusivity/arrhenius.csv`, rows A1-04, A2-03 and A3-05 of the T978 set); the buffer takes the kernel value (a choice of this test, no registered law). The contrasted set is a test choice, not a correlation: its five diffusivities span $10^{-16}$ to $10^{-9}$ m² s⁻¹ ($D_{\mathrm{SiC}}$ to $D_{\mathrm{buffer}}$), seven decades, which drives `accuracy_floor()` to $1.6\times10^{-9}$.

| Geometry | Diffusivities (m² s⁻¹) | $\alpha_1$ (s⁻¹) | `accuracy_floor()` | Reference |
|---|---|---|---|---|
| uniform sphere, radius 175 µm | $D = 1.0\times10^{-14}$ | $3.22\times10^{-6}$ | $2.2\times10^{-16}$ | **closed form**: series of the physics module (`physics::reference`), many terms, with the closed-form tail of the decayed fraction and a quadrature of the superposition integral |
| homogeneous five-layer particle | all five $1.0\times10^{-13}$ | $6.35\times10^{-6}$ | $3.7\times10^{-15}$ to $4.6\times10^{-15}$ | **closed form**: kernel-step modes of `homogeneous_sphere`, 40 000 modes |
| reference particle 1000 °C | $D_{\mathrm{kernel}} = D_{\mathrm{buffer}} = 1.14\times10^{-15}$, $D_{\mathrm{IPyC}} = D_{\mathrm{OPyC}} = 2.55\times10^{-15}$, $D_{\mathrm{SiC}} = 5.43\times10^{-18}$ | $1.44\times10^{-9}$ | $3.4\times10^{-14}$ | **self reference**: the same evaluation with 16 384 shooting modes |
| reference particle 1600 °C | $D_{\mathrm{kernel}} = D_{\mathrm{buffer}} = 1.68\times10^{-13}$, $D_{\mathrm{IPyC}} = D_{\mathrm{OPyC}} = 2.69\times10^{-13}$, $D_{\mathrm{SiC}} = 3.64\times10^{-15}$ | $9.30\times10^{-7}$ | $7.5\times10^{-15}$ to $1.4\times10^{-14}$ | self reference, 16 384 modes |
| strongly contrasted | $D_{\mathrm{kernel}} = 1.0\times10^{-11}$, $D_{\mathrm{buffer}} = 1.0\times10^{-9}$, $D_{\mathrm{IPyC}} = 1.0\times10^{-12}$, $D_{\mathrm{SiC}} = 1.0\times10^{-16}$, $D_{\mathrm{OPyC}} = 1.0\times10^{-12}$ | $2.67\times10^{-8}$ | $1.6\times10^{-9}$ | self reference, 16 384 modes |

Table 8.1. The geometry is the registered AGR-1 baseline (kernel 175 µm; outer radii of buffer, IPyC, SiC and OPyC 278.5, 317.9, 353.2 and 394.2 µm), except for the uniform sphere. $\alpha_1$ is the slowest mode rate. A closed-form reference is independent of the code under test; a self reference measures the error of the mode count and not the error of the shooting solution (Section 8.6).

* **Mode counts:** 16, 64, 256, 1024 and 2048 (the committed test: 8, 16, 32 and 64); references with at least 8 times as many modes (16 384 for the layered sets, the homogeneous particle 40 000 closed-form modes).
* **Decay constants** (s⁻¹, `data/nuclear/decay.csv`): none, Cs-137 ($7.30\times10^{-10}$), Cs-134 ($1.06\times10^{-8}$), I-131 ($1.00\times10^{-6}$), Kr-85m ($4.30\times10^{-5}$) and Ag-110 ($2.82\times10^{-2}$): from the longest to the shortest half-life of the registered unstable nuclides (the file registers also, for example, Sr-90 at $7.6\times10^{-10}$ s⁻¹, Ag-110m at $3.2\times10^{-8}$ s⁻¹ and Ba-137m at $4.5\times10^{-3}$ s⁻¹, which lie inside the covered range). The ratio $\lambda/\alpha_1$ reaches $1.96\times10^{7}$.
* **Sources and initial inventory:** decay only ($M^0 = 1$ mol); a source constant on ($M^0 = 0$ and $M^0 = 1$ mol); on and switched off at $3/\alpha_1$; off and switched on at $3/\alpha_1$; on, off from $1/\alpha_1$ and on again from $3/\alpha_1$; and a history of thirteen pieces of length $2/\alpha_1$ with rates $G\cdot\left\{1, 0, 0.5, 2, 0, 1, 0.1, 3, 0, 1, 0.5, 0, 1\right\}$, the last continuing. The rate is $G = 1/\left(V_{\mathrm K}\alpha_1^{-1}\right)$ mol m⁻³ s⁻¹.
* **Times:** from $0.3\,t_v$ of the largest mode count (below $t_v$, where the mask withholds, so that the table also shows the wrong early values) to $1000/\alpha_1$, 3 points per decade (the committed test: to $300/\alpha_1$, 1.5 per decade).
* **Quantities compared:** $M$, $M_{\mathrm{rel}}$ and, for a source, $\Phi$ (the flux of the closed-form references is available for $M^0 = 0$ only; the self references give it always).

**Quality of the references.** The homogeneous particle reference is the kernel-step series of `homogeneous_sphere` with closed-form eigenvalues and weights (no shooting). The uniform-sphere reference uses the physics module's series for the retained and decayed fractions of the uniform sphere. Two changes make it accurate to rounding: the series of the decayed fraction is completed beyond the summed terms by its closed-form tail, $\frac{6}{\left(\pi\right)^2}\lambda\sum_{k>n}\frac{1}{\left(k\right)^2\left(\left(k\right)^2a_1 + \lambda\right)}$ with $\sum_{k\ge1}\frac{1}{\left(k\right)^2\left(\left(k\right)^2 + x^2\right)} = \frac{1}{x^2}\left[\frac{\left(\pi\right)^2}{6} - \frac{\pi x\coth(\pi x) - 1}{2\left(x\right)^2}\right]$ (series in $x^2$ for $x < 0.5$); and the superposition integral over the source piece is evaluated by Gauss–Legendre rules in $\sqrt\theta$, in which the release of the sphere, which grows as $\sqrt\theta$, is smooth. Tests (`the_uniform_reference_agrees_with_the_physics_module`) compare the reference with the series summed directly from the small terms to the large ones: **OBSERVED** agreement within $10^{-15}$ for $\lambda$ from 0 to $4.3\times10^{-5}$ s⁻¹ and times from $0.01/a_1$ to $40/a_1$, and within $10^{-12}$ with the physics module's own naive forward sums (their rounding). A first version of this reference, with a cut of the integral at $10^{-4}$ of its range, had errors of $10^{-5}$ and was discarded.

### 8.4 Results

Table 8.2 gives, for each geometry, mode count $N$ and quantity, the largest **excess** over the sampled times, decay constants and sources,

$$
\text{excess} = \max\ \frac{\left|\text{value} - \text{reference}\right| - B}{\varepsilon_{\mathrm{floor}}\,A}\quad\left(\text{for }\Phi:\ S_\Phi\text{ in place of }A\right),
$$

in units of the floor term: how many times $\varepsilon_{\mathrm{floor}}A$ the observed error exceeds the computed bound $B$ at the worst sampled point. (A negative value would mean that the computed bound alone is enough everywhere.)

| Geometry | $N$ | $M$ | $M_{\mathrm{rel}}$ | $\Phi$ |
|---|---|---|---|---|
| uniform sphere | 16 | 1.009 | 6.523 | 0.597 |
| | 64 | 3.533 | 6.958 | 1.045 |
| | 256 | 3.533 | 6.523 | 1.045 |
| | 1024 | 3.533 | 6.958 | 1.045 |
| | 2048 | 3.533 | 6.523 | 1.045 |
| homogeneous particle | 16 | 0.918 | 0.720 | 0.543 |
| | 64 | 2.480 | 3.059 | 1.272 |
| | 256 | 2.341 | 3.325 | 1.202 |
| | 1024 | 2.761 | 2.793 | 1.887 |
| | 2048 | 3.459 | 3.448 | 2.582 |
| reference particle 1000 °C | 16 | 0.031 | 0.013 | 0.051 |
| | 64 | 0.863 | 0.847 | 0.847 |
| | 256 | 1.461 | 1.415 | 1.415 |
| | 1024 | 1.957 | 1.953 | 1.953 |
| | 2048 | 1.143 | 1.148 | 0.768 |
| reference particle 1600 °C | 16 | 0.315 | 0.303 | 0.121 |
| | 64 | 0.588 | 0.803 | 0.519 |
| | 256 | 1.918 | 1.913 | 1.169 |
| | 1024 | 1.510 | 1.485 | 0.735 |
| | 2048 | 1.302 | 1.303 | 0.799 |
| strongly contrasted | 16 | 0.016 | 0.016 | 0.016 |
| | 64 | 0.037 | 0.037 | 0.037 |
| | 256 | 0.065 | 0.065 | 0.065 |
| | 1024 | 0.038 | 0.038 | 0.038 |
| | 2048 | 0.011 | 0.011 | 0.011 |

Table 8.2 (full set: 73 364 sampled values over the five geometries; the program prints the number of points per row and the largest ratio of error to estimate, which is below 1 in every row). The smaller set of the committed test (mode counts 8 to 64, Cs-134, Kr-85m and Ag-110, times to $300/\alpha_1$) has the largest excess 3.53 ($M$), 8.37 ($M_{\mathrm{rel}}$, uniform sphere, $N = 16$) and 1.94 ($\Phi$, homogeneous particle, $N = 64$).

**OBSERVED**, over both sets:

1. The estimate holds at every sampled value, and near $t_v$ it is nearly reached: the largest ratio of error to estimate is 0.997 in the full set (uniform sphere, 16 modes) and 0.998 in the smaller set (8 modes), where the computed bound (52) is almost attained and the floor term is negligible. For the homogeneous particle and the layered sets the largest ratio is 0.43 or less (0.011 or less for the contrasted set), because the factors are generous there and the bound is not attained.
2. Beyond $B$ the excess never exceeds 3.53 ($M$), 8.37 ($M_{\mathrm{rel}}$) and 2.58 ($\Phi$) floor terms. The excess does not grow with the mode count (uniform sphere: 3.53 at 64 and at 2048 modes); it does not grow with $\lambda$ or with the time up to $1000/\alpha_1$, once the scale $A$ is used. (In scratch runs of this work, which the committed code does not reproduce, the plain input $M^0 + M_{\mathrm{gen}}$ as scale gave 20 floors at late times with decay, and the error of $\Phi$ with $M^0 > 0$ gave 97 floors at the first time after $t_v$; the age weighting and the term $M^0/t$ in the scales removed both.)
3. The largest excesses are in the closed-form geometries: the uniform sphere, whose modes are closed form (no shooting error), shows 3.5 to 8.4 floor terms of **rounding** of the evaluation and of the reference (its floor is the rounding unit, $2.2\times10^{-16}$); the homogeneous particle adds the shooting error, 2.5 to 3.5; the layered sets stay below 2.0 (their floor is 34 to $7\times10^{6}$ times larger, and their self reference shares the shooting error).
4. The weights of the shooting solution of the homogeneous particle differ from the closed-form weights by at most $4.0\times10^{-15}$ per mode (1.1 floors), and by $1.0\times10^{-14}$ (2.9 floors) in sum over 64 modes, $1.3\times10^{-14}$ (3.4 floors) over 256 and $2.0\times10^{-14}$ (4.3 floors) over 1024 (the example prints them). This is the observed $\varepsilon_{\mathrm{series}}$ of Section 8.2, point 2, and the basis of $f = 4$ in $\Delta$ (a test asserts that the sum stays below $4\varepsilon_{\mathrm{floor}}$ up to 256 modes).

### 8.5 The rule

From the observed excess (largest over both sets) with a margin of at least 2:

| Quantity | Largest excess | Factor | Margin |
|---|---|---|---|
| $M$: $c_M$ (`EXT_FACTOR_INVENTORY`) | 3.53 | 10 | 2.8 |
| $M_{\mathrm{rel}}$: $c_R$ (`EXT_FACTOR_RELEASE`) | 8.37 | 24 | 2.9 |
| $\Phi$: $c_\Phi$ (`EXT_FACTOR_FLUX`) | 2.58 | 8 | 3.1 |

The estimate (54) with these factors is the bound $e_{\mathrm{ext}}(N, q)$: at every sampled value of both sets $\left|\text{value} - \text{reference}\right| \le e$ (the largest ratio of error to estimate is below 1, a test asserts it at every point of the smaller set, an ignored test and the program at every point of the full set). The dependence on the mode count is in $B$ (through $\Delta$ and $t_v$) and in $\varepsilon_{\mathrm{floor}}$ (which grows with $N$ where the shooting solution loses digits); the factors themselves did not need to depend on $N$ in $[16, 2048]$.

**The mask, item 3 for a run with decay or a source.** A value is drawn where all hold:

1. items 1 and 2 of Section 4;
2. the run lies in the regime of Table 8.1 (`REGIME_*`): $16 \le N \le 2048$; $\varepsilon_{\mathrm{floor}} \le 2.0\times10^{-9}$; $\lambda \le 1.9\times10^{7}\alpha_1$; $\alpha_1t \le 1000$; at most 13 source pieces;
3. the estimated relative error (55) is at most 0.05.

Otherwise the value is withheld with its reason: `BeforeMinValidTime`, `CoverSensitivity`, `OutsideEstimatedRegime` (with the limit that is exceeded), `ExtensionErrorTooLarge` (with the estimated relative error), or `Undefined`. The existing series (no source, no decay) keeps its item 3 ($f_{\mathrm{series}}$). The constant `EXTENSION_FLOOR_REESTIMATED` is `true`: decision 68(c) authorises it once the independent verifier passes this section.

**Mask against the references.** The production function `KernelRun::drawn` was asked for $M/M^0$, $F$, $F_{\mathrm{gen}}$ and $\beta$ at every sampled point and compared with the references. Smaller set: 14 692 values asked, 4 391 drawn, 3 494 withheld before $t_v$, 1 671 outside the regime (the 8-mode series), 1 254 for an estimated error above 5 %, the rest by the cover test or for a quantity not defined; 7 095 unmasked values have a relative error above 5 % and all 7 095 are withheld. Full set: 76 095 values asked, 32 157 drawn, 19 023 withheld before $t_v$, 2 186 outside the regime (essentially the 1000 °C particle with Ag-110, whose ratio $\lambda/\alpha_1 = 1.96\times10^{7}$ lies just above the regime constant $1.9\times10^{7}$), 5 616 for an estimated error above 5 %, the rest by the cover test or for a quantity not defined; 36 155 unmasked values have a relative error above 5 % and all 36 155 are withheld. In both, no drawn value has a relative error above 5 % (the largest is 0.73 % in both sets), and every value whose unmasked relative error exceeds 5 % is withheld, among them every value before $t_v$ and the early values that still look plausible (a test asserts both). $F_{\mathrm{post}}$ was not sampled against references (its estimate (55) is the propagation of the estimates of the two states); it is derived, not measured.

**Mutants.** A test fails if the floor term of (54) is removed, if the computed bound is removed, if the factors are divided by 8, or if the computed bound is halved where it is not rounding: each leaves a sampled error above the estimate.

### 8.6 What is proved, observed and not proven

* **PROVED:** the structure (54) (model error bounded by (52), (53); series error and rounding proportional to $A$ or $S_\Phi$) and the bound (53) (Sections 3.1 and 8.2), given the completeness premise of Section 0 and the convention for $\theta \ge t_v$.
* **OBSERVED, over the designed set** (Table 8.1; 73 364 values of the full set, 5 geometries, 5 mode counts from 16 to 2048, 6 decay settings, 7 source histories, times $0.3\,t_v$ to $1000/\alpha_1$): the factors $c_M$, $c_R$, $c_\Phi$ of Section 8.5 with margin 2.8 to 3.1, and the shooting error of the weights of the homogeneous particle (Section 8.4).
* **NOT PROVEN:**
  1. The factors are not bounds proved for every geometry. They hold at every sampled point; the next geometry may differ.
  2. For the layered sets (the two reference particles and the contrasted set) the reference is a self reference: it contains the same shooting solution, so the shooting error of those sets is **not** measured. Their numbers measure the mode-count error and the rounding of the evaluation. For them the term $c\,\varepsilon_{\mathrm{floor}}A$ rests on `accuracy_floor()` being the error of the shooting solution up to the factors of Section 8.5, an assertion of the existing module (it states that the true error stayed 10 to 20 times below the estimate against the finite-volume solver on its test inputs) that this section does not re-verify; the closed-form homogeneous particle supports it for its own material only.
  3. The convention for $\theta \ge t_v$ ($e^{-40}\sum_\mu\left|w_\mu\right|$) is neglected; $\sum_\mu\left|w_\mu\right|$ is not bounded.
  4. $F_{\mathrm{post}}$ is not sampled; geometry other than the registered radii (kernel 175 µm) and the uniform sphere; $N > 2048$; $N < 16$; more than 13 source pieces; $\alpha_1t > 1000$; $\lambda > 1.9\times10^{7}\alpha_1$; $\varepsilon_{\mathrm{floor}} > 2\times10^{-9}$.
* **Withheld, with the reason `OutsideEstimatedRegime`:** every run outside the regime of Section 8.5. The estimate does not claim anything there, and no value is drawn.

### 8.7 Cost of the table

The smaller set takes tens of seconds in a release build and about 30 s as a debug test (the five geometries share one run); the full set runs for tens of minutes (the shooting solution with 16 384 modes costs about the square of the mode count). `cargo test --release --lib -- --ignored full_set` repeats the assertions of the full set.

## 9. Integration notes

* **Merge `716036a`** ("Merge the verified BTCS physics branch into the analytical extensions branch", no message body: it is pushed and is not rewritten, so its content is stated here). It merged `feat/btcs-physics` at `0e99250`, which had passed its own independent verification (round 2), into this branch. It brought Document 5 (generation, decay, temperature), the production BTCS with the physics terms (`src/numerical/physics_run.rs`, `physics_config.rs`, the changes of `btcs.rs` and of the exports, the tests `tests/btcs_physics.rs` and `tests/btcs_physics_cli.rs`), and the changes of Documents 1, 2 and 4. This branch needs it for check (iv) of Section 5 (the production solver `run_physics`, test `tests/analytical_extensions_btcs.rs`); the module `physics::reference` (uniform sphere with decay) came earlier with `7e0c99e` and is the closed-form reference of Section 8.
* **Section 8 and decision 68(c):** with this section the constant `EXTENSION_FLOOR_REESTIMATED` is set to `true` in the code. The decision authorises it once the independent verifier passes the re-estimate; if the verification fails, the constant goes back to `false` and `KernelRun::drawn` withholds the extension again (`Withheld::FloorNotReestimated`).
