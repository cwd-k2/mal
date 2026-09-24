# VS Code specifics for scripts/dev.nu: bundling the language server and launching a compatible VS Code.

const EXTENSION = path self ../editors/vscode

# Copy the built language server next to the extension so it can start it.
export def bundle-server [server: string] {
    mkdir ($EXTENSION | path join server)
    cp --force $server ($EXTENSION | path join server/mal-lsp)
}

# Launch VS Code with the extension, either as an Extension Development Host or, where that is not
# supported (for example WSL remote-cli), by installing a packaged VSIX into the remote host.
export def launch [
    repository: string
    server: string
    --dry-run
    --code-command: string
] {
    let candidates = if $code_command == null { launcher_candidates } else { [$code_command] }
    let launcher = ($candidates | each {|candidate| inspect_launcher $candidate $EXTENSION } | compact | get -o 0)
    if $launcher == null {
        let checked = if ($candidates | is-empty) { "none" } else { $candidates | str join ", " }
        error make {
            msg: $"No compatible VS Code CLI was found. Checked: ($checked). Use --code-command to select one explicitly, or --prepare-only in a GUI-less environment."
        }
    }

    $env.MAL_LSP_PATH = $server
    let development_option = $"--extensionDevelopmentPath=($EXTENSION)"
    if $dry_run {
        if $launcher.mode == "development" {
            print $"Development launch: ($launcher.command) ($development_option) ($repository)"
        } else {
            print $"Remote install: package a VSIX, install it with ($launcher.command), then open ($repository)"
        }
        return
    }

    if $launcher.mode == "development" {
        print "Starting the VS Code Extension Development Host..."
        run-external $launcher.command $development_option $repository
        return
    }

    let vsix = "/tmp/mal-language-support.vsix"
    print "Packaging and installing the extension into the remote host..."
    cd $EXTENSION
    npm exec -- vsce package --out $vsix --allow-missing-repository
    run-external $launcher.command "--install-extension" $vsix "--force"
    print "Opening the repository with the installed remote extension..."
    run-external $launcher.command "--new-window" $repository
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
