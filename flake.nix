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
        nativeCheckInputs = [ pkgs.clang pkgs.lld ];
        postInstall = ''
          wrapProgram $out/bin/malc \
            --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.clang pkgs.lld ]}
          install -Dm644 $src/LICENSE $out/share/licenses/malc/LICENSE
        '';
        meta = {
          mainProgram = "malc";
          license = with pkgs.lib.licenses; [ mit mit0 ];
        };
      };
      mal-lsp = pkgs.rustPlatform.buildRustPackage {
        pname = "mal-lsp";
        version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile ./VERSION);
        src = ./.;
        cargoRoot = "tools/mal-lsp";
        buildAndTestSubdir = "tools/mal-lsp";
        cargoLock.lockFile = ./tools/mal-lsp/Cargo.lock;
        postInstall = ''
          install -Dm644 $src/LICENSE $out/share/licenses/mal-lsp/LICENSE
        '';
        meta = {
          mainProgram = "mal-lsp";
          license = with pkgs.lib.licenses; [ mit mit0 ];
        };
      };
      vscode-check = pkgs.buildNpmPackage {
        pname = "mal-language-support-check";
        version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile ./VERSION);
        src = ./editors/vscode;
        npmDepsHash = "sha256-AimvSkY/IpOuZeQk4Km2PeL/RKSN+2pm9crKJeHheUI=";
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
          nushell
          clang
          lld
          hyperfine
          valgrind
          time
          nodejs
        ];
        RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
      };
    };
}
