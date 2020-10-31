{ sources ? import ./nix/sources.nix, pkgs ? import sources.nixpkgs { } }:
with pkgs;
let
  rust = import ./nix/rust.nix { inherit sources; };
  naersk = pkgs.callPackage sources.naersk {
    rustc = rust;
    cargo = rust;
  };
  src = builtins.filterSource
    (path: type: type != "directory" || builtins.baseNameOf path != "target")
    ./.;
in {
  backend = naersk.buildPackage {
    name = "wasmcloud_backend";
    inherit src;
    buildInputs = with pkgs; [ openssl pkg-config ];
  };
}
