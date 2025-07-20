{
  inputs = {
    nixpkgs.url = "github:nixOs/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs = { nixpkgs.follows = "nixpkgs"; };
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs = { nixpkgs.follows = "nixpkgs"; };
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = { self, nixpkgs, devenv, systems, ... } @ inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = forEachSystem (system: {
        devenv-up = self.devShells.${system}.default.config.procfileScript;
      });

      devShells = forEachSystem
        (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
          in
          {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules = [
                {
                  packages = with pkgs;
                    [
                      cargo-leptos
                      sass
                      pdfium-binaries
                    ];

                  dotenv.enable = true;
                  # https://devenv.sh/reference/options/

                  languages.rust = {
                    enable = true;
                    channel = "stable";
                    targets = [ "wasm32-unknown-unknown" ];
                  };

                  enterShell = ''
                    export PATH="${pkgs.pdfium-binaries}/lib:$PATH"
                    export PDFIUM_DYNAMIC_LIB_PATH="${pkgs.pdfium-binaries}/lib"
                    export PDFIUM_DEBUG_PATH="${pkgs.pdfium-binaries}/lib/libpdfium.so"
                  '';
                }
              ];
            };
          });
    };
}
