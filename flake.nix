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
          pkgs.nixd
          pkgs.rust-analyzer
          pkgs.rustc
          pkgs.rustfmt
          pkgs.lld

          # TODO waiting on #470538
          (pkgs.buildWasmBindgenCli rec {
            src = pkgs.fetchCrate {
              pname = "wasm-bindgen-cli";
              version = "0.2.106";
              hash = "sha256-M6WuGl7EruNopHZbqBpucu4RWz44/MSdv6f0zkYw+44=";
            };
            cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
              inherit src;
              inherit (src) pname version;
              hash = "sha256-ElDatyOwdKwHg3bNH/1pcxKI7LXkhsotlDPQjiLHBwA=";
            };
          })
        ];
      };
    };
  });
}
