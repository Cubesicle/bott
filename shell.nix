with (import <nixpkgs> { });
mkShell {
  buildInputs = [
    cargo
    cargo-cross
    docker
    rustup
  ];
  shellHook = ''
  '';
}
