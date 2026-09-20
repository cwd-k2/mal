#!/usr/bin/env nu

const SCRIPT_PATH = path self editor-dev.nu
const REPOSITORY_ROOT = path self ..

def main [
    editor: string  # Editor to prepare and launch: helix or neovim.
    --prepare-only  # Build the server and editor runtimes without launching an editor.
] {
    if not ($editor in ["helix", "neovim"]) {
        error make { msg: $"Unsupported editor '($editor)'; expected helix or neovim." }
    }

    if $env.MAL_EDITOR_DEV_SHELL? != "1" {
        mut forwarded = [$editor]
        if $prepare_only {
            $forwarded = ($forwarded | append "--prepare-only")
        }
        let forwarded_arguments = $forwarded
        do --capture-errors {
            ^nix develop $REPOSITORY_ROOT --command env MAL_EDITOR_DEV_SHELL=1 nu $SCRIPT_PATH ...$forwarded_arguments
        }
        return
    }

    let server_manifest = ($REPOSITORY_ROOT | path join tools/mal-lsp/Cargo.toml)
    let server_path = ($REPOSITORY_ROOT | path join tools/mal-lsp/target/release/mal-lsp)
    let grammar_path = ($REPOSITORY_ROOT | path join editors/tree-sitter-mal)
    let artifact_root = ($REPOSITORY_ROOT | path join .artifacts/editor-runtime)
    let parser_path = ($artifact_root | path join mal.so)
    let helix_runtime = ($artifact_root | path join helix)
    let neovim_runtime = ($artifact_root | path join neovim)

    print "Building mal-lsp..."
    cargo build --manifest-path $server_manifest --locked --release

    print "Generating and testing tree-sitter-mal..."
    cd $grammar_path
    tree-sitter generate
    tree-sitter test
    let mal_sources = (rg --files $REPOSITORY_ROOT -g "*.mal" | lines)
    tree-sitter parse --quiet ...$mal_sources

    mkdir $artifact_root
    tree-sitter build --output $parser_path $grammar_path

    let helix_grammar_directory = ($helix_runtime | path join grammars)
    let helix_query_directory = ($helix_runtime | path join queries/mal)
    let neovim_parser_directory = ($neovim_runtime | path join parser)
    let neovim_query_directory = ($neovim_runtime | path join queries/mal)
    mkdir $helix_grammar_directory
    mkdir $helix_query_directory
    mkdir $neovim_parser_directory
    mkdir $neovim_query_directory
    cp --force $parser_path ($helix_grammar_directory | path join mal.so)
    cp --force $parser_path ($neovim_parser_directory | path join mal.so)
    cp --force ($grammar_path | path join queries/highlights.scm) $helix_query_directory
    cp --force ($grammar_path | path join queries/indents.scm) $helix_query_directory
    cp --force ($grammar_path | path join queries/textobjects.scm) $helix_query_directory
    cp --force ($grammar_path | path join queries/highlights.scm) $neovim_query_directory

    if $prepare_only {
        print $"Ready: ($server_path)"
        return
    }

    cd $REPOSITORY_ROOT
    if $editor == "helix" {
        $env.HELIX_RUNTIME = $helix_runtime
        run-external hx $REPOSITORY_ROOT
    } else {
        run-external nvim "--cmd" "set exrc" $REPOSITORY_ROOT
    }
}
