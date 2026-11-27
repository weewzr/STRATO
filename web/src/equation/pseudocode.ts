// Verbatim blocks of docs/mathematics/btcs/03_rust_pseudocode.md (the Rust-oriented pseudocode of the
// BTCS solver), for the Pseudocode section of the Numerical Interpretation. GENERATED from that file; the
// unit test tests/pseudocode.test.ts reads the document and fails if a block here differs from it by one
// character. Nothing is rewritten: a routine is shown as the document gives it.

export interface PseudocodeBlock {
  /** The identifier and the heading of the routine in the document, for example "PS-05 `assemble` : matrix assembly". */
  id: string;
  title: string;
  /** The code block, exactly as in the document. */
  code: string;
}

/** The routines of the baseline BTCS run (document 03, Section 2), in the order of one run. */
export const PSEUDOCODE: readonly PseudocodeBlock[] = [
  {
    "id": "PS-05",
    "title": "PS-05 `assemble` : matrix assembly",
    "code": "fn assemble(grid, coeff, dt) -> Result<Assembly> {\n    let n = grid.n\n    let mut lower = vec![0.0; n]; let mut diag = vec![0.0; n]; let mut upper = vec![0.0; n]\n\n    // Row 0: centre, F12 (PS-06)\n    let (b0, u0) = centre_row(coeff.fo[Region::Kernel.index()])\n    diag[0] = b0; upper[0] = u0                        // lower[0] is absent\n\n    // Rows 1 .. N-1: the one general row, F11, for every class of node (interior, interface, row N-1)\n    for i in 1..n {                                    // 1 <= i <= N-1\n        let k_in  = coeff.k_outer[i - 1]               // K_{i,inner}\n        let k_out = coeff.k_outer[i]                   // K_{i,outer}\n        let l = -dt * k_in  / grid.v[i]                // L_i  (F11)\n        let u = -dt * k_out / grid.v[i]                // U_i  (F11)\n        lower[i] = l\n        diag[i]  = 1.0 - l - u                         // B_i = 1 + |L_i| + |U_i|  (F11)\n        upper[i] = u\n    }\n    lower[1] = 0.0                                     // F13: exact zero (K_{1,inner} = K_{0,outer} = 0.0)\n    // Row N-1: F17. The boundary value C_N = 0 (F18) removes the matrix entry, not the diagonal.\n    upper[n - 1] = 0.0                                 // U_{N-1} stays inside diag[n-1], not a matrix entry\n\n    for every entry: require finite\n    return Assembly { matrix: Tridiagonal { lower, diag, upper }, coeff, dt }\n}"
  },
  {
    "id": "PS-10",
    "title": "PS-10 `ThomasFactors` : tridiagonal solution",
    "code": "fn new(a: &Tridiagonal) -> Result<ThomasFactors> {\n    let n = a.diag.len()\n    let mut m = vec![0.0; n]; let mut w = vec![0.0; n]\n    m[0] = a.diag[0]\n    for i in 1..n {\n        w[i] = a.lower[i] / m[i - 1]                       // w_i = L_i / m_{i-1}   (<= 0)\n        m[i] = a.diag[i] - w[i] * a.upper[i - 1]           // m_i = B_i - w_i U_{i-1}\n    }\n    for i in 0..n { require m[i] finite and > 0.0 }        // the hypothesis of P6\n    return ThomasFactors { w, m, upper: a.upper.clone() }\n}\n\nfn solve(&self, rhs, x, work) {                            // work = b'\n    let n = self.m.len()\n    work[0] = rhs[0]\n    for i in 1..n { work[i] = rhs[i] - self.w[i] * work[i - 1] }        // b'_i = b_i - w_i b'_{i-1}\n    x[n - 1] = work[n - 1] / self.m[n - 1]\n    for i in (0..n - 1).rev() { x[i] = (work[i] - self.upper[i] * x[i + 1]) / self.m[i] }\n}"
  },
  {
    "id": "PS-11",
    "title": "PS-11 `Stepper::step` : one time step",
    "code": "fn step(&mut self, c_old, c_new) {\n    // F20: right-hand side b_i^j = C_i^j, 0 <= i <= N-1\n    // (Phase G1 adds the source term here, PS-19; the boundary adds nothing, F18)\n    self.rhs[..n].copy_from_slice(&c_old[..n])\n    self.factors.solve(&self.rhs, &mut c_new[..n], &mut self.work)     // PS-10: C_0^{j+1} .. C_{N-1}^{j+1}\n    c_new[n] = 0.0                                                    // F18 (PS-08)\n}"
  },
  {
    "id": "PS-12",
    "title": "PS-12 `inventory` : inventory accounting",
    "code": "fn inventory(grid, c) -> f64 {\n    let mut sum = 0.0\n    for i in 1..grid.n { sum += grid.v[i] * c[i] }      // F25: 1 <= i <= N-1; V_0 = 0 and C_N = 0\n    4.0 * PI * sum\n}"
  },
  {
    "id": "PS-13",
    "title": "PS-13 `release_rate` and `Accumulator` : release calculation",
    "code": "fn release_rate(k_last, c_last) -> f64 { 4.0 * PI * k_last * c_last }       // F26\n\n// in the run loop, after the step that produced c_new (level j + 1):\nlet phi = release_rate(coeff.k_outer[n - 1], c_new[n - 1])                  // the stored K, never recomputed\nreleased.add(dt * phi)                                                      // F27, compensated\nlet f = released.value() / initial_inventory                               // F28\n\nfn add(&mut self, x) {                      // compensated (Kahan) summation of the increments\n    let y = x - self.compensation\n    let t = self.sum + y\n    self.compensation = (t - self.sum) - y\n    self.sum = t\n}"
  },
  {
    "id": "PS-16",
    "title": "PS-16 `run` : time stepping",
    "code": "fn run(cfg, limits, snapshot_times) -> Result<RunResult> {\n    let input = validate_input(cfg, limits)?                              // PS-01\n    let grid = Grid::build(&input)?                                       // PS-02, PS-03\n    let mut stepper = Stepper::new(&grid, input.d, input.dt)?             // PS-04, PS-05, PS-10\n    let (mut c_old, _theta, m0) = initial_vector(&grid, input.c_init)     // PS-09\n    let mut c_new = vec![0.0; grid.n + 1]\n    let mut released = Accumulator::new()\n    let mut series = vec![ (0.0, row_at_start(&grid, &c_old, m0)) ]       // j = 0: release_rate = None, F = 0, residual = 0.0 by definition\n    for step in 1..=input.steps {                                         // j = step - 1  ->  j + 1 = step\n        stepper.step(&c_old, &mut c_new)                                  // PS-11\n        let phi = release_rate(stepper.k_last(), c_new[grid.n - 1])       // PS-13, F26\n        released.add(input.dt * phi)                                      // F27\n        swap(&mut c_old, &mut c_new)                                      // c_old now holds C^{j+1}\n        let t = step as f64 * input.dt\n        if step % input.output_every == 0 || step == input.steps {\n            let m = inventory(&grid, &c_old)                              // PS-12\n            let row = StepDiagnostics {\n                inventory: m,\n                cumulative_release: released.value(),\n                release_fraction: released.value() / m0,                  // F28\n                release_rate: Some(phi),\n                balance_residual: balance_residual(m, released.value(), m0),   // PS-14\n                min_concentration: min(&c_old[..=grid.n]),\n                max_concentration: max(&c_old[..=grid.n]),\n            }\n            require every field of row finite\n            series.push((t, row))\n        }\n        keep a profile copy for every requested snapshot time that this step reaches (nearest output step, never interpolated)\n    }\n    return RunResult { ... , final_profile: c_old }\n}"
  }
];

/** Rows of the table of hooks of PS-19 (document 03): where each mechanism enters the routines above. */
export const HOOKS = {
  "G1": {
    "mechanism": "G1 generation, source $G$ ($\\mathrm{mol\\ m^{-3}\\ s^{-1}}$)",
    "entersAt": "PS-11 right-hand side (F20); PS-14 balance; maybe PS-13",
    "change": "`rhs[i] = c_old[i] + source_term(i)`; new accumulator `generated`; new `StepDiagnostics.generated`"
  },
  "G2": {
    "mechanism": "G2 decay, $\\lambda$ ($\\mathrm{s^{-1}}$)",
    "entersAt": "PS-05 diagonals of (F11), (F12), (F17); PS-14 balance",
    "change": "`diag[i] += decay_term(i)`; new accumulator `decayed`; new `StepDiagnostics.decayed`"
  },
  "G3": {
    "mechanism": "G3 temperature, $D_{\\mathrm{material}}(T)$",
    "entersAt": "PS-03 `DiffusivitySet`; PS-04, PS-05, PS-10 on every change",
    "change": "`Stepper::rebuild` is called whenever $T$ changes; the same $D$ values feed (F6), (F8), (F11), (F12), (F14), (F16) and (F26)"
  }
} as const;
