// Test fixture: a copy of the generated research catalogue with some statuses replaced.
import catalogueJson from '../src/generated/research_catalogue.json';
import { parseCatalogue, type Catalogue } from '../src/research/catalogue';
import type { ResearchStatus } from '../src/research/status';

export function fixtureCatalogue(statuses: Record<string, ResearchStatus>): Catalogue {
  const raw = JSON.parse(JSON.stringify(catalogueJson)) as { data: { mechanisms: { mechanism_id: string; status: string }[] } };
  for (const m of raw.data.mechanisms) if (m.mechanism_id in statuses) m.status = statuses[m.mechanism_id] as string;
  return parseCatalogue(raw);
}
