// Research-status system (prompt/TESTING_AND_VALIDATION.md Section 3.1) and "no active toggle without
// solver support" (Section 3.2): chips, isAvailable(), the solved stage, the request filter, and the
// statuses of the builder mechanisms against the generated catalogue and prompt/PROGRESS.md (M7).
import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import catalogueJson from '../src/generated/research_catalogue.json';
import { BTCS_CAPABILITY, filterRequestedMechanisms, isAvailable, simulationMethod, solvedStage } from '../src/equation/availability';
import { CATALOGUE, statusOf } from '../src/research/catalogue';
import { StatusChip } from '../src/research/StatusChip';
import { STATUS_INFO, STATUSES, type ResearchStatus } from '../src/research/status';
import { fixtureCatalogue } from './catalogueFixture';

// Vitest runs from web/; the repository root is its parent.
const REPO = join(process.cwd(), '..');

describe('status chips', () => {
  const shapes: Record<ResearchStatus, string> = {
    PENDING: 'dotted',
    RESEARCHED: 'dashed',
    IMPLEMENTED: 'outline',
    VERIFIED: 'fill',
    VALIDATED: 'fill-check',
  };

  it('defines exactly the five statuses, each with its own shape', () => {
    expect([...STATUSES]).toEqual(['PENDING', 'RESEARCHED', 'IMPLEMENTED', 'VERIFIED', 'VALIDATED']);
    expect(new Set(STATUSES.map((s) => STATUS_INFO[s].shape)).size).toBe(5);
  });

  for (const s of STATUSES) {
    it(`${s}: word, shape class and accessible name`, () => {
      render(<StatusChip status={s} mechanism="fixture" />);
      const chip = document.querySelector<HTMLElement>('[data-status-for="fixture"]');
      if (!chip) throw new Error('no chip');
      expect(chip.dataset.status).toBe(s);
      expect(chip.className).toContain(`status-chip--${shapes[s]}`);
      expect(chip.querySelector('.status-chip__word')?.textContent).toBe(s);
      expect(chip.textContent).toBe(`Research status: ${s}`);
      expect(chip.querySelector('svg')?.getAttribute('aria-hidden')).toBe('true');
      // Only VALIDATED carries the check mark.
      expect(chip.querySelector('svg path') !== null).toBe(s === 'VALIDATED');
      expect(chip.getAttribute('title')).toBe(STATUS_INFO[s].definition);
    });
  }

  it('is distinct from the result labels (no badge class)', () => {
    render(<StatusChip status="VERIFIED" mechanism="fixture" />);
    expect(screen.getByText('VERIFIED').closest('.badge')).toBeNull();
  });
});

describe('isAvailable(): live only for VERIFIED with the capability present', () => {
  for (const s of STATUSES) {
    for (const present of [true, false]) {
      it(`status ${s}, capability ${present ? 'present' : 'absent'}`, () => {
        const cat = fixtureCatalogue({ generation: s });
        const caps = present ? ['generation'] : [];
        expect(isAvailable('generation', caps, cat)).toBe(s === 'VERIFIED' && present);
      });
    }
  }

  it('is false for an id that the catalogue does not hold, even if reported', () => {
    expect(isAvailable('not_a_mechanism', ['not_a_mechanism'])).toBe(false);
  });

  it('with the generated catalogue and the v1.0 capabilities only diffusion_layered is live', () => {
    const caps = ['diffusion_layered'];
    expect(CATALOGUE.mechanisms.filter((m) => isAvailable(m.mechanism_id, caps)).map((m) => m.mechanism_id)).toEqual(['diffusion_layered']);
    expect(CATALOGUE.mechanisms.filter((m) => isAvailable(m.mechanism_id, [])).length).toBe(0);
  });
});

