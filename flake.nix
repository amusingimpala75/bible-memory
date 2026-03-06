{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } ({ lib, ... }: {
    systems = lib.systems.flakeExposed;

    perSystem = { pkgs, ... }: {
      devShells.default = pkgs.mkShell {
        packages = [
          pkgs.cargo
          pkgs.dioxus-cli
          pkgs.lld
          pkgs.nixd
          pkgs.rust-analyzer
          pkgs.rustc
          pkgs.rustfmt
          # TODO waiting on #470538
          (pkgs.buildWasmBindgenCli rec {
            src = pkgs.fetchCrate {
              pname = "wasm-bindgen-cli";
              version = "0.2.114";
              hash = "sha256-xrCym+rFY6EUQFWyWl6OPA+LtftpUAE5pIaElAIVqW0=";
            };
            cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
              inherit src;
              inherit (src) pname version;
              hash = "sha256-Z8+dUXPQq7S+Q7DWNr2Y9d8GMuEdSnq00quUR0wDNPM=";
            };
          })
        ];
      };
    };
  });
}
