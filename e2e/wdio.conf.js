import os from 'os';
import path from 'path';
import fs from 'fs';
import { spawn, spawnSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Keep track of the tauri-driver process
let tauriDriver;

export const config = {
  specs: ['./test/specs/**/*.e2e.js'],
  maxInstances: 1,

  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: '../src-tauri/target/release/handy',
      },
    },
  ],

  // Log level: trace | debug | info | warn | error | silent
  logLevel: 'info',

  // Default timeout for all waitFor* commands
  waitforTimeout: 10000,

  // Default timeout for requests to the Selenium/WebDriver server
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  // Framework to use
  framework: 'mocha',

  // Test reporters
  reporters: ['spec'],

  // Mocha options
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  // Hook run before any tests start
  onPrepare: function () {
    // Check if the app is already built
    const appPath = path.resolve(__dirname, '..', 'src-tauri', 'target', 'release', 'handy');
    if (!fs.existsSync(appPath)) {
      console.log('Building Tauri application...');
      const buildResult = spawnSync('cargo', ['build', '--release'], {
        cwd: path.resolve(__dirname, '..', 'src-tauri'),
        stdio: 'inherit',
      });

      if (buildResult.status !== 0) {
        throw new Error('Failed to build the Tauri application');
      }
    } else {
      console.log('Using existing build:', appPath);
    }
  },

  // Hook run before a test starts
  beforeSession: function () {
    const tauriDriverPath = path.resolve(
      os.homedir(),
      '.cargo',
      'bin',
      'tauri-driver'
    );

    console.log('Starting tauri-driver from:', tauriDriverPath);

    tauriDriver = spawn(tauriDriverPath, [], {
      stdio: [null, process.stdout, process.stderr],
    });
  },

  // Hook run after a test ends
  afterSession: function () {
    if (tauriDriver) {
      console.log('Stopping tauri-driver...');
      tauriDriver.kill();
    }
  },
};
