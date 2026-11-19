<script lang="ts"></script>

<div class="splash" role="status" aria-label="Chargement">
  <img src="/loader.webp" alt="Termomix" width="393" height="283" />
</div>

<style>
  .splash { animation: in 0.25s ease-out; position: fixed; inset: 0; z-index: 100; display: grid; place-items: center; background: var(--bg); }
  @keyframes in { from { opacity: 0; } }
  img { max-width: 60vw; height: auto; user-select: none; -webkit-user-drag: none; }
</style>
