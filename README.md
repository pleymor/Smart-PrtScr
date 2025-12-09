# Simple PrintScreen

Logiciel de capture d'écran pour Windows avec support multi-écrans et horodatage automatique.

## Fonctionnalités

- 📸 **Capture de l'écran actif** : Appuyez sur `PrtScr` pour capturer l'écran où se trouve votre curseur
- ✂️ **Sélection rectangulaire** : Appuyez sur `Ctrl+PrtScr` pour dessiner une zone personnalisée à capturer
- 🖥️ **Multi-écrans** : Support complet des configurations multi-moniteurs
- 🕐 **Horodatage automatique** : Chaque capture inclut un header avec la date et l'heure
- 💾 **Configuration persistante** : Choisissez votre dossier de destination (sauvegarde automatique)
- 🎯 **Dossier par défaut** : Utilise le dossier "Captures d'écran" de Windows par défaut

## Installation

1. Clonez ou téléchargez ce projet
2. Ouvrez un terminal dans le dossier du projet
3. Installez les dépendances :
   ```bash
   npm install
   ```

## Utilisation

### Méthode 1 : Double-cliquer sur start.bat (Recommandé)
1. Double-cliquez sur le fichier `start.bat` dans le dossier du projet
2. L'application se lancera automatiquement

### Méthode 2 : Ligne de commande
1. Ouvrez **PowerShell** ou **l'Invite de commandes Windows** (PAS Git Bash)
2. Naviguez vers le dossier du projet
3. Exécutez :
   ```cmd
   npm start
   ```

### Après le lancement

1. Une fenêtre de configuration s'ouvrira. Vous pouvez :
   - Voir le dossier de destination actuel
   - Changer le dossier de destination
   - Réinitialiser au dossier par défaut

2. Utilisez les raccourcis clavier globaux :
   - **PrtScr** : Capture l'écran actif (où se trouve le curseur)
   - **Ctrl+PrtScr** : Ouvre une fenêtre de sélection pour capturer une zone spécifique

## ⚠️ Note importante

**Sur Windows, vous DEVEZ utiliser PowerShell, CMD ou le fichier start.bat pour lancer l'application.**
Git Bash/MSYS n'est pas compatible avec Electron sur Windows en raison de conflits de chemins.

## Raccourcis clavier

| Raccourci | Action |
|-----------|--------|
| `PrtScr` | Capture l'écran actif |
| `Ctrl+PrtScr` | Sélection rectangulaire |
| `Échap` | Annuler la sélection (en mode sélection) |

## Configuration

- Le dossier de destination par défaut est : `%USERPROFILE%\Pictures\Screenshots`
- Vous pouvez changer ce dossier via l'interface de configuration
- Votre choix est sauvegardé automatiquement et persistera entre les sessions

## Format des captures

Les captures d'écran sont sauvegardées au format PNG avec :
- Un header contenant l'horodatage (date et heure)
- Nom de fichier : `Screenshot_YYYY-MM-DDTHH-MM-SS.png`

## Technologies utilisées

- **Electron** : Framework d'application desktop
- **screenshot-desktop** : Capture d'écran multi-moniteurs
- **sharp** : Traitement d'images
- **node-global-key-listener** : Écoute des raccourcis clavier globaux
- **electron-store** : Persistance de la configuration

## Licence

ISC
