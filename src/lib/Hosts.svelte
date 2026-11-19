<script lang="ts">
  import { api, type Group, type Server } from './api';
  import { app, attempt, confirmDlg, openTab, reload } from './state.svelte';
  import Icon from './Icon.svelte';
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
    if (!(await confirmDlg(`Supprimer « ${s.name} » ?`, 'Cette action est définitive.'))) return;
    await attempt(() => api.deleteServer(s.id));
    await reload();
  }

  async function duplicate(s: Server) {
    await attempt(() => api.saveServer({ ...s, id: '', name: `${s.name} (copie)`, password: undefined, key_passphrase: undefined }));
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
    if (!(await confirmDlg(`Supprimer le groupe « ${g.name} » ?`, 'Les serveurs du groupe sont conservés.'))) return;
    await attempt(() => api.deleteGroup(g.id));
    await reload();
  }

  const ago = (t: number | null) => {
    if (!t) return 'Jamais connecté';
    const d = Math.floor(Date.now() / 1000 - t);
    if (d < 90) return "À l'instant";
    if (d < 5400) return `Il y a ${Math.round(d / 60)} min`;
    if (d < 129600) return `Il y a ${Math.round(d / 3600)} h`;
    return `Il y a ${Math.round(d / 86400)} j`;
  };
</script>

<div class="page">
  <header>
    <div>
      <h1>Serveurs</h1>
      <p class="muted">{app.servers.length} enregistré{app.servers.length > 1 ? 's' : ''}</p>
    </div>
    <div class="actions">
      <button class="btn" onclick={() => { groupDlg = { id: '', name: '' }; groupName = ''; }}><Icon name="folderPlus" /> Groupe</button>
      <button class="btn primary" onclick={() => (editing = null)}><Icon name="plus" /> Serveur</button>
    </div>
  </header>

  <div class="bar">
    <div class="search">
      <Icon name="search" size={15} />
      <input class="input" placeholder="Rechercher un serveur" bind:value={query} />
    </div>
    <form class="quick" onsubmit={(e) => { e.preventDefault(); quickConnect(); }}>
      <Icon name="terminal" size={15} />
      <input class="input mono" placeholder="Connexion rapide : utilisateur@hôte:port" bind:value={quick} />
      <button class="btn" type="submit" disabled={!quick.trim()}>Connecter</button>
    </form>
  </div>

  {#if !query && recents.length}
    <section>
      <h3>Récents</h3>
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
          <h3>{sec.group?.name ?? 'Sans groupe'}</h3>
          <span class="count">{sec.servers.length}</span>
        </button>
        {#if sec.group}
          <div class="sec-actions">
            <button class="btn ghost sm icon" title="Renommer" onclick={() => { groupDlg = sec.group; groupName = sec.group!.name; }}><Icon name="edit" size={14} /></button>
            <button class="btn ghost sm icon" title="Supprimer le groupe" onclick={() => removeGroup(sec.group!)}><Icon name="trash" size={14} /></button>
          </div>
        {/if}
      </div>
      {#if !collapsed[sec.group?.id ?? 'none']}
        <div class="grid">
          {#each sec.servers as s (s.id)}
            {@render card(s, true)}
          {:else}
            <p class="empty">Groupe vide.</p>
          {/each}
        </div>
      {/if}
    </section>
  {/each}

  {#if !app.servers.length}
    <div class="welcome">
      <img src="/logo-mark.png" alt="" width="72" height="72" />
      <h2>Aucun serveur pour le moment</h2>
      <p class="muted">Ajoutez votre premier serveur, ou importez vos données depuis un autre ordinateur.</p>
      <button class="btn primary" onclick={() => (editing = null)}><Icon name="plus" /> Ajouter un serveur</button>
    </div>
  {/if}
</div>

{#snippet card(s: Server, actions: boolean)}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="card" onclick={() => openTab(s)} ondblclick={(e) => e.preventDefault()}>
    <div class="avatar">{s.name.slice(0, 1).toUpperCase()}</div>
    <div class="info">
      <div class="name">{s.name}</div>
      <div class="addr mono">{s.username}@{s.host}{s.port !== 22 ? `:${s.port}` : ''}</div>
      <div class="meta">
        <Icon name={s.auth === 'key' ? 'key' : 'terminal'} size={12} /> {s.auth === 'key' ? 'Clé' : 'Mot de passe'} · {ago(s.last_connected)}
      </div>
    </div>
    {#if actions}
      <div class="card-actions" onclick={(e) => e.stopPropagation()}>
        <button class="btn ghost sm icon" title="Modifier" onclick={() => (editing = s)}><Icon name="edit" size={14} /></button>
        <button class="btn ghost sm icon" title="Dupliquer" onclick={() => duplicate(s)}><Icon name="copy" size={14} /></button>
        <button class="btn ghost sm icon" title="Supprimer" onclick={() => remove(s)}><Icon name="trash" size={14} /></button>
      </div>
    {/if}
  </div>
{/snippet}

{#if editing !== undefined}
  <ServerForm server={editing} onclose={() => (editing = undefined)} />
{/if}

{#if groupDlg}
  <Modal title={groupDlg.id ? 'Renommer le groupe' : 'Nouveau groupe'} width={400} onclose={() => (groupDlg = null)}>
    <div class="field">
      <label for="gname">Nom</label>
      <!-- svelte-ignore a11y_autofocus -->
      <input id="gname" class="input" bind:value={groupName} autofocus onkeydown={(e) => e.key === 'Enter' && saveGroup()} />
    </div>
    {#snippet footer()}
      <button class="btn" onclick={() => (groupDlg = null)}>Annuler</button>
      <button class="btn primary" onclick={saveGroup} disabled={!groupName.trim()}>Enregistrer</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .page { height: 100%; overflow: auto; padding: 28px 32px 40px; display: flex; flex-direction: column; gap: 22px; }
  header { display: flex; justify-content: space-between; align-items: flex-end; }
  h1 { font-size: 20px; font-weight: 650; letter-spacing: -0.01em; }
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
  .card:hover { background: var(--surface-2); border-color: var(--border-strong); }
  .avatar { flex: none; width: 36px; height: 36px; border-radius: 8px; background: var(--surface-3); display: grid; place-items: center; font-weight: 650; color: var(--accent); }
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
