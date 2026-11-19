<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { api, type Progress } from './lib/api';
  import { app, apply, closeTab, macroEditor, toast, updateTransfer } from './lib/state.svelte';
  import Sidebar from './lib/Sidebar.svelte';
  import Hosts from './lib/Hosts.svelte';
  import Macros from './lib/Macros.svelte';
  import MacroEditor from './lib/MacroEditor.svelte';
  import Terminal from './lib/Terminal.svelte';
  import DataDialog from './lib/DataDialog.svelte';
  import Dialogs from './lib/Dialogs.svelte';
  import Toasts from './lib/Toasts.svelte';
  import Splash from './lib/Splash.svelte';
  import Icon from './lib/Icon.svelte';

  let showData = $state(false);
  let splash = $state(true);

  onMount(() => {
    const min = new Promise((r) => setTimeout(r, 1800)); // laisse l'animation de chargement se jouer
    Promise.all([api.getState(), min])
      .then(([s]) => { apply(s); app.ready = true; })
      .catch((e) => toast(String(e), 'error'))
      .finally(() => (splash = false));
    const un = listen<Progress>('transfer', (e) => updateTransfer(e.payload));
    return () => { un.then((f) => f()); };
  });

  function onKey(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'w' && app.active) { e.preventDefault(); closeTab(app.active); }
    if (e.ctrlKey && e.key === 'Tab' && app.tabs.length) {
      e.preventDefault();
      const i = app.tabs.findIndex((t) => t.id === app.active);
      app.active = app.tabs[(i + (e.shiftKey ? -1 : 1) + app.tabs.length) % app.tabs.length].id;
    }
  }
</script>

<svelte:window onkeydown={onKey} onkeydowncapture={(e) => e.key === 'F5' && e.preventDefault()} />

{#if splash}<Splash />{/if}

{#if app.ready}
  <div class="shell">
    <Sidebar ondata={() => (showData = true)} />
    <div class="content">
      {#if app.tabs.length}
        <div class="tabs" role="tablist">
          {#each app.tabs as t (t.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_interactive_supports_focus -->
            <div class="tab" class:on={app.active === t.id} role="tab" aria-selected={app.active === t.id}
              onclick={() => (app.active = t.id)} onauxclick={(e) => e.button === 1 && closeTab(t.id)}>
              <Icon name="terminal" size={14} />
              <span class="tt">{t.title}</span>
              <button class="x" aria-label="Fermer l'onglet" onclick={(e) => { e.stopPropagation(); closeTab(t.id); }}><Icon name="x" size={12} /></button>
            </div>
          {/each}
        </div>
      {/if}
      <div class="stage">
        {#each app.tabs as t (t.id)}
          <div class="pane" class:show={app.active === t.id}><Terminal tab={t} active={app.active === t.id} /></div>
        {/each}
        {#if !app.active}
          <div class="pane show">{#if app.view === 'hosts'}<Hosts />{:else}<Macros />{/if}</div>
        {/if}
      </div>
    </div>
  </div>
{/if}

{#if showData}<DataDialog onclose={() => (showData = false)} />{/if}
{#if macroEditor.open}{#key macroEditor.macro}<MacroEditor />{/key}{/if}
<Dialogs />
<Toasts />

<style>
  .shell { height: 100%; display: flex; }
  .content { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .tabs { flex: none; display: flex; gap: 2px; padding: 6px 8px 0; background: var(--bg); border-bottom: 1px solid var(--border); overflow-x: auto; }
  .tab { display: flex; align-items: center; gap: 8px; max-width: 220px; height: 32px; padding: 0 8px 0 12px; border-radius: 8px 8px 0 0; color: var(--muted); cursor: default; border: 1px solid transparent; border-bottom: 0; }
  .tab:hover { background: var(--surface); }
  .tab.on { background: var(--surface); color: var(--text); border-color: var(--border); }
  .tt { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .x { display: grid; place-items: center; width: 18px; height: 18px; padding: 0; background: none; border: 0; border-radius: 4px; color: var(--faint); }
  .x:hover { background: var(--surface-3); color: var(--text); }
  .stage { flex: 1; min-height: 0; position: relative; }
  .pane { position: absolute; inset: 0; display: none; }
  .pane.show { display: block; }
</style>
