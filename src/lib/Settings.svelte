<script lang="ts">
  import { app, updateSettings } from './state.svelte';
  import { t } from './i18n.svelte';
  import Modal from './Modal.svelte';
  import Icon from './Icon.svelte';
  import Toggle from './Toggle.svelte';
  import DataPanel from './DataPanel.svelte';

  const tabs = [
    { id: 'general', icon: 'settings', label: 'Général' },
    { id: 'terminal', icon: 'terminal', label: 'Terminal' },
    { id: 'connection', icon: 'plug', label: 'Connexion' },
    { id: 'data', icon: 'archive', label: 'Données' },
    { id: 'about', icon: 'info', label: 'À propos' },
  ] as const;

  let s = $derived(app.settings);
  const num = (e: Event, min: number, max: number) => Math.min(max, Math.max(min, Number((e.currentTarget as HTMLInputElement).value) || min));
  const close = () => (app.settingsOpen = false);
</script>

<Modal title={t('Réglages')} width={760} onclose={close} flush>
  <div class="layout">
    <nav>
      {#each tabs as tb}
        <button class:on={app.settingsTab === tb.id} onclick={() => (app.settingsTab = tb.id)}><Icon name={tb.icon} size={15} /> {t(tb.label)}</button>
      {/each}
    </nav>

    <div class="content">
      {#if app.settingsTab === 'general'}
        <h3>{t('Général')}</h3>
        <div class="opt">
          <div><div class="l">{t('Langue')}</div><div class="d">{t('Langue de l\'interface.')}</div></div>
          <select class="input w" value={s.language} onchange={(e) => updateSettings({ language: e.currentTarget.value as 'fr' | 'en' })}>
            <option value="fr">Français</option>
            <option value="en">English</option>
          </select>
        </div>
        <div class="opt">
          <div><div class="l">{t('Animation de démarrage')}</div><div class="d">{t('Affiche le logo animé au lancement de l\'application.')}</div></div>
          <Toggle checked={s.splash} label={t('Animation de démarrage')} onchange={(v) => updateSettings({ splash: v })} />
        </div>
        <div class="opt">
          <div><div class="l">{t('Confirmer la fermeture d\'une session')}</div><div class="d">{t('Demande une confirmation avant de fermer un onglet connecté.')}</div></div>
          <Toggle checked={s.confirm_close} label={t('Confirmer la fermeture d\'une session')} onchange={(v) => updateSettings({ confirm_close: v })} />
        </div>
      {:else if app.settingsTab === 'terminal'}
        <h3>{t('Terminal')}</h3>
        <div class="opt">
          <div><div class="l">{t('Taille de la police')}</div><div class="d">{s.font_size} px</div></div>
          <input class="range" type="range" min="10" max="24" step="1" value={s.font_size} oninput={(e) => updateSettings({ font_size: num(e, 10, 24) })} />
        </div>
        <div class="opt">
          <div><div class="l">{t('Interligne')}</div><div class="d">{s.line_height.toFixed(2)}</div></div>
          <input class="range" type="range" min="1" max="1.6" step="0.05" value={s.line_height} oninput={(e) => updateSettings({ line_height: Number(e.currentTarget.value) })} />
        </div>
        <div class="opt">
          <div><div class="l">{t('Forme du curseur')}</div></div>
          <select class="input w" value={s.cursor_style} onchange={(e) => updateSettings({ cursor_style: e.currentTarget.value as any })}>
            <option value="bar">{t('Barre')}</option>
            <option value="block">{t('Bloc')}</option>
            <option value="underline">{t('Souligné')}</option>
          </select>
        </div>
        <div class="opt">
          <div><div class="l">{t('Curseur clignotant')}</div></div>
          <Toggle checked={s.cursor_blink} label={t('Curseur clignotant')} onchange={(v) => updateSettings({ cursor_blink: v })} />
        </div>
        <div class="opt">
          <div><div class="l">{t('Copier à la sélection')}</div><div class="d">{t('Copie automatiquement le texte sélectionné dans le presse-papiers.')}</div></div>
          <Toggle checked={s.copy_on_select} label={t('Copier à la sélection')} onchange={(v) => updateSettings({ copy_on_select: v })} />
        </div>
        <div class="opt">
          <div><div class="l">{t('Historique de défilement')}</div><div class="d">{t('Nombre de lignes conservées.')}</div></div>
          <select class="input w" value={s.scrollback} onchange={(e) => updateSettings({ scrollback: Number(e.currentTarget.value) })}>
            {#each [1000, 5000, 10000, 50000, 100000] as n}<option value={n}>{n.toLocaleString(s.language)}</option>{/each}
          </select>
        </div>
        <p class="hint">{t('Les réglages du terminal s\'appliquent immédiatement aux sessions ouvertes.')}</p>
      {:else if app.settingsTab === 'connection'}
        <h3>{t('Connexion')}</h3>
        <div class="opt">
          <div><div class="l">{t('Reconnexion automatique')}</div><div class="d">{t('Se reconnecte tout seul après une coupure réseau. Une commande « exit » ne déclenche pas de reconnexion.')}</div></div>
          <Toggle checked={s.auto_reconnect} label={t('Reconnexion automatique')} onchange={(v) => updateSettings({ auto_reconnect: v })} />
        </div>
        <div class="opt" class:off={!s.auto_reconnect}>
          <div><div class="l">{t('Nombre de tentatives')}</div></div>
          <input class="input w" type="number" min="1" max="50" value={s.reconnect_attempts} disabled={!s.auto_reconnect} onchange={(e) => updateSettings({ reconnect_attempts: num(e, 1, 50) })} />
        </div>
        <div class="opt" class:off={!s.auto_reconnect}>
          <div><div class="l">{t('Délai entre tentatives (secondes)')}</div></div>
          <input class="input w" type="number" min="1" max="120" value={s.reconnect_delay} disabled={!s.auto_reconnect} onchange={(e) => updateSettings({ reconnect_delay: num(e, 1, 120) })} />
        </div>
        <div class="opt">
          <div><div class="l">{t('Signal de maintien (secondes)')}</div><div class="d">{t('Évite que la connexion soit coupée par inactivité. 0 pour désactiver.')}</div></div>
          <input class="input w" type="number" min="0" max="600" value={s.keepalive} onchange={(e) => updateSettings({ keepalive: num(e, 0, 600) })} />
        </div>
        <div class="opt">
          <div><div class="l">{t('Délai de connexion (secondes)')}</div></div>
          <input class="input w" type="number" min="3" max="120" value={s.connect_timeout} onchange={(e) => updateSettings({ connect_timeout: num(e, 3, 120) })} />
        </div>
        <p class="hint">{t('Ces réglages s\'appliquent aux prochaines connexions.')}</p>
      {:else if app.settingsTab === 'data'}
        <h3>{t('Données')}</h3>
        <DataPanel />
        <div class="path"><span class="l">{t('Dossier de données')}</span><code class="mono">{app.dataDir}</code></div>
      {:else}
        <div class="about">
          <img src="/logo-full.png" alt="Termomix" width="240" />
          <p class="ver">{t('Version {v}', { v: app.version })}</p>
          <p class="muted">{t('Client SSH moderne, local et sans cloud.')}</p>
          <dl>
            <dt>{t('Interface')}</dt><dd>Tauri 2 · Svelte 5 · xterm.js · CodeMirror</dd>
            <dt>{t('Moteur SSH')}</dt><dd>Rust · russh</dd>
            <dt>{t('Chiffrement')}</dt><dd>ChaCha20-Poly1305 · Argon2id</dd>
            <dt>{t('Dossier de données')}</dt><dd class="mono">{app.dataDir}</dd>
          </dl>
        </div>
      {/if}
    </div>
  </div>
</Modal>

<style>
  .layout { display: flex; min-height: 440px; }
  nav { width: 170px; flex: none; padding: 12px 8px; border-right: 1px solid var(--border); display: flex; flex-direction: column; gap: 2px; }
  nav button { display: flex; align-items: center; gap: 9px; height: 32px; padding: 0 10px; background: none; border: 0; border-radius: 7px; color: var(--muted); font-weight: 500; text-align: left; }
  nav button:hover { background: var(--surface-2); color: var(--text); }
  nav button.on { background: var(--surface-2); color: var(--text); }
  .content { flex: 1; min-width: 0; padding: 18px 24px 22px; overflow: auto; max-height: calc(100vh - 160px); display: flex; flex-direction: column; gap: 4px; }
  h3 { font-family: var(--brand); font-size: 15px; font-weight: 600; margin-bottom: 8px; }
  .opt { display: flex; align-items: center; justify-content: space-between; gap: 20px; padding: 11px 0; border-bottom: 1px solid var(--border); }
  .opt.off { opacity: 0.5; }
  .l { font-weight: 500; }
  .d { font-size: 12px; color: var(--faint); max-width: 340px; margin-top: 1px; }
  .w { width: 140px; flex: none; }
  .range { width: 160px; accent-color: #7f8cf8; }
  .hint { margin-top: 10px; }
  .path { margin-top: 18px; display: flex; flex-direction: column; gap: 4px; }
  .path code { font-size: 12px; color: var(--muted); user-select: text; word-break: break-all; }
  .about { display: flex; flex-direction: column; align-items: center; gap: 6px; padding-top: 10px; text-align: center; }
  .ver { font-family: var(--brand); font-weight: 600; margin-top: 6px; }
  dl { display: grid; grid-template-columns: auto 1fr; gap: 6px 16px; margin: 18px 0 0; text-align: left; font-size: 12.5px; }
  dt { color: var(--faint); }
  dd { margin: 0; color: var(--muted); user-select: text; word-break: break-all; }
</style>
