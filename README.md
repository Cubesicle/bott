# rBot
A Geometry Dash replay bot written in Rust.

## Building
* To build this project, you will need to install [Nix](https://nixos.org/download.html).
* Next, simply run `make` to build the project.
* You can also run `make release` to build an optimized version of the dll.
* The compiled dll will be found in `target/i686-pc-windows-gnu`.

## Faster Builds
This project uses podman for cross compilation for Windows via Linux. If your builds are slow, podman is probably using the VFS storage driver instead of fuse-overlayfs. To fix that, create a new file called `storage.conf` in `~/.config/containers`, and paste the following in the configuration:
```
[storage]
  driver = "overlay"
```
After that, start up the nix shell by running `nix-shell`, then run `podman system reset`, and delete the libpod db by running `rm ~/.local/share/containers/storage/libpod/bolt_state.db`.

## Running the bot
Find a dll injector online and inject the dll into Geometry dash.
