{
  description = "mal v0.6 toolchain and development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      lib = pkgs.lib;
      version = lib.removeSuffix "\n" (builtins.readFile ./VERSION);

      # Only what the Rust workspace needs, so editor and documentation edits do not rebuild it.
      rustSource = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          ./Cargo.toml
          ./Cargo.lock
          ./VERSION
          ./LICENSE
          ./crates
          ./examples
        ];
      };

      # One command-line package of the Cargo workspace.
      rustPackage = { pname, package, license, nativeBuildInputs ? [ ], nativeCheckInputs ? [ ], postInstall ? "" }:
        pkgs.rustPlatform.buildRustPackage {
          inherit pname version nativeBuildInputs nativeCheckInputs;
          src = rustSource;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "--package" package ];
          cargoTestFlags = [ "--package" package ];
          postInstall = ''
            install -Dm644 $src/LICENSE $out/share/licenses/${pname}/LICENSE
          '' + postInstall;
          meta = {
            mainProgram = pname;
            inherit license;
          };
        };

      malc = rustPackage {
        pname = "malc";
        package = "mal-compiler";
        # The C11 runtime linked into every program is MIT-0.
        license = with lib.licenses; [ mit mit0 ];
        nativeBuildInputs = [ pkgs.makeWrapper ];
        nativeCheckInputs = [ pkgs.clang pkgs.lld ];
        postInstall = ''
          wrapProgram $out/bin/malc \
            --prefix PATH : ${lib.makeBinPath [ pkgs.clang pkgs.lld ]}
        '';
      };
      mal-fmt = rustPackage {
        pname = "mal-fmt";
        package = "mal-fmt";
        license = lib.licenses.mit;
      };
      mal-lsp = rustPackage {
        pname = "mal-lsp";
        package = "mal-lsp";
        license = lib.licenses.mit;
      };

      editor-runtime = pkgs.stdenv.mkDerivation {
        pname = "mal-editor-runtime";
        inherit version;
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
          license = lib.licenses.mit;
        };
      };

      toolchain = pkgs.symlinkJoin {
        name = "mal-toolchain-${version}";
        paths = [ malc mal-fmt mal-lsp editor-runtime ];
        meta = {
          description = "Compiler, formatter, language server, and editor runtime for mal development";
          mainProgram = "malc";
          license = with lib.licenses; [ mit mit0 ];
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
        inherit version;
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

      app = package: description: {
        type = "app";
        program = lib.getExe package;
        meta = { inherit description; };
      };
    in
    {
      packages.${system} = {
        default = malc;
        inherit malc mal-fmt mal-lsp editor-runtime toolchain;
      };

      apps.${system} = {
        default = app malc "mal v0.6 compiler";
        malc = app malc "mal v0.6 compiler";
        mal-fmt = app mal-fmt "mal v0.6 formatter";
        mal-lsp = app mal-lsp "mal v0.6 language server";
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
          ripgrep
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
        # rust-analyzer resolves the standard library sources from here.
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
      };
    };
}
