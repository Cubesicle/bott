with (import <nixpkgs> { });
mkShell {
  buildInputs = [
    cargo
  ];
  shellHook = ''
  '';
}
