#!/usr/bin/env nu

const SCRIPT_PATH = path self vscode-dev.nu
const REPOSITORY_ROOT = path self ..

def main [
    --prepare-only  # Build and install dependencies without launching VS Code.
    --dry-run  # Prepare and print the selected launch command without opening VS Code.
    --code-command: string  # Explicit VS Code CLI executable.
] {
    if $env.MAL_VSCODE_DEV_SHELL? != "1" {
        mut forwarded = []
        if $prepare_only {
            $forwarded = ($forwarded | append "--prepare-only")
        }
        if $dry_run {
            $forwarded = ($forwarded | append "--dry-run")
        }
        if $code_command != null {
            $forwarded = ($forwarded | append ["--code-command", $code_command])
        }
        let forwarded_arguments = $forwarded
        do --capture-errors {
            ^nix develop $REPOSITORY_ROOT --command env MAL_VSCODE_DEV_SHELL=1 nu $SCRIPT_PATH ...$forwarded_arguments
        }
        return
    }

    let server_manifest = ($REPOSITORY_ROOT | path join tools/mal-lsp/Cargo.toml)
    let extension_path = ($REPOSITORY_ROOT | path join editors/vscode)
    let server_path = ($REPOSITORY_ROOT | path join tools/mal-lsp/target/release/mal-lsp)
    let bundled_server = ($extension_path | path join server/mal-lsp)

    print "Building mal-lsp..."
    do --capture-errors {
        ^cargo build --manifest-path $server_manifest --locked --release
    }

    print "Preparing VS Code extension dependencies..."
    do --capture-errors {
        ^npm install --prefix $extension_path --ignore-scripts
    }
    mkdir ($extension_path | path join server)
    cp --force $server_path $bundled_server

    if $prepare_only {
        print $"Ready: ($server_path)"
        return
    }
    let candidates = if $code_command == null {
        launcher_candidates
    } else {
        [$code_command]
    }
    let launcher = ($candidates | each {|candidate|
        inspect_launcher $candidate $extension_path
    } | compact | get -o 0)
    if $launcher == null {
        let checked = if ($candidates | is-empty) {
            "none"
        } else {
            $candidates | str join ", "
        }
        error make {
            msg: $"No compatible VS Code CLI was found. Checked: ($checked). Use --code-command to select one explicitly, or --prepare-only in a GUI-less environment."
        }
    }

    $env.MAL_LSP_PATH = $server_path
    let development_option = $"--extensionDevelopmentPath=($extension_path)"
    if $dry_run {
        if $launcher.mode == "development" {
            print $"Development launch: ($launcher.command) ($development_option) ($REPOSITORY_ROOT)"
        } else {
            print $"Remote install: package a VSIX, install it with ($launcher.command), then open ($REPOSITORY_ROOT)"
        }
        return
    }

    if $launcher.mode == "development" {
        print "Starting the VS Code Extension Development Host..."
        do --capture-errors {
            run-external $launcher.command $development_option $REPOSITORY_ROOT
        }
        return
    }

    let vsix_path = "/tmp/mal-language-support.vsix"
    print "Packaging and installing the extension into the remote host..."
    do --capture-errors {
        cd $extension_path
        ^npm exec -- vsce package --out $vsix_path --allow-missing-repository
    }
    do --capture-errors {
        run-external $launcher.command "--install-extension" $vsix_path "--force"
    }
    print "Opening the repository with the installed remote extension..."
    do --capture-errors {
        run-external $launcher.command "--new-window" $REPOSITORY_ROOT
    }
}

def launcher_candidates [] {
    mut candidates = []
    for name in ["code", "code-insiders"] {
        let command = (which $name | get -o 0 | get -o path)
        if $command != null {
            $candidates = ($candidates | append ($command | into string))
        }
    }

    let system_installations = [
        "/mnt/c/Program Files/Microsoft VS Code/bin/code"
        "/mnt/c/Program Files/Microsoft VS Code Insiders/bin/code-insiders"
    ] | where {|path| $path | path exists}
    let user_installations = (
        (glob "/mnt/c/Users/*/AppData/Local/Programs/Microsoft VS Code/bin/code")
        | append (glob "/mnt/c/Users/*/AppData/Local/Programs/Microsoft VS Code Insiders/bin/code-insiders")
        | each {|path| $path | into string}
    )
    $candidates | append $system_installations | append $user_installations | uniq
}

def inspect_launcher [command: string, extension_path: string] {
    let version_probe = (run-external $command "--version" | complete)
    let version = $version_probe.stdout | lines | get -o 0 | default ""
    if $version_probe.exit_code != 0 or not ($version =~ '^\d+\.\d+\.\d+') {
        return null
    }

    let option = $"--extensionDevelopmentPath=($extension_path)"
    let development_probe = (run-external $command $option "--version" | complete)
    let output = [$development_probe.stdout $development_probe.stderr] | str join "\n"
    let mode = if $development_probe.exit_code == 0 and not ($output | str contains "Ignoring option") {
        "development"
    } else {
        "remote"
    }
    { command: $command, mode: $mode, version: $version }
}
