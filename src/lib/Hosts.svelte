<script lang="ts">
  import { api, type Group, type Server } from './api';
  import { app, attempt, confirmDlg, openTab, reload } from './state.svelte';
  import Icon from './Icon.svelte';
  import OsIcon from './OsIcon.svelte';
  import { t } from './i18n.svelte';
  import Modal from './Modal.svelte';
  import ServerForm from './ServerForm.svelte';

  let query = $state('');
  let quick = $state('');
  let editing = $state<Server | null | undefined>(undefined); // undefined = fermé, null = nouveau
  let groupDlg = $state<Group | null>(null);
  let groupName = $state('');
  let collapsed = $state<Record<string, boolean>>({});

  const match = (s: Server) => {
    const q = query.trim().toLowerCase();
    return !q || [s.name, s.host, s.username].some((v) => v.toLowerCase().includes(q));
  };

  let sections = $derived.by(() => {
    const list = app.servers.filter(match).sort((a, b) => a.name.localeCompare(b.name));
    const out = app.groups
      .slice()
      .sort((a, b) => a.name.localeCompare(b.name))
      .map((g) => ({ group: g as Group | null, servers: list.filter((s) => s.group_id === g.id) }));
    out.push({ group: null, servers: list.filter((s) => !s.group_id || !app.groups.some((g) => g.id === s.group_id)) });
    return out.filter((s) => s.servers.length || (s.group && !query));
  });

  let recents = $derived(
    app.servers
      .filter((s) => s.last_connected)
      .sort((a, b) => (b.last_connected ?? 0) - (a.last_connected ?? 0))
      .slice(0, 4),
  );

  function quickConnect() {
    const m = quick.trim().match(/^(?:([^@\s]+)@)?([^:\s]+)(?::(\d{1,5}))?$/);
    if (!m) return;
    const [, user, host, port] = m;
    const s: Server = { id: '', name: host, host, port: port ? +port : 22, username: user ?? '', group_id: null, auth: 'password', key_path: null, startup_command: null, notes: null, last_connected: null };
    openTab(s, true);
    quick = '';
  }

  async function remove(s: Server) {
    if (!(await confirmDlg(t('Supprimer « {name} » ?', { name: s.name }), t('Cette action est définitive.')))) return;
    await attempt(() => api.deleteServer(s.id));
    await reload();
  }

  async function duplicate(s: Server) {
    await attempt(() => api.saveServer({ ...s, id: '', name: t('{name} (copie)', { name: s.name }), password: undefined, key_passphrase: undefined }));
    await reload();
  }

  async function saveGroup() {
    const g = groupDlg!;
    const ok = await attempt(() => api.saveGroup({ id: g.id, name: groupName }));
    if (ok) {
      groupDlg = null;
      await reload();
    }
  }

  async function removeGroup(g: Group) {
    if (!(await confirmDlg(t('Supprimer le groupe « {name} » ?', { name: g.name }), t('Les serveurs du groupe sont conservés.')))) return;
    await attempt(() => api.deleteGroup(g.id));
    await reload();
  }

  const ago = (ts: number | null) => {
    if (!ts) return t('Jamais connecté');
    const d = Math.floor(Date.now() / 1000 - ts);
    if (d < 90) return t("À l'instant");
    if (d < 5400) return t('Il y a {n} min', { n: Math.round(d / 60) });
    if (d < 129600) return t('Il y a {n} h', { n: Math.round(d / 3600) });
    return t('Il y a {n} j', { n: Math.round(d / 86400) });
  };
</script>

