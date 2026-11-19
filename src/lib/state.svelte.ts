import { lang } from './i18n.svelte';
import { api, defaultSettings, errMsg, type AppState, type Group, type Macro, type Server, type Settings } from './api';

export interface Tab {
  id: string;
  /** Serveur enregistré ou connexion rapide. */
  server: Server;
  adhoc: boolean;
  title: string;
  /** Session SSH ouverte (renseigné par le terminal). */
  connected?: boolean;
}

export const app = $state({
  ready: false,
  groups: [] as Group[],
  servers: [] as Server[],
  macros: [] as Macro[],
  settings: { ...defaultSettings } as Settings,
  dataDir: '',
  version: '',
  view: 'hosts' as 'hosts' | 'macros',
  settingsOpen: false,
  settingsTab: 'general' as 'general' | 'terminal' | 'connection' | 'data' | 'about',
  tabs: [] as Tab[],
  active: null as string | null,
  toasts: [] as { id: number; kind: 'info' | 'error' | 'ok'; text: string }[],
});

export function apply(s: AppState) {
  app.groups = s.groups;
  app.servers = s.servers;
  app.macros = s.macros;
  app.settings = { ...defaultSettings, ...s.settings };
  lang.value = app.settings.language;
  app.dataDir = s.data_dir;
  app.version = s.version;
}

export async function reload() {
  apply(await api.getState());
}

let toastId = 0;
export function toast(text: string, kind: 'info' | 'error' | 'ok' = 'info') {
  const id = ++toastId;
  app.toasts.push({ id, kind, text });
  setTimeout(() => (app.toasts = app.toasts.filter((t) => t.id !== id)), kind === 'error' ? 6000 : 3000);
}

export async function attempt<T>(fn: () => Promise<T>): Promise<T | undefined> {
  try {
    return await fn();
  } catch (e) {
    toast(errMsg(e), 'error');
  }
}

let tabSeq = 0;
export function openTab(server: Server, adhoc = false) {
  // Plusieurs sessions simultanées sur le même serveur sont autorisées.
  const n = app.tabs.filter((t) => t.server.id === server.id && !adhoc).length;
  const tab: Tab = {
    id: `tab-${++tabSeq}`,
    server,
    adhoc,
    title: n ? `${server.name} (${n + 1})` : server.name,
  };
  app.tabs.push(tab);
  app.active = tab.id;
}

export function closeTab(id: string) {
  const i = app.tabs.findIndex((t) => t.id === id);
  if (i < 0) return;
  app.tabs.splice(i, 1);
  if (app.active === id) app.active = app.tabs[Math.min(i, app.tabs.length - 1)]?.id ?? null;
}

export function macrosFor(serverId: string) {
  return app.macros.filter((m) => m.server_ids.length === 0 || m.server_ids.includes(serverId));
}

// ---------- Boîtes de dialogue (remplacent confirm/prompt natifs) ----------

export interface DialogReq {
  kind: 'confirm' | 'prompt';
  title: string;
  message?: string;
  value?: string;
  danger?: boolean;
  resolve: (v: any) => void;
}
export const dlg = $state({ current: null as DialogReq | null });

export const confirmDlg = (title: string, message?: string, danger = true) =>
  new Promise<boolean>((resolve) => (dlg.current = { kind: 'confirm', title, message, danger, resolve }));

export const promptDlg = (title: string, message = '', value = '') =>
  new Promise<string | null>((resolve) => (dlg.current = { kind: 'prompt', title, message, value, resolve }));

// ---------- Éditeur de macro (ouvert depuis la page Macros ou l'enregistrement) ----------

export const macroEditor = $state({ open: false, macro: null as Partial<import('./api').Macro> | null });
export function editMacro(m: Partial<import('./api').Macro> | null = null) {
  macroEditor.macro = m;
  macroEditor.open = true;
}

// ---------- Transferts SFTP ----------

export const transfers = $state({ list: [] as import('./api').Progress[] });
export function updateTransfer(p: import('./api').Progress) {
  const i = transfers.list.findIndex((t) => t.id === p.id);
  if (i >= 0) transfers.list[i] = p;
  else transfers.list.push(p);
  if (p.state === 'done') setTimeout(() => (transfers.list = transfers.list.filter((t) => t.id !== p.id)), 3500);
}

export async function updateSettings(patch: Partial<Settings>) {
  app.settings = { ...app.settings, ...patch };
  lang.value = app.settings.language;
  await attempt(() => api.saveSettings($state.snapshot(app.settings) as Settings));
}
