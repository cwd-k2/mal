#!/usr/bin/env nu
# Development tasks for the mal repository.
#
#   nu scripts/dev.nu check [--fast]       Verify the whole repository
#   nu scripts/dev.nu vscode               Launch VS Code with the extension and language server
#   nu scripts/dev.nu helix                Prepare the editor runtime and launch Helix
#   nu scripts/dev.nu neovim               Prepare the editor runtime and launch Neovim
#
# Every task re-runs itself inside `nix develop` when the toolchain is not on PATH.

use vscode.nu

const SCRIPT_PATH = path self dev.nu
const REPOSITORY_ROOT = path self ..

def main [] {
    help main
}

# Verify Rust, Tree-sitter, the VS Code extension, and the Nix flake.
def "main check" [
    --fast  # Skip the VS Code package and the Nix flake checks.
] {
    if (reenter ["check"] --flags (if $fast { ["--fast"] } else { [] })) { return }
    cd $REPOSITORY_ROOT

    step "Rust format" { cargo fmt --all --check }
    step "Rust lint" { cargo clippy --workspace --all-targets --locked -- -D warnings }
    step "Rust tests" { cargo test --workspace --locked }
    step "Tree-sitter generation" { check_tree_sitter_generation }
    step "Tree-sitter corpus" { cd editors/tree-sitter-mal; tree-sitter test }
    step "Tree-sitter repository sources" {
        cd editors/tree-sitter-mal
        tree-sitter parse --quiet ...(rg --files ../.. -g "*.mal" | lines)
    }
    step "Tree-sitter queries" {
        cd editors/tree-sitter-mal
        for query in [highlights indents textobjects] {
            tree-sitter query $"queries/($query).scm" ../../editors/vscode/test/highlight.mal | ignore
        }
    }
    if $fast { return }

    step "VS Code dependencies" { npm --prefix editors/vscode ci }
    step "VS Code tests" { npm --prefix editors/vscode test }
    step "VS Code package" {
        vscode bundle-server (build_server)
        cd editors/vscode
        npm exec -- vsce package --out /tmp/mal-language-support-test.vsix --allow-missing-repository
    }
    step "Nix flake" { nix flake check }
}

# Launch VS Code with this repository's extension and a freshly built mal-lsp.
def "main vscode" [
    --prepare-only  # Build and install dependencies without launching VS Code.
    --dry-run  # Prepare and print the selected launch command without opening VS Code.
    --code-command: string  # Explicit VS Code CLI executable.
] {
    let flags = [
        (if $prepare_only { ["--prepare-only"] } else { [] })
        (if $dry_run { ["--dry-run"] } else { [] })
        (if $code_command != null { ["--code-command" $code_command] } else { [] })
    ] | flatten
    if (reenter ["vscode"] --flags $flags) { return }
    cd $REPOSITORY_ROOT

    let server = (build_server)
    print "Preparing VS Code extension dependencies..."
    npm install --prefix editors/vscode --ignore-scripts
    vscode bundle-server $server
    if $prepare_only {
        print $"Ready: ($server)"
        return
    }
    vscode launch $REPOSITORY_ROOT $server --dry-run=$dry_run --code-command $code_command
}

# Prepare the Tree-sitter runtime and mal-lsp, then launch Helix.
def "main helix" [--prepare-only] {
    launch_terminal_editor "helix" $prepare_only
}

# Prepare the Tree-sitter runtime and mal-lsp, then launch Neovim.
def "main neovim" [--prepare-only] {
    launch_terminal_editor "neovim" $prepare_only
}

def launch_terminal_editor [editor: string, prepare_only: bool] {
    if (reenter [$editor] --flags (if $prepare_only { ["--prepare-only"] } else { [] })) { return }
    cd $REPOSITORY_ROOT

    let server = (build_server)
    let runtime = (prepare_runtime)
    if $prepare_only {
        print $"Ready: ($server)"
        return
    }
    if $editor == "helix" {
        $env.HELIX_RUNTIME = ($runtime | path join helix)
        run-external hx $REPOSITORY_ROOT
    } else {
        run-external nvim "--cmd" "set exrc" $REPOSITORY_ROOT
    }
}

def step [name: string, action: closure] {
    print $"==> ($name)"
    do $action
}

# Re-run this script's subcommand inside `nix develop` when the toolchain is missing.
# Returns true when the work was delegated and the caller should stop.
def reenter [subcommand: list<string>, --flags: list<string> = []]: nothing -> bool {
    let tools = ["cargo" "tree-sitter" "rg" "node"]
    if ($env.MAL_DEV_SHELL? == "1") or ($tools | all {|tool| (which $tool | is-not-empty) }) {
        return false
    }
    print "Entering the pinned development environment..."
    ^nix develop $REPOSITORY_ROOT --command env MAL_DEV_SHELL=1 nu $SCRIPT_PATH ...$subcommand ...$flags
    true
}

# Build the language server in release mode and return its path.
def build_server []: nothing -> string {
    print "Building mal-lsp..."
    cargo build --manifest-path ($REPOSITORY_ROOT | path join Cargo.toml) --package mal-lsp --locked --release
    $REPOSITORY_ROOT | path join target/release/mal-lsp
}

# Regenerate the parser and lay out the runtime directories Helix and Neovim read.
# Returns the runtime root.
def prepare_runtime []: nothing -> string {
    let grammar = ($REPOSITORY_ROOT | path join editors/tree-sitter-mal)
    let root = ($REPOSITORY_ROOT | path join .artifacts/editor-runtime)
    let parser = ($root | path join mal.so)

    print "Generating and testing tree-sitter-mal..."
    cd $grammar
    tree-sitter generate
    tree-sitter test
    tree-sitter parse --quiet ...(rg --files $REPOSITORY_ROOT -g "*.mal" | lines)
    mkdir $root
    tree-sitter build --output $parser $grammar

    let helix = ($root | path join helix)
    let neovim = ($root | path join neovim)
    mkdir ($helix | path join grammars) ($helix | path join queries/mal)
    mkdir ($neovim | path join parser) ($neovim | path join queries/mal)
    cp --force $parser ($helix | path join grammars/mal.so)
    cp --force $parser ($neovim | path join parser/mal.so)
    for query in [highlights indents textobjects] {
        cp --force ($grammar | path join $"queries/($query).scm") ($helix | path join queries/mal)
    }
    cp --force ($grammar | path join queries/highlights.scm) ($neovim | path join queries/mal)
    $root
}

# Fail when regenerating the parser changes any committed generated file.
def check_tree_sitter_generation [] {
    cd editors/tree-sitter-mal
    let generated = [
        src/grammar.json
        src/node-types.json
        src/parser.c
        src/tree_sitter/alloc.h
        src/tree_sitter/array.h
        src/tree_sitter/parser.h
    ]
    let before = ($generated | each {|path| open --raw $path | hash sha256 })
    tree-sitter generate
    let after = ($generated | each {|path| open --raw $path | hash sha256 })
    if $before != $after {
        error make { msg: "tree-sitter generated sources were stale; commit the regenerated files" }
    }
}
