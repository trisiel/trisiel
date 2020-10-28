let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  pkgs = import sources.nixpkgs { };
in pkgs.mkShell rec {
  buildInputs = with pkgs; [ rust diesel-cli postgresql pgcli cargo-watch ];

  DATABASE_URL = "postgresql://postgres:hunter2@localhost:5432/wasmcloud";
  ROCKET_DATABASES = ''
    { main_data = { url = "${DATABASE_URL}" } }'';
}
