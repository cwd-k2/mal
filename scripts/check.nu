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
