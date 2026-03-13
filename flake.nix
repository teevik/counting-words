{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      crane,
      ...
    }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
      ];
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          # Native nightly toolchain
          rustNightly = pkgs.rust-bin.nightly.latest.default;
          craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustNightly);

          src = craneLib.cleanCargoSource ./.;

          commonArgs = {
            inherit src;
            pname = "counting-words";
            version = "0.1.0";
            strictDeps = true;
          };

          # Build dependencies only -- cached until Cargo.toml/Cargo.lock change
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          # Cross-compilation setup (aarch64-linux-musl)
          pkgsCross = import nixpkgs {
            inherit system;
            crossSystem.config = "aarch64-unknown-linux-musl";
            overlays = [ rust-overlay.overlays.default ];
          };

          # Host nightly toolchain with cross target added
          rustNightlyCross = pkgs.rust-bin.nightly.latest.default.override {
            targets = [ "aarch64-unknown-linux-musl" ];
          };

          craneLibCross = (crane.mkLib pkgsCross).overrideToolchain (_: rustNightlyCross);

          crossArgs = {
            inherit src;
            pname = "counting-words";
            version = "0.1.0";
            strictDeps = true;

            # Statically link the target binary (only affects aarch64, not build scripts)
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = "-C target-feature=+crt-static";
          };

          # Cross dependencies -- cached separately
          cargoArtifactsCross = craneLibCross.buildDepsOnly crossArgs;

        in
        {
          default = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          cross-aarch64-linux = craneLibCross.buildPackage (
            crossArgs
            // {
              cargoArtifacts = cargoArtifactsCross;
            }
          );
        }
      );
    };
}
