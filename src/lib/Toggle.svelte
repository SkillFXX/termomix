<script lang="ts">
  let { checked, onchange, label }: { checked: boolean; onchange: (v: boolean) => void; label: string } = $props();
</script>

<button class="sw" class:on={checked} role="switch" aria-checked={checked} aria-label={label} onclick={() => onchange(!checked)}><i></i></button>

<style>
  .sw { flex: none; position: relative; width: 36px; height: 20px; padding: 0; border-radius: 20px; border: 1px solid var(--border-strong); background: var(--surface-3); transition: background 0.15s, border-color 0.15s; }
  .sw i { position: absolute; top: 2px; left: 2px; width: 14px; height: 14px; border-radius: 50%; background: var(--muted); transition: transform 0.15s, background 0.15s; }
  .sw.on { background: var(--grad); border-color: transparent; }
  .sw.on i { transform: translateX(16px); background: #fff; }
</style>
