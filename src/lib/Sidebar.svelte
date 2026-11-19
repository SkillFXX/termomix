<script lang="ts">
  import { app } from './state.svelte';
  import Icon from './Icon.svelte';
  let { ondata }: { ondata: () => void } = $props();

  const go = (v: 'hosts' | 'macros') => { app.view = v; app.active = null; };
</script>

<nav>
  <div class="brand"><img src="/logo-mark.png" alt="" width="30" height="30" /><span>Termo<b class="grad-text">mix</b></span></div>
  <button class:on={!app.active && app.view === 'hosts'} onclick={() => go('hosts')}><Icon name="server" /> Serveurs</button>
  <button class:on={!app.active && app.view === 'macros'} onclick={() => go('macros')}><Icon name="macro" /> Macros</button>
  <div class="grow"></div>
  <button onclick={ondata}><Icon name="archive" /> Export / Import</button>
  <div class="ver">v{app.version}</div>
</nav>

<style>
  nav { width: 208px; flex: none; display: flex; flex-direction: column; gap: 2px; padding: 14px 10px 10px; background: linear-gradient(180deg, rgba(154, 116, 245, 0.06), transparent 40%), var(--surface); border-right: 1px solid var(--border); }
  .brand { display: flex; align-items: center; gap: 9px; padding: 2px 8px 16px; font-family: var(--brand); font-weight: 600; font-size: 16px; letter-spacing: -0.01em; }
  .brand b { font-weight: 700; }
  button { position: relative; display: flex; align-items: center; gap: 10px; height: 34px; padding: 0 10px; background: none; border: 0; border-radius: 8px; color: var(--muted); font-weight: 500; text-align: left; }
  button:hover { background: var(--surface-2); color: var(--text); }
  button.on { background: var(--surface-2); color: var(--text); }
  button.on::before { content: ''; position: absolute; left: -10px; top: 8px; bottom: 8px; width: 3px; border-radius: 0 3px 3px 0; background: var(--grad); box-shadow: 0 0 12px rgba(127, 140, 248, 0.7); }
  .grow { flex: 1; }
  .ver { padding: 6px 10px 0; font-size: 11px; color: var(--faint); }
</style>
