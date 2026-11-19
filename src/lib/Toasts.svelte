<script lang="ts">
  import { app, transfers } from './state.svelte';
  const pct = (d: number, t: number) => (t ? Math.min(100, Math.round((d / t) * 100)) : 100);
</script>

<div class="stack">
  {#each transfers.list as t (t.id)}
    <div class="toast tr {t.state}">
      <div class="top"><span class="n">{t.state === 'error' ? 'Échec : ' : ''}{t.name}</span><span class="p">{t.state === 'running' ? pct(t.done, t.total) + ' %' : t.state === 'done' ? 'Terminé' : ''}</span></div>
      {#if t.state === 'error'}<div class="e">{t.error}</div>{:else}<div class="bar"><i style:width="{pct(t.done, t.total)}%"></i></div>{/if}
    </div>
  {/each}
  {#each app.toasts as t (t.id)}
    <div class="toast {t.kind}">{t.text}</div>
  {/each}
</div>

<style>
  .stack { position: fixed; right: 16px; bottom: 16px; z-index: 80; display: flex; flex-direction: column; gap: 8px; width: 320px; pointer-events: none; }
  .toast { padding: 10px 12px; background: var(--surface-2); border: 1px solid var(--border-strong); border-radius: 10px; box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4); }
  .toast.error { border-color: var(--danger); }
  .toast.ok { border-color: var(--success); }
  .tr.error { border-color: var(--danger); }
  .top { display: flex; justify-content: space-between; gap: 10px; }
  .n { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .p { color: var(--muted); font-size: 12px; }
  .bar { height: 3px; margin-top: 8px; background: var(--surface-3); border-radius: 3px; overflow: hidden; }
  .bar i { display: block; height: 100%; background: var(--accent); transition: width 0.15s; }
  .e { margin-top: 4px; font-size: 12px; color: var(--danger); }
</style>
