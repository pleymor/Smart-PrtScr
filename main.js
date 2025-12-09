const { app, BrowserWindow, ipcMain, dialog, screen, Tray, Menu, nativeImage, desktopCapturer } = require('electron');
const path = require('path');
const fs = require('fs');
const sharp = require('sharp');

let Store;
let store;
let gkl;
let GlobalKeyboardListener;

let mainWindow = null;
let selectionWindow = null;
let filenameDialog = null;
let tray = null;
let currentScreenshotBuffer = null; // Stocker la capture actuelle
let pendingScreenshot = null; // Image en attente de nom de fichier

// Obtenir le dossier par défaut (dossier Captures d'écran de Windows)
function getDefaultScreenshotPath() {
  const userProfile = process.env.USERPROFILE;
  const screenshotsPath = path.join(userProfile, 'Pictures', 'Screenshots');

  // Créer le dossier s'il n'existe pas
  if (!fs.existsSync(screenshotsPath)) {
    fs.mkdirSync(screenshotsPath, { recursive: true });
  }

  return screenshotsPath;
}

// Obtenir le chemin de sauvegarde configuré
function getSavePath() {
  return store.get('screenshotPath', getDefaultScreenshotPath());
}

// Générer un nom de fichier avec horodatage (sans extension)
function generateDefaultFilename() {
  const now = new Date();
  const timestamp = now.toISOString().replace(/:/g, '-').replace(/\..+/, '');
  return `Screenshot_${timestamp}`;
}

// Ajouter l'horodatage dans le header de l'image
async function addTimestampToImage(imageBuffer) {
  const now = new Date();
  const timestamp = now.toLocaleString('fr-FR');

  const metadata = await sharp(imageBuffer).metadata();
  const headerHeight = 30;

  const header = Buffer.from(
    `<svg width="${metadata.width}" height="${headerHeight}">
      <rect width="${metadata.width}" height="${headerHeight}" fill="#333333"/>
      <text x="10" y="20" font-family="Arial" font-size="14" fill="white">${timestamp}</text>
    </svg>`
  );

  const headerImage = await sharp(header).png().toBuffer();

  const finalImage = await sharp({
    create: {
      width: metadata.width,
      height: metadata.height + headerHeight,
      channels: 4,
      background: { r: 255, g: 255, b: 255, alpha: 1 }
    }
  })
  .composite([
    { input: headerImage, top: 0, left: 0 },
    { input: imageBuffer, top: headerHeight, left: 0 }
  ])
  .png()
  .toBuffer();

  return finalImage;
}

// Créer la fenêtre de sélection
async function createSelectionWindow() {
  try {
    // Capturer l'écran AVANT d'afficher l'overlay avec desktopCapturer
    const primaryDisplay = screen.getPrimaryDisplay();
    const { width, height } = primaryDisplay.size;
    const scaleFactor = primaryDisplay.scaleFactor;

    // Obtenir les sources d'écran
    const sources = await desktopCapturer.getSources({
      types: ['screen'],
      thumbnailSize: { width: width * scaleFactor, height: height * scaleFactor }
    });

    if (sources.length === 0) {
      console.error('No screen sources found');
      return;
    }

    // Utiliser le premier écran (ou l'écran où se trouve le curseur)
    const cursorPoint = screen.getCursorScreenPoint();
    const activeDisplay = screen.getDisplayNearestPoint(cursorPoint);

    // Trouver la source correspondante
    let source = sources[0];
    for (const s of sources) {
      if (s.display_id === String(activeDisplay.id)) {
        source = s;
        break;
      }
    }

    // Convertir le thumbnail NativeImage en PNG buffer
    const imgBuffer = source.thumbnail.toPNG();

    // Stocker la capture pour la réutiliser lors de la sélection
    currentScreenshotBuffer = imgBuffer;

    // Convertir l'image en base64
    const base64Image = imgBuffer.toString('base64');

    selectionWindow = new BrowserWindow({
      fullscreen: true,
      frame: false,
      transparent: true,
      alwaysOnTop: true,
      skipTaskbar: true,
      webPreferences: {
        nodeIntegration: true,
        contextIsolation: false
      }
    });

    selectionWindow.loadFile('selection.html');
    selectionWindow.setIgnoreMouseEvents(false);

    // Envoyer la capture à la fenêtre
    selectionWindow.webContents.on('did-finish-load', () => {
      selectionWindow.webContents.send('screenshot-data', base64Image);
    });

    selectionWindow.on('closed', () => {
      selectionWindow = null;
      currentScreenshotBuffer = null; // Nettoyer la mémoire
    });
  } catch (error) {
    console.error('Error creating selection window:', error);
  }
}

