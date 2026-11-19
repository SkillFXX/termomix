<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { api, blankServer, type Server } from './api';
  import { app, attempt, reload } from './state.svelte';
  import Modal from './Modal.svelte';
  import Icon from './Icon.svelte';
  import { t } from './i18n.svelte';

  let { server, onclose }: { server: Server | null; onclose: () => void } = $props();

  // Initialisation unique à l'ouverture du formulaire.
  /* svelte-ignore state_referenced_locally */
  const isNew = !server;
  /* svelte-ignore state_referenced_locally */
  let f = $state<Server>(server ? { ...server, password: undefined, key_passphrase: undefined } : blankServer());
  let pw = $state('');
  let pass = $state('');
  let clearPw = $state(false);

  async function pickKey() {
    const p = await open({ title: t('Choisir une clé privée'), multiple: false, directory: false });
    if (typeof p === 'string') f.key_path = p;
  }

  async function save() {
    const out: Server = { ...f, port: Number(f.port) || 22 };
    if (clearPw) out.password = '';
    else if (pw) out.password = pw;
    if (pass) out.key_passphrase = pass;
    const ok = await attempt(() => api.saveServer(out));
    if (ok) {
      await reload();
      onclose();
    }
  }
</script>

<Modal title={isNew ? t('Nouveau serveur') : t('Modifier le serveur')} {onclose}>
  <div class="field">
    <label for="sf-name">{t('Nom')}</label>
    <input id="sf-name" class="input" bind:value={f.name} placeholder={t('Production web')} />
  </div>
  <div class="row">
    <div class="field" style="flex: 3">
      <label for="sf-host">{t('Hôte')}</label>
      <input id="sf-host" class="input mono" bind:value={f.host} placeholder={t('192.168.1.10 ou exemple.fr')} />
    </div>
    <div class="field" style="flex: 1">
      <label for="sf-port">Port</label>
      <input id="sf-port" class="input mono" type="number" min="1" max="65535" bind:value={f.port} />
    </div>
  </div>
  <div class="row">
    <div class="field">
      <label for="sf-user">{t('Utilisateur')}</label>
      <input id="sf-user" class="input mono" bind:value={f.username} placeholder="root" autocomplete="off" />
    </div>
    <div class="field">
      <label for="sf-group">{t('Groupe')}</label>
      <select id="sf-group" class="input" bind:value={f.group_id}>
        <option value={null}>{t('Aucun')}</option>
        {#each app.groups as g}<option value={g.id}>{g.name}</option>{/each}
      </select>
    </div>
  </div>

  <div class="field">
    <label for="sf-auth">{t('Authentification')}</label>
    <select id="sf-auth" class="input" bind:value={f.auth}>
      <option value="password">{t('Mot de passe')}</option>
      <option value="key">{t('Clé privée')}</option>
    </select>
  </div>

  {#if f.auth === 'password'}
    <div class="field">
      <label for="sf-pw">{t('Mot de passe')}</label>
      <input id="sf-pw" class="input" type="password" bind:value={pw} autocomplete="new-password"
        placeholder={f.has_password && !clearPw ? t('•••••••• (enregistré)') : t('Laisser vide pour le demander à chaque connexion')} />
      {#if f.has_password}
        <label class="check"><input type="checkbox" bind:checked={clearPw} /> {t('Supprimer le mot de passe enregistré')}</label>
      {/if}
    </div>
  {:else}
    <div class="field">
      <label for="sf-key">{t('Fichier de clé privée')}</label>
      <div class="row">
        <input id="sf-key" class="input mono" bind:value={f.key_path} placeholder="~/.ssh/id_ed25519" />
        <button class="btn" style="flex: 0 0 auto" onclick={pickKey}><Icon name="folder" /> {t('Parcourir')}</button>
      </div>
    </div>
    <div class="field">
      <label for="sf-pass">{t('Phrase secrète de la clé')}</label>
      <input id="sf-pass" class="input" type="password" bind:value={pass} autocomplete="new-password"
        placeholder={f.has_passphrase ? t('•••••••• (enregistrée)') : t('Optionnel')} />
    </div>
  {/if}

  <div class="field">
    <label for="sf-cmd">{t('Commande au démarrage')}</label>
    <input id="sf-cmd" class="input mono" bind:value={f.startup_command} placeholder="cd /var/www && ls" />
    <span class="hint">{t('Exécutée automatiquement à l\'ouverture du shell.')}</span>
  </div>
  <div class="field">
    <label for="sf-notes">{t('Notes')}</label>
    <textarea id="sf-notes" class="input" rows="2" bind:value={f.notes} style="font-family: var(--font)"></textarea>
  </div>

  {#snippet footer()}
    <button class="btn" onclick={onclose}>{t('Annuler')}</button>
    <button class="btn primary" onclick={save}>{t('Enregistrer')}</button>
  {/snippet}
</Modal>

<style>
  .check { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--muted); font-weight: 400; }
</style>
