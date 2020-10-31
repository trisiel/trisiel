let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  pkgs = import sources.nixpkgs { };
in pkgs.mkShell rec {
  buildInputs = with pkgs; with elmPackages; [
    # rust
    rust
    diesel-cli
    postgresql
    pgcli
    cargo-watch
    pkg-config
    openssl

    # elm
    elm2nix
    elm
    elm-language-server
  ];

  B2_CREDFILE = "./var/secret/b2-creds.txt";
  B2_MODULE_BUCKET_NAME = "wasmcloud-modules";
  DATABASE_URL = "postgresql://postgres:hunter2@localhost:5432/wasmcloud";
  JWT_SECRET = "hunter2";
  ROCKET_DATABASES = ''{ main_data = { url = "${DATABASE_URL}" } }'';
  RUST_LOG = "info,wasmcloud=debug";
}
