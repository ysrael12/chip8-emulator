# Chip8 rust emulator 
### motivations to make this project 
- Learn rust 
- Try to make a chip8 emulator
- Play some old chip8 games
- Create a full system withou ai (i know ai is very good, but this project is for learning rust, and review some rust concepts)

# Chip8 specifications 

## Memory 
- 4,096 bytes (4KB)
i will trate the memory as a simple byte array

# CPU

### Registers
- 16 8-bit registers
- v0 to vF (F- hexadecimal)
- vF is used as a flag register 

### Instruction set
- 35 opcodes each taking 2 bytes (16-bit)

### Stack
- 16 levels


## Display
- 64x32 pixels

## timers
- 2 timers (delay and sound)

## keyboard
- 16 keys (0 to F)

note : F is because i use hexadecimal keys

# How to run

```
cd chip8-emulator
cargo run --release -- path/to/game.ch8
```

## Where to download games
Roms can be downloaded from this repo: https://github.com/kripod/chip8-roms

## Key mapping
The chip8 keypad is mapped to the left side of the keyboard:

```
chip8:        keyboard:
1 2 3 C       1 2 3 4
4 5 6 D  -->  Q W E R
7 8 9 E       A S D F
A 0 B F       Z X C V
```

Esc closes the emulator.

## Xbox controller
An xbox controller (or any xinput gamepad) also works, covering all 16 keys:

```
chip8:   controller:
2 4 6 8  d-pad up / left / right / down
5 0 1 3  A / B / X / Y
7 9      LB / RB
C D      LT / RT
E F      view / menu
A B      left / right stick click
```
