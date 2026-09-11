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
      malcApp = {
        type = "app";
        program = "${malc}/bin/malc";
        meta.description = "mal v0.5 reference compiler";
      };
    in
    {
      packages.${system} = {
        default = malc;
        inherit malc;
      };

      apps.${system} = {
        default = malcApp;
        malc = malcApp;
      };

      checks.${system}.malc = malc;

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          sccache
          clang
          lld
          nodejs
        ];
        RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
      };
    };
}
