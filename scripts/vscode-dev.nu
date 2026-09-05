#!/usr/bin/env nu

const SCRIPT_PATH = path self vscode-dev.nu
const REPOSITORY_ROOT = path self ..

def main [
    --prepare-only  # Build and install dependencies without launching VS Code.
] {
    if $env.MAL_VSCODE_DEV_SHELL? != "1" {
        let forwarded = if $prepare_only { ["--prepare-only"] } else { [] }
        do --capture-errors {
            ^nix develop $REPOSITORY_ROOT --command env MAL_VSCODE_DEV_SHELL=1 nu $SCRIPT_PATH ...$forwarded
        }
        return
    }

    let server_manifest = ($REPOSITORY_ROOT | path join tools/mal-lsp/Cargo.toml)
    let extension_path = ($REPOSITORY_ROOT | path join editors/vscode)
    let server_path = ($REPOSITORY_ROOT | path join tools/mal-lsp/target/debug/mal-lsp)

    print "Building mal-lsp..."
    do --capture-errors {
        ^cargo build --manifest-path $server_manifest --locked
    }

    print "Preparing VS Code extension dependencies..."
    do --capture-errors {
        ^npm install --prefix $extension_path --ignore-scripts
    }

    if $prepare_only {
        print $"Ready: ($server_path)"
        return
    }
    if (which code | is-empty) {
        error make { msg: "`code` was not found on PATH" }
    }

    $env.MAL_LSP_PATH = $server_path
    print "Starting the VS Code Extension Development Host..."
    do --capture-errors {
        ^code --extensionDevelopmentPath $extension_path $REPOSITORY_ROOT
    }
}
