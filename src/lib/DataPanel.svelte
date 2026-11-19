<script lang="ts">
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { api, errMsg } from './api';
  import { app, reload, toast } from './state.svelte';
  import { t } from './i18n.svelte';
  import Icon from './Icon.svelte';

  let mode = $state<'export' | 'import'>('export');
  let secrets = $state(true);
  let pass = $state('');
  let pass2 = $state('');
  let replace = $state(false);
  let busy = $state(false);
  let importPath = $state<string | null>(null);
  let needPass = $state(false);
  let error = $state('');

  function switchMode(m: 'export' | 'import') {
    mode = m;
    error = '';
    pass = pass2 = '';
    needPass = false;
  }

  async function doExport() {
    error = '';
    if (secrets && pass.length < 8) return (error = t('La phrase secrète doit faire au moins 8 caractères.'));
    if (pass !== pass2 && (secrets || pass)) return (error = t('Les deux phrases secrètes sont différentes.'));
    const stamp = new Date().toISOString().slice(0, 10);
    const path = await save({ title: t('Exporter mes données'), defaultPath: `termomix-${stamp}.termomix`, filters: [{ name: t('Export Termomix'), extensions: ['termomix'] }] });
    if (!path) return;
    busy = true;
    try {
      await api.exportData(path, pass || null, secrets);
      toast(t('Export enregistré'), 'ok');
      pass = pass2 = '';
    } catch (e) {
      error = errMsg(e);
    } finally {
      busy = false;
    }
  }

  async function pick() {
    const p = await open({ title: t('Importer un export'), multiple: false, filters: [{ name: t('Export Termomix'), extensions: ['termomix'] }] });
    if (typeof p === 'string') {
      importPath = p;
      needPass = false;
      error = '';
    }
  }

  async function doImport() {
    if (!importPath) return;
    error = '';
    busy = true;
    try {
      const r = await api.importData(importPath, pass || null, replace);
      await reload();
      toast(t('{s} serveur(s) et {m} macro(s) importés', { s: r.servers, m: r.macros }), 'ok');
      importPath = null;
      pass = '';
    } catch (e) {
      const m = String((e as any)?.message ?? e);
      if (m === 'PASSPHRASE_REQUIRED') {
        needPass = true;
        error = t('Ce fichier est chiffré : saisissez sa phrase secrète.');
      } else error = errMsg(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="panel">
  <div class="seg">
    <button class:on={mode === 'export'} onclick={() => switchMode('export')}>{t('Exporter')}</button>
    <button class:on={mode === 'import'} onclick={() => switchMode('import')}>{t('Importer')}</button>
  </div>

  {#if mode === 'export'}
    <p class="muted">{t('Crée un fichier unique à copier sur une clé USB ou un autre ordinateur. Rien ne passe par un serveur distant.')}</p>
    <label class="check"><input type="checkbox" bind:checked={secrets} /> {t('Inclure les mots de passe et phrases secrètes')}</label>
    <div class="field">
      <label for="e-p1">{secrets ? t('Phrase secrète') : t('Phrase secrète (optionnelle)')}</label>
      <input id="e-p1" class="input" type="password" bind:value={pass} autocomplete="new-password" />
    </div>
    {#if pass || secrets}
      <div class="field">
        <label for="e-p2">{t('Confirmer la phrase secrète')}</label>
        <input id="e-p2" class="input" type="password" bind:value={pass2} autocomplete="new-password" />
      </div>
    {/if}
    <p class="hint">{t('Le fichier est chiffré (Argon2id + ChaCha20-Poly1305) dès qu\'une phrase secrète est définie. Sans elle, il est impossible de le relire : conservez-la. Les clés privées ne sont pas copiées, seulement leur chemin.')}</p>
    <div class="actions"><button class="btn primary" disabled={busy} onclick={doExport}>{t('Exporter…')}</button></div>
  {:else}
    <p class="muted">{t('Importez un fichier .termomix créé sur un autre ordinateur.')}</p>
    <div class="row">
      <input class="input mono" readonly value={importPath ?? ''} placeholder={t('Aucun fichier sélectionné')} />
      <button class="btn" style="flex: 0 0 auto" onclick={pick}><Icon name="folder" /> {t('Choisir')}</button>
    </div>
    {#if needPass}
      <div class="field">
        <label for="i-p">{t('Phrase secrète')}</label>
        <input id="i-p" class="input" type="password" bind:value={pass} onkeydown={(e) => e.key === 'Enter' && doImport()} />
      </div>
    {/if}
    <label class="check"><input type="checkbox" bind:checked={replace} /> {t('Remplacer toutes mes données actuelles')}</label>
    <p class="hint">
      {replace
        ? t('Attention : les {s} serveurs et {m} macros actuels seront remplacés.', { s: app.servers.length, m: app.macros.length })
        : t('Par défaut, les éléments importés sont ajoutés ou mis à jour, rien n\'est supprimé.')}
    </p>
    <div class="actions"><button class="btn primary" disabled={busy || !importPath} onclick={doImport}>{t('Importer')}</button></div>
  {/if}
  {#if error}<p class="error">{error}</p>{/if}
</div>

<style>
  .panel { display: flex; flex-direction: column; gap: 14px; }
  .seg { display: flex; gap: 4px; padding: 3px; background: var(--bg); border-radius: 8px; }
  .seg button { flex: 1; height: 28px; border: 0; background: none; border-radius: 6px; color: var(--muted); font-weight: 500; }
  .seg button.on { background: var(--surface-3); color: var(--text); }
  .check { display: flex; align-items: center; gap: 8px; }
  .actions { display: flex; justify-content: flex-end; }
  .error { color: var(--danger); }
</style>
