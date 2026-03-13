{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
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

          rustNightly = pkgs.rust-bin.nightly.latest.default;

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustNightly;
            rustc = rustNightly;
          };

          pkgsCross = import nixpkgs {
            inherit system;
            crossSystem.config = "aarch64-unknown-linux-musl";
            overlays = [ rust-overlay.overlays.default ];
          };

          # Host nightly toolchain with cross target added
          rustNightlyCross = pkgs.rust-bin.nightly.latest.default.override {
            targets = [ "aarch64-unknown-linux-musl" ];
          };

          # Use makeRustPlatform from the cross package set (so hooks target aarch64)
          # but with the HOST nightly toolchain (so binaries actually run on x86_64).
          # Note: use pkgsCross directly, not pkgsCross.pkgsStatic -- the musl target
          # already produces static Rust binaries, and pkgsStatic adds -static flags
          # to the CC wrapper which breaks build scripts (they get linked without a
          # C runtime since only the aarch64 static libc is available, not x86_64's).
          rustPlatformCross = pkgsCross.makeRustPlatform {
            cargo = rustNightlyCross;
            rustc = rustNightlyCross;
          };

        in
        {
          default = rustPlatform.buildRustPackage {
            pname = "matmul-simd";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };

          cross-aarch64-linux = rustPlatformCross.buildRustPackage {
            pname = "matmul-simd";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            # Statically link the target binary (only affects aarch64, not build scripts)
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS = "-C target-feature=+crt-static";
          };
        }
      );
    };
}
