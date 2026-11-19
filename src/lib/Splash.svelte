<script lang="ts">
  import { onMount } from 'svelte';

  let { onfinished, leaving = false }: { onfinished: () => void; leaving?: boolean } = $props();

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

</script>

<div class="splash" class:leaving role="status" aria-label="Chargement">
  <img
    src="/loader.webp"
    alt="Termomix"
    width="393"
    height="283"
    onload={() => setTimeout(finish, DURATION)}
    onerror={finish}
  />
</div>

<style>
  .splash { position: fixed; inset: 0; z-index: 100; display: grid; place-items: center; background: var(--bg); }
  /* Sortie pilotée en CSS : l'élément reste invisible (et non remis à zéro) jusqu'à son retrait. */
  .splash.leaving { opacity: 0; visibility: hidden; pointer-events: none; transition: opacity 0.6s ease-in-out, visibility 0s 0.6s; }
  img { max-width: 60vw; height: auto; user-select: none; -webkit-user-drag: none; }
  .leaving img { opacity: 0; transform: translateY(-22px) scale(1.04); transition: opacity 0.6s ease-in-out, transform 0.6s ease-in-out; }
</style>
