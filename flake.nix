{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ inputs.rust-overlay.overlays.default ];
        pkgs = import inputs.nixpkgs { inherit system overlays; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            (pkgs.lib.hiPrio (
              pkgs.rust-bin.stable.latest.minimal.override {
                extensions = [
                  "rust-docs"
                  "clippy"
                  "llvm-tools"
                ];
              }
            ))
            (pkgs.rust-bin.selectLatestNightlyWith (
              toolchain:
              toolchain.minimal.override {
                extensions = [
                  "rust-analyzer"
                  "rustfmt"
                ];
              }
            ))
          ];
          shellHook = ''
            export CARGO_HOME="$PWD/.cargo"
            export PATH="$CARGO_HOME/bin:$PATH"
            mkdir -p .cargo
            echo '*' > .cargo/.gitignore
          '';
        };
      }
    );
}
