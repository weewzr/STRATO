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

The inner product is $\langle f,g\rangle = \int_0^{R_5}r^2fg\,dr$ and $\varphi_\mu$ are the eigenfunctions of the layered operator (regular centre, perfect contact, $\varphi_\mu(R_5)=0$) with $\|\varphi_\mu\|^2 = \langle\varphi_\mu,\varphi_\mu\rangle$. The weights are $w_\mu = \dfrac{\langle\chi,\varphi_\mu\rangle\langle 1,\varphi_\mu\rangle}{\|\varphi_\mu\|^2\langle\chi,1\rangle}$ with $\chi$ the kernel indicator; this is the formula of Section 4.1 of the science design ($\langle\chi,1\rangle = \left(R_1\right)^3/3$).

**Premise used throughout (NOT PROVEN in this repository, standard theory):** the eigenfunctions $\varphi_\mu$ are complete in the weighted $L^2$ space of the layered operator (regular Sturm–Liouville problem with piecewise-constant coefficients).

## 1. The formulas

### 1.1 The mode equation (B1)

Write $C(r,t) = \sum_\mu c_\mu(t)\varphi_\mu(r)$. The decay-free, source-free operator has $\varphi_\mu$ as eigenfunctions with rate $\alpha_\mu$. Decay adds $-\lambda C$ with the same $\lambda$ in every layer, so each mode gets the rate $a_\mu = \alpha_\mu + \lambda$. The source $G(t)\chi$ has the kernel indicator as its spatial shape, so its projection on mode $\mu$ is $G(t)\langle\chi,\varphi_\mu\rangle/\|\varphi_\mu\|^2$. With $m_\mu = 4\pi c_\mu\langle1,\varphi_\mu\rangle$ (the inventory in mode $\mu$, because the inventory of $c_\mu\varphi_\mu$ is $4\pi\int r^2c_\mu\varphi_\mu\,dr$), projection and multiplication by $4\pi\langle1,\varphi_\mu\rangle$ give, since $4\pi\langle\chi,1\rangle = V_{\mathrm K}$,

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

**(a) $\sum_\mu\left|w_\mu\right| < \infty$ and $\sum_\mu w_\mu = 1$ (PROVED given completeness).** By Bessel's inequality $\sum_\mu\langle\chi,\varphi_\mu\rangle^2/\|\varphi_\mu\|^2 \le \langle\chi,\chi\rangle$ and the same for $1$, and Cauchy–Schwarz gives $\sum_\mu|w_\mu| \le \sqrt{\langle\chi,\chi\rangle\langle1,1\rangle}/\langle\chi,1\rangle < \infty$. Parseval's identity for the complete system gives $\langle\chi,1\rangle = \sum_\mu\langle\chi,\varphi_\mu\rangle\langle1,\varphi_\mu\rangle/\|\varphi_\mu\|^2$, that is $\sum_\mu w_\mu = 1$. The weights change sign, so a partial sum can be near 1 by cancellation, and the **missing weight** $W_{\mathrm{miss}} = 1 - \sum_{\mu\le\mu_{\max}}w_\mu$ is reported as a diagnostic only.

**(b) The existing series is the case $\lambda = 0$ (PROVED).** $R(\theta) = \sum_\mu w_\mu e^{-\alpha_\mu\theta}$, hence

$$
\sum_\mu w_\mu e^{-a_\mu\theta} = e^{-\lambda\theta}R(\theta). \tag{47}
$$

### 2.2 The problem with a truncated (43)

Sum (43) over the held modes only. The terms of the omitted modes are $w_\mu\alpha_\mu(1 - e^{-a_\mu t})/a_\mu \approx w_\mu$ (they have decayed), so the truncated sum is short by about $W_{\mathrm{miss}}$ at all times, which is 1e-3 or larger for the series of the tests and far above a release fraction of 1e-5. The existing $F_0 = 1 - \sum_\mu w_\mu e^{-\alpha_\mu t}$ avoids it by (a). Completing the omitted modes with $W_{\mathrm{miss}}$ is exact for $\lambda = 0$, but for $\lambda > 0$ it misses $\lambda\sum_{\mu>\mu_{\max}}w_\mu/a_\mu$, of size $\lambda W_{\mathrm{miss}}/\alpha_{\max}$. **OBSERVED** (test scratch, $D = 10^{-10}$ m² s⁻¹, $\lambda = 5\times10^{-3}$ s⁻¹): at $F_{\mathrm{rel}} = 3.19\times10^{-5}$ the completed truncation gave $3.46\times10^{-5}$ with 21 modes, $3.25\times10^{-5}$ with 42 and $3.195\times10^{-5}$ with 96, the error scaling as $1/\alpha_{\max}$, and the same for the source sums. That error is far above `accuracy_floor()` and not bounded a priori.

