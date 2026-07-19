'use strict';

/**
 * Preload script — runs in the renderer process with a limited Node.js
 * context before the page loads.  Exposes only what the frontend needs
 * via contextBridge so the renderer never has direct Node/Electron access.
 */

const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronBridge', {
  /** Returns the configured backend URL (e.g. "http://127.0.0.1:8080") */
  getBackendUrl: () => ipcRenderer.invoke('get-backend-url'),

  /** true when running inside Electron */
  isElectron: true,

  /** Platform string: "linux" | "darwin" | "win32" */
  platform: process.platform,
});
