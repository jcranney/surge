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
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt pre-commit rustPackages.clippy 
            cargo-watch cargo-machete pkg-config libcamera
            clang
            clang-tools
          ];
          hardeningDisable = [ "fortify" ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LIBCLANG_PATH = with pkgs; "${libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS =
            with pkgs;
            "-isystem ${libclang.lib}/lib/clang/${lib.versions.major (lib.getVersion clang)}/include";
        };
      }
    );
}
