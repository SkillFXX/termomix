<script lang="ts">
  import { api, type Macro } from './api';
  import { app, attempt, confirmDlg, editMacro, reload } from './state.svelte';
  import Icon from './Icon.svelte';

  async function remove(m: Macro) {
    if (!(await confirmDlg(`Supprimer la macro « ${m.name} » ?`))) return;
    await attempt(() => api.deleteMacro(m.id));
    await reload();
  }
  async function duplicate(m: Macro) {
    await attempt(() => api.saveMacro({ ...m, id: '', name: `${m.name} (copie)` }));
    await reload();
  }
  const scope = (m: Macro) =>
    m.server_ids.length === 0
      ? 'Tous les serveurs'
      : m.server_ids.map((id) => app.servers.find((s) => s.id === id)?.name).filter(Boolean).join(', ');
</script>

<div class="page">
  <header>
    <div>
      <h1>Macros</h1>
      <p class="muted">Séquences de commandes à lancer d'un clic dans n'importe quel terminal.</p>
    </div>
    <button class="btn primary" onclick={() => editMacro(null)}><Icon name="plus" /> Macro</button>
  </header>

  {#each app.macros as m (m.id)}
    <div class="macro">
      <div class="head">
        <div>
          <div class="name">{m.name}</div>
          <div class="sub">{m.description ?? ''}{m.description ? ' · ' : ''}{scope(m)}</div>
        </div>
        <div class="acts">
          <button class="btn ghost sm icon" title="Modifier" onclick={() => editMacro(m)}><Icon name="edit" size={14} /></button>
          <button class="btn ghost sm icon" title="Dupliquer" onclick={() => duplicate(m)}><Icon name="copy" size={14} /></button>
          <button class="btn ghost sm icon" title="Supprimer" onclick={() => remove(m)}><Icon name="trash" size={14} /></button>
        </div>
      </div>
      <pre class="mono">{m.commands.join('\n')}</pre>
    </div>
  {:else}
    <div class="welcome">
      <h2>Aucune macro</h2>
      <p class="muted">Dans un terminal, cliquez sur « Enregistrer », tapez vos commandes puis arrêtez : elles deviennent une macro réutilisable. Vous pouvez aussi en créer une à la main.</p>
    </div>
  {/each}
</div>

<style>
  .page { height: 100%; overflow: auto; padding: 28px 32px 40px; display: flex; flex-direction: column; gap: 14px; }
  header { display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: 8px; }
  h1 { font-size: 22px; font-weight: 600; letter-spacing: -0.01em; }
  .macro { background: var(--surface); border: 1px solid var(--border); border-radius: 10px; padding: 12px 14px; }
  .head { display: flex; justify-content: space-between; align-items: flex-start; }
  .name { font-weight: 600; }
  .sub { font-size: 12px; color: var(--faint); }
  .acts { display: flex; opacity: 0; }
  .macro:hover .acts { opacity: 1; }
  pre { margin: 8px 0 0; padding: 10px 12px; background: var(--bg); border-radius: 6px; font-size: 12px; color: var(--muted); overflow: auto; max-height: 160px; user-select: text; }
  .welcome { margin: 50px auto; max-width: 420px; text-align: center; display: flex; flex-direction: column; gap: 8px; }
  .welcome h2 { font-size: 16px; }
</style>
