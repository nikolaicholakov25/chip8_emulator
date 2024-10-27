# Chip8 Emulator on Rust


## Run Locally

Clone the project

```bash
  git clone https://github.com/nikolaicholakov25/chip8_emulator.git
```

Build the project

```bash
  make build
```
## Try playing games

Run the game of your choise by passing the path to the game.
Optionally you can pass the speed of the game (ticks per frame) as a second argument.

Example playing snek game with default speed

```bash
make run-game game=snek.ch8
```
or
```bash
cargo run --manifest-path ./desktop/Cargo.toml ./games/snek.ch8
```


Example playing snek game with custom speed

```bash
make run-game game=snek.ch8 speed=20
```
or
```bash
cargo run --manifest-path ./desktop/Cargo.toml ./games/snek.ch8 20
```


## Notes

The window frame was built using [Rust-SDL2](https://docs.rs/crate/sdl2/0.37.0) crate, with the "bundled" feature. You will need a C compiler installed on your machine for the project to work properly.

Quote from the Rust-SDL2 docs:

`Since 0.31, this crate supports a feature named "bundled" which compiles SDL2 from source and links it automatically. While this should work for any architecture, you will need a C compiler (like gcc, clang, or MS's own compiler) to use this feature properly.`

Steps for how to install this can be found in [Rust-SDL2's](https://docs.rs/crate/sdl2/0.37.0) docs.

## Acknowledgements

 - [Chip8 emulator guide followed](https://github.com/aquova/chip8-book)

## Credits
The provided Chip-8 games are supplied from [Zophar's Domain](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html) and [Chip8 Archive](https://johnearnest.github.io/chip8Archive/?sort=platform#chip8). Original author unknown.