### 2.3 The evaluation used

Integrate (47) and use (a). For a unit kernel-step inventory released with decay,

$$
F_\lambda(\theta) = \sum_\mu w_\mu\alpha_\mu\frac{1 - e^{-a_\mu\theta}}{a_\mu} = \sum_\mu w_\mu\left(1 - e^{-a_\mu\theta}\right) - \lambda\sum_\mu w_\mu\frac{1 - e^{-a_\mu\theta}}{a_\mu} = 1 - e^{-\lambda\theta}R(\theta) - \lambda E(\theta), \tag{48}
$$

with $E(\theta) = \int_0^\theta e^{-\lambda s}R(s)\,ds = \sum_\mu w_\mu(1 - e^{-a_\mu\theta})/a_\mu$. The exchange of sum and integral is justified by $\sum_\mu|w_\mu|/a_\mu \le \sum_\mu|w_\mu|/\alpha_1 < \infty$. **PROVED.** Equation (48) uses the complete set only through $R$.

Because $R$ is the remaining fraction of the existing series, (48) needs no tail of the modal sum. The series holds the modes $\mu \le \mu_{\max}$ and is exact for $\theta \ge t_v$ up to the content $e^{-40}$ of the omitted modes (the module's convention behind `min_valid_time`). For $\theta < t_v$ the series is not used: $R$ is replaced by $1$. The model remaining fraction is

$$
\hat R(\theta) = \begin{cases}1, & \theta < t_v,\\ \sum_{\mu\le\mu_{\max}}w_\mu e^{-\alpha_\mu\theta}, & \theta \ge t_v,\end{cases}\qquad \Delta = 1 - \sum_{\mu\le\mu_{\max}}w_\mu e^{-\alpha_\mu t_v} + \text{(the series' accuracy floor)}. \tag{49}
$$

$F_0$ is not decreasing, so $0 \le 1 - R(\theta) = F_0(\theta) \le F_0(t_v) \le \Delta$ for $\theta < t_v$. **PROVED** (monotonicity: $C \ge 0$ by the parabolic maximum principle for non-negative data and zero surface value, so the outward flux at the absorbing surface is $\ge 0$ and the cumulative release does not decrease). $\Delta$ is computed from the series itself; it is large if the cover is too small, and then the mask rejects the value.

## 3. Closed forms of the evaluation

With $\hat R$ in place of $R$, define for $\theta \ge 0$ (units of $\theta$: s), with $\ell(x) = \int_0^xe^{-\lambda s}ds = \left(1 - e^{-\lambda x}\right)/\lambda$ ($=x$ for $\lambda=0$) and $\ell_2(x) = \int_0^x\ell = \left(\lambda x - 1 + e^{-\lambda x}\right)/\lambda^2$ ($= x^2/2$ for $\lambda = 0$), and $c_\mu = w_\mu e^{-a_\mu t_v}$, $L = \theta - t_v$:

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
\left|\hat M_{\mathrm{rel}} - M_{\mathrm{rel}}\right| &\le \Delta\left[M^0\left(\mathbf 1_{t<t_v} + \lambda\min(t,t_v)\right) + V_{\mathrm K}\sum_kG_k\left(\ell_k + \lambda t_v\,\mathrm{len}_k\right)\right],\\
\left|\hat\Phi - \Phi\right| &\le 2\Delta V_{\mathrm K}\sum_{k:\ s_k<t}G_k\left(1 + \lambda t_v\right)\quad (t \ge t_v),
\end{aligned}\tag{52}
$$

and $\left|\hat M_{\mathrm{dec}} - M_{\mathrm{dec}}\right|$ is bounded by the sum of the first two. Derivation: $\hat F_\lambda - F_\lambda = -e^{-\lambda\theta}e(\theta) - \lambda\int_0^\theta e^{-\lambda s}e(s)\,ds$, so $-\Delta\left(\mathbf 1_{\theta<t_v} + \lambda\min(\theta,t_v)\right) \le \hat F_\lambda - F_\lambda \le 0$; integrating over each increment gives the second line, and $\hat M - M$ is the same with $e^{-\lambda\theta}e(\theta)\ge 0$. **PROVED** apart from the convention for $\theta\ge t_v$ and the mode truncation of $\Delta$. They are computed and returned by the code (`ModalState::inventory_error_bound`, `release_error_bound`, `flux_error_bound`); the flux bound is valid for $t\ge t_v$ only. They are bounds on the model error, not on the shooting error of the eigenvalues and weights (that is `accuracy_floor()`, Section 4).

**OBSERVED** (decay, no source, $D = 10^{-10}$ m² s⁻¹, $\lambda = 5\times10^{-3}$ s⁻¹, 21 modes, $t_v = 17.8$ s): at $t = 20$ s the error against the independent closed form is $7.8\times10^{-8}$, the bound $1.0\times10^{-6}$, the release fraction $3.2\times10^{-5}$.

## 4. The drawing mask (science design Section 4.2)

No a-priori tail bound for the *modal sums* is claimed. A value $V(t)$ of $F$, $F_{\mathrm{gen}}$, $F_{\mathrm{post}}$, $M/M^0$ or $\beta$ is drawn only if all hold; otherwise no value is returned, with the reason (`Withheld`):

1. $t \ge t_v$ (for $F_{\mathrm{post}}$ also $t_e$);
2. $\left|V_{\alpha_{\max}} - V_{\alpha_{\max}/4}\right| \le 0.05\left|V_{\alpha_{\max}}\right|$, where the second value uses only the modes with $\alpha_\mu \le \alpha_{\max}/4$ (and so $t_v' = 4t_v$ in (49)) (**CONVENTION:** the labelled 5 % of the science design, not a derived bound);
3. for a release quantity, $f\cdot$`accuracy_floor()` $\le 0.05\,V$ with $V$ as a fraction of $M^0 + M_{\mathrm{gen}}$ (for $F_{\mathrm{post}}$ of $M(t_e)$; for $\beta$ the value itself, **CONVENTION**), with the factor $f$ below.

Item 2 is stricter than the science design may have intended near $t_v$: for the extension quantities the reduced-cover value has no release before $4t_v$ (Section 2.3), so a release that is not already negligible at $t_v$ fails item 2 for $t_v \le t < 4t_v$. That is conservative, not wrong. **The factor $f$ (FINDING).** The science design assumed that `accuracy_floor()`, the estimate for the decay-free `release_fraction`, also holds for (43) to (46) (check (v)), and asks that item 3 use the observed error if it does not. **OBSERVED:** at values of 1 to 100 times the floor (five times, one homogeneous layer, 67 modes, floor $5.30\times10^{-16}$) the difference between the shooting series and the closed-form modes is $1.55\times10^{-15}$, 3.34 times the floor (worst case). The decay-free series differs from the closed-form modes by the same $1.55\times10^{-15}$ ($2.55\times10^{-15}$ against $1.0\times10^{-15}$ at the first point): the offset belongs to the existing series (its weights differ from the closed-form weights by up to $3.8\times10^{-15}$), and the extension adds none. The code therefore uses $f = 4$ (`FLOOR_FACTOR_FROM_CHECK_V`), the observed ratio rounded up: a **CONVENTION** from one geometry. The floor of the existing series is to be re-estimated; until then the literal check (v) criterion (difference $\le$ floor) is **not met**.

## 5. The five verifier checks

All results are numerical verifications (cargo tests in `src/analytical/extensions.rs`), not experimental validations.

| Check | Statement | Result |
|---|---|---|
| (i) | $G = 0$, $\lambda = 0$ reproduces `release_fraction` bit for bit; $\lambda\to0$ and $G\to0$ within the series' `accuracy_floor()` | OBSERVED: bit for bit (equality asserted); $\lambda = 10^{-30}$ to $10^{-20}$ s⁻¹ and $G = 10^{-30}$ to $10^{-20}$ mol m⁻³ s⁻¹ within the floor |
| (ii) | $\lambda = 0$, constant $G$, $t\to\infty$: $\Phi\to V_{\mathrm K}G$ and $M_\infty = 4\pi\int_0^{R_5}C_{\mathrm{ss}}r^2dr$ with D5 (14) integrated layer by layer in closed form | OBSERVED: five different diffusivities ($10^{-12}$, $4\times10^{-10}$, $8\times10^{-13}$, $2\times10^{-14}$, $7\times10^{-13}$ m² s⁻¹), 64 modes: $M$ against D5 (14) $5.6\times10^{-13}$ relative; $\Phi/(V_{\mathrm K}G)$ equals 1 to $10^{-9}$; $\beta\to1$. The truncated modal sum $V_{\mathrm K}G\sum w/\alpha$ differs from D5 (14) by $1.5\times10^{-9}$ (tail of order $N^{-3}$, OBSERVED not proved). With decay (one material, closed form with $\sinh$ matching at $R_1$): $2.0\times10^{-15}$ relative |
| (iii) | one homogeneous layer reproduces D5 (21) and the Duhamel row | OBSERVED: shooting modes against the closed-form modes of `homogeneous_sphere` agree to $10^{-9}$ in $M$, $M_{\mathrm{rel}}$, $M_{\mathrm{dec}}$ (decay and two source segments); decay against the closed form $F_\lambda = e^{-\lambda\theta}F_0 + \lambda\int_0^\theta e^{-\lambda s}F_0$ (rigorous series of `homogeneous_sphere`) and against the physics module's uniform-start sphere within its stated tail bound $6/(\pi^2N)$; source against the superposition of that closed form |
| (iv) | BTCS with the D5 rows converges to (B5) at order 2 in space, 1 in time | **NOT RUN as specified**: the BTCS with D5 rows is on `feat/btcs-physics`, which is not in this branch's base. A **surrogate** was run: a test-local scalar scheme (backward Euler, centred differences on $u = rC$, uniform spacing, kernel surface a node with half weight), NOT the production solver. OBSERVED: self-convergence ratio 4.000 in space (order 2.000) and 2.29 in time (order 1.20, pre-asymptotic); the extrapolated limit equals (44) at $2.0\times10^{-10}$ relative, the finest run $2.0\times10^{-5}$. Check (iv) with the production solver remains to be run at integration |
| (v) | floor test at 1 to 100 times `accuracy_floor()` | **FINDING, not met literally**: worst ratio 3.34 (Section 4); the extension adds no error beyond the existing series |

Refusals (each tested): unequal decay constants; negative or non-finite $\lambda$, $M^0$, $t$, source rates, breakpoints; breakpoints not starting at 0 or not strictly increasing; rate and breakpoint counts that do not match; non-positive kernel volume; an empty or inconsistent series; a quantity that is undefined ($F$ with $M^0 = 0$, $\beta$ with $G = 0$, $F_{\mathrm{post}}$ before $t_e$).

## 6. The applicability function (science design Section 4.3)

`applicability(stage, source, decay, temperature)` returns `Exact`, `Extension(Decay | Source | SourceWithDecay)` or `NotApplicable(reason)`. Order of the checks: invalid input; a temperature that varies in time (a series of breakpoints is constant only when every value is exactly equal); a mechanism the stage does not contain (stages 1 and 4: no source, no decay; stage 2: no decay); a source that is not piecewise constant, or not uniform in the kernel; unequal decay constants. A source whose rates are all zero counts as no source, and $\lambda = 0$ as no decay. Then:

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
| Error bounds (52) | PROVED, except the neglected $e^{-40}\sum_\mu\left|w_\mu\right|$ (CONVENTION) |
| Mask items 1 to 3 | CONVENTION (5 %, $f = 4$); no a-priori bound of the modal sums |
| Cover/4 sensitivity (item 2) is a bound on the truncation error | NOT PROVEN |
| `accuracy_floor()` is the error of (43) to (46) | NOT MET: 3.34 times in check (v) |
| Check (iv) with the production BTCS | NOT RUN (surrogate only) |
