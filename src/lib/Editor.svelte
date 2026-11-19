<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { EditorState, type Extension } from '@codemirror/state';
  import { EditorView, drawSelection, highlightActiveLine, highlightActiveLineGutter, keymap, lineNumbers } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
  import { bracketMatching, foldGutter, HighlightStyle, indentOnInput, StreamLanguage, syntaxHighlighting } from '@codemirror/language';
  import { highlightSelectionMatches, searchKeymap } from '@codemirror/search';
  import { tags as tg } from '@lezer/highlight';
  import { api, errMsg } from './api';
  import { app, confirmDlg, toast } from './state.svelte';
  import { t } from './i18n.svelte';
  import Icon from './Icon.svelte';

  let { sessionId, path, onclose }: { sessionId: string; path: string; onclose: () => void } = $props();

  let host: HTMLDivElement;
  let view: EditorView | undefined;
  let status = $state<'loading' | 'ready' | 'error'>('loading');
  let error = $state('');
  let dirty = $state(false);
  let saving = $state(false);
  let eol = $state('\n');
  let saved = '';
  /* svelte-ignore state_referenced_locally */
  const name = path.split('/').pop() ?? path;

  // Chargement paresseux des langages : seul celui du fichier ouvert est téléchargé.
  async function language(file: string): Promise<Extension | null> {
    const ext = file.includes('.') ? file.split('.').pop()!.toLowerCase() : '';
    const base = file.toLowerCase();
    switch (true) {
      case ['js', 'mjs', 'cjs', 'jsx'].includes(ext): return (await import('@codemirror/lang-javascript')).javascript();
      case ['ts', 'tsx'].includes(ext): return (await import('@codemirror/lang-javascript')).javascript({ typescript: true, jsx: ext === 'tsx' });
      case ext === 'json': return (await import('@codemirror/lang-json')).json();
      case ['py'].includes(ext): return (await import('@codemirror/lang-python')).python();
      case ['html', 'htm', 'vue', 'svelte'].includes(ext): return (await import('@codemirror/lang-html')).html();
      case ['css', 'scss'].includes(ext): return (await import('@codemirror/lang-css')).css();
      case ['md', 'markdown'].includes(ext): return (await import('@codemirror/lang-markdown')).markdown();
      case ['yml', 'yaml'].includes(ext): return (await import('@codemirror/lang-yaml')).yaml();
      case ['xml', 'svg'].includes(ext): return (await import('@codemirror/lang-xml')).xml();
      case ext === 'sql': return (await import('@codemirror/lang-sql')).sql();
      case ['sh', 'bash', 'zsh', 'env', 'profile', 'bashrc', 'zshrc'].includes(ext) || base.startsWith('.bash') || base.startsWith('.zsh') || base === '.profile':
        return StreamLanguage.define((await import('@codemirror/legacy-modes/mode/shell')).shell);
      case ext === 'toml': return StreamLanguage.define((await import('@codemirror/legacy-modes/mode/toml')).toml);
      case ['ini', 'conf', 'cfg', 'properties'].includes(ext): return StreamLanguage.define((await import('@codemirror/legacy-modes/mode/properties')).properties);
      case base === 'dockerfile': return StreamLanguage.define((await import('@codemirror/legacy-modes/mode/dockerfile')).dockerFile);
      case base.includes('nginx'): return StreamLanguage.define((await import('@codemirror/legacy-modes/mode/nginx')).nginx);
      default: return null;
    }
  }

  const theme = EditorView.theme(
    {
      '&': { height: '100%', backgroundColor: '#0b0d15', color: '#dfe3f2' },
      '.cm-scroller': { fontFamily: "'JetBrains Mono', monospace", fontSize: '13px', lineHeight: '1.55' },
      '.cm-content': { caretColor: '#7f8cf8' },
      '.cm-cursor': { borderLeftColor: '#7f8cf8' },
      '&.cm-focused': { outline: 'none' },
      '.cm-gutters': { backgroundColor: '#0b0d15', color: '#454c70', border: 'none' },
      '.cm-activeLine': { backgroundColor: 'rgba(127,140,248,0.06)' },
      '.cm-activeLineGutter': { backgroundColor: 'rgba(127,140,248,0.08)', color: '#8d95b5' },
      '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection': { backgroundColor: 'rgba(127,140,248,0.3) !important' },
      '.cm-searchMatch': { backgroundColor: 'rgba(240,185,90,0.25)' },
      '.cm-panels': { backgroundColor: '#181c2a', color: '#e8eaf5' },
      '.cm-panels input, .cm-panels button': { fontFamily: 'inherit' },
      '.cm-panels input': { backgroundColor: '#0b0d15', border: '1px solid #353c63', borderRadius: '4px', color: '#e8eaf5' },
      '.cm-panels button': { backgroundImage: 'none', backgroundColor: '#222740', border: '1px solid #353c63', borderRadius: '4px', color: '#e8eaf5' },
    },
    { dark: true },
  );

  const highlight = HighlightStyle.define([
    { tag: [tg.keyword, tg.operatorKeyword, tg.modifier], color: '#b48cf7' },
    { tag: [tg.string, tg.special(tg.string)], color: '#7fe0b0' },
    { tag: [tg.number, tg.bool, tg.null, tg.atom], color: '#f0b95a' },
    { tag: [tg.comment, tg.lineComment, tg.blockComment], color: '#5c6489', fontStyle: 'italic' },
    { tag: [tg.function(tg.variableName), tg.function(tg.propertyName)], color: '#6fc8ff' },
    { tag: [tg.typeName, tg.className, tg.tagName], color: '#6fc8ff' },
    { tag: [tg.propertyName, tg.attributeName], color: '#8fb4ff' },
    { tag: [tg.definition(tg.variableName), tg.variableName], color: '#dfe3f2' },
    { tag: [tg.operator, tg.punctuation, tg.bracket], color: '#8d95b5' },
    { tag: [tg.heading], color: '#6fc8ff', fontWeight: '600' },
    { tag: [tg.link, tg.url], color: '#6fc8ff', textDecoration: 'underline' },
    { tag: tg.invalid, color: '#f2667f' },
  ]);

  onMount(async () => {
    let text: string;
    try {
      text = await api.sftpReadText(sessionId, path);
    } catch (e) {
      error = errMsg(e);
      status = 'error';
      return;
    }
    eol = text.includes('\r\n') ? '\r\n' : '\n';
    const lang = await language(name).catch(() => null);
    const exts: Extension[] = [
      lineNumbers(), foldGutter(), highlightActiveLine(), highlightActiveLineGutter(), history(), drawSelection(),
      indentOnInput(), bracketMatching(), highlightSelectionMatches(), syntaxHighlighting(highlight), theme,
      EditorState.lineSeparator.of(eol),
      keymap.of([{ key: 'Mod-s', preventDefault: true, run: () => (save(), true) }, indentWithTab, ...defaultKeymap, ...historyKeymap, ...searchKeymap]),
      EditorView.updateListener.of((u) => { if (u.docChanged) dirty = u.state.doc.toString() !== saved; }),
    ];
    if (lang) exts.push(lang);
    view = new EditorView({ parent: host, state: EditorState.create({ doc: text, extensions: exts }) });
    saved = view.state.doc.toString();
    status = 'ready';
    view.focus();
  });

  onDestroy(() => view?.destroy());

  async function save() {
    if (!view || saving) return;
    saving = true;
    try {
      const content = view.state.sliceDoc(0, view.state.doc.length);
      await api.sftpWriteText(sessionId, path, content);
      saved = view.state.doc.toString();
      dirty = false;
      toast(t('Fichier enregistré'), 'ok');
    } catch (e) {
      toast(errMsg(e), 'error');
    } finally {
      saving = false;
    }
  }

  async function close() {
    if (dirty && !(await confirmDlg(t('Fermer sans enregistrer ?'), t('Les modifications de « {name} » seront perdues.', { name })))) return;
    onclose();
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && !document.querySelector('.cm-panel') && close()} />

