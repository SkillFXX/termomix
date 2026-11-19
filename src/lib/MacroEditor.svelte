<script lang="ts">
  import { api, type Macro } from './api';
  import { app, attempt, macroEditor, reload } from './state.svelte';
  import Modal from './Modal.svelte';

  const init = macroEditor.macro ?? {};
  let name = $state(init.name ?? '');
  let description = $state(init.description ?? '');
  let text = $state((init.commands ?? []).join('\n'));
  let delay = $state(init.delay_ms ?? 150);
  let scoped = $state((init.server_ids?.length ?? 0) > 0);
  let ids = $state<string[]>([...(init.server_ids ?? [])]);

  const close = () => (macroEditor.open = false);

  async function save() {
    const m: Macro = {
      id: init.id ?? '',
      name,
      description: description || null,
      commands: text.split('\n'),
      delay_ms: Math.max(0, Number(delay) || 0),
      server_ids: scoped ? ids : [],
    };
    if (await attempt(() => api.saveMacro(m))) {
      await reload();
      close();
    }
  }
</script>

<Modal title={init.id ? 'Modifier la macro' : 'Nouvelle macro'} onclose={close} width={560}>
  <div class="field">
    <label for="m-name">Nom</label>
    <!-- svelte-ignore a11y_autofocus -->
    <input id="m-name" class="input" bind:value={name} placeholder="Mise à jour du système" autofocus />
  </div>
  <div class="field">
    <label for="m-cmd">Commandes (une par ligne)</label>
    <textarea id="m-cmd" class="input" rows="8" bind:value={text} spellcheck="false" placeholder={'sudo apt update\nsudo apt upgrade -y'}></textarea>
  </div>
  <div class="row">
    <div class="field">
      <label for="m-delay">Délai entre commandes (ms)</label>
      <input id="m-delay" class="input mono" type="number" min="0" step="50" bind:value={delay} />
    </div>
    <div class="field">
      <label for="m-desc">Description</label>
      <input id="m-desc" class="input" bind:value={description} />
    </div>
  </div>
  <div class="field">
    <label for="m-scope">Disponible sur</label>
    <select id="m-scope" class="input" onchange={(e) => (scoped = e.currentTarget.value === 'some')} value={scoped ? 'some' : 'all'}>
      <option value="all">Tous les serveurs</option>
      <option value="some">Certains serveurs</option>
    </select>
    {#if scoped}
      <div class="servers">
        {#each app.servers as s (s.id)}
          <label class="check"><input type="checkbox" value={s.id} bind:group={ids} /> {s.name}</label>
        {/each}
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <button class="btn" onclick={close}>Annuler</button>
    <button class="btn primary" onclick={save}>Enregistrer</button>
  {/snippet}
</Modal>

<style>
  .servers { display: grid; grid-template-columns: 1fr 1fr; gap: 6px 12px; max-height: 140px; overflow: auto; padding: 8px 10px; border: 1px solid var(--border); border-radius: var(--radius); }
  .check { display: flex; align-items: center; gap: 6px; font-size: 12.5px; color: var(--text); font-weight: 400; }
</style>
