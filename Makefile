all: build

clean:
	nix-shell --run "cargo clean"

build:
	nix-shell --run "sg docker 'cross build --target i686-pc-windows-gnu'"

release:
	nix-shell --run "sg docker 'cross build --target i686-pc-windows-gnu --release'"
