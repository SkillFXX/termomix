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
- Éditeur de fichiers distants intégré (coloration syntaxique, Ctrl+S, ouverture par double-clic dans l'explorateur)
- Explorateur SFTP : navigation, envoi/téléchargement (dossiers inclus, glisser-déposer), renommer, supprimer
- Macros : enregistrement des commandes tapées ou création manuelle, lancement en un clic, par serveur ou globales
- Réglages : langue (français / anglais), terminal, reconnexion automatique, signal de maintien
- Export/Import en fichier `.termomix`, chiffré (Argon2id + ChaCha20-Poly1305), sans cloud

## Données

Base chiffrée `termomix.db` + `device.key` dans le dossier de données de l'application
(`~/.local/share/app.termomix.desktop`). Les empreintes d'hôtes sont mémorisées à la première connexion (TOFU) et
la connexion est refusée si elles changent.

L'icône source (1024 px) est dans `branding/` ; les icônes de l'application se régénèrent avec `npx tauri icon branding/icon-1024.png`.

## Licence

Le code est publié sous licence [MIT](LICENSE).

Le logo et l'identité visuelle de Termomix (`public/`, `branding/`, `docs/logo.png`, icônes de l'application) sont exclus de cette licence : tous droits réservés. Les logos de distributions affichés dans l'application appartiennent à leurs propriétaires respectifs. Les polices (Inter, Sora, JetBrains Mono) sont distribuées sous licence SIL OFL.
