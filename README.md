# Anyrun Plugins

Various anyrun plugins I've written for [anyrun](https://github.com/anyrun-org/anyrun) for my own use cases.

## Installing

**home-manager**
If you're using home-manager, you can add the following to your config:

```nix
# flake.nix
{
    inputs = {
        anyrun-plugins.url = "github:Robert-MacWha/anyrun-plugins/main";
    };

    outputs = {
        anyrun-plugins
        ...
    };
    {
        homeConfigurations = {
            username = home-manager.lib.homeManagerConfiguration {
                extraSpecialArgs = {
                    anyrun-plugins = anyrun-plugins.packages.${system};
                };
            };
        };
    };
}
```

```nix
# home.nix
{
  lib,
  pkgs,
  anyrun-plugins,
  ...
}:
{
    programs.anyrun = {
        config = {
            plugins = [
                "${anyrun-plugins.watson}/lib/libanyrun_watson.so"
                "${anyrun-plugins.timestamp}/lib/libanyrun_timestamp.so"
                "${anyrun-plugins.vscode}/lib/libanyrun_vscode.so"
                "${anyrun-plugins.todo}/lib/libanyrun_todo.so"
            ];
        };
    };
}
```

**Manual Install**
Clone the repository, build the plugins, and copy the resulting `.so` files to your anyrun plugins directory (usually `~/.config/anyrun/plugins`).

```bash
git clone git@github.com:Robert-MacWha/anyrun-plugins.git
cd anyrun-plugins
cargo build --release
cp target/release/lib*.so ~/.config/anyrun/plugins/
```