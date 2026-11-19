<script lang="ts">
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { api, errMsg } from './api';
  import { app, reload, toast } from './state.svelte';
  import Modal from './Modal.svelte';
  import Icon from './Icon.svelte';

  let { onclose }: { onclose: () => void } = $props();
  let tab = $state<'export' | 'import'>('export');
  let secrets = $state(true);
  let pass = $state('');
  let pass2 = $state('');
  let replace = $state(false);
  let busy = $state(false);
  let importPath = $state<string | null>(null);
  let needPass = $state(false);
  let error = $state('');

  async function doExport() {
    error = '';
    if (secrets && pass.length < 8) return (error = 'La phrase secrète doit faire au moins 8 caractères.');
    if (pass !== pass2 && (secrets || pass)) return (error = 'Les deux phrases secrètes sont différentes.');
    const stamp = new Date().toISOString().slice(0, 10);
    const path = await save({ title: 'Exporter mes données', defaultPath: `termomix-${stamp}.termomix`, filters: [{ name: 'Export Termomix', extensions: ['termomix'] }] });
    if (!path) return;
    busy = true;
    try {
      await api.exportData(path, pass || null, secrets);
      toast('Export enregistré', 'ok');
      onclose();
    } catch (e) {
      error = errMsg(e);
    } finally {
      busy = false;
    }
  }

  async function pick() {
    const p = await open({ title: 'Importer un export', multiple: false, filters: [{ name: 'Export Termomix', extensions: ['termomix'] }] });
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
      toast(`${r.servers} serveur(s) et ${r.macros} macro(s) importés`, 'ok');
      onclose();
    } catch (e) {
      const m = errMsg(e);
      if (m === 'PASSPHRASE_REQUIRED') { needPass = true; error = 'Ce fichier est chiffré : saisissez sa phrase secrète.'; }
      else error = m;
    } finally {
      busy = false;
    }
  }
</script>

<Modal title="Données" {onclose} width={500}>
  <div class="tabs">
    <button class:on={tab === 'export'} onclick={() => { tab = 'export'; error = ''; pass = pass2 = ''; }}>Exporter</button>
    <button class:on={tab === 'import'} onclick={() => { tab = 'import'; error = ''; pass = pass2 = ''; }}>Importer</button>
  </div>

  {#if tab === 'export'}
    <p class="muted">Crée un fichier unique à copier sur une clé USB ou un autre ordinateur. Rien ne passe par un serveur distant.</p>
    <label class="check"><input type="checkbox" bind:checked={secrets} /> Inclure les mots de passe et phrases secrètes</label>
    <div class="field">
      <label for="e-p1">Phrase secrète {secrets ? '' : '(optionnelle)'}</label>
      <input id="e-p1" class="input" type="password" bind:value={pass} autocomplete="new-password" />
    </div>
    {#if pass || secrets}
      <div class="field">
        <label for="e-p2">Confirmer la phrase secrète</label>
        <input id="e-p2" class="input" type="password" bind:value={pass2} autocomplete="new-password" />
      </div>
    {/if}
    <p class="hint">Le fichier est chiffré (Argon2id + ChaCha20-Poly1305) dès qu'une phrase secrète est définie. Sans elle, il est impossible de le relire : conservez-la. Les clés privées ne sont pas copiées, seulement leur chemin.</p>
  {:else}
    <p class="muted">Importez un fichier .termomix créé sur un autre ordinateur.</p>
    <div class="row">
      <input class="input mono" readonly value={importPath ?? ''} placeholder="Aucun fichier sélectionné" />
      <button class="btn" style="flex: 0 0 auto" onclick={pick}><Icon name="folder" /> Choisir</button>
    </div>
    {#if needPass}
      <div class="field">
        <label for="i-p">Phrase secrète</label>
        <input id="i-p" class="input" type="password" bind:value={pass} onkeydown={(e) => e.key === 'Enter' && doImport()} />
      </div>
    {/if}
    <label class="check"><input type="checkbox" bind:checked={replace} /> Remplacer toutes mes données actuelles</label>
    <p class="hint">{replace ? `Attention : les ${app.servers.length} serveurs et ${app.macros.length} macros actuels seront remplacés.` : 'Par défaut, les éléments importés sont ajoutés ou mis à jour, rien n\'est supprimé.'}</p>
  {/if}

  {#if error}<p class="error">{error}</p>{/if}

  {#snippet footer()}
    <button class="btn" onclick={onclose}>Fermer</button>
    {#if tab === 'export'}
      <button class="btn primary" disabled={busy} onclick={doExport}>Exporter…</button>
    {:else}
      <button class="btn primary" disabled={busy || !importPath} onclick={doImport}>Importer</button>
    {/if}
  {/snippet}
</Modal>

<style>
  .tabs { display: flex; gap: 4px; padding: 3px; background: var(--bg); border-radius: 8px; }
  .tabs button { flex: 1; height: 28px; border: 0; background: none; border-radius: 6px; color: var(--muted); font-weight: 500; }
  .tabs button.on { background: var(--surface-3); color: var(--text); }
  .check { display: flex; align-items: center; gap: 8px; }
  .error { color: var(--danger); }
</style>
