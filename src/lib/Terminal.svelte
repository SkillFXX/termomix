<script lang="ts">
  import { onDestroy, onMount, untrack } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { SearchAddon } from '@xterm/addon-search';
  import { WebLinksAddon } from '@xterm/addon-web-links';
  import { WebglAddon } from '@xterm/addon-webgl';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { api, errMsg, type Macro } from './api';
  import { app, editMacro, macrosFor, toast, type Tab } from './state.svelte';
  import { t } from './i18n.svelte';
  import Icon from './Icon.svelte';
  import Sftp from './Sftp.svelte';

  let { tab, active }: { tab: Tab; active: boolean } = $props();
  const server = untrack(() => tab.server);

  type Status = 'ask' | 'connecting' | 'open' | 'closed' | 'error' | 'reconnecting';
  let status = $state<Status>('connecting');
  let error = $state('');
  let sessionId = $state<string | null>(null);
  let sftpOpen = $state(false);
  let sftpWidth = $state(420);
  let showSearch = $state(false);
  let query = $state('');
  let macroMenu = $state(false);
  let attempt = $state(0);
  let lastSecret: string | undefined;
  let cancelReconnect = false;

  // Saisie à la volée
  let askUser = $state(server.username);
  let askSecret = $state('');

  // Enregistrement de macro
  let recording = $state(false);
  let recorded = $state<string[]>([]);
  let line = '';

  let host: HTMLDivElement;
  let term: Terminal;
  let fit: FitAddon;
  let search: SearchAddon;
  let ro: ResizeObserver;
  let unlisten: UnlistenFn | undefined;
  let disposed = false;

  const theme = {
    background: '#0b0d15', foreground: '#dfe3f2', cursor: '#7f8cf8', cursorAccent: '#0b0d15',
    selectionBackground: 'rgba(127,140,248,0.32)',
    black: '#1a1e2a', red: '#f2667f', green: '#46d39a', yellow: '#f0b95a', blue: '#6aa8ff', magenta: '#b46cf5', cyan: '#52c7f5', white: '#dfe3ec',
    brightBlack: '#5d6579', brightRed: '#ff8097', brightGreen: '#6fe6b4', brightYellow: '#f7cd85', brightBlue: '#8dbcff', brightMagenta: '#c0a8ff', brightCyan: '#85dcfa', brightWhite: '#ffffff',
  };

  onMount(async () => {
    term = new Terminal({
      fontFamily: "'JetBrains Mono', monospace",
      fontSize: app.settings.font_size,
      scrollback: app.settings.scrollback,
      cursorStyle: app.settings.cursor_style,
      cursorBlink: app.settings.cursor_blink,
      lineHeight: app.settings.line_height,
      allowProposedApi: true,
      theme,
    });
    fit = new FitAddon();
    search = new SearchAddon();
    term.loadAddon(fit);
    term.loadAddon(search);
    term.loadAddon(new WebLinksAddon());
    term.open(host);
    try {
      const gl = new WebglAddon();
      gl.onContextLoss(() => gl.dispose());
      term.loadAddon(gl);
    } catch { /* rendu canvas par défaut */ }
    fit.fit();

    term.onData((d) => {
      if (recording) track(d);
      if (sessionId) api.sshWrite(sessionId, d).catch(() => {});
    });
    term.onSelectionChange(() => {
      if (app.settings.copy_on_select && term.hasSelection()) navigator.clipboard.writeText(term.getSelection()).catch(() => {});
    });
    term.onResize(({ cols, rows }) => sessionId && api.sshResize(sessionId, cols, rows).catch(() => {}));
    term.attachCustomKeyEventHandler((e) => {
      if (e.type !== 'keydown' || !e.ctrlKey || !e.shiftKey) return true;
      if (e.code === 'KeyC') { navigator.clipboard.writeText(term.getSelection()).catch(() => {}); return false; }
      if (e.code === 'KeyV') { navigator.clipboard.readText().then((t) => term.paste(t)).catch(() => {}); return false; }
      if (e.code === 'KeyF') { showSearch = true; return false; }
      return true;
    });

    ro = new ResizeObserver(() => active && requestAnimationFrame(() => { try { fit.fit(); } catch {} }));
    ro.observe(host);

    unlisten = await listen<{ id: string; clean: boolean }>('ssh-closed', (e) => {
      if (e.payload.id === sessionId && !disposed) onClosed(e.payload.id, e.payload.clean);
    });

    const needsInput = tab.adhoc || (server.auth === 'password' && !server.has_password);
    if (needsInput) status = 'ask';
    else connect();
  });

  $effect(() => {
    if (active && term) {
      requestAnimationFrame(() => { try { fit.fit(); } catch {} term.focus(); });
    }
  });

  async function onClosed(id: string, clean: boolean) {
    sessionId = null;
    api.sshClose(id).catch(() => {});
    if (!clean && app.settings.auto_reconnect) return autoReconnect();
    status = 'closed';
    term.write(`\r\n\x1b[90m— ${t('Connexion fermée')} —\x1b[0m\r\n`);
  }

  async function autoReconnect() {
    status = 'reconnecting';
    cancelReconnect = false;
    term.write(`\r\n\x1b[33m— ${t('Connexion perdue')} —\x1b[0m\r\n`);
    const max = app.settings.reconnect_attempts;
    for (attempt = 1; attempt <= max && !cancelReconnect && !disposed; attempt++) {
      await sleep(app.settings.reconnect_delay * 1000);
      if (cancelReconnect || disposed) break;
      if (await connect(lastSecret, true)) {
        term.write(`\r\n\x1b[32m— ${t('Reconnecté')} —\x1b[0m\r\n`);
        return;
      }
    }
    if (disposed) return;
    status = 'closed';
    term.write(`\r\n\x1b[90m— ${t('Reconnexion abandonnée')} —\x1b[0m\r\n`);
  }

  async function connect(secret?: string, silent = false): Promise<boolean> {
    if (secret) lastSecret = secret;
    if (!silent) {
      status = 'connecting';
      error = '';
    }
    const adhoc = tab.adhoc ? { ...server, username: askUser.trim() || server.username } : undefined;
    try {
      const r = await api.sshConnect(
        { server_id: tab.adhoc ? undefined : server.id, adhoc, secret, cols: term.cols, rows: term.rows },
        (b) => term.write(b),
      );
      if (disposed) { api.sshClose(r.session_id); return false; }
      sessionId = r.session_id;
      status = 'open';
      askSecret = '';
      if (r.new_host) toast(t('Nouvelle empreinte enregistrée pour {host}', { host: server.host }), 'info');
      term.focus();
      return true;
    } catch (e) {
      if (!silent) {
        error = errMsg(e);
        status = 'error';
      }
      return false;
    }
  }

  async function forget() {
    await api.forgetHostKey(server.host, server.port);
    toast(t('Empreinte oubliée. Elle sera enregistrée à la prochaine connexion.'), 'ok');
    connect(askSecret || undefined);
  }

  onDestroy(() => {
    disposed = true;
    ro?.disconnect();
    unlisten?.();
    if (sessionId) api.sshClose(sessionId).catch(() => {});
    term?.dispose();
  });

  // ----- Macros -----
  const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

  async function runMacro(m: Macro) {
    macroMenu = false;
    if (!sessionId) return toast(t('Aucune session ouverte'), 'error');
    const id = sessionId;
    for (const cmd of m.commands) {
      await api.sshWrite(id, cmd + '\r').catch(() => {});
      await sleep(m.delay_ms);
    }
    term.focus();
  }

  function track(d: string) {
    if (d.startsWith('\x1b')) return; // flèches, touches de fonction : l'historique n'est pas fiable à reconstruire
    for (const ch of d) {
      if (ch === '\r') { if (line.trim()) recorded.push(line); line = ''; }
      else if (ch === '\x7f') line = line.slice(0, -1);
      else if (ch === '\x03' || ch === '\x15') line = '';
      else if (ch >= ' ') line += ch;
    }
  }

  function toggleRecord() {
    if (!recording) {
      recorded = [];
      line = '';
      recording = true;
      term.focus();
      return;
    }
    recording = false;
    if (!recorded.length) return toast(t('Aucune commande enregistrée'), 'info');
    editMacro({ name: '', commands: [...recorded], delay_ms: 150, server_ids: [], description: null });
  }

  // Réglages appliqués en direct aux terminaux ouverts.
  $effect(() => {
    const s = app.settings;
    if (!term) return;
    term.options.fontSize = s.font_size;
    term.options.lineHeight = s.line_height;
    term.options.cursorStyle = s.cursor_style;
    term.options.cursorBlink = s.cursor_blink;
    term.options.scrollback = s.scrollback;
    if (active) requestAnimationFrame(() => { try { fit.fit(); } catch {} });
  });

  $effect(() => { tab.connected = status === 'open'; });

  // ----- Recherche -----
  $effect(() => { if (query && search) search.findNext(query); });
  function closeSearch() { showSearch = false; query = ''; search?.clearDecorations(); term.focus(); }

  // ----- Séparateur SFTP -----
  function startDrag(e: PointerEvent) {
    const startX = e.clientX, startW = sftpWidth;
    const move = (ev: PointerEvent) => { sftpWidth = Math.min(Math.max(startW + (startX - ev.clientX), 300), 800); };
    const up = () => { window.removeEventListener('pointermove', move); window.removeEventListener('pointerup', up); fit.fit(); };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  }

  let macros = $derived(macrosFor(server.id));
