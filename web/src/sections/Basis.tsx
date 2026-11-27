import '../design/basis.css';
import { DEFAULT_CASE, findCase } from '../data/case';
import { layerColour } from '../design/layers';
import { equationTex } from '../equation/physics';
import { Tex } from '../equation/Tex';
import { sciText } from '../format';
import reference from '../generated/agr1_reference.json';
import { METHOD_1, METHOD_2 } from '../methods';
import { ringsOf } from '../particle/geometry';
import { hrefDocumentation } from '../route';
import { SectionHeading, TableBox } from './common';

// The Basis page: what STRATO is for and the model it solves, in diagrams, one equation and small tables.
// Every value comes from the generated registers (the AGR-1 reference geometry, the synthetic case file);
// nothing on it is a new scientific statement. The derivations, the limits and the evidence are in
// Scientific Documentation.

const SETS = reference.data.sets;
const AGR1 = SETS.find((s) => s.set_id === reference.data.default_set) ?? SETS[0];

const W = 200;
const H = 176;
const CX = 100;
const CY = 88;
const R = 82;

function LayerDiagram() {
  if (!AGR1) return null;
  const rings = ringsOf(AGR1.layers.map((l) => ({ name: l.layer, outer_radius: l.outer_radius_um })));
  if (!rings) return null;
  const k = R / 104; // ringsOf draws to a radius of 104 units
  return (
    <figure className="basis__figure" data-testid="basis-particle">
      <svg
        className="basis__svg"
        viewBox={`0 0 ${W} ${H}`}
        role="img"
        aria-label={`TRISO particle drawn to scale from the registered AGR-1 dimensions, five concentric layers from the centre: ${AGR1.layers.map((l) => l.layer).join(', ')}.`}
      >
        {[...rings].reverse().map((ring) => (
          <circle key={ring.index} cx={CX} cy={CY} r={ring.outer * k} className="basis__ring" style={{ ['--ring' as string]: layerColour(ring.index) }} />
        ))}
        <line x1={CX} y1={CY} x2={CX + R} y2={CY} className="basis__radius" />
      </svg>
      <figcaption className="figure__caption">
        <strong>Figure 1.</strong> Five concentric layers, AGR-1 reference geometry (set {AGR1.set_id}), drawn to scale.
      </figcaption>
    </figure>
  );
}

