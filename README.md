# Smart PrtScr

Logiciel de capture d'écran pour Windows avec support multi-écrans, horodatage automatique et icône système.

## Fonctionnalités

- **Sélection rectangulaire** : Dessinez une zone personnalisée à capturer
- **Capture plein écran** : Appuyez sur `PrtScr` pour capturer l'écran entier
- **Multi-écrans** : Support complet des configurations multi-moniteurs
- **Horodatage automatique** : Chaque capture inclut un bandeau avec la date et l'heure
- **Configuration persistante** : Choisissez votre dossier de destination (sauvegarde automatique)
- **Dossier par défaut** : Utilise le dossier "Captures d'écran" de Windows par défaut
- **Icône système (System Tray)** : L'application se minimise dans la barre des tâches
- **Démarrage automatique** : Option pour lancer l'application au démarrage de Windows
- **Mode silencieux** : Fonctionne en arrière-plan sans fenêtre intrusive
- **Format configurable** : PNG ou JPEG

## Installation

### Option 1 : Utiliser l'installateur (Recommandé)

1. Téléchargez le fichier `Smart PrtScr Setup.exe` depuis les releases
2. Exécutez l'installateur
3. Suivez les instructions à l'écran
4. L'application se lancera automatiquement après l'installation

### Option 2 : Version portable (Sans installation)

1. Téléchargez le fichier `Smart PrtScr Portable.exe` depuis les releases
2. Placez-le dans un dossier de votre choix
3. Double-cliquez pour lancer

### Option 3 : Développement

```bash
# Installer les dépendances
npm install

# Lancer en mode développement
npm run tauri:dev

# Construire pour production
npm run tauri:build
```

## Utilisation

### Première utilisation

1. Après l'installation, l'application démarre automatiquement
2. Une icône apparaît dans la barre des tâches (près de l'horloge)
3. Clic gauche sur l'icône pour ouvrir les paramètres
4. Clic droit sur l'icône pour accéder au menu rapide

### Raccourcis clavier

| Raccourci | Action |
|-----------|--------|
| `PrtScr` | Ouvre la fenêtre de sélection |
| `Entrée` | Capturer tout l'écran |
| `Échap` | Annuler la capture |

### Menu de l'icône système

Clic droit sur l'icône dans la barre des tâches pour :
- Ouvrir la fenêtre de configuration
- Capturer avec sélection
- Ouvrir le dossier de sauvegarde
- Quitter l'application

## Configuration

### Dossier de destination
- **Par défaut** : `%USERPROFILE%\Pictures\Screenshots`
- Modifiable via l'interface de configuration
- Les modifications sont sauvegardées automatiquement

### Format d'image
- **PNG** : Qualité maximale, fichiers plus volumineux
- **JPEG** : Fichiers plus légers

### Bandeau d'horodatage
- Position configurable (haut/bas)
- Alignement configurable (gauche/centre/droite)
- Couleur de fond et de texte personnalisables
- Mode overlay (texte directement sur l'image)

## Format des captures

Les captures d'écran sont sauvegardées avec :
- Un bandeau contenant l'horodatage (date et heure)
- Nom de fichier : `YYYY-MM-DDTHH-MM-SS_capture.png` ou `.jpg`

## Technologies utilisées

- **Tauri v2** : Framework d'application desktop léger et sécurisé
- **Rust** : Backend performant pour la capture et le traitement d'images
- **screenshots** : Capture d'écran multi-moniteurs
- **image/imageproc** : Traitement d'images
- **ab_glyph** : Rendu de texte pour l'horodatage

## Structure du projet

```
simple-printscreen/
├── src-tauri/           # Backend Rust (Tauri)
│   ├── src/lib.rs       # Logique principale
│   ├── Cargo.toml       # Dépendances Rust
│   └── tauri.conf.json  # Configuration Tauri
├── src/                 # Frontend (HTML/JS)
│   ├── index.html       # Interface principale
│   ├── selection.html   # Interface de sélection
│   └── filename-dialog.html  # Dialogue de sauvegarde
└── package.json         # Configuration npm
```

## Notes importantes

- **Windows 10/11** : L'application est optimisée pour Windows 10 et 11

## Licence

ISC

## Contribution

Les contributions sont les bienvenues ! N'hésitez pas à ouvrir une issue ou une pull request.
