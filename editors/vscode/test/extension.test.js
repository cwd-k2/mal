'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');

const extension = require('../extension');

test('starts the prepared server and stops it during deactivation', async () => {
  const starts = [];
  const stops = [];
  const clients = [];
  class LanguageClient {
    constructor(id, name, serverOptions, clientOptions) {
      clients.push({ id, name, serverOptions, clientOptions });
    }

    start() {
      starts.push(true);
    }

    stop() {
      stops.push(true);
      return Promise.resolve();
    }
  }
  const vscode = {
    workspace: {
      getConfiguration(section) {
        assert.equal(section, 'mal');
        return { get: () => '/configured/mal-lsp' };
      },
    },
  };
  const languageClient = { LanguageClient, TransportKind: { stdio: 'stdio' } };
  const context = {
    subscriptions: [],
    asAbsolutePath: (path) => `/extension/${path}`,
  };

  const client = extension.activateWith(vscode, languageClient, context, {
    MAL_LSP_PATH: '/prepared/mal-lsp',
  });
  assert.equal(clients[0].serverOptions.run.command, '/prepared/mal-lsp');
  assert.deepEqual(clients[0].clientOptions.documentSelector, [
    { scheme: 'file', language: 'mal' },
  ]);
  assert.deepEqual(starts, [true]);
  assert.deepEqual(context.subscriptions, [client]);

  await extension.deactivate();
  assert.deepEqual(stops, [true]);
});

test('uses the server bundled in an installed VSIX by default', async () => {
  let command;
  class LanguageClient {
    constructor(_id, _name, serverOptions) {
      command = serverOptions.run.command;
    }

    start() {}

    stop() {
      return Promise.resolve();
    }
  }
  const vscode = {
    workspace: {
      getConfiguration: () => ({ get: () => '' }),
    },
  };
  const context = {
    subscriptions: [],
    asAbsolutePath: (path) => `/installed-extension/${path}`,
  };

  extension.activateWith(
    vscode,
    { LanguageClient, TransportKind: { stdio: 'stdio' } },
    context,
    {},
  );
  assert.equal(command, '/installed-extension/server/mal-lsp');
  await extension.deactivate();
});