// Capturer la sélection et ouvrir le dialogue de nom
async function captureSelection(bounds) {
  try {
    // Utiliser la capture déjà prise (sans l'overlay)
    if (!currentScreenshotBuffer) {
      console.error('No screenshot buffer available');
      return;
    }

    const croppedImage = await sharp(currentScreenshotBuffer)
      .extract({
        left: Math.round(bounds.x),
        top: Math.round(bounds.y),
        width: Math.round(bounds.width),
        height: Math.round(bounds.height)
      })
      .toBuffer();

    const finalImage = await addTimestampToImage(croppedImage);

    // Stocker l'image en attente et ouvrir le dialogue
    pendingScreenshot = {
      image: finalImage,
      defaultFilename: generateDefaultFilename()
    };

    openFilenameDialog();
  } catch (error) {
    console.error('Error capturing selection:', error);
  }
}

// Ouvrir le dialogue pour le nom de fichier
function openFilenameDialog() {
  if (filenameDialog) {
    filenameDialog.focus();
    return;
  }

  filenameDialog = new BrowserWindow({
    width: 450,
    height: 180,
    frame: false,
    resizable: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    }
  });

  filenameDialog.loadFile('filename-dialog.html');

  filenameDialog.webContents.on('did-finish-load', () => {
    if (pendingScreenshot) {
      filenameDialog.webContents.send('default-filename', pendingScreenshot.defaultFilename);
    }
  });

  filenameDialog.on('closed', () => {
    filenameDialog = null;
  });
}

// Sauvegarder la capture avec le nom choisi
async function saveScreenshotWithName(filename) {
  if (!pendingScreenshot) {
    console.error('No pending screenshot to save');
    return;
  }

  try {
    const savePath = getSavePath();
    const fullFilename = `${filename}.png`;
    const fullPath = path.join(savePath, fullFilename);

    // Vérifier que le dossier existe
    if (!fs.existsSync(savePath)) {
      fs.mkdirSync(savePath, { recursive: true });
    }

    await sharp(pendingScreenshot.image).toFile(fullPath);

    console.log(`Screenshot saved: ${fullPath}`);

    // Notification via tray
    if (tray) {
      tray.displayBalloon({
        title: 'Capture réussie',
        content: `Sauvegardée : ${fullFilename}`
      });
    }

    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('screenshot-saved', fullPath);
    }
  } catch (error) {
    console.error('Error saving screenshot:', error);
  } finally {
    pendingScreenshot = null;
  }
}