function LayerTable() {
  if (!AGR1) return null;
  return (
    <TableBox label="Layers of the TRISO particle and their registered outer radii">
      <table className="table table--compact" data-testid="basis-layers">
        <caption className="visually-hidden">Layers of the TRISO particle and their registered outer radii</caption>
        <thead>
          <tr>
            <th scope="col">Layer</th>
            <th scope="col">Material</th>
            <th scope="col" className="num-col">
              Outer radius (µm)
            </th>
          </tr>
        </thead>
        <tbody>
          {AGR1.layers.map((l, i) => (
            <tr key={l.row_id}>
              <th scope="row">
                <span className="layer-dot" style={{ ['--ring' as string]: layerColour(i) }} aria-hidden="true" />
                {l.layer}
              </th>
              <td>{l.material}</td>
              <td className="num-col num" data-value={String(l.outer_radius_um)}>
                {l.outer_radius_text}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </TableBox>
  );
}

// The fundamental assumptions of the model, as stated in Scientific Documentation (assumptions and limits).
const ASSUMPTIONS: { assumption: string; meaning: string }[] = [
  { assumption: 'Spherical symmetry', meaning: 'The centre is symmetric; only the radius matters.' },
  { assumption: 'Layer interfaces', meaning: 'Perfect contact between layers.' },
  { assumption: 'Outer surface', meaning: 'A perfect sink.' },
  { assumption: 'Initial inventory', meaning: 'Uniform in the kernel.' },
  { assumption: 'Diffusivity', meaning: 'One constant value in each layer.' },
  { assumption: 'Temperature', meaning: 'Constant.' },
];

function Assumptions() {
  return (
    <TableBox label="Fundamental assumptions">
      <table className="table table--compact" data-testid="basis-assumptions">
        <caption className="visually-hidden">Fundamental assumptions</caption>
        <thead>
          <tr>
            <th scope="col">Assumption</th>
            <th scope="col">In the model</th>
          </tr>
        </thead>
        <tbody>
          {ASSUMPTIONS.map((a) => (
            <tr key={a.assumption}>
              <th scope="row">{a.assumption}</th>
              <td>{a.meaning}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </TableBox>
  );
}

function Benchmark() {
  const entry = findCase(DEFAULT_CASE.id);
  if (!entry || !AGR1) return null;
  const synthetic = entry.config.regions;
  return (
    <TableBox label="Synthetic benchmark against the literature-based fuel configuration">
      <table className="table table--compact" data-testid="basis-benchmark">
        <caption className="visually-hidden">Synthetic benchmark against the literature-based fuel configuration</caption>
        <thead>
          <tr>
            <th scope="col">Layer</th>
            <th scope="col" className="num-col">
              Benchmark outer radius (m)
            </th>
            <th scope="col" className="num-col">
              Benchmark diffusivity (m²/s)
            </th>
            <th scope="col" className="num-col">
              AGR-1 outer radius (µm)
            </th>
          </tr>
        </thead>
        <tbody>
          {synthetic.map((r, i) => (
            <tr key={r.name}>
              <th scope="row">{r.name}</th>
              <td className="num-col num">{sciText(r.outer_radius_m)}</td>
              <td className="num-col num">{sciText(r.diffusivity_m2_s)}</td>
              <td className="num-col num">{AGR1.layers[i]?.outer_radius_text ?? ''}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </TableBox>
  );
}

export function Basis() {
  return (
    <article className="section section--doc basis" data-section="basis">
      <SectionHeading lede="Fission-product diffusion through the five layers of a TRISO fuel particle, solved two ways that check each other.">
        Basis
      </SectionHeading>

      <section className="basis__block" aria-labelledby="basis-particle-title">
        <h2 id="basis-particle-title">The TRISO particle</h2>
        <div className="basis__pair">
          <LayerDiagram />
          <LayerTable />
        </div>
      </section>

      <section className="basis__block" aria-labelledby="basis-eq-title">
        <h2 id="basis-eq-title">Fission-product diffusion</h2>
        <div className="basis__eq" data-testid="basis-equation" role="group" aria-label="Baseline governing equation">
          <Tex latex={equationTex('baseline')} display />
        </div>
        <p className="basis__note">
          Concentration <i>C</i> at radius <i>r</i> and time <i>t</i>, with a constant diffusivity <i>D</i> in each layer.
        </p>
      </section>

      <section className="basis__block" aria-labelledby="basis-assume-title">
        <h2 id="basis-assume-title">Fundamental assumptions</h2>
        <Assumptions />
      </section>

      <section className="basis__block" aria-labelledby="basis-methods-title">
        <h2 id="basis-methods-title">Two methods</h2>
        <div className="basis__methods">
          <div className="basis__method basis__method--a" data-testid="basis-method-a">
            <h3>{METHOD_1}</h3>
            <p>Central differences in space on a grid with nodes on the layer interfaces; Backward Euler in time.</p>
          </div>
          <div className="basis__method basis__method--b" data-testid="basis-method-b">
            <h3>{METHOD_2}</h3>
            <p>Eigenfunction series of the five-layer sphere, for a constant diffusivity per layer, no source and a constant temperature.</p>
          </div>
        </div>
      </section>

      <section className="basis__block" aria-labelledby="basis-bench-title">
        <h2 id="basis-bench-title">Synthetic benchmark</h2>
        <Benchmark />
        <p className="basis__note">
          The benchmark verifies the solvers; it is not a TRISO measurement. <a href={hrefDocumentation('about-model')}>Assumptions and derivations</a>
        </p>
      </section>
    </article>
  );
}
