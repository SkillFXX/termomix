<script lang="ts">
  import { dlg } from './state.svelte';
  import Modal from './Modal.svelte';

  let value = $state('');
  $effect(() => { if (dlg.current?.kind === 'prompt') value = dlg.current.value ?? ''; });

  function done(v: any) {
    const c = dlg.current;
    dlg.current = null;
    c?.resolve(v);
  }
</script>

{#if dlg.current}
  {@const c = dlg.current}
  <Modal title={c.title} width={420} onclose={() => done(c.kind === 'confirm' ? false : null)}>
    {#if c.message}<p class="msg">{c.message}</p>{/if}
    {#if c.kind === 'prompt'}
      <!-- svelte-ignore a11y_autofocus -->
      <input class="input" bind:value autofocus onkeydown={(e) => e.key === 'Enter' && value.trim() && done(value.trim())} />
    {/if}
    {#snippet footer()}
      <button class="btn" onclick={() => done(c.kind === 'confirm' ? false : null)}>Annuler</button>
      {#if c.kind === 'confirm'}
        <button class="btn" class:primary={!c.danger} class:danger={c.danger} onclick={() => done(true)}>Confirmer</button>
      {:else}
        <button class="btn primary" disabled={!value.trim()} onclick={() => done(value.trim())}>Valider</button>
      {/if}
    {/snippet}
  </Modal>
{/if}

<style>
  .msg { color: var(--muted); line-height: 1.5; }
</style>