<div class="overlay">
  <div class="editor" role="dialog" aria-label={name}>
    <header>
      <Icon name="file" size={15} />
      <span class="title">{name}</span>
      {#if dirty}<span class="dot" title={t('Modifications non enregistrées')}></span>{/if}
      <span class="path mono">{path}</span>
      <span class="grow"></span>
      <button class="btn primary sm" disabled={!dirty || saving} onclick={save}>{t('Enregistrer')}</button>
      <button class="btn ghost sm icon" aria-label={t('Fermer')} onclick={close}><Icon name="x" size={14} /></button>
    </header>
    <div class="body">
      <div class="cm" bind:this={host} class:hide={status !== 'ready'}></div>
      {#if status === 'loading'}<p class="info">{t('Chargement…')}</p>{/if}
      {#if status === 'error'}
        <p class="info err">
          {#if error.startsWith('FILE_TOO_LARGE')}{t('Fichier trop volumineux pour l\'édition en ligne (5 Mo maximum). Téléchargez-le plutôt.')}
          {:else if error === 'NOT_TEXT'}{t('Ce fichier n\'est pas du texte UTF-8, il ne peut pas être modifié ici.')}
          {:else}{error}{/if}
        </p>
      {/if}
    </div>
    <footer>
      <span>{t('Ctrl+S pour enregistrer · Ctrl+F pour rechercher')}</span>
      <span>{eol === '\r\n' ? 'CRLF' : 'LF'} · UTF-8</span>
    </footer>
  </div>
</div>

<style>
  .overlay { position: fixed; inset: 0; z-index: 60; background: rgba(5, 6, 9, 0.7); display: grid; place-items: center; }
  .editor { width: min(1100px, calc(100vw - 48px)); height: calc(100vh - 56px); display: flex; flex-direction: column; background: var(--bg); border: 1px solid var(--border-strong); border-radius: 12px; overflow: hidden; box-shadow: 0 24px 60px rgba(0, 0, 0, 0.55); }
  header { display: flex; align-items: center; gap: 8px; padding: 8px 10px 8px 14px; background: var(--surface); border-bottom: 1px solid var(--border); color: var(--muted); }
  .title { color: var(--text); font-weight: 600; }
  .path { font-size: 11.5px; color: var(--faint); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; min-width: 0; }
  .grow { flex: 1; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--warn); flex: none; }
  .body { position: relative; flex: 1; min-height: 0; }
  .cm { height: 100%; }
  .cm.hide { visibility: hidden; }
  .info { position: absolute; inset: 0; display: grid; place-items: center; text-align: center; padding: 30px; color: var(--muted); }
  .info.err { color: var(--danger); }
  footer { display: flex; justify-content: space-between; padding: 5px 14px; font-size: 11.5px; color: var(--faint); background: var(--surface); border-top: 1px solid var(--border); }
</style>
