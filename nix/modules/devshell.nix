{ inputs, ... }:
{
  perSystem = { config, self', pkgs, lib, ... }: {
    devShells.default = pkgs.mkShell {
      name = "thermal-printer-shell";
      inputsFrom = [
        self'.devShells.rust
        config.treefmt.build.devShell
      ];
      packages = with pkgs; [
        just
        nixd # Nix language server
        cargo-watch
        config.process-compose.cargo-doc-live.outputs.package
        udev # Needed for the serial port driver
      ];
      env = {
        # Required by rust-analyzer
        # RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
        # PKG_CONFIG_PATH = "${pkgs.udev.dev}/lib/pkgconfig";
        # PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig;${pkgs.udev.dev}/lib/pkgconfig";
        CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG = "true";
      };
    };
  };
}
