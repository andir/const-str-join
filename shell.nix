with import <nixpkgs> {};
mkShell {
  nativeBuildInputs = [
    cargo
    rustfmt
    clippy
    cargo-expand
  ];
}
