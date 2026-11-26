export const CAPTION_MAX_CHARS: number;
export const PROSE_MAX_WORDS: number;
export const PROSE_MAX_WORDS_BY_PAGE: Record<string, number>;
export const DOCUMENTATION_PAGE: string;
export const RULES: string[];
export const RULE_MESSAGES: Record<string, string>;
export const HASH_PATTERN: RegExp;
export const PATH_PATTERN: RegExp;
export const DISCLAIMERS: { id: string; pattern: RegExp }[];
export const TEMPORARY_EXCEPTIONS: Record<string, string[]>;

export interface PageSnapshot {
  id: string;
  page: string;
  text: string;
  shellText?: string;
  captions: string[];
  proseWords: number;
}

export interface RuleResult {
  rule: string;
  name: string;
  failures: string[];
  excepted: string[];
}

export function evaluateSnapshots(snapshots: PageSnapshot[], exceptions?: Record<string, string[]>): { results: RuleResult[]; notes: string[] };
