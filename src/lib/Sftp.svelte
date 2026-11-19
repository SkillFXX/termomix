<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { api, errMsg, type Entry } from './api';
  import { attempt, confirmDlg, promptDlg, toast } from './state.svelte';
  import Icon from './Icon.svelte';

  let { sessionId, active }: { sessionId: string; active: boolean } = $props();

  let path = $state('');
  let entries = $state<Entry[]>([]);
  let loading = $state(false);
  let selected = $state<Set<string>>(new Set());
  let pathInput = $state('');
  let hover = $state(false);
  let unlisten: (() => void) | undefined;

  let sorted = $derived(
    entries.slice().sort((a, b) => Number(b.is_dir) - Number(a.is_dir) || a.name.localeCompare(b.name, undefined, { sensitivity: 'base' })),
  );
  let crumbs = $derived(
    path.split('/').filter(Boolean).map((n, i, arr) => ({ n, p: '/' + arr.slice(0, i + 1).join('/') })),
  );

  const join = (b: string, n: string) => (b.endsWith('/') ? b + n : `${b}/${n}`);

  async function load(p: string) {
    loading = true;
    try {
      const l = await api.sftpList(sessionId, p);
      path = l.path;
      pathInput = l.path;
      entries = l.entries;
      selected = new Set();
    } catch (e) {
      toast(errMsg(e), 'error');
      pathInput = path;
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    load('');
    unlisten = await getCurrentWebview().onDragDropEvent(async (ev) => {
      if (!active) return;
      const p = ev.payload;
      if (p.type === 'over') hover = true;
      else if (p.type === 'leave') hover = false;
      else if (p.type === 'drop') {
        hover = false;
        await uploadPaths(p.paths);
      }
    });
  });
  onDestroy(() => unlisten?.());

  async function uploadPaths(paths: string[]) {
    const dir = path;
    for (const l of paths) await attempt(() => api.sftpUpload(sessionId, l, dir));
    if (path === dir) load(dir);
  }

  async function pickUpload() {
    const r = await open({ multiple: true, title: 'Envoyer des fichiers' });
    if (r) await uploadPaths(Array.isArray(r) ? r : [r]);
  }

  async function pickUploadDir() {
    const r = await open({ directory: true, title: 'Envoyer un dossier' });
    if (typeof r === 'string') await uploadPaths([r]);
  }

  async function downloadSel() {
    const items = sorted.filter((e) => selected.has(e.name));
    if (!items.length) return toast('Sélectionnez un fichier', 'info');
    const dir = await open({ directory: true, title: 'Télécharger vers…' });
    if (typeof dir !== 'string') return;
    for (const e of items) await attempt(() => api.sftpDownload(sessionId, join(path, e.name), e.is_dir, dir));
  }

  async function mkdir() {
    const n = await promptDlg('Nouveau dossier', 'Nom du dossier');
    if (n && (await attempt(() => api.sftpMkdir(sessionId, join(path, n))) !== undefined)) load(path);
  }

  async function rename() {
    const [name] = [...selected];
    if (selected.size !== 1) return;
    const n = await promptDlg('Renommer', '', name);
    if (n && n !== name && (await attempt(() => api.sftpRename(sessionId, join(path, name), join(path, n)))) !== undefined) load(path);
  }

  async function remove() {
    const items = sorted.filter((e) => selected.has(e.name));
    if (!items.length) return;
    const msg = items.length === 1 ? `« ${items[0].name} »` : `${items.length} éléments`;
    if (!(await confirmDlg(`Supprimer ${msg} ?`, items.some((i) => i.is_dir) ? 'Les dossiers sont supprimés avec tout leur contenu. Action définitive.' : 'Action définitive.'))) return;
    for (const e of items) await attempt(() => api.sftpRemove(sessionId, join(path, e.name), e.is_dir));
    load(path);
  }

  function click(e: MouseEvent, en: Entry) {
    const s = new Set(e.ctrlKey || e.metaKey ? selected : []);
    if (e.ctrlKey || e.metaKey) s.has(en.name) ? s.delete(en.name) : s.add(en.name);
    else s.add(en.name);
    selected = s;
  }

  const open2 = (en: Entry) => en.is_dir && load(join(path, en.name));
  const up = () => load(path.replace(/\/[^/]*\/?$/, '') || '/');

  function size(n: number) {
    if (n < 1024) return `${n} o`;
    const u = ['Ko', 'Mo', 'Go', 'To'];
    let i = -1;
    do { n /= 1024; i++; } while (n >= 1024 && i < 3);
    return `${n.toFixed(n < 10 ? 1 : 0)} ${u[i]}`;
  }
  const date = (t: number | null) => (t ? new Date(t * 1000).toLocaleDateString('fr-FR', { day: '2-digit', month: 'short', year: '2-digit' }) : '');
