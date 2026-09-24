{
  description = "mal v0.6 toolchain and development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      malc = pkgs.rustPlatform.buildRustPackage {
        pname = "malc";
        version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile ./VERSION);
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
        cargoBuildFlags = [ "--package" "mal-compiler" ];
        cargoTestFlags = [ "--package" "mal-compiler" ];
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
      mal-fmt = pkgs.rustPlatform.buildRustPackage {
        pname = "mal-fmt";
        version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile ./VERSION);
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
        cargoBuildFlags = [ "--package" "mal-fmt" ];
        cargoTestFlags = [ "--package" "mal-fmt" ];
        postInstall = ''
          install -Dm644 $src/LICENSE $out/share/licenses/mal-fmt/LICENSE
        '';
        meta = {
          mainProgram = "mal-fmt";
          license = pkgs.lib.licenses.mit;
        };
      };
      mal-lsp = pkgs.rustPlatform.buildRustPackage {
        pname = "mal-lsp";
        version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile ./VERSION);
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
        cargoBuildFlags = [ "--package" "mal-lsp" ];
        cargoTestFlags = [ "--package" "mal-lsp" ];
        postInstall = ''
          install -Dm644 $src/LICENSE $out/share/licenses/mal-lsp/LICENSE
        '';
        meta = {
          mainProgram = "mal-lsp";
          license = with pkgs.lib.licenses; [ mit mit0 ];
        };
      };
      editor-runtime = pkgs.stdenv.mkDerivation {
        pname = "mal-editor-runtime";
        version = pkgs.lib.strings.removeSuffix "\n" (builtins.readFile ./VERSION);
        src = ./editors/tree-sitter-mal;

        dontConfigure = true;

        buildPhase = ''
          runHook preBuild
          $CC -std=c11 -O2 -shared -fPIC -Isrc -o mal.so src/parser.c
          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          install -Dm755 mal.so $out/parser/mal.so
          install -Dm755 mal.so $out/grammars/mal.so
          install -Dm644 queries/highlights.scm $out/queries/mal/highlights.scm
          install -Dm644 queries/indents.scm $out/queries/mal/indents.scm
          install -Dm644 queries/textobjects.scm $out/queries/mal/textobjects.scm
          install -Dm644 ${./LICENSE} $out/share/licenses/mal-editor-runtime/LICENSE
          install -Dm644 $src/third-party/tree-sitter/LICENSE \
            $out/share/licenses/mal-editor-runtime/tree-sitter-LICENSE
          runHook postInstall
        '';

        meta = {
          description = "Tree-sitter parser and queries for mal editor integrations";
          license = pkgs.lib.licenses.mit;
        };
      };
      editor-runtime-check = pkgs.runCommand "mal-editor-runtime-check" {
        nativeBuildInputs = [ pkgs.binutils ];
      } ''
        test -s ${editor-runtime}/parser/mal.so
        test -s ${editor-runtime}/grammars/mal.so
        cmp ${editor-runtime}/parser/mal.so ${editor-runtime}/grammars/mal.so
        test -s ${editor-runtime}/queries/mal/highlights.scm
        test -s ${editor-runtime}/queries/mal/indents.scm
        test -s ${editor-runtime}/queries/mal/textobjects.scm
        readelf --file-header ${editor-runtime}/parser/mal.so >/dev/null
        nm --dynamic --defined-only ${editor-runtime}/parser/mal.so \
          | grep ' tree_sitter_mal$' >/dev/null
        touch $out
      '';
      toolchain = pkgs.symlinkJoin {
        name = "mal-toolchain-${pkgs.lib.strings.removeSuffix "\n" (builtins.readFile ./VERSION)}";
        paths = [ malc mal-fmt mal-lsp editor-runtime ];
        meta = {
          description = "Compiler, formatter, language server, and editor runtime for mal development";
          mainProgram = "malc";
          license = with pkgs.lib.licenses; [ mit mit0 ];
        };
      };
      toolchain-check = pkgs.runCommand "mal-toolchain-check" { } ''
        test -x ${toolchain}/bin/malc
        test -x ${toolchain}/bin/mal-fmt
        test -x ${toolchain}/bin/mal-lsp
        test -s ${toolchain}/parser/mal.so
        test -s ${toolchain}/grammars/mal.so
        test -s ${toolchain}/queries/mal/highlights.scm
        test -s ${toolchain}/queries/mal/indents.scm
        test -s ${toolchain}/queries/mal/textobjects.scm
        touch $out
      '';
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
        meta.description = "mal v0.6 compiler";
      };
    in
    {
      packages.${system} = {
        default = malc;
        inherit malc mal-fmt mal-lsp editor-runtime toolchain;
      };

      apps.${system} = {
        default = malcApp;
        malc = malcApp;
      };

      checks.${system} = {
        inherit malc mal-fmt mal-lsp vscode-check;
        editor-runtime = editor-runtime-check;
        toolchain = toolchain-check;
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
          tree-sitter
          helix-unwrapped
          neovim
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
