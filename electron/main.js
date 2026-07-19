'use strict';

/**
 * TaskBoard — Electron main process
 *
 * The app is a thin shell around the existing web frontend.
 * It connects to a running TaskBoard backend (URL configurable via
 * the TASKBOARD_URL env var or the --url CLI flag) and displays it
 * in a frameless BrowserWindow with native OS chrome.
 *
 * Default URL: http://127.0.0.1:8080
 * Override:    TASKBOARD_URL=http://192.168.1.10:8080 ./TaskBoard
 *              ./TaskBoard --url http://192.168.1.10:8080
 */

const { app, BrowserWindow, Menu, shell, ipcMain, dialog } = require('electron');
const path  = require('path');

// ── Parse CLI / env config ────────────────────────────────────────────────────
function resolveBackendUrl() {
  const argIdx = process.argv.findIndex(a => a === '--url');
  if (argIdx !== -1 && process.argv[argIdx + 1]) {
    return process.argv[argIdx + 1];
  }
  return process.env.TASKBOARD_URL || 'http://127.0.0.1:8080';
}

const BACKEND_URL = resolveBackendUrl();

// ── Window creation ───────────────────────────────────────────────────────────
let mainWindow = null;

function createWindow() {
  mainWindow = new BrowserWindow({
    width:  1280,
    height: 800,
    minWidth:  480,
    minHeight: 320,
    title: 'TaskBoard',
    backgroundColor: '#171614',   // matches --color-bg dark token; avoids white flash
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
    // Platform-specific chrome
    ...(process.platform === 'darwin' ? {
      titleBarStyle: 'hiddenInset',
      trafficLightPosition: { x: 14, y: 14 },
    } : {
      frame: true,
    }),
  });

  // Load the backend URL
  mainWindow.loadURL(BACKEND_URL).catch(() => {
    // Backend not yet reachable — show a waiting page and retry
    showLoadingPage();
  });

  // Open external links in the OS browser, not in the Electron window
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: 'deny' };
  });

  // Retry connection when the page fails to load (backend may be starting up)
  mainWindow.webContents.on('did-fail-load', (_event, errorCode) => {
    // ERR_CONNECTION_REFUSED (-102) — backend not up yet
    if (errorCode === -102 || errorCode === -105) {
      scheduleRetry();
    }
  });

  mainWindow.on('closed', () => { mainWindow = null; });
}

// ── Loading / retry logic ─────────────────────────────────────────────────────
let retryTimer = null;
let retryCount = 0;
const MAX_RETRIES = 30;   // ~30 seconds before giving up

function showLoadingPage() {
  if (!mainWindow) return;
  const escaped = BACKEND_URL.replace(/'/g, "\\'");
  mainWindow.webContents.loadURL(`data:text/html;charset=utf-8,${encodeURIComponent(`
    <!DOCTYPE html>
    <html style="background:#171614;color:#cdccca;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0">
    <body style="text-align:center">
      <div>
        <div style="font-size:2rem;margin-bottom:1rem">⬛ TaskBoard</div>
        <p style="color:#797876">Connecting to <code style="color:#4f98a3">${escaped}</code>…</p>
        <p style="color:#5a5957;font-size:.85rem;margin-top:.5rem">Make sure the backend is running.</p>
      </div>
    </body></html>
  `)}`);
}

function scheduleRetry() {
  if (retryCount >= MAX_RETRIES) {
    dialog.showErrorBox(
      'TaskBoard — connection failed',
      `Could not reach the backend at ${BACKEND_URL} after ${MAX_RETRIES} attempts.\n\nMake sure the server is running, then restart the app.`
    );
    return;
  }
  retryTimer = setTimeout(() => {
    retryCount++;
    if (mainWindow) mainWindow.loadURL(BACKEND_URL).catch(() => {});
  }, 1000);
}

// ── Application menu ──────────────────────────────────────────────────────────
function buildMenu() {
  const isMac = process.platform === 'darwin';

  const template = [
    ...(isMac ? [{ role: 'appMenu' }] : []),
    {
      label: 'View',
      submenu: [
        { role: 'reload' },
        { role: 'forceReload' },
        { type: 'separator' },
        { role: 'resetZoom' },
        { role: 'zoomIn' },
        { role: 'zoomOut' },
        { type: 'separator' },
        { role: 'togglefullscreen' },
        { type: 'separator' },
        {
          label: 'Toggle Developer Tools',
          accelerator: isMac ? 'Alt+Command+I' : 'Ctrl+Shift+I',
          click: () => mainWindow?.webContents.toggleDevTools(),
        },
      ],
    },
    {
      label: 'Connection',
      submenu: [
        {
          label: `Backend: ${BACKEND_URL}`,
          enabled: false,
        },
        { type: 'separator' },
        {
          label: 'Reconnect',
          accelerator: isMac ? 'Command+R' : 'F5',
          click: () => {
            retryCount = 0;
            mainWindow?.loadURL(BACKEND_URL).catch(() => showLoadingPage());
          },
        },
      ],
    },
    {
      role: 'help',
      submenu: [
        {
          label: 'Open in browser',
          click: () => shell.openExternal(BACKEND_URL),
        },
      ],
    },
    ...(!isMac ? [{ role: 'fileMenu' }] : []),
  ];

  Menu.setApplicationMenu(Menu.buildFromTemplate(template));
}

// ── IPC ───────────────────────────────────────────────────────────────────────
ipcMain.handle('get-backend-url', () => BACKEND_URL);

// ── App lifecycle ─────────────────────────────────────────────────────────────
app.whenReady().then(() => {
  buildMenu();
  createWindow();

  // macOS: re-create window when dock icon is clicked and no windows are open
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  clearTimeout(retryTimer);
  if (process.platform !== 'darwin') app.quit();
});

// Security: prevent new windows from navigating to unexpected origins
app.on('web-contents-created', (_event, contents) => {
  contents.on('will-navigate', (event, url) => {
    if (!url.startsWith(BACKEND_URL)) {
      event.preventDefault();
      shell.openExternal(url);
    }
  });
});
