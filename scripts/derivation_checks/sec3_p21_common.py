"""Shared helpers for the STRATO Phase 2.1 derivation checks (sec3_p21_*.py).

Everything here is written from the equations of Phase 2.1 and is independent of
the Rust code in src/. Two tools are provided:

1. Multilayer: eigenpairs of the layered-sphere Sturm-Liouville problem
       (1/r^2) d/dr (r^2 D_i dphi/dr) + Lambda phi = 0,
   phi regular at r = 0, c and D dphi/dr continuous at every interface,
   phi(R) = 0 (absorbing surface).  With u = r phi each layer has
   u'' + (Lambda/D_i) u = 0, and the state (u, q = D u') is carried outward
   with the jump rule q_{i+1} = q_i + (D_{i+1} - D_i) u / R_i  (Phase 1).
2. SphereFV: a conservative cell-centred finite-volume solver for
       dc/dt = (1/r^2) d/dr (r^2 D dc/dr) - lambda c + S
   with the theta method (Crank-Nicolson by default) and exact discrete
   inventory book-keeping (released and decayed amounts).

SI units throughout: r in m, D in m^2 s^-1, Lambda and lambda in s^-1,
c in mol m^-3, S in mol m^-3 s^-1, inventories in mol.

This module writes nothing to disk.
"""
import math

import numpy as np
from scipy.linalg import solve_banded
from scipy.optimize import brentq

GL_NODES, GL_WEIGHTS = np.polynomial.legendre.leggauss(16)


def report(name, ok, detail=""):
    """Print one PASS/FAIL line and return the boolean."""
    print(f"{'PASS' if ok else 'FAIL'}  {name}  {detail}")
    return bool(ok)


class Multilayer:
    """Layered sphere with outer radii R_1..R_L (m) and diffusivities D_1..D_L."""

    def __init__(self, radii, diffs):
        self.R = np.asarray(radii, dtype=float)
        self.D = np.asarray(diffs, dtype=float)
        self.Rin = np.concatenate([[0.0], self.R[:-1]])
        self.Rout = self.R[-1]

    # ---- layer transfer -------------------------------------------------
    def walk(self, lam):
        """States (u, q) at the inner edge of every layer and final (u, q) at R.

        Start (u, q) = (0, D_1) so that u'(0) = 1, i.e. u ~ r near the centre.
        """
        u, q = 0.0, self.D[0]
        states = []
        for i in range(len(self.D)):
            if i > 0:  # interface jump rule, flux continuity of D (u' - u/r)
                q = q + (self.D[i] - self.D[i - 1]) * u / self.Rin[i]
            states.append((u, q))
            D = self.D[i]
            k = math.sqrt(lam / D)
            L = self.R[i] - self.Rin[i]
            cs, sn = math.cos(k * L), math.sin(k * L)
            u, q = u * cs + q / (D * k) * sn, -D * k * u * sn + q * cs
        return states, u, q

    def g(self, z):
        """Characteristic function u(R) at Lambda = z^2."""
        return self.walk(z * z)[1]

    # ---- eigenvalues ----------------------------------------------------
    def eigenvalues(self, n_modes, oversample=40):
        """First n_modes eigenvalues Lambda_n (s^-1), bracketed on z = sqrt(Lambda)."""
        travel = np.sum((self.R - self.Rin) / np.sqrt(self.D))  # s^(1/2)
        dz = math.pi / travel / oversample
        roots = []
        z0, g0 = dz, self.g(dz)
        while len(roots) < n_modes:
            z1 = z0 + dz
            g1 = self.g(z1)
            if g0 == 0.0:
                roots.append(z0)
            elif g0 * g1 < 0.0:
                roots.append(brentq(self.g, z0, z1, xtol=1e-15 * z1, rtol=1e-15, maxiter=200))
            z0, g0 = z1, g1
        z = np.array(roots[:n_modes])
        return z * z

    # ---- eigenfunctions -------------------------------------------------
    def u_eval(self, lam, r):
        """u = r phi for eigenvalue lam at radii r (array)."""
        states, _, _ = self.walk(lam)
        r = np.asarray(r, dtype=float)
        out = np.zeros_like(r)
        for i, (u0, q0) in enumerate(states):
            D = self.D[i]
            k = math.sqrt(lam / D)
            m = (r >= self.Rin[i]) & (r <= self.R[i])
            s = r[m] - self.Rin[i]
            out[m] = u0 * np.cos(k * s) + q0 / (D * k) * np.sin(k * s)
        return out

    def du_at_R(self, lam):
        """u'(R) for eigenvalue lam."""
        _, _, q = self.walk(lam)
        return q / self.D[-1]

    def grid(self, kmax_per_layer, rad_per_panel=3.0):
        """Composite Gauss-Legendre nodes and weights over [0, R], layer by layer."""
        rs, ws, layer = [], [], []
        for i in range(len(self.D)):
            a, b = self.Rin[i], self.R[i]
            npan = max(2, int(math.ceil(kmax_per_layer[i] * (b - a) / rad_per_panel)) + 1)
            edges = np.linspace(a, b, npan + 1)
            h = 0.5 * np.diff(edges)
            mid = 0.5 * (edges[1:] + edges[:-1])
            rs.append((mid[:, None] + h[:, None] * GL_NODES[None, :]).ravel())
            ws.append((h[:, None] * GL_WEIGHTS[None, :]).ravel())
            layer.append(np.full(rs[-1].size, i))
        return np.concatenate(rs), np.concatenate(ws), np.concatenate(layer)

    def mode_integrals(self, lams, src_layers):
        """Per-mode integrals (weight 1 for u, i.e. r^2 for phi):

        N_n  = int_0^R u_n^2 dr            = int r^2 phi_n^2 dr       (m^3 * scale^2)
        I_n  = int_0^R r u_n dr            = int r^2 phi_n dr
        Is_n = int_{source layers} r u_n dr
        dU_n = u_n'(R)
        """
        N, I, Is, dU = (np.zeros(len(lams)) for _ in range(4))
        src = np.zeros(len(self.D), dtype=bool)
        src[list(src_layers)] = True
        for n, lam in enumerate(lams):
            kmax = np.sqrt(lam / self.D)
            r, w, lay = self.grid(kmax)
            u = self.u_eval(lam, r)
            N[n] = np.sum(w * u * u)
            I[n] = np.sum(w * r * u)
            Is[n] = np.sum((w * r * u)[src[lay]])
            dU[n] = self.du_at_R(lam)
        return N, I, Is, dU

    def zero_crossings(self, lam, npts):
        """Interior sign changes of u_n (Sturm oscillation check: equals n - 1)."""
        r = np.linspace(0.0, self.Rout, npts)[1:-1]
        u = self.u_eval(lam, r)
        return int(np.sum(u[1:] * u[:-1] < 0.0))