<div class="page">
  <header>
    <div>
      <h1>{t('Serveurs')}</h1>
      <p class="muted">{t(app.servers.length > 1 ? '{n} enregistrés' : '{n} enregistré', { n: app.servers.length })}</p>
    </div>
    <div class="actions">
      <button class="btn" onclick={() => { groupDlg = { id: '', name: '' }; groupName = ''; }}><Icon name="folderPlus" /> {t('Groupe')}</button>
      <button class="btn primary" onclick={() => (editing = null)}><Icon name="plus" /> {t('Serveur')}</button>
    </div>
  </header>

  <div class="bar">
    <div class="search">
      <Icon name="search" size={15} />
      <input class="input" placeholder={t('Rechercher un serveur')} bind:value={query} />
    </div>
    <form class="quick" onsubmit={(e) => { e.preventDefault(); quickConnect(); }}>
      <Icon name="terminal" size={15} />
      <input class="input mono" placeholder={t('Connexion rapide : utilisateur@hôte:port')} bind:value={quick} />
      <button class="btn" type="submit" disabled={!quick.trim()}>{t('Connecter')}</button>
    </form>
  </div>

  {#if !query && recents.length}
    <section>
      <h3>{t('Récents')}</h3>
      <div class="grid">
        {#each recents as s (s.id)}
          {@render card(s, false)}
        {/each}
      </div>
    </section>
  {/if}

  {#each sections as sec (sec.group?.id ?? 'none')}
    <section>
      <div class="sec-head">
        <button class="toggle" onclick={() => (collapsed[sec.group?.id ?? 'none'] = !collapsed[sec.group?.id ?? 'none'])}>
          <span class="chev" class:open={!collapsed[sec.group?.id ?? 'none']}><Icon name="chevron" size={14} /></span>
          <h3>{sec.group?.name ?? t('Sans groupe')}</h3>
          <span class="count">{sec.servers.length}</span>
        </button>
        {#if sec.group}
          <div class="sec-actions">
            <button class="btn ghost sm icon" title={t('Renommer')} onclick={() => { groupDlg = sec.group; groupName = sec.group!.name; }}><Icon name="edit" size={14} /></button>
            <button class="btn ghost sm icon" title={t('Supprimer le groupe')} onclick={() => removeGroup(sec.group!)}><Icon name="trash" size={14} /></button>
          </div>
        {/if}
      </div>
      {#if !collapsed[sec.group?.id ?? 'none']}
        <div class="grid">
          {#each sec.servers as s (s.id)}
            {@render card(s, true)}
          {:else}
            <p class="empty">{t('Groupe vide.')}</p>
          {/each}
        </div>
      {/if}
    </section>
  {/each}

  {#if !app.servers.length}
    <div class="welcome">
      <img src="/logo-full.png" alt="Termomix" width="260" />
      <h2>{t('Aucun serveur pour le moment')}</h2>
      <p class="muted">{t('Ajoutez votre premier serveur, ou importez vos données depuis un autre ordinateur.')}</p>
      <button class="btn primary" onclick={() => (editing = null)}><Icon name="plus" /> {t('Ajouter un serveur')}</button>
    </div>
  {/if}
</div>

{#snippet card(s: Server, actions: boolean)}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="card" onclick={() => openTab(s)} ondblclick={(e) => e.preventDefault()}>
    {#if s.os}<OsIcon os={s.os} size={36} title={s.os_name ?? ""} />{:else}<div class="avatar">{s.name.slice(0, 1).toUpperCase()}</div>{/if}
    <div class="info">
      <div class="name">{s.name}</div>
      <div class="addr mono">{s.username}@{s.host}{s.port !== 22 ? `:${s.port}` : ''}</div>
      <div class="meta">
        <Icon name={s.auth === 'key' ? 'key' : 'terminal'} size={12} /> {s.auth === 'key' ? t('Clé') : t('Mot de passe')} · {ago(s.last_connected)}
      </div>
    </div>
    {#if actions}
      <div class="card-actions" onclick={(e) => e.stopPropagation()}>
        <button class="btn ghost sm icon" title={t('Modifier')} onclick={() => (editing = s)}><Icon name="edit" size={14} /></button>
        <button class="btn ghost sm icon" title={t('Dupliquer')} onclick={() => duplicate(s)}><Icon name="copy" size={14} /></button>
        <button class="btn ghost sm icon" title={t('Supprimer')} onclick={() => remove(s)}><Icon name="trash" size={14} /></button>
      </div>
    {/if}
  </div>
{/snippet}

{#if editing !== undefined}
  <ServerForm server={editing} onclose={() => (editing = undefined)} />
{/if}

{#if groupDlg}
  <Modal title={groupDlg.id ? t('Renommer le groupe') : t('Nouveau groupe')} width={400} onclose={() => (groupDlg = null)}>
    <div class="field">
      <label for="gname">{t('Nom')}</label>
      <!-- svelte-ignore a11y_autofocus -->
      <input id="gname" class="input" bind:value={groupName} autofocus onkeydown={(e) => e.key === 'Enter' && saveGroup()} />
    </div>
    {#snippet footer()}
      <button class="btn" onclick={() => (groupDlg = null)}>{t('Annuler')}</button>
      <button class="btn primary" onclick={saveGroup} disabled={!groupName.trim()}>{t('Enregistrer')}</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .page { height: 100%; overflow: auto; padding: 28px 32px 40px; display: flex; flex-direction: column; gap: 22px; }
  header { display: flex; justify-content: space-between; align-items: flex-end; }
  h1 { font-size: 22px; font-weight: 600; letter-spacing: -0.01em; }
  .actions { display: flex; gap: 8px; }
  .bar { display: flex; gap: 12px; }
  .search, .quick { position: relative; display: flex; align-items: center; gap: 8px; color: var(--faint); }
  .search { flex: 1; }
  .quick { flex: 1.3; }
  .search :global(svg), .quick :global(svg) { position: absolute; left: 10px; pointer-events: none; }
  .search .input, .quick .input { padding-left: 32px; }
  h3 { font-size: 12px; font-weight: 600; color: var(--muted); text-transform: uppercase; letter-spacing: 0.06em; }
  section { display: flex; flex-direction: column; gap: 10px; }
  .sec-head { display: flex; align-items: center; justify-content: space-between; }
  .sec-head .sec-actions { opacity: 0; transition: opacity 0.1s; }
  .sec-head:hover .sec-actions { opacity: 1; }
  .toggle { display: flex; align-items: center; gap: 6px; background: none; border: 0; padding: 2px 0; color: inherit; }
  .chev { display: inline-flex; transition: transform 0.12s; color: var(--faint); }
  .chev.open { transform: rotate(90deg); }
  .count { font-size: 11px; color: var(--faint); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(270px, 1fr)); gap: 10px; }
  .card { position: relative; display: flex; gap: 12px; align-items: center; padding: 12px; background: var(--surface); border: 1px solid var(--border); border-radius: 10px; cursor: pointer; transition: border-color 0.12s, background 0.12s; }
  .card { overflow: hidden; isolation: isolate; }
  .card::after { content: ''; position: absolute; inset: 0; z-index: -1; pointer-events: none; background: linear-gradient(105deg, transparent 38%, rgba(255, 255, 255, 0.07) 50%, transparent 62%); transform: translateX(-110%); }
  .card:hover::after { transform: translateX(110%); transition: transform 0.5s ease-out; }
  @media (prefers-reduced-motion: reduce) { .card::after { display: none; } }
  .card:hover { background: var(--surface-2); border-color: #4a537f; }
  .avatar { flex: none; width: 36px; height: 36px; border-radius: 8px; background: linear-gradient(var(--surface-2), var(--surface-2)) padding-box, var(--grad) border-box; border: 1px solid transparent; display: grid; place-items: center; font-family: var(--brand); font-weight: 650; color: var(--text); }
  .info { min-width: 0; flex: 1; }
  .name { font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .addr { font-size: 12px; color: var(--muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .meta { display: flex; align-items: center; gap: 5px; font-size: 11.5px; color: var(--faint); margin-top: 2px; }
  .card-actions { position: absolute; top: 6px; right: 6px; display: flex; opacity: 0; background: var(--surface-2); border-radius: 6px; }
  .card:hover .card-actions { opacity: 1; }
  .empty { color: var(--faint); font-size: 12px; }
  .welcome { margin: 60px auto; display: flex; flex-direction: column; align-items: center; gap: 10px; text-align: center; max-width: 380px; }
  .welcome h2 { font-size: 16px; }
</style>
