// Usage : npm run version:set -- 0.2.0
// Met à jour le numéro de version aux 5 endroits où il est écrit.
import { readFileSync, writeFileSync } from 'node:fs';

const v = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(v ?? '')) {
  console.error('Usage : npm run version:set -- X.Y.Z   (ex. 0.2.0)');
  process.exit(1);
}

const json = (file, edit) => {
  const data = JSON.parse(readFileSync(file, 'utf8'));
  edit(data);
  writeFileSync(file, JSON.stringify(data, null, 2) + '\n');
};
const text = (file, re, rep) => {
  const src = readFileSync(file, 'utf8');
  if (!re.test(src)) throw new Error(`Version introuvable dans ${file}`);
  writeFileSync(file, src.replace(re, rep));
};

json('package.json', (d) => (d.version = v));
json('package-lock.json', (d) => {
  d.version = v;
  if (d.packages?.['']) d.packages[''].version = v;
});
text('src-tauri/tauri.conf.json', /^(\s*"version": )"[^"]+"/m, `$1"${v}"`);
text('src-tauri/Cargo.toml', /^version = "[^"]+"/m, `version = "${v}"`);
text('src-tauri/Cargo.lock', /(name = "termomix"\r?\nversion = )"[^"]+"/, `$1"${v}"`);

console.log(`Version ${v} appliquée.\nEnsuite : git commit -am "Version ${v}" && git tag v${v}`);