</script>

<div class="wrap" class:hidden={!active}>
  <div class="toolbar">
    <div class="state">
      <span class="dot {status}"></span>
      <span class="mono who">{askUser || server.username}@{server.host}{server.port !== 22 ? `:${server.port}` : ''}</span>
    </div>
    <div class="tools">
      <div class="menu-anchor">
        <button class="btn sm" disabled={status !== 'open'} onclick={() => (macroMenu = !macroMenu)}><Icon name="macro" size={14} /> {t('Macros')}</button>
        {#if macroMenu}
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div class="scrim" onclick={() => (macroMenu = false)}></div>
          <div class="menu">
            {#each macros as m (m.id)}
              <button onclick={() => runMacro(m)}>
                <Icon name="play" size={12} />
                <span class="mn">{m.name}</span>
                <span class="mc">{t('{n} cmd', { n: m.commands.length })}</span>
              </button>
            {:else}
              <p>{t('Aucune macro. Enregistrez-en une avec le bouton rouge.')}</p>
            {/each}
          </div>
        {/if}
      </div>
      <button class="btn sm" class:rec={recording} disabled={status !== 'open'} onclick={toggleRecord} title={t('Enregistrer les commandes tapées')}>
        <Icon name={recording ? 'stop' : 'record'} size={14} />
        {recording ? t('Arrêter ({n})', { n: recorded.length }) : t('Enregistrer')}
      </button>
      <button class="btn sm icon" title={t('Rechercher (Ctrl+Maj+F)')} onclick={() => (showSearch = !showSearch)}><Icon name="search" size={14} /></button>
      <button class="btn sm" class:on={sftpOpen} disabled={status !== 'open'} onclick={() => { sftpOpen = !sftpOpen; setTimeout(() => fit.fit(), 30); }}>
        <Icon name="folder" size={14} /> {t('Fichiers')}
      </button>
    </div>
  </div>

  <div class="main">
    <div class="termbox">
      <div class="term" bind:this={host}></div>

      {#if status === 'reconnecting'}
        <div class="banner">
          <div class="spinner sm"></div>
          <span>{t('Connexion perdue. Reconnexion {n}/{max}…', { n: attempt, max: app.settings.reconnect_attempts })}</span>
          <button class="btn sm" onclick={() => { cancelReconnect = true; }}>{t('Annuler')}</button>
        </div>
      {/if}

      {#if showSearch}
        <div class="searchbar">
          <!-- svelte-ignore a11y_autofocus -->
          <input class="input" placeholder={t('Rechercher')} bind:value={query} autofocus
            onkeydown={(e) => { if (e.key === 'Enter') e.shiftKey ? search.findPrevious(query) : search.findNext(query); if (e.key === 'Escape') closeSearch(); }} />
          <button class="btn ghost sm icon" onclick={() => search.findPrevious(query)}><Icon name="up" size={14} /></button>
          <button class="btn ghost sm icon" onclick={() => search.findNext(query)}><Icon name="down" size={14} /></button>
          <button class="btn ghost sm icon" onclick={closeSearch}><Icon name="x" size={14} /></button>
        </div>
      {/if}

      {#if status === 'ask' || status === 'error' || status === 'connecting' || status === 'closed'}
        <div class="overlay">
          <div class="panel">
            {#if status === 'connecting'}
              <div class="spinner"></div>
              <p>{t('Connexion à {host}…', { host: server.host })}</p>
            {:else if status === 'ask'}
              <h3>{server.name}</h3>
              {#if tab.adhoc}
                <div class="field"><label for="a-user">{t('Utilisateur')}</label><input id="a-user" class="input mono" bind:value={askUser} /></div>
              {/if}
              <div class="field">
                <label for="a-sec">{t('Mot de passe')}</label>
                <!-- svelte-ignore a11y_autofocus -->
                <input id="a-sec" class="input" type="password" bind:value={askSecret} autofocus onkeydown={(e) => e.key === 'Enter' && askUser && connect(askSecret)} />
              </div>
              <button class="btn primary" disabled={!askUser.trim()} onclick={() => connect(askSecret)}>{t('Se connecter')}</button>
            {:else if status === 'error'}
              <div class="err"><Icon name="alert" /> <span>{error.replace('HOST_KEY_CHANGED: ', '')}</span></div>
              {#if error.startsWith('HOST_KEY_CHANGED')}
                <p class="hint">{t('Cela peut signaler une réinstallation du serveur, ou une interception. Ne continuez que si vous êtes certain du changement.')}</p>
                <button class="btn danger" onclick={forget}>{t("Oublier l'ancienne empreinte et se connecter")}</button>
              {:else}
                {#if server.auth === 'password' || server.auth === 'key'}
                  <div class="field">
                    <label for="r-sec">{server.auth === 'key' ? t('Phrase secrète de la clé') : t('Mot de passe')}</label>
                    <input id="r-sec" class="input" type="password" bind:value={askSecret} onkeydown={(e) => e.key === 'Enter' && connect(askSecret)} />
                  </div>
                {/if}
                <button class="btn primary" onclick={() => connect(askSecret)}>{t('Réessayer')}</button>
              {/if}
            {:else}
              <p>{t('Session terminée.')}</p>
              <button class="btn primary" onclick={() => { term.reset(); connect(); }}>{t('Se reconnecter')}</button>
            {/if}
          </div>
        </div>
      {/if}
    </div>

    {#if sftpOpen && sessionId}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="split" onpointerdown={startDrag}></div>
      <div class="side" style:width="{sftpWidth}px">
        <Sftp {sessionId} {active} />
      </div>
    {/if}
  </div>
</div>

<style>
  .wrap { height: 100%; display: flex; flex-direction: column; }
  .wrap.hidden { display: none; }
  .toolbar { height: 40px; flex: none; display: flex; align-items: center; justify-content: space-between; padding: 0 12px; border-bottom: 1px solid var(--border); background: var(--surface); }
  .state { display: flex; align-items: center; gap: 8px; min-width: 0; }
  .who { font-size: 12px; color: var(--muted); }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--faint); }
  .dot.open { background: var(--success); }
  .dot.connecting, .dot.ask { background: var(--warn); }
  .dot.error { background: var(--danger); }
  .tools { display: flex; gap: 6px; align-items: center; }
  .btn.on { background: var(--surface-3); border-color: var(--border-strong); }
  .btn.rec { color: var(--danger); border-color: var(--danger); }
  .btn.rec :global(svg) { fill: currentColor; }
  .menu-anchor { position: relative; }
  .scrim { position: fixed; inset: 0; z-index: 9; }
  .menu { position: absolute; right: 0; top: 32px; z-index: 10; min-width: 240px; max-height: 320px; overflow: auto; background: var(--surface-2); border: 1px solid var(--border-strong); border-radius: 10px; padding: 4px; box-shadow: 0 12px 32px rgba(0, 0, 0, 0.45); }
  .menu button { display: flex; align-items: center; gap: 8px; width: 100%; padding: 7px 10px; border: 0; background: none; border-radius: 6px; text-align: left; }
  .menu button:hover { background: var(--surface-3); }
  .menu .mn { flex: 1; }
  .menu .mc { color: var(--faint); font-size: 11px; }
  .menu p { padding: 10px; color: var(--muted); font-size: 12px; max-width: 240px; }
  .main { flex: 1; min-height: 0; display: flex; }
  .termbox { position: relative; flex: 1; min-width: 0; padding: 8px 0 4px 12px; background: #0b0d15; }
  .term { height: 100%; }
  .term :global(.xterm-viewport) { background: transparent !important; }
  .split { width: 5px; cursor: col-resize; background: var(--border); flex: none; }
  .split:hover { background: var(--grad); }
  .side { flex: none; min-width: 0; background: var(--surface); }
  .searchbar { position: absolute; top: 10px; right: 18px; display: flex; gap: 2px; align-items: center; padding: 4px; background: var(--surface-2); border: 1px solid var(--border-strong); border-radius: 10px; width: 300px; }
  .searchbar .input { height: 26px; border: 0; background: transparent; }
  .banner { position: absolute; top: 10px; left: 50%; transform: translateX(-50%); z-index: 5; display: flex; align-items: center; gap: 10px; padding: 6px 8px 6px 12px; background: var(--surface-2); border: 1px solid var(--warn); border-radius: 10px; box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45); }
  .spinner.sm { width: 14px; height: 14px; }
  .overlay { position: absolute; inset: 0; display: grid; place-items: center; background: rgba(13, 15, 20, 0.78); }
  .panel { width: 340px; display: flex; flex-direction: column; gap: 12px; padding: 22px; background: var(--surface); border: 1px solid var(--border-strong); border-radius: 12px; }
  .panel h3 { font-size: 14px; }
  .err { display: flex; gap: 8px; color: var(--danger); line-height: 1.4; }
  .err :global(svg) { flex: none; margin-top: 1px; }
  .spinner { width: 22px; height: 22px; border-radius: 50%; border: 2px solid var(--surface-3); border-top-color: #7f8cf8; animation: spin 0.8s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
