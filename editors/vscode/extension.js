'use strict';

const fs = require('node:fs');

let activeClient;

function activateWith(vscode, languageClient, context, environment = process.env) {
  const configuredCommand = vscode.workspace
    .getConfiguration('mal')
    .get('server.path', '');
  const bundledCommand = context.asAbsolutePath('server/mal-lsp');
  const command = environment.MAL_LSP_PATH || configuredCommand || bundledCommand;
  if (command === bundledCommand && process.platform !== 'win32') {
    try {
      fs.chmodSync(command, 0o755);
    } catch {
      // LanguageClient reports a useful process launch error if the bundle is absent.
    }
  }
  const serverOptions = {
    run: { command, transport: languageClient.TransportKind.stdio },
    debug: { command, transport: languageClient.TransportKind.stdio },
  };
  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'mal' }],
  };
  activeClient = new languageClient.LanguageClient(
    'mal',
    'mal Language Server',
    serverOptions,
    clientOptions,
  );
  context.subscriptions.push(activeClient);
  activeClient.start();
  return activeClient;
}

function activate(context) {
  return activateWith(
    require('vscode'),
    require('vscode-languageclient/node'),
    context,
  );
}

function deactivate() {
  const client = activeClient;
  activeClient = undefined;
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate, activateWith };
