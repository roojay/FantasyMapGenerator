import { en } from './en';
import { zh } from './zh';
import type { Language } from '../types/map';

export type { TranslationKeys } from './en';

const dictionaries = { en, zh } as const;

export function getT(lang: Language) {
  return dictionaries[lang];
}

export { en, zh };
