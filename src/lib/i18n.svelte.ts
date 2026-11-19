// Le français est la langue source : les clés sont les textes français.
// Une clé absente du dictionnaire anglais s'affiche telle quelle.
import en from './en';

export const lang = $state({ value: 'fr' as 'fr' | 'en' });

export function t(key: string, vars?: Record<string, string | number>): string {
  let out = lang.value === 'en' ? (en[key] ?? key) : key;
  if (vars) for (const [k, v] of Object.entries(vars)) out = out.replaceAll(`{${k}}`, String(v));
  return out;
}

/** Messages d'erreur renvoyés par le backend (en français). */
export function translateError(msg: string): string {
  if (lang.value !== 'en') return msg;
  if (en[msg]) return en[msg];
  for (const [re, rep] of errorPatterns) {
    const m = msg.match(re);
    if (m) return rep.replace(/\$(\d)/g, (_, i) => m[+i] ?? '');
  }
  return msg;
}

const errorPatterns: [RegExp, string][] = [
  [/^Délai dépassé en se connectant à (.+)$/, 'Timed out connecting to $1'],
  [/^Connexion impossible : (.+)$/, 'Unable to connect: $1'],
  [/^HOST_KEY_CHANGED: l'empreinte de (.+) a changé \(attendue (.+), reçue (.+)\)\. Connexion refusée\.$/, 'The fingerprint of $1 has changed (expected $2, received $3). Connection refused.'],
  [/^Lecture de la clé impossible \((.+)\) : (.+)$/, 'Unable to read the key ($1): $2'],
];