class SphereFV:
    """Conservative spherical finite volumes with decay and source (theta method)."""

    def __init__(self, radii, diffs, cells_per_layer):
        radii = np.asarray(radii, dtype=float)
        rin = np.concatenate([[0.0], radii[:-1]])
        faces, dcell, lay = [0.0], [], []
        for i, (a, b, n) in enumerate(zip(rin, radii, cells_per_layer)):
            faces.extend(np.linspace(a, b, n + 1)[1:])
            dcell.extend([diffs[i]] * n)
            lay.extend([i] * n)
        self.rf = np.array(faces)                      # face radii (m)
        self.rc = 0.5 * (self.rf[1:] + self.rf[:-1])   # cell centres (m)
        self.Dc = np.array(dcell)                      # cell diffusivity (m^2 s^-1)
        self.layer = np.array(lay)
        self.V = 4.0 / 3.0 * np.pi * (self.rf[1:] ** 3 - self.rf[:-1] ** 3)  # m^3
        A = 4.0 * np.pi * self.rf ** 2                 # face areas (m^2)
        n = self.rc.size
        G = np.zeros(n + 1)                            # face conductances (m^3 s^-1)
        for j in range(1, n):
            res = (self.rf[j] - self.rc[j - 1]) / self.Dc[j - 1] + (self.rc[j] - self.rf[j]) / self.Dc[j]
            G[j] = A[j] / res
        G[n] = A[n] * self.Dc[-1] / (self.rf[n] - self.rc[-1])  # absorbing surface, c(R) = 0
        self.G = G
        # K C = net outflow from each cell (mol s^-1 per unit c)
        self.diag = G[:-1] + G[1:]
        self.off = -G[1:-1]

    def K_apply(self, C):
        out = self.diag * C
        out[:-1] += self.off * C[1:]
        out[1:] += self.off * C[:-1]
        return out

    def step(self, C, dt, lam, S, theta=0.5):
        """One theta step. S is the cell source (mol m^-3 s^-1), constant over the step.

        Returns new C, released amount and decayed amount over the step (mol).
        """
        V = self.V
        ab = np.zeros((3, C.size))
        ab[0, 1:] = theta * self.off
        ab[1, :] = V / dt + theta * (self.diag + lam * V)
        ab[2, :-1] = theta * self.off
        rhs = V / dt * C - (1.0 - theta) * (self.K_apply(C) + lam * V * C) + V * S
        Cn = solve_banded((1, 1), ab, rhs)
        Cbar = theta * Cn + (1.0 - theta) * C
        rel = dt * self.G[-1] * Cbar[-1]
        dec = dt * lam * np.sum(V * Cbar)
        return Cn, rel, dec

    def inventory(self, C):
        return float(np.sum(self.V * C))
