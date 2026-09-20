#!/usr/bin/env nu

def step [name: string, action: closure] {
    print $"==> ($name)"
    do $action
}

step "compiler format" {
    cargo fmt --manifest-path compiler/Cargo.toml --check
}
step "compiler lint" {
    cargo clippy --manifest-path compiler/Cargo.toml --all-targets -- -D warnings
}
step "compiler tests" {
    cargo test --manifest-path compiler/Cargo.toml
}
step "language server format" {
    cargo fmt --manifest-path tools/mal-lsp/Cargo.toml --check
}
step "language server lint" {
    cargo clippy --manifest-path tools/mal-lsp/Cargo.toml --all-targets --locked -- -D warnings
}
step "language server tests" {
    cargo test --manifest-path tools/mal-lsp/Cargo.toml --locked
}
step "Tree-sitter generation" {
    cd editors/tree-sitter-mal
    let generated_files = [
        src/grammar.json
        src/node-types.json
        src/parser.c
        src/tree_sitter/alloc.h
        src/tree_sitter/array.h
        src/tree_sitter/parser.h
    ]
    let before = ($generated_files | each {|path| open --raw $path | hash sha256})
    tree-sitter generate
    let after = ($generated_files | each {|path| open --raw $path | hash sha256})
    if $before != $after {
        error make { msg: "tree-sitter generated sources were stale; commit the regenerated files" }
    }
}
step "Tree-sitter corpus" {
    cd editors/tree-sitter-mal
    tree-sitter test
}
step "Tree-sitter repository sources" {
    cd editors/tree-sitter-mal
    let sources = (rg --files ../.. -g "*.mal" | lines)
    tree-sitter parse --quiet ...$sources
}
step "Tree-sitter queries" {
    cd editors/tree-sitter-mal
    for query in [highlights indents textobjects] {
        tree-sitter query $"queries/($query).scm" ../../editors/vscode/test/highlight.mal | ignore
    }
}
step "VS Code dependencies" {
    npm --prefix editors/vscode ci
}
step "VS Code server bundle" {
    nu scripts/vscode-dev.nu --prepare-only
}
step "VS Code tests" {
    npm --prefix editors/vscode test
}
step "VS Code package" {
    cd editors/vscode
    npm exec -- vsce package --out /tmp/mal-language-support-test.vsix --allow-missing-repository
}
step "Nix flake" {
    nix flake check
}
