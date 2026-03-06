{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } (
    {
      self,
      lib,
      ...
    }: {
      systems = lib.systems.flakeExposed;

      perSystem = { pkgs, ... }: let
        cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
        rev = toString (self.shortRev or self.dirtyShortRev or self.lastModified or "unknown");
        deps = with pkgs; [
          binaryen
          cargo
          dioxus-cli
          lld
          rustc
          # TODO waiting on #470538
          (buildWasmBindgenCli rec {
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
      in
        {
          # Loosely based on this comment:
          # https://github.com/DioxusLabs/dioxus/discussions/4229#discussioncomment-13470839
          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = "${cargoToml.package.version}-${rev}";
            src = lib.sources.cleanSource ./.;
            strictDeps = true;
            nativeBuildInputs = [
              pkgs.rustPlatform.bindgenHook
            ] ++ deps;
            buildPhase = ''
              dx build --release --platform web
            '';
            installPhase = ''
              mkdir -p $out
              cp -R target/dx/$pname/release/web $out/
            '';
            cargoLock.lockFile = ./Cargo.lock;
          };
          devShells.default = pkgs.mkShell {
            packages = [
              pkgs.nixd
              pkgs.rust-analyzer
              pkgs.rustfmt
            ] ++ deps;
          };
        };
    });
}
