with (import <nixpkgs> { });
mkShell {
  buildInputs = [
    cargo
    docker
    rustup
  ];
  shellHook = ''
  '';
}
