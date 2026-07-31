# Chip8 rust emulator

A [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) emulator written in Rust. It runs the classic games from the 70s/80s (Pong, Tetris, Blitz, Space Invaders...) in a desktop window, with keyboard and Xbox controller support.

![Rust](https://img.shields.io/badge/rust-2024_edition-orange) ![Platform](https://img.shields.io/badge/platform-windows_|_linux-blue)

## Motivations

- Learn Rust and review some concepts by building a full system
- Understand how an emulator works (fetch, decode, execute)
- Play some old chip8 games


## Features

- All 35 CHIP-8 opcodes implemented (original COSMAC VIP behavior)
- 64x32 monochrome display scaled 16x, running at 60 fps
- ~600 CPU instructions per second (10 cycles per frame)
- Delay and sound timers at 60hz, with beep sound (windows only)
- Keyboard and Xbox controller input, working at the same time
- Unit tests covering the CPU core

## How to run

You need [Rust](https://rustup.rs/) installed. Then:

```
cd chip8-emulator
cargo run --release -- path/to/game.ch8
```

Esc closes the emulator.

On Linux you may need some system libraries first (Ubuntu/Debian):

```
sudo apt install libxkbcommon-dev libwayland-dev
```

Note: on Linux everything works except the beep sound, which uses a windows api.

### Where to download games

Roms can be downloaded from this repo: https://github.com/kripod/chip8-roms

## Controls

The CHIP-8 has a 16 key hexadecimal keypad. It is mapped to the left side of a qwerty keyboard:

```
chip8:        keyboard:
1 2 3 C       1 2 3 4
4 5 6 D  -->  Q W E R
7 8 9 E       A S D F
A 0 B F       Z X C V
```

An Xbox controller (or any xinput gamepad) also works, covering all 16 keys:

| Chip8 key | Controller                       |
| --------- | -------------------------------- |
| 2 4 6 8   | d-pad up / left / right / down   |
| 5 0 1 3   | A / B / X / Y                    |
| 7 9       | LB / RB                          |
| C D       | LT / RT                          |
| E F       | view / menu                      |
| A B       | left / right stick click         |

Tip: most games use **5** (keyboard **W**, controller **A**) as the action button and **2 4 6 8** as directions.

## Chip8 specifications

| Component   | Description                                              |
| ----------- | -------------------------------------------------------- |
| Memory      | 4,096 bytes (4KB), treated as a simple byte array        |
| Registers   | 16 8-bit registers, v0 to vF (vF is the flag register)   |
| Instructions| 35 opcodes, each taking 2 bytes (16-bit)                 |
| Stack       | 16 levels, for routine calls                             |
| Display     | 64x32 monochrome pixels, sprites drawn with xor          |
| Timers      | 2 timers (delay and sound), decrementing at 60hz         |
| Keyboard    | 16 keys, 0 to F (hexadecimal)                            |

Programs are loaded at address `0x200`, the fontset at `0x50`.

## Project structure

```
chip8-emulator/src/
├── main.rs    window, main loop, keyboard and gamepad input
├── chip8.rs   cpu: fetch -> decode -> execute cycle, all opcodes
├── memory.rs  memory, stack and register types
└── inout.rs   display, keyboard, fontset and beep
```

## Tests

The CPU core has unit tests (registers, carry flag, call/return, sprite collision, bcd):

```
cd chip8-emulator
cargo test
```
