{
  description = "mtgbrain — MTGJSON cards in a queryable SQLite DB for LLM-driven card search";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.callPackage ./nix/package.nix { };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            clippy
            rust-analyzer
            rustfmt
            curl
            cacert
            jq
            just
            sqlite
          ];
          # curl in the dev shell needs an explicit CA bundle.
          shellHook = ''
            export SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt
          '';
        };
      }
    );
}
