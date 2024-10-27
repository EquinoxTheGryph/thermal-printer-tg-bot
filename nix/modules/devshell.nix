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
      ];
    };
  };
}
