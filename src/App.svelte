<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { api, errMsg, type Progress } from './lib/api';
  import { app, apply, closeTab, reload, confirmDlg, macroEditor, toast, updateTransfer } from './lib/state.svelte';
  import { t } from './lib/i18n.svelte';
  import Sidebar from './lib/Sidebar.svelte';
  import Hosts from './lib/Hosts.svelte';
  import Macros from './lib/Macros.svelte';
  import MacroEditor from './lib/MacroEditor.svelte';
  import Terminal from './lib/Terminal.svelte';
  import Settings from './lib/Settings.svelte';
  import Dialogs from './lib/Dialogs.svelte';
  import Toasts from './lib/Toasts.svelte';
  import Splash from './lib/Splash.svelte';
  import Icon from './lib/Icon.svelte';
  import OsIcon from './lib/OsIcon.svelte';

  let splash = $state(true);

  let animDone = $state(false);
  let failed = $state(false);
  let leaving = $state(false);

  onMount(() => {
    api.getState()
      .then((s) => { apply(s); app.ready = true; })
      .catch((e) => { toast(errMsg(e), 'error'); failed = true; });
    const un = listen<Progress>('transfer', (e) => updateTransfer(e.payload));
    // Le système d'un serveur vient d'être détecté : on rafraîchit les cartes et les onglets.
    const un2 = listen('servers-changed', () => reload());
    return () => { un.then((f) => f()); un2.then((f) => f()); };
  });

  // L'écran de chargement part dès que les données sont prêtes ET que l'animation est terminée.
  $effect(() => {
    if (!leaving && (failed || (app.ready && (animDone || !app.settings.splash)))) {
      leaving = true;
      // Retrait du DOM une fois le fondu terminé (il est déjà invisible à ce moment).
      setTimeout(() => (splash = false), 900);
    }
  });

  async function requestClose(id: string) {
    const tab = app.tabs.find((x) => x.id === id);
    if (tab?.connected && app.settings.confirm_close) {
      if (!(await confirmDlg(t('Fermer « {name} » ?', { name: tab.title }), t('La session SSH sera déconnectée.'), false))) return;
    }
    closeTab(id);
  }

  function onKey(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'w' && app.active) { e.preventDefault(); requestClose(app.active); }
    if (e.ctrlKey && e.key === 'Tab' && app.tabs.length) {
      e.preventDefault();
      const i = app.tabs.findIndex((t) => t.id === app.active);
      app.active = app.tabs[(i + (e.shiftKey ? -1 : 1) + app.tabs.length) % app.tabs.length].id;
    }
  }
</script>

<svelte:window onkeydown={onKey} onkeydowncapture={(e) => e.key === 'F5' && e.preventDefault()} />

{#if splash}<Splash {leaving} onfinished={() => (animDone = true)} />{/if}

{#if app.ready}
  <div class="shell" class:enter={leaving || !splash}>
    <Sidebar />
    <div class="content">
      {#if app.tabs.length}
        <div class="tabs" role="tablist">
          {#each app.tabs as tb (tb.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_interactive_supports_focus -->
            <div class="tab" class:on={app.active === tb.id} role="tab" aria-selected={app.active === tb.id}
              onclick={() => (app.active = tb.id)} onauxclick={(e) => e.button === 1 && requestClose(tb.id)}>
              {#if app.servers.find((x) => x.id === tb.server.id)?.os}<OsIcon os={app.servers.find((x) => x.id === tb.server.id)?.os} size={14} />{:else}<Icon name="terminal" size={14} />{/if}
              <span class="tt">{tb.title}</span>
              <button class="x" aria-label={t("Fermer l'onglet")} onclick={(e) => { e.stopPropagation(); requestClose(tb.id); }}><Icon name="x" size={12} /></button>
            </div>
          {/each}
        </div>
      {/if}
      <div class="stage">
        {#each app.tabs as tb (tb.id)}
          <div class="pane" class:show={app.active === tb.id}><Terminal tab={tb} active={app.active === tb.id} /></div>
        {/each}
        {#if !app.active}
          <div class="pane show">{#if app.view === 'hosts'}<Hosts />{:else}<Macros />{/if}</div>
        {/if}
      </div>
    </div>
  </div>
{/if}

{#if app.settingsOpen}<Settings />{/if}
{#if macroEditor.open}{#key macroEditor.macro}<MacroEditor />{/key}{/if}
<Dialogs />
<Toasts />

<style>
  .shell { height: 100%; display: flex; opacity: 0; transform: translateY(18px); }
  .shell.enter { opacity: 1; transform: none; transition: opacity 0.6s cubic-bezier(0.22, 0.8, 0.2, 1), transform 0.6s cubic-bezier(0.22, 0.8, 0.2, 1); }
  .content { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .tabs { flex: none; display: flex; gap: 2px; padding: 6px 8px 0; background: var(--bg); border-bottom: 1px solid var(--border); overflow-x: auto; }
  .tab { display: flex; align-items: center; gap: 8px; max-width: 220px; height: 32px; padding: 0 8px 0 12px; border-radius: 8px 8px 0 0; color: var(--muted); cursor: default; border: 1px solid transparent; border-bottom: 0; }
  .tab:hover { background: var(--surface); }
  .tab.on {
    color: var(--text);
    border-color: var(--border);
    border-top-color: transparent;
    /* Le filet dégradé est un fond : il est donc découpé par les coins arrondis de l'onglet. */
    background-image: var(--grad), linear-gradient(var(--surface), var(--surface));
    background-size: 100% 2px, 100% 100%;
    background-repeat: no-repeat;
    background-origin: border-box;
    background-clip: border-box;
  }
  .tt { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .x { display: grid; place-items: center; width: 18px; height: 18px; padding: 0; background: none; border: 0; border-radius: 4px; color: var(--faint); }
  .x:hover { background: var(--surface-3); color: var(--text); }
  .stage { flex: 1; min-height: 0; position: relative; }
  .pane { position: absolute; inset: 0; display: none; }
  .pane.show { display: block; }
</style>
