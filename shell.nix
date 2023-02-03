with (import <nixpkgs> { });
mkShell {
  buildInputs = [
    rustup
  ];
  shellHook = ''
    rustup target add i686-pc-windows-gnu
  '';
}
