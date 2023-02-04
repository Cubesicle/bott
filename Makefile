run = nix-shell --pure --run

all: build

clean:
	$(run) "cargo clean"

build:
	$(run) "cross b --target i686-pc-windows-gnu"

release:
	$(run) "cross b -r --target i686-pc-windows-gnu"
