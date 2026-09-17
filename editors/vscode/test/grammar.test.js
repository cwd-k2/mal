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

test("keeps a closing parenthesis inside ')' in a Symbol literal token", async () => {
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

test('highlights v0.6 memory syntax', async () => {
  const grammar = await loadGrammar();
  const line = 'region := address@u8@16usize; size := #address + 0x10bytes; pair := <-cursor;';
  const tokens = grammar.tokenizeLine(line).tokens.map((token) => ({
    text: line.slice(token.startIndex, token.endIndex),
    scopes: token.scopes,
  }));

  for (const operator of ['@', '#', '+', '<-']) {
    assert.ok(
      tokens
        .find((token) => token.text === operator)
        .scopes.includes('keyword.operator.mal'),
    );
  }
  for (const literal of ['16usize', '0x10bytes']) {
    assert.ok(
      tokens
        .find((token) => token.text === literal)
        .scopes.includes('constant.numeric.integer.decimal.mal') ||
      tokens
        .find((token) => token.text === literal)
        .scopes.includes('constant.numeric.integer.hexadecimal.mal'),
    );
  }
});

test('highlights receiver-first callees as functions', async () => {
  const grammar = await loadGrammar();
  const line = 'result := source.transform(1).finish();';
  const tokens = grammar.tokenizeLine(line).tokens.map((token) => ({
    text: line.slice(token.startIndex, token.endIndex),
    scopes: token.scopes,
  }));

  for (const token of tokens.filter((candidate) => candidate.text === '.')) {
    assert.ok(token.scopes.includes('punctuation.accessor.mal'));
  }
  for (const name of ['transform', 'finish']) {
    const token = tokens.find((candidate) => candidate.text === name);
    assert.ok(token.scopes.includes('entity.name.function.mal'));
  }
});

test('highlights unary and binary Symbol operators', async () => {
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

test('highlights expression-body arrows and control keywords', async () => {
  const grammar = await loadGrammar();
  const line =
    'choose := (condition) -> [left, right] => if (condition) then left(1) else when (false) right(2);';
  const tokens = grammar.tokenizeLine(line).tokens.map((token) => ({
    text: line.slice(token.startIndex, token.endIndex),
    scopes: token.scopes,
  }));

  const arrows = tokens.filter((token) => ['->', '=>'].includes(token.text));
  assert.deepEqual(
    arrows.map((token) => token.text),
    ['->', '=>'],
  );
  assert.ok(arrows.every((token) => token.scopes.includes('keyword.operator.mal')));
  for (const keyword of ['if', 'then', 'else', 'when']) {
    assert.ok(
      tokens
        .find((token) => token.text === keyword)
        .scopes.includes('keyword.control.mal'),
    );
  }
});

test('highlights requirements and private identifiers', async () => {
  const grammar = await loadGrammar();
  const line = 'require "./support.mal"; _value :: _Type := value;';
  const tokens = grammar.tokenizeLine(line).tokens.map((token) => ({
    text: line.slice(token.startIndex, token.endIndex),
    scopes: token.scopes,
  }));

  assert.ok(
    tokens
      .find((token) => token.text === 'require')
      .scopes.includes('keyword.other.require.mal'),
  );
  assert.ok(
    tokens
      .find((token) => token.text === '_value')
      .scopes.includes('variable.other.mal'),
  );
  assert.ok(
    tokens
      .find((token) => token.text === '_Type')
      .scopes.includes('entity.name.type.mal'),
  );
});

test('classifies lowercase names outside the keyword set as value identifiers', async () => {
  const grammar = await loadGrammar();
  const line = 'ordinary := value;';
  const token = grammar
    .tokenizeLine(line)
    .tokens.map((candidate) => ({
      text: line.slice(candidate.startIndex, candidate.endIndex),
      scopes: candidate.scopes,
    }))
    .find((candidate) => candidate.text === 'ordinary');

  assert.ok(token.scopes.includes('variable.other.mal'));
  assert.ok(!token.scopes.includes('keyword.control.mal'));
});
