{
  description = "Anyrun plugin collection";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    systems.url = "github:nix-systems/default-linux";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    {
      self,
      flake-parts,
      systems,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import systems;

      perSystem =
        { pkgs, ... }:
        let
          lockFile = ./Cargo.lock;
          mkPlugin =
            name: extraBuildInputs:
            pkgs.rustPlatform.buildRustPackage rec {
              pname = "libanyrun_${name}";
              version = "0.1.0";
              src = ./.;
              buildAndTestSubdir = "anyrun-${name}";
              cargoLock = {
                lockFile = lockFile;
                outputHashes = {
                  "anyrun-macros-25.9.3" = "sha256-GLGdVJSPH0LnsO64Biw0WFJaj1PlltYxgH13f+FGWgQ=";
                  "anyrun-interface-25.9.3" = "sha256-ynLb+3Y+sbrNc2HD1VTRNBj2GKm44CGENTJZwvn0Xt0=";
                };
              };

              buildInputs = extraBuildInputs;

              cargoBuildFlags = [ "--lib" ];

              installPhase = ''
                mkdir -p $out/lib
                cp target/x86_64-unknown-linux-gnu/release/libanyrun_${name}.so $out/lib/
              '';
            };
        in
        {
          packages = {
            watson = mkPlugin "watson" [ ];
            timestamp = mkPlugin "timestamp" [ ];
            vscode = mkPlugin "vscode" [ pkgs.sqlite ];
            todo = mkPlugin "todo" [ ];
          };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              rustfmt
            ];
          };
        };
    };
}
