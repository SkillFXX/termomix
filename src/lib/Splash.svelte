<script lang="ts">
  import { onMount } from 'svelte';
  import { cubicInOut } from 'svelte/easing';
  import { fade } from 'svelte/transition';

  let { onfinished }: { onfinished: () => void } = $props();

  // L'animation (30 images x 100 ms) se joue une seule fois puis reste sur sa dernière image :
  // le fondu de sortie démarre donc exactement à la fin, sans reprise de boucle.
  const DURATION = 3000;
  let done = false;
  const finish = () => {
    if (!done) {
      done = true;
      onfinished();
    }
  };

  onMount(() => {
    const safety = setTimeout(finish, DURATION + 2500); // si l'image ne charge pas
    return () => clearTimeout(safety);
  });

  // Le logo glisse légèrement vers le haut en s'estompant.
  function lift(_node: Element) {
    return { duration: 600, easing: cubicInOut, css: (t: number, u: number) => `opacity: ${t}; transform: translateY(${-u * 22}px) scale(${1 + u * 0.04})` };
  }
</script>

<div class="splash" role="status" aria-label="Chargement" out:fade={{ duration: 600 }}>
  <img
    src="/loader.webp"
    alt="Termomix"
    width="393"
    height="283"
    out:lift
    onload={() => setTimeout(finish, DURATION)}
    onerror={finish}
  />
</div>

<style>
  .splash { position: fixed; inset: 0; z-index: 100; display: grid; place-items: center; background: var(--bg); }
  img { max-width: 60vw; height: auto; user-select: none; -webkit-user-drag: none; }
</style>
