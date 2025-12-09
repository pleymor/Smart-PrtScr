# Simple PrintScreen

Logiciel de capture d'écran pour Windows avec support multi-écrans, horodatage automatique et icône système.

## ✨ Fonctionnalités

- 📸 **Capture de l'écran actif** : Appuyez sur `PrtScr` pour capturer l'écran où se trouve votre curseur
- ✂️ **Sélection rectangulaire** : Appuyez sur `Ctrl+PrtScr` pour dessiner une zone personnalisée à capturer
- 🖥️ **Multi-écrans** : Support complet des configurations multi-moniteurs
- 🕐 **Horodatage automatique** : Chaque capture inclut un header avec la date et l'heure
- 💾 **Configuration persistante** : Choisissez votre dossier de destination (sauvegarde automatique)
- 🎯 **Dossier par défaut** : Utilise le dossier "Captures d'écran" de Windows par défaut
- 🔔 **Icône système (System Tray)** : L'application se minimise dans la barre des tâches
- 🚀 **Démarrage automatique** : Option pour lancer l'application au démarrage de Windows
- 🔕 **Mode silencieux** : Fonctionne en arrière-plan sans fenêtre intrusive

## 📥 Installation

### Option 1 : Utiliser l'installateur (Recommandé pour utilisateurs finaux)

1. Téléchargez le fichier `Simple PrintScreen Setup.exe` depuis les releases
2. Exécutez l'installateur
3. Suivez les instructions à l'écran
4. L'application se lancera automatiquement après l'installation

### Option 2 : Version portable (Sans installation)

1. Téléchargez le fichier `Simple PrintScreen Portable.exe` depuis les releases
2. Placez-le dans un dossier de votre choix
3. Double-cliquez pour lancer

### Option 3 : Développement (Pour les développeurs)

1. Clonez ou téléchargez ce projet
2. Ouvrez **PowerShell** ou **CMD** dans le dossier du projet
3. Installez les dépendances :
   ```bash
   npm install
   ```
4. Lancez l'application :
   ```bash
   npm start
   ```

## 🎯 Utilisation

### Première utilisation

1. Après l'installation, l'application démarre automatiquement
2. Une fenêtre de configuration s'affiche
3. Configurez vos préférences :
   - **Dossier de sauvegarde** : Choisissez où sauvegarder vos captures
   - **Démarrage automatique** : Cochez pour lancer au démarrage de Windows
4. L'application se minimise dans la barre des tâches (icône près de l'horloge)

### Utilisation quotidienne

L'application fonctionne en arrière-plan. Utilisez simplement les raccourcis clavier :

- **PrtScr** : Capture l'écran actif (où se trouve le curseur)
- **Ctrl+PrtScr** : Ouvre une fenêtre de sélection pour capturer une zone spécifique
- **Échap** : Annuler la sélection (en mode sélection)

### Menu de l'icône système

Clic droit sur l'icône dans la barre des tâches pour :
- Ouvrir la fenêtre de configuration
- Capturer l'écran directement
- Ouvrir le mode sélection
- Ouvrir le dossier de sauvegarde
- Activer/désactiver le démarrage automatique
- Quitter l'application

## ⚙️ Configuration

### Dossier de destination
- **Par défaut** : `%USERPROFILE%\Pictures\Screenshots`
- Modifiable via l'interface de configuration
- Les modifications sont sauvegardées automatiquement

### Démarrage automatique
- Activez cette option pour que l'application démarre avec Windows
- L'application se lance en mode masqué (icône système uniquement)
- Accessible via l'interface ou le menu de l'icône système

## 📸 Format des captures

Les captures d'écran sont sauvegardées au format PNG avec :
- Un header contenant l'horodatage (date et heure en français)
- Nom de fichier : `Screenshot_YYYY-MM-DDTHH-MM-SS.png`
- Qualité PNG maximale

## 🛠️ Construction de l'installateur

Pour les développeurs souhaitant créer l'installateur :

```bash
# Installer les dépendances
npm install

# Créer l'installateur Windows
npm run build

# Créer la version portable
npm run build:portable

# Créer les deux versions
npm run dist
```

Les fichiers seront générés dans le dossier `dist/`.

## 📋 Raccourcis clavier

| Raccourci | Action |
|-----------|--------|
| `PrtScr` | Capture l'écran actif |
| `Ctrl+PrtScr` | Sélection rectangulaire |
| `Échap` | Annuler la sélection |

## ⚠️ Notes importantes

- **PowerShell/CMD requis** : Pour le développement, utilisez PowerShell ou CMD (Git Bash n'est pas compatible avec Electron sur Windows)
- **Droits administrateur** : Peuvent être nécessaires pour la capture de touches globales sur certains systèmes
- **Windows 10/11** : L'application est optimisée pour Windows 10 et 11

## 💻 Technologies utilisées

- **Electron** : Framework d'application desktop
- **screenshot-desktop** : Capture d'écran multi-moniteurs
- **sharp** : Traitement d'images
- **node-global-key-listener** : Écoute des raccourcis clavier globaux
- **electron-store** : Persistance de la configuration
- **electron-builder** : Création de l'installateur Windows

## 📦 Distribution via le Windows Store

L'application est en cours de soumission au Microsoft Store. En attendant, utilisez les installateurs disponibles dans les releases GitHub.

## 🐛 Problèmes connus

- Sur certains systèmes, les droits administrateur peuvent être nécessaires pour capturer les touches globales
- L'icône système peut ne pas s'afficher correctement si Windows Explorer est redémarré (relancer l'application)

## 📝 Licence

ISC

## 🤝 Contribution

Les contributions sont les bienvenues ! N'hésitez pas à ouvrir une issue ou une pull request.

## 📧 Support

Pour toute question ou problème, ouvrez une issue sur GitHub.
