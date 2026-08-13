{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      rec {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt pre-commit rustPackages.clippy 
            cargo-watch cargo-machete pkg-config cfitsio libclang.dev
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          # NIX_LD_LIBRARY_PATH =
          # PATH = "${./libASICameraSDK/include}";
        };
      }
    );
}
