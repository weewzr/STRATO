export function build(): { what: string; generator: string; nuclides: Record<string, { decay_constant_per_s: number; half_life: number; half_life_unit: string; source: string }> };
