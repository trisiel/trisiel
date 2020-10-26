let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  pkgs = import sources.nixpkgs { };
in pkgs.mkShell {
  buildInputs = with pkgs; [ rust diesel-cli postgresql pgcli ];

  ROCKET_DATABASES = ''{ main_data = { url = "postgresql://postgres:hunter2@localhost:5432/wasmcloud" } }'';
}