// Créer la fenêtre principale
function createMainWindow() {
  if (mainWindow) {
    mainWindow.show();
    mainWindow.focus();
    return;
  }

  mainWindow = new BrowserWindow({
    width: 600,
    height: 380,
    frame: false,
    resizable: false,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    },
    icon: path.join(__dirname, 'icon.ico')
  });

  mainWindow.loadFile('index.html');
  mainWindow.setMenuBarVisibility(false);

  // Cacher au lieu de fermer
  mainWindow.on('close', (event) => {
    if (!app.isQuitting) {
      event.preventDefault();
      mainWindow.hide();
    }
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

// Créer le system tray
function createTray() {
  // Créer une icône simple en PNG
  const iconData = Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAAsTAAALEwEAmpwYAAAAnElEQVR4nGNgGAWMDAwMDP///2dgYPjPwMDwn4GB4T8DQ8N/BgaG/wwMDP8ZGBj+MzAw/GdgYPjPwMDwn4GB4T8DA8N/BgaG/wwMDP8ZGBj+MzAw/GdgYPjPwMDwn4GB4T8DA8N/BgaG/wwMDP8ZGBj+MzAw/GdgYPjPwMDwn4GB4T8DA8N/BgaG/wwMDP8ZGBj+MzAw/GdgYPgPAAwMDAwAHhEI3KFxPLsAAAAASUVORK5CYII=',
    'base64'
  );

  const trayIcon = nativeImage.createFromBuffer(iconData);

  tray = new Tray(trayIcon);

  const contextMenu = Menu.buildFromTemplate([
    {
      label: 'Simple PrintScreen',
      enabled: false
    },
    { type: 'separator' },
    {
      label: 'Ouvrir',
      click: () => {
        createMainWindow();
      }
    },
    {
      label: 'Capturer (PrtScr)',
      click: () => {
        createSelectionWindow();
      }
    },
    { type: 'separator' },
    {
      label: 'Dossier de sauvegarde',
      click: () => {
        const savePath = getSavePath();
        require('electron').shell.openPath(savePath);
      }
    },
    { type: 'separator' },
    {
      label: 'Démarrage automatique',
      type: 'checkbox',
      checked: app.getLoginItemSettings().openAtLogin,
      click: (item) => {
        app.setLoginItemSettings({
          openAtLogin: item.checked,
          openAsHidden: item.checked
        });
      }
    },
    { type: 'separator' },
    {
      label: 'Quitter',
      click: () => {
        app.isQuitting = true;
        app.quit();
      }
    }
  ]);

  tray.setToolTip('Simple PrintScreen - Prêt');
  tray.setContextMenu(contextMenu);

  // Double-clic pour ouvrir la fenêtre
  tray.on('double-click', () => {
    createMainWindow();
  });
}

// Fonction pour initialiser les gestionnaires IPC
function setupIpcHandlers() {
  ipcMain.handle('get-save-path', () => {
    return getSavePath();
  });

  ipcMain.handle('set-save-path', async () => {
    const result = await dialog.showOpenDialog(mainWindow, {
      properties: ['openDirectory']
    });

    if (!result.canceled && result.filePaths.length > 0) {
      const newPath = result.filePaths[0];
      store.set('screenshotPath', newPath);
      return newPath;
    }

    return null;
  });

  ipcMain.handle('reset-save-path', () => {
    const defaultPath = getDefaultScreenshotPath();
    store.set('screenshotPath', defaultPath);
    return defaultPath;
  });

  ipcMain.handle('get-auto-start', () => {
    return app.getLoginItemSettings().openAtLogin;
  });

  ipcMain.handle('set-auto-start', (event, enabled) => {
    app.setLoginItemSettings({
      openAtLogin: enabled,
      openAsHidden: enabled
    });
    return enabled;
  });

  ipcMain.on('selection-complete', async (event, bounds) => {
    // Capturer d'abord, AVANT de fermer la fenêtre (qui efface le buffer)
    if (bounds && bounds.width > 0 && bounds.height > 0) {
      await captureSelection(bounds);
    }

    if (selectionWindow) {
      selectionWindow.close();
    }
  });

  ipcMain.on('selection-cancel', () => {
    if (selectionWindow) {
      selectionWindow.close();
    }
  });

  ipcMain.on('close-window', () => {
    if (mainWindow) {
      mainWindow.close();
    }
  });

  // Gestionnaires pour le dialogue de nom de fichier
  ipcMain.on('filename-submit', async (event, filename) => {
    if (filenameDialog) {
      filenameDialog.close();
    }
    await saveScreenshotWithName(filename);
  });

  ipcMain.on('filename-cancel', () => {
    if (filenameDialog) {
      filenameDialog.close();
    }
    pendingScreenshot = null;
  });
}

// Fonction pour initialiser l'écouteur de touches globales
function setupKeyListener() {
  try {
    GlobalKeyboardListener = require('node-global-key-listener').GlobalKeyboardListener;
    gkl = new GlobalKeyboardListener();

    gkl.addListener((e) => {
      if (e.name === 'PRINT SCREEN' && e.state === 'DOWN') {
        console.log('PrtScr pressed - Opening selection window');
        createSelectionWindow();
      }
    });

    console.log('Global keyboard listener initialized successfully');
  } catch (error) {
    console.error('Error initializing keyboard listener:', error);
    console.log('You may need to run the application as Administrator for global key capture.');
  }
}

// Application lifecycle
app.whenReady().then(async () => {
  // Initialiser electron-store
  Store = (await import('electron-store')).default;
  store = new Store();

  // Créer le system tray
  createTray();

  // Configurer les gestionnaires IPC
  setupIpcHandlers();

  // Configurer l'écouteur de touches
  setupKeyListener();

  // Créer la fenêtre principale (masquée si démarrage auto)
  const openAtLogin = app.getLoginItemSettings().openAtLogin;
  if (!openAtLogin || !app.getLoginItemSettings().wasOpenedAsHidden) {
    createMainWindow();
  }

  app.on('activate', () => {
    createMainWindow();
  });
});

// Ne pas quitter quand toutes les fenêtres sont fermées
app.on('window-all-closed', (e) => {
  e.preventDefault();
});

app.on('before-quit', () => {
  app.isQuitting = true;
});

app.on('will-quit', () => {
  if (gkl && gkl.kill) {
    gkl.kill();
  }
});
