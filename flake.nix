{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{
    flake-parts,
    systems,
    ...
  }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = import systems;
    perSystem = { pkgs, system, ... }: {
      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ (inputs.rust-overlay.overlays.default) ];
      };

      devShells.default =
        let
          toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
            extensions = [ "rust-src" ];
          };
        in
        pkgs.mkShell {
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          packages = with pkgs; [
            toolchain
            cargo-expand
            worker-build
            nodejs
          ];
        };
    };
  };
}
