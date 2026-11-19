# Termomix

Client SSH moderne (Tauri 2 + Rust + Svelte). Local-first : aucune donnée ne quitte votre machine.

## Prérequis (Linux)

```bash
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev librsvg2-dev libayatana-appindicator3-dev libssl-dev pkg-config
```

## Développement

```bash
npm install
npm run tauri dev      # lance l'application
npm run tauri build    # paquets (deb, AppImage…)
```

## Fonctionnalités

- Serveurs, groupes, mot de passe ou clé privée, commande de démarrage, connexion rapide `user@hôte:port`
- Terminal xterm.js (WebGL), onglets, recherche, copier/coller `Ctrl+Maj+C/V`
- Explorateur SFTP : navigation, envoi/téléchargement (dossiers inclus, glisser-déposer), renommer, supprimer
- Macros : enregistrement des commandes tapées ou création manuelle, lancement en un clic, par serveur ou globales
- Export/Import en fichier `.termomix`, chiffré (Argon2id + ChaCha20-Poly1305), sans cloud

## Données

Base chiffrée `termomix.db` + `device.key` dans le dossier de données de l'application
(`~/.local/share/app.termomix.desktop`). Les empreintes d'hôtes sont mémorisées à la première connexion (TOFU) et
la connexion est refusée si elles changent.

Les sources originales des logos sont dans `_src/`.
