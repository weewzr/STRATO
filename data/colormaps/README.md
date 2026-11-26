# Colour maps of the web demo

Display data, not scientific data. The web demo colours the particle by the concentration that the
Rust solver returns, through one of these two tables (`prompt/DESIGN_SPECIFICATION.md` Section 9:
cividis by default, viridis as the option, never a rainbow map).

| File | Table | Copied from |
|---|---|---|
| `cividis.csv` | `_cividis_data`, 256 entries | matplotlib 3.11.2, `lib/matplotlib/_cm_listed.py` |
| `viridis.csv` | `_viridis_data`, 256 entries | matplotlib 3.11.2, `lib/matplotlib/_cm_listed.py` |

How the copy was made (2026-11-25): matplotlib 3.11.2 was installed from PyPI into a virtual
environment; the two lists were copied from `_cm_listed.py` (SHA-256 of that file
`ddad3698f5129ceb1792a445371286c08bc9080298e657b3054aea19c9659ef9`) with their decimal text
unchanged, one row per entry (`index,r,g,b`, components in 0 to 1). The copied values were compared
with `matplotlib.colormaps[name](range(256))` and agree exactly (largest difference 0.0).

`strato-webtools colormaps` (crates/strato-webtools) reads these files and writes
`web/src/generated/colormaps.json`, which adds the 8-bit form that matplotlib itself uses for
`bytes=True` (`int(255 * value)`).

Origin and licence:

- matplotlib is distributed under the Matplotlib License (a PSF-style licence),
  Copyright (c) 2012- Matplotlib Development Team; All Rights Reserved.
  https://matplotlib.org/stable/project/license.html. The tables are copied unchanged.
- cividis: J. R. Nuñez, C. R. Anderton and R. S. Renslow (2018), "Optimizing colormaps with
  consideration for color vision deficiency to enable accurate interpretation of scientific data",
  PLOS ONE 13(7): e0199239, https://doi.org/10.1371/journal.pone.0199239.
- viridis: the default colour map of matplotlib since version 2.0.