</script>

<div class="sftp" class:hover>
  <div class="bar">
    <button class="btn ghost sm icon" title="Dossier parent" onclick={up}><Icon name="up" size={14} /></button>
    <button class="btn ghost sm icon" title="Actualiser" onclick={() => load(path)}><Icon name="refresh" size={14} /></button>
    <input class="input mono" bind:value={pathInput} onkeydown={(e) => e.key === 'Enter' && load(pathInput)} spellcheck="false" />
  </div>
  <div class="bar actions">
    <button class="btn sm" onclick={pickUpload}><Icon name="upload" size={14} /> Envoyer</button>
    <button class="btn sm icon" title="Envoyer un dossier" onclick={pickUploadDir}><Icon name="folderPlus" size={14} /></button>
    <button class="btn sm" disabled={!selected.size} onclick={downloadSel}><Icon name="download" size={14} /> Télécharger</button>
    <span class="grow"></span>
    <button class="btn ghost sm icon" title="Nouveau dossier" onclick={mkdir}><Icon name="plus" size={14} /></button>
    <button class="btn ghost sm icon" title="Renommer" disabled={selected.size !== 1} onclick={rename}><Icon name="edit" size={14} /></button>
    <button class="btn ghost sm icon" title="Supprimer" disabled={!selected.size} onclick={remove}><Icon name="trash" size={14} /></button>
  </div>

  <div class="crumbs">
    <button onclick={() => load('/')}>/</button>
    {#each crumbs as c}<span>›</span><button onclick={() => load(c.p)}>{c.n}</button>{/each}
  </div>

  <div class="list" class:loading>
    {#each sorted as en (en.name)}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="item" class:sel={selected.has(en.name)} onclick={(e) => click(e, en)} ondblclick={() => open2(en)}>
        <span class="ic" class:dir={en.is_dir}><Icon name={en.is_dir ? 'folder' : 'file'} size={15} /></span>
        <span class="nm" title={en.name}>{en.name}</span>
        <span class="sz">{en.is_dir ? '' : size(en.size)}</span>
        <span class="dt">{date(en.mtime)}</span>
      </div>
    {:else}
      {#if !loading}<p class="empty">Dossier vide</p>{/if}
    {/each}
  </div>
  {#if hover}<div class="drop">Déposez pour envoyer vers {path}</div>{/if}
</div>

<style>
  .sftp { position: relative; height: 100%; display: flex; flex-direction: column; }
  .bar { display: flex; gap: 4px; align-items: center; padding: 8px 8px 0; }
  .bar.actions { padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  .bar .input { height: 26px; font-size: 12px; }
  .grow { flex: 1; }
  .crumbs { display: flex; flex-wrap: wrap; align-items: center; gap: 2px; padding: 6px 10px; font-size: 12px; color: var(--faint); }
  .crumbs button { background: none; border: 0; padding: 1px 4px; border-radius: 4px; color: var(--muted); }
  .crumbs button:hover { background: var(--surface-3); color: var(--text); }
  .list { flex: 1; overflow: auto; padding: 0 6px 8px; }
  .list.loading { opacity: 0.5; }
  .item { display: grid; grid-template-columns: 20px 1fr auto 62px; gap: 8px; align-items: center; padding: 5px 8px; border-radius: 6px; cursor: default; }
  .item:hover { background: var(--surface-2); }
  .item.sel { background: rgba(82, 199, 245, 0.14); }
  .ic { color: var(--faint); display: inline-flex; }
  .ic.dir { color: var(--accent); }
  .nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sz, .dt { font-size: 11.5px; color: var(--faint); text-align: right; white-space: nowrap; }
  .empty { padding: 24px; text-align: center; color: var(--faint); }
  .drop { position: absolute; inset: 6px; display: grid; place-items: center; text-align: center; padding: 20px; border: 2px dashed var(--accent); border-radius: 10px; background: rgba(13, 15, 20, 0.88); color: var(--accent); }
</style>
