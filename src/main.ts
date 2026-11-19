import '@fontsource-variable/inter';
import '@fontsource-variable/sora';
import '@fontsource/jetbrains-mono/400.css';
import '@fontsource/jetbrains-mono/700.css';
import '@xterm/xterm/css/xterm.css';
import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';

if (import.meta.env.DEV && !('__TAURI_INTERNALS__' in window)) {
  const { installMock } = await import('./lib/devmock');
  installMock();
}

// Reflet au survol (voir app.css) : lancé à l'entrée du pointeur et mené à son terme.
document.addEventListener('pointerover', (e) => {
  const el = (e.target as Element | null)?.closest?.('.btn.primary, .card') as HTMLElement | null;
  if (!el || el.classList.contains('sweeping') || (el as HTMLButtonElement).disabled) return;
  if (e.relatedTarget instanceof Node && el.contains(e.relatedTarget)) return;
  el.classList.add('sweeping');
  el.addEventListener('animationend', () => el.classList.remove('sweeping'), { once: true });
});

mount(App, { target: document.getElementById('app')! });
