import { Channel, invoke } from '@tauri-apps/api/core';

export type AuthKind = 'password' | 'key';

export interface Server {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  group_id: string | null;
  auth: AuthKind;
  /** Envoyé uniquement à l'enregistrement. undefined = conserver, '' = effacer. */
  password?: string;
  has_password?: boolean;
  key_path: string | null;
  key_passphrase?: string;
  has_passphrase?: boolean;
  startup_command: string | null;
  notes: string | null;
  last_connected: number | null;
}

export interface Group {
  id: string;
  name: string;
}

export interface Macro {
  id: string;
  name: string;
  description: string | null;
  commands: string[];
  delay_ms: number;
  server_ids: string[];
}

export interface Settings {
  font_size: number;
  scrollback: number;
  cursor_style: 'bar' | 'block' | 'underline';
}

export interface AppState {
  groups: Group[];
  servers: Server[];
  macros: Macro[];
  settings: Settings;
  data_dir: string;
  version: string;
}

export interface Entry {
  name: string;
  is_dir: boolean;
  is_symlink: boolean;
  size: number;
  mtime: number | null;
  mode: number | null;
}

export interface Listing {
  path: string;
  entries: Entry[];
}

export interface Progress {
  id: string;
  name: string;
  done: number;
  total: number;
  state: 'running' | 'done' | 'error';
  error: string | null;
}

export interface Connected {
  session_id: string;
  fingerprint: string;
  new_host: boolean;
}

export const blankServer = (): Server => ({
  id: '',
  name: '',
  host: '',
  port: 22,
  username: '',
  group_id: null,
  auth: 'password',
  key_path: null,
  startup_command: null,
  notes: null,
  last_connected: null,
});

export const api = {
  getState: () => invoke<AppState>('get_state'),
  saveServer: (server: Server) => invoke<Server>('save_server', { server }),
  deleteServer: (id: string) => invoke<void>('delete_server', { id }),
  saveGroup: (group: Group) => invoke<Group>('save_group', { group }),
  deleteGroup: (id: string) => invoke<void>('delete_group', { id }),
  saveMacro: (m: Macro) => invoke<Macro>('save_macro', { m }),
  deleteMacro: (id: string) => invoke<void>('delete_macro', { id }),
  saveSettings: (settings: Settings) => invoke<void>('save_settings', { settings }),
  forgetHostKey: (host: string, port: number) => invoke<void>('forget_host_key', { host, port }),

  sshConnect: (
    req: { server_id?: string; adhoc?: Server; secret?: string; cols: number; rows: number },
    onData: (bytes: Uint8Array) => void,
  ) => {
    const ch = new Channel<ArrayBuffer>();
    ch.onmessage = (buf) => onData(new Uint8Array(buf));
    return invoke<Connected>('ssh_connect', { req, onData: ch });
  },
  sshWrite: (id: string, data: string) => invoke<void>('ssh_write', { id, data }),
  sshResize: (id: string, cols: number, rows: number) => invoke<void>('ssh_resize', { id, cols, rows }),
  sshClose: (id: string) => invoke<void>('ssh_close', { id }),

  sftpList: (id: string, path: string) => invoke<Listing>('sftp_list', { id, path }),
  sftpMkdir: (id: string, path: string) => invoke<void>('sftp_mkdir', { id, path }),
  sftpRename: (id: string, from: string, to: string) => invoke<void>('sftp_rename', { id, from, to }),
  sftpRemove: (id: string, path: string, isDir: boolean) => invoke<void>('sftp_remove', { id, path, isDir }),
  sftpUpload: (id: string, local: string, remoteDir: string) => invoke<void>('sftp_upload', { id, local, remoteDir }),
  sftpDownload: (id: string, remote: string, isDir: boolean, localDir: string) =>
    invoke<void>('sftp_download', { id, remote, isDir, localDir }),

  exportData: (path: string, passphrase: string | null, includeSecrets: boolean) =>
    invoke<void>('export_data', { path, passphrase, includeSecrets }),
  importData: (path: string, passphrase: string | null, replace: boolean) =>
    invoke<{ servers: number; macros: number }>('import_data', { path, passphrase, replace }),
};

export const errMsg = (e: unknown) => (typeof e === 'string' ? e : e instanceof Error ? e.message : String(e));
