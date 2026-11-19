<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';
  let { title, onclose, width = 520, children, footer }: { title: string; onclose: () => void; width?: number; children: Snippet; footer?: Snippet } = $props();
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="overlay" onmousedown={(e) => e.target === e.currentTarget && onclose()}>
  <div class="modal" style:width="{width}px" role="dialog" aria-label={title}>
    <header>
      <h2>{title}</h2>
      <button class="btn ghost icon sm" onclick={onclose} aria-label="Fermer"><Icon name="x" /></button>
    </header>
    <div class="body">{@render children()}</div>
    {#if footer}<footer>{@render footer()}</footer>{/if}
  </div>
</div>

<style>
  .overlay { position: fixed; inset: 0; background: rgba(5, 6, 9, 0.62); display: flex; align-items: center; justify-content: center; z-index: 50; }
  .modal { max-width: calc(100vw - 48px); max-height: calc(100vh - 48px); display: flex; flex-direction: column; background: var(--surface); border: 1px solid var(--border-strong); border-radius: 12px; box-shadow: 0 24px 60px rgba(0, 0, 0, 0.55), 0 0 40px rgba(110, 120, 250, 0.08); }
  header { display: flex; align-items: center; justify-content: space-between; padding: 14px 16px 12px 20px; border-bottom: 1px solid var(--border); }
  h2 { font-size: 14px; font-weight: 600; font-family: var(--brand); }
  .body { padding: 18px 20px; overflow: auto; display: flex; flex-direction: column; gap: 14px; }
  footer { padding: 12px 20px; border-top: 1px solid var(--border); display: flex; justify-content: flex-end; gap: 8px; }
</style>
