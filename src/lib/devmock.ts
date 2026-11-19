// Uniquement en développement dans un navigateur simple (hors Tauri) : données factices pour travailler l'interface.
import { mockIPC } from '@tauri-apps/api/mocks';

const now = Math.floor(Date.now() / 1000);
const srv = (id: string, name: string, host: string, g: string | null, k = false, last: number | null = null) => ({
  id, name, host, port: 22, username: 'root', group_id: g, auth: k ? 'key' : 'password', has_password: !k,
  key_path: null, startup_command: null, notes: null, last_connected: last,
});

export function installMock() {
  mockIPC((cmd) => {
    if (cmd === 'get_state')
      return {
        groups: [{ id: 'g1', name: 'Production' }, { id: 'g2', name: 'Maison' }],
        servers: [
          srv('1', 'web-01', '51.15.20.11', 'g1', true, now - 300),
          srv('2', 'db-01', '51.15.20.12', 'g1', true, now - 86400),
          srv('3', 'NAS', '192.168.1.20', 'g2', false, now - 7200),
          srv('4', 'Raspberry Pi', '192.168.1.42', 'g2'),
          srv('5', 'vps-test', 'test.exemple.fr', null),
        ],
        macros: [{ id: 'm1', name: 'Mise à jour', description: 'apt', commands: ['sudo apt update', 'sudo apt upgrade -y'], delay_ms: 150, server_ids: [] }],
        settings: { font_size: 14, scrollback: 10000, cursor_style: 'bar' },
        data_dir: '/tmp', version: '0.1.0',
      };
  });
}
