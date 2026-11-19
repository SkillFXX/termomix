<script lang="ts">
  import { t } from './i18n.svelte';

  let { host, phase }: { host: string; phase: 'connect' | 'auth' | 'shell' } = $props();

  const steps = [
    { id: 'connect', label: 'Connexion au serveur' },
    { id: 'auth', label: 'Authentification' },
    { id: 'shell', label: 'Ouverture du terminal' },
  ] as const;
  let current = $derived(steps.findIndex((s) => s.id === phase));
</script>

<div class="loader" role="status" aria-live="polite">
  <div class="mark">
    <i class="ring"></i>
    <img src="/logo-mark.png" alt="" width="38" height="38" />
  </div>
  <div class="host mono">{host}</div>
  <ol>
    {#each steps as s, i}
      <li class:done={i < current} class:now={i === current}>
        <span class="dot">
          {#if i < current}
            <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12l5 5 9-10" /></svg>
          {/if}
        </span>
        {t(s.label)}
      </li>
    {/each}
  </ol>
</div>

<style>
  /* Apparition retardée : une connexion quasi instantanée n'affiche pas de clignotement. */
  .loader { display: flex; flex-direction: column; align-items: center; gap: 14px; animation: appear 0.3s ease-out 0.25s both; }
  @keyframes appear { from { opacity: 0; transform: translateY(6px); } }
  .mark { position: relative; width: 76px; height: 76px; display: grid; place-items: center; }
  .ring { position: absolute; inset: 0; border-radius: 50%; background: conic-gradient(from 0deg, transparent 0 50%, #4fd3ff 78%, #9a74f5 100%); -webkit-mask: radial-gradient(farthest-side, transparent calc(100% - 3px), #000 calc(100% - 2.5px)); mask: radial-gradient(farthest-side, transparent calc(100% - 3px), #000 calc(100% - 2.5px)); animation: spin 1.1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .host { font-size: 13px; color: var(--text); }
  ol { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 7px; min-width: 190px; }
  li { display: flex; align-items: center; gap: 10px; color: var(--faint); transition: color 0.2s; }
  li.now { color: var(--text); }
  li.done { color: var(--muted); }
  .dot { width: 16px; height: 16px; border-radius: 50%; display: grid; place-items: center; border: 1.5px solid var(--border-strong); flex: none; transition: background 0.2s, border-color 0.2s; }
  li.now .dot { border-color: #7f8cf8; background: radial-gradient(circle, #7f8cf8 0 3px, transparent 3.5px); animation: pulse 1.2s ease-in-out infinite; }
  li.done .dot { border-color: transparent; background: var(--success); color: #06140d; }
  @keyframes pulse { 50% { border-color: transparent; } }
  @media (prefers-reduced-motion: reduce) { .ring, li.now .dot { animation-duration: 3s; } }
</style>
