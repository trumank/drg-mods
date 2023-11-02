# AssemblyStorms's DRG Mods

Source for my blueprint mods is under `Content/_AssemblyStorm/`

## Building

Many mods require special post-cook build scripts and also individual packaging
and zipping which is all done via a rust script in [`src/`](src/). This script also runs
Unreal Engine via CLI to allow for cooking, building, and packaging in a single
command.

To build:
1. point `UNREAL` to the root of your Unreal Engine 4.27 installation in `.env` (see [`.env.example`](.env.example)).
2. install the [rust toolchain](https://www.rust-lang.org/learn/get-started)
3. run `cargo run --release` in a shell in the root of the repository
4. built mods will saved to `PackagedMods/`

Note: You may need to open the project in Unreal Engine and run through the
initial setup/compilation once before being able to build from command line.

## Mods

- https://mod.io/g/drg/m/sandbox-utilities
- https://mod.io/g/drg/m/custom-difficulty
- https://mod.io/g/drg/m/mission-selector
- https://mod.io/g/drg/m/build-inspector
- https://mod.io/g/drg/m/no-ragdolls
- https://mod.io/g/drg/m/a-better-modding-menu
- https://mod.io/g/drg/m/skip-start-screen
- https://mod.io/g/drg/m/mission-log
- https://mod.io/g/drg/m/take-me-home-to-the-nest
- https://mod.io/g/drg/m/no-lithophage-missions

Maintained versions of [ArcticEcho's mods](https://github.com/ArcticEcho/My-DRG-Mods)
- https://mod.io/g/drg/m/better-spectator-reloaded
- https://mod.io/g/drg/m/better-post-processing-reloaded
- https://mod.io/g/drg/m/advanced-darkness-reloaded

## Credits
- Samamstar - creator of DRGLib, an excellent library of utility and debugging tools
- ArcticEcho - original creator of many mods in this repo
