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

mount(App, { target: document.getElementById('app')! });
