const { app, BrowserWindow, ipcMain, dialog, screen } = require('electron');
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
let isCtrlPressed = false;

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

// Ajouter l'horodatage dans le header de l'image (métadonnées EXIF)
async function addTimestampToImage(imageBuffer) {
  const now = new Date();
  const timestamp = now.toLocaleString('fr-FR');

  // Ajouter une bande en haut avec l'horodatage
  const metadata = await sharp(imageBuffer).metadata();
  const headerHeight = 30;

  // Créer un header avec le timestamp
  const header = Buffer.from(
    `<svg width="${metadata.width}" height="${headerHeight}">
      <rect width="${metadata.width}" height="${headerHeight}" fill="#333333"/>
      <text x="10" y="20" font-family="Arial" font-size="14" fill="white">${timestamp}</text>
    </svg>`
  );

  const headerImage = await sharp(header).png().toBuffer();

  // Combiner le header et l'image
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

// Capturer l'écran actif
async function captureActiveScreen() {
  try {
    const displays = screen.getAllDisplays();
    const cursorPoint = screen.getCursorScreenPoint();

    // Trouver l'écran où se trouve le curseur
    const activeDisplay = screen.getDisplayNearestPoint(cursorPoint);
    const displayIndex = displays.findIndex(d => d.id === activeDisplay.id);

    // Capturer l'écran actif
    const imgBuffer = await screenshot({ screen: displayIndex });

    // Ajouter l'horodatage
    const finalImage = await addTimestampToImage(imgBuffer);

    // Sauvegarder
    const savePath = getSavePath();
    const filename = generateFilename();
    const fullPath = path.join(savePath, filename);

    await sharp(finalImage).toFile(fullPath);

    console.log(`Screenshot saved: ${fullPath}`);

    // Notification
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('screenshot-saved', fullPath);
    }
  } catch (error) {
    console.error('Error capturing screen:', error);
  }
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

    // Capturer l'écran complet
    const imgBuffer = await screenshot({ screen: displayIndex });

    // Extraire la zone sélectionnée
    const croppedImage = await sharp(imgBuffer)
      .extract({
        left: Math.round(bounds.x),
        top: Math.round(bounds.y),
        width: Math.round(bounds.width),
        height: Math.round(bounds.height)
      })
      .toBuffer();

    // Ajouter l'horodatage
    const finalImage = await addTimestampToImage(croppedImage);

    // Sauvegarder
    const savePath = getSavePath();
    const filename = generateFilename();
    const fullPath = path.join(savePath, filename);

    await sharp(finalImage).toFile(fullPath);

    console.log(`Screenshot saved: ${fullPath}`);

    // Notification
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send('screenshot-saved', fullPath);
    }
  } catch (error) {
    console.error('Error capturing selection:', error);
  }
}

// Créer la fenêtre principale
function createMainWindow() {
  mainWindow = new BrowserWindow({
    width: 500,
    height: 300,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    },
    icon: path.join(__dirname, 'icon.png')
  });

  mainWindow.loadFile('index.html');

  mainWindow.on('closed', () => {
    mainWindow = null;
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

    gkl.addListener((e, down) => {
      if (e.name === 'LEFT CTRL' || e.name === 'RIGHT CTRL') {
        isCtrlPressed = down['LEFT CTRL'] || down['RIGHT CTRL'];
      }

      if (e.name === 'PRINT SCREEN' && e.state === 'DOWN') {
        if (isCtrlPressed) {
          // Ctrl + PrtScr : sélection rectangulaire
          console.log('Ctrl+PrtScr pressed - Opening selection window');
          createSelectionWindow();
        } else {
          // PrtScr : capture de l'écran actif
          console.log('PrtScr pressed - Capturing active screen');
          captureActiveScreen();
        }
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
  // Initialiser electron-store (module ES6)
  Store = (await import('electron-store')).default;
  store = new Store();

  // Configurer les gestionnaires IPC
  setupIpcHandlers();

  // Configurer l'écouteur de touches
  setupKeyListener();

  // Créer la fenêtre principale
  createMainWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createMainWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('will-quit', () => {
  if (gkl && gkl.kill) {
    gkl.kill();
  }
});
