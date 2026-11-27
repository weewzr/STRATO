// The variable legend under the governing equation (final directive, Section 9): symbol, meaning and SI
// unit of ONLY the variables of the displayed equation. Display text: nothing here is evaluated.
import { mechanismsOf, type PhysicsId, type ScenarioId } from './capability';
import { D_CONSTANT, D_SCENARIO, D_TEMPERATURE, G_SCENARIO, isTimed } from './physics';

export interface LegendRow {
  id: 'C' | 't' | 'r' | 'D' | 'G' | 'lambda' | 'T';
  /** TeX of the symbol as the equation shows it. */
  symbol: string;
  meaning: string;
  /** TeX of the SI unit. */
  unit: string;
}

const U = {
  conc: String.raw`\mathrm{mol/m^{3}}`,
  time: String.raw`\mathrm{s}`,
  radius: String.raw`\mathrm{m}`,
  diff: String.raw`\mathrm{m^{2}/s}`,
  gen: String.raw`\mathrm{mol/(m^{3}\,s)}`,
  decay: String.raw`\mathrm{s^{-1}}`,
  temp: String.raw`\mathrm{K}`,
};

/**
 * The rows for a model equation and an operation. 'future' shows the combined equation, so its rows are
 * those of the combined model. An operation that prescribes a history changes the symbols G and D(r,T) of the
 * rows that exist; it never adds a row for a term the equation does not show.
 */
export function legendRows(physics: PhysicsId, scenario: ScenarioId | null): LegendRow[] {
  const mech = mechanismsOf(physics);
  const timed = isTimed(scenario);
  const rows: LegendRow[] = [
    { id: 'C', symbol: 'C', meaning: 'Concentration', unit: U.conc },
    { id: 't', symbol: 't', meaning: 'Time', unit: U.time },
    { id: 'r', symbol: 'r', meaning: 'Radius', unit: U.radius },
    { id: 'D', symbol: !mech.includes('temperature') ? D_CONSTANT : timed ? D_SCENARIO : D_TEMPERATURE, meaning: 'Diffusivity', unit: U.diff },
  ];
  if (mech.includes('generation')) rows.push({ id: 'G', symbol: timed ? G_SCENARIO : 'G', meaning: 'Generation rate', unit: U.gen });
  if (mech.includes('decay')) rows.push({ id: 'lambda', symbol: String.raw`\lambda`, meaning: 'Radioactive decay constant', unit: U.decay });
  if (mech.includes('temperature')) rows.push({ id: 'T', symbol: timed ? 'T(t)' : 'T', meaning: 'Temperature', unit: U.temp });
  return rows;
}
