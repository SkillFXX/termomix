<script lang="ts">
  import { app } from './state.svelte';
  import Icon from './Icon.svelte';
  import { t } from './i18n.svelte';

  const go = (v: 'hosts' | 'macros') => { app.view = v; app.active = null; };
</script>

<nav>
  <button class:on={!app.active && app.view === 'hosts'} onclick={() => go('hosts')}><Icon name="server" /> {t('Serveurs')}</button>
  <button class:on={!app.active && app.view === 'macros'} onclick={() => go('macros')}><Icon name="macro" /> {t('Macros')}</button>
  <div class="grow"></div>
  <button onclick={() => { app.settingsTab = 'general'; app.settingsOpen = true; }}><Icon name="settings" /> {t('Réglages')}</button>
  <div class="ver">v{app.version}</div>
</nav>

<style>
  nav { width: 208px; flex: none; display: flex; flex-direction: column; gap: 2px; padding: 12px 10px 10px; background: var(--surface); border-right: 1px solid var(--border); }
  button { position: relative; display: flex; align-items: center; gap: 10px; height: 34px; padding: 0 10px; background: none; border: 0; border-radius: 8px; color: var(--muted); font-weight: 500; text-align: left; }
  button:hover { background: var(--surface-2); color: var(--text); }
  button.on { background: var(--surface-2); color: var(--text); }
  button.on::before { content: ''; position: absolute; left: -10px; top: 8px; bottom: 8px; width: 3px; border-radius: 0 3px 3px 0; background: var(--grad); }
  .grow { flex: 1; }
  .ver { padding: 6px 10px 0; font-size: 11px; color: var(--faint); }
</style>