describe('solved stage and the solver request filter', () => {
  it('is 0 before the wasm module reports capabilities and for the previous solver, and 12 with the BTCS capability', () => {
    expect(solvedStage([])).toBe(0);
    expect(solvedStage(['diffusion_layered'])).toBe(0);
    expect(solvedStage([BTCS_CAPABILITY])).toBe(12);
    // Reporting a capability without the live status changes nothing.
    expect(solvedStage([BTCS_CAPABILITY, 'generation', 'decay'])).toBe(12);
  });

  it('names the solver that runs the simulation', () => {
    expect(simulationMethod([])).toBe('loading');
    expect(simulationMethod(['diffusion_layered'])).toBe('previous');
    expect(simulationMethod([BTCS_CAPABILITY])).toBe('btcs');
    // The BTCS capability alone is not enough when the catalogue row is not at the live status.
    expect(simulationMethod([BTCS_CAPABILITY], fixtureCatalogue({ diffusion_layered: 'IMPLEMENTED' }))).toBe('previous');
  });

  it('advances through the previews only in order', () => {
    const cat = fixtureCatalogue({ generation: 'VERIFIED', decay: 'VERIFIED' });
    expect(solvedStage([BTCS_CAPABILITY, 'generation', 'decay'], cat)).toBe(16);
    expect(solvedStage([BTCS_CAPABILITY, 'decay'], cat)).toBe(12);
  });

  it('drops every mechanism for which isAvailable() is false and reports it', () => {
    const r = filterRequestedMechanisms(['diffusion_layered', 'generation', 'decay', 'diffusivity_temperature'], ['diffusion_layered']);
    expect(r.send).toEqual(['diffusion_layered']);
    expect(r.dropped).toEqual(['generation', 'decay', 'diffusivity_temperature']);
    const none = filterRequestedMechanisms(['generation'], ['generation'], fixtureCatalogue({ generation: 'IMPLEMENTED' }));
    expect(none.send).toEqual([]);
  });
});

describe('builder mechanism statuses on 2026-11-24', () => {
  // The status table of owner decision M7 in prompt/PROGRESS.md, by mechanism name there.
  const NAME_TO_ID: Record<string, string> = {
    'Five-layer diffusion, constant diffusivity per layer': 'diffusion_layered',
    'Fission-product generation': 'generation',
    'Radioactive decay': 'decay',
    'Inventory accounting with decay and source': 'inventory_accounting',
    'Temperature-dependent diffusivity, layer-specific laws': 'diffusivity_temperature',
    'Interface partition coefficients': 'interface_partition',
  };

  it('equal the table recorded in prompt/PROGRESS.md (M7)', () => {
    const progress = readFileSync(join(REPO, 'prompt', 'PROGRESS.md'), 'utf8');
    const table = progress.slice(progress.indexOf('| Mechanism | Status | Domain | Basis |'));
    const rows = new Map<string, string>();
    for (const line of table.split('\n').slice(2)) {
      if (!line.startsWith('|')) break;
      const cells = line.split('|').map((c) => c.trim());
      rows.set(cells[1] ?? '', cells[2] ?? '');
    }
    for (const [name, id] of Object.entries(NAME_TO_ID)) {
      expect(rows.get(name), name).toBeDefined();
      expect(statusOf(CATALOGUE, id), `${id} against "${name}"`).toBe(rows.get(name));
    }
    // The layer-specific laws share the row of temperature dependence in that table.
    expect(statusOf(CATALOGUE, 'diffusivity_layer_laws')).toBe(rows.get('Temperature-dependent diffusivity, layer-specific laws'));
    // Nothing is VALIDATED.
    expect(CATALOGUE.mechanisms.some((m) => m.status === 'VALIDATED')).toBe(false);
  });

  it('the generated catalogue holds the builder rows and carries its provenance header', () => {
    expect(catalogueJson.label).toBe('CATALOGUE');
    expect(catalogueJson.kind).toBe('research_catalogue');
    expect(catalogueJson.generator).toMatch(/^scripts\/web\/generate_data\.py v\d+$/);
    for (const i of catalogueJson.inputs) expect(i.sha256).toMatch(/^[0-9a-f]{64}$/);
    const ids = CATALOGUE.mechanisms.map((m) => m.mechanism_id);
    for (const id of ['diffusion_layered', 'generation', 'decay', 'inventory_accounting', 'diffusivity_temperature', 'diffusivity_layer_laws', 'interface_partition']) {
      expect(ids).toContain(id);
    }
  });
});
