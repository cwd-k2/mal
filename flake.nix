{
  description = "mal v0.5 reference compiler and development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      malc = pkgs.rustPlatform.buildRustPackage {
        pname = "malc";
        version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile ./VERSION);
        src = ./.;
        cargoRoot = "compiler";
        buildAndTestSubdir = "compiler";
        cargoLock.lockFile = ./compiler/Cargo.lock;
        nativeBuildInputs = [ pkgs.makeWrapper ];
        nativeCheckInputs = [ pkgs.clang ];
        postInstall = ''
          wrapProgram $out/bin/malc \
            --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.clang ]}
        '';
        meta.mainProgram = "malc";
      };
      mal-lsp = pkgs.rustPlatform.buildRustPackage {
        pname = "mal-lsp";
        version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile ./VERSION);
        src = ./.;
        cargoRoot = "tools/mal-lsp";
        buildAndTestSubdir = "tools/mal-lsp";
        cargoLock.lockFile = ./tools/mal-lsp/Cargo.lock;
        meta.mainProgram = "mal-lsp";
      };
      vscode-check = pkgs.buildNpmPackage {
        pname = "mal-language-support-check";
        version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile ./VERSION);
        src = ./editors/vscode;
        npmDepsHash = "sha256-MWI0HAKkzwb7yNeEw/WoplgkpqCBR6K0RrDUqbQeU+o=";
        npmRebuildFlags = [ "--ignore-scripts" ];
        dontNpmBuild = true;
        doCheck = true;
        checkPhase = ''
          runHook preCheck
          npm test
          runHook postCheck
        '';
        installPhase = ''
          runHook preInstall
          mkdir -p $out
          touch $out/passed
          runHook postInstall
        '';
      };
      malcApp = {
        type = "app";
        program = "${malc}/bin/malc";
        meta.description = "mal v0.5 reference compiler";
      };
    in
    {
      packages.${system} = {
        default = malc;
        inherit malc mal-lsp;
      };

      apps.${system} = {
        default = malcApp;
        malc = malcApp;
      };

      checks.${system} = {
        inherit malc mal-lsp vscode-check;
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
          sccache
          clang
          lld
          nodejs
        ];
        RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
      };
    };
}
