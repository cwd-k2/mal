'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const oniguruma = require('vscode-oniguruma');
const textmate = require('vscode-textmate');

const extensionRoot = path.join(__dirname, '..');
const grammarPath = path.join(extensionRoot, 'syntaxes', 'mal.tmLanguage.json');

async function loadGrammar() {
  const wasmPath = require.resolve('vscode-oniguruma/release/onig.wasm');
  const wasm = fs.readFileSync(wasmPath);
  await oniguruma.loadWASM(
    wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength),
  );
  const source = fs.readFileSync(grammarPath, 'utf8');
  const registry = new textmate.Registry({
    onigLib: Promise.resolve({
      createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
      createOnigString: (value) => new oniguruma.OnigString(value),
    }),
    loadGrammar: async (scopeName) =>
      scopeName === 'source.mal'
        ? textmate.parseRawGrammar(source, grammarPath)
        : null,
  });
  return registry.loadGrammar('source.mal');
}

test("keeps a closing parenthesis inside ')' in an Engram literal token", async () => {
  const grammar = await loadGrammar();
  const line = "call(')');";
  const tokens = grammar.tokenizeLine(line).tokens.map((token) => ({
    text: line.slice(token.startIndex, token.endIndex),
    scopes: token.scopes,
  }));
  const closingParentheses = tokens.filter((token) => token.text === ')');

  assert.equal(closingParentheses.length, 2);
  assert.ok(closingParentheses[0].scopes.includes('string.quoted.single.mal'));
  assert.ok(closingParentheses[0].scopes.includes('constant.character.mal'));
  assert.ok(!closingParentheses[1].scopes.includes('string.quoted.single.mal'));
  assert.ok(closingParentheses[1].scopes.includes('punctuation.definition.mal'));
});

test('highlights the storage-size sigil and its type separately', async () => {
  const grammar = await loadGrammar();
  const line = 'size := @Engram + @UInt8;';
  const tokens = grammar.tokenizeLine(line).tokens.map((token) => ({
    text: line.slice(token.startIndex, token.endIndex),
    scopes: token.scopes,
  }));

  for (const token of tokens.filter((candidate) => candidate.text === '@')) {
    assert.ok(token.scopes.includes('keyword.operator.mal'));
  }
  for (const name of ['Engram', 'UInt8']) {
    const token = tokens.find((candidate) => candidate.text === name);
    assert.ok(token.scopes.includes('entity.name.type.mal'));
  }
});

test('highlights unary and binary Engram operators', async () => {
  const grammar = await loadGrammar();
  const line = 'length := #value; byte := value # 1u64;';
  const tokens = grammar.tokenizeLine(line).tokens.map((token) => ({
    text: line.slice(token.startIndex, token.endIndex),
    scopes: token.scopes,
  }));

  const hashes = tokens.filter((token) => token.text === '#');
  assert.equal(hashes.length, 2);
  assert.ok(hashes.every((token) => token.scopes.includes('keyword.operator.mal')));
});
