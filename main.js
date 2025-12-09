const { app, BrowserWindow, ipcMain, dialog, screen, Tray, Menu, nativeImage } = require('electron');
const path = require('path');
const fs = require('fs');
const screenshot = require('screenshot-desktop');
const sharp = require('sharp');

let Store;
let store;
let gkl;
let GlobalKeyboardListener;

let mainWindow = null;
let selectionWindow = null;
let tray = null;

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

// Générer un nom de fichier avec horodatage
function generateFilename() {
  const now = new Date();
  const timestamp = now.toISOString().replace(/:/g, '-').replace(/\..+/, '');
  return `Screenshot_${timestamp}.png`;
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
function createSelectionWindow() {
  const displays = screen.getAllDisplays();
  const primaryDisplay = screen.getPrimaryDisplay();
  const { width, height } = primaryDisplay.bounds;

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

  selectionWindow.on('closed', () => {
    selectionWindow = null;
  });
}

// Capturer la sélection
async function captureSelection(bounds) {
  try {
    const displays = screen.getAllDisplays();
    const cursorPoint = screen.getCursorScreenPoint();
    const activeDisplay = screen.getDisplayNearestPoint(cursorPoint);
    const displayIndex = displays.findIndex(d => d.id === activeDisplay.id);

    const imgBuffer = await screenshot({ screen: displayIndex });

    const croppedImage = await sharp(imgBuffer)
      .extract({
        left: Math.round(bounds.x),
        top: Math.round(bounds.y),
        width: Math.round(bounds.width),
        height: Math.round(bounds.height)
      })
      .toBuffer();

    const finalImage = await addTimestampToImage(croppedImage);

    const savePath = getSavePath();
    const filename = generateFilename();
    const fullPath = path.join(savePath, filename);

    await sharp(finalImage).toFile(fullPath);

    console.log(`Screenshot saved: ${fullPath}`);

    // Notification via tray
    if (tray) {
      tray.displayBalloon({
        title: 'Capture réussie',
        content: `Sauvegardée : ${filename}`
      });
    }

    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('screenshot-saved', fullPath);
    }
  } catch (error) {
    console.error('Error capturing selection:', error);
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
    width: 500,
    height: 400,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    },
    icon: path.join(__dirname, 'icon.ico')
  });

  mainWindow.loadFile('index.html');

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
      label: 'Capturer (sélection)',
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

  ipcMain.on('selection-complete', (event, bounds) => {
    if (selectionWindow) {
      selectionWindow.close();
    }

    if (bounds && bounds.width > 0 && bounds.height > 0) {
      captureSelection(bounds);
    }
  });

  ipcMain.on('selection-cancel', () => {
    if (selectionWindow) {
      selectionWindow.close();
    }
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
