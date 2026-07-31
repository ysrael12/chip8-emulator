/* using our modules inout and memory */
/* mounting our machine  */
use crate::inout::{self, Timer};
use crate::memory;

pub struct Chip8 {
    /* memory */
    memory: memory::Memory,
    registers: memory::registers,
    index_pointer: memory::index_pointer,
    program_counter: memory::program_counter,
    stack: memory::Stack,

    /* inout */
    pub display: inout::Display,
    pub keyboard: inout::Keyboard,
    pub delay_timer: Timer,
    pub sound_timer: Timer,

    //state of the simple pseudo random generator (opcode CXNN)
    random_state: u16,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Self {
            memory: [0; memory::MEMORY_SIZE],
            registers: [0; 16],
            index_pointer: 0,
            program_counter: memory::START_ADDRESS as memory::program_counter,
            stack: memory::Stack::new(),
            display: inout::Display::new(),
            keyboard: inout::Keyboard::new(),
            delay_timer: 0,
            sound_timer: 0,
            random_state: 0xACE1,
        };
        //load fontset
        Self::load_fontset(&mut chip8.memory);
        chip8
    }

    pub fn load_fontset(memory: &mut memory::Memory) {
        let start = memory::FONTSET_START_ADDRESS;
        memory[start..start + inout::FONTSET.len()].copy_from_slice(&inout::FONTSET);
    }

    //load program into memory
    pub fn load_program(&mut self, program: &[u8]) {
        let start = memory::START_ADDRESS;
        let end = start + program.len();
        if end > memory::MEMORY_SIZE {
            panic!("program too big: {} bytes", program.len());
        }
        self.memory[start..end].copy_from_slice(program);
        self.program_counter = start as memory::program_counter;
    }

    //timers decrement at 60hz, called once per frame
    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    //xorshift, enough randomness for games
    fn random_byte(&mut self) -> u8 {
        let mut value = self.random_state;
        value ^= value << 7;
        value ^= value >> 9;
        value ^= value << 8;
        self.random_state = value;
        value as u8
    }

    //one fetch -> decode -> execute cycle
    pub fn emulate_cycle(&mut self) {
        //fetch: each opcode takes 2 bytes
        let pc = self.program_counter as usize;
        let opcode = u16::from_be_bytes([self.memory[pc], self.memory[pc + 1]]);
        self.program_counter += 2;

        //decode: split the opcode into the usual fields
        let nnn = opcode & 0x0FFF; //address
        let nn = (opcode & 0x00FF) as u8; //byte constant
        let n = (opcode & 0x000F) as u8; //nibble constant
        let x = ((opcode & 0x0F00) >> 8) as usize; //register vX
        let y = ((opcode & 0x00F0) >> 4) as usize; //register vY

        //execute
        match opcode & 0xF000 {
            0x0000 => match opcode {
                //00E0 - clear the screen
                0x00E0 => self.display.clear(),
                //00EE - return from routine
                0x00EE => self.program_counter = self.stack.pop(),
                _ => panic!("unknown opcode {:04X}", opcode),
            },
            //1NNN - jump to address
            0x1000 => self.program_counter = nnn,
            //2NNN - call routine
            0x2000 => {
                self.stack.push(self.program_counter);
                self.program_counter = nnn;
            }
            //3XNN - skip next if vX == NN
            0x3000 => {
                if self.registers[x] == nn {
                    self.program_counter += 2;
                }
            }
            //4XNN - skip next if vX != NN
            0x4000 => {
                if self.registers[x] != nn {
                    self.program_counter += 2;
                }
            }
            //5XY0 - skip next if vX == vY
            0x5000 => {
                if self.registers[x] == self.registers[y] {
                    self.program_counter += 2;
                }
            }
            //6XNN - set vX = NN
            0x6000 => self.registers[x] = nn,
            //7XNN - add NN to vX (no carry flag)
            0x7000 => self.registers[x] = self.registers[x].wrapping_add(nn),
            0x8000 => match n {
                //8XY0 - set vX = vY
                0x0 => self.registers[x] = self.registers[y],
                //8XY1 - vX |= vY
                0x1 => self.registers[x] |= self.registers[y],
                //8XY2 - vX &= vY
                0x2 => self.registers[x] &= self.registers[y],
                //8XY3 - vX ^= vY
                0x3 => self.registers[x] ^= self.registers[y],
                //8XY4 - vX += vY, vF is the carry flag
                0x4 => {
                    let (result, carry) = self.registers[x].overflowing_add(self.registers[y]);
                    self.registers[x] = result;
                    self.registers[0xF] = carry as u8;
                }
                //8XY5 - vX -= vY, vF = 1 when there is no borrow
                0x5 => {
                    let (result, borrow) = self.registers[x].overflowing_sub(self.registers[y]);
                    self.registers[x] = result;
                    self.registers[0xF] = (!borrow) as u8;
                }
                //8XY6 - shift right, vF keeps the dropped bit
                0x6 => {
                    let dropped = self.registers[x] & 1;
                    self.registers[x] >>= 1;
                    self.registers[0xF] = dropped;
                }
                //8XY7 - vX = vY - vX, vF = 1 when there is no borrow
                0x7 => {
                    let (result, borrow) = self.registers[y].overflowing_sub(self.registers[x]);
                    self.registers[x] = result;
                    self.registers[0xF] = (!borrow) as u8;
                }
                //8XYE - shift left, vF keeps the dropped bit
                0xE => {
                    let dropped = self.registers[x] >> 7;
                    self.registers[x] <<= 1;
                    self.registers[0xF] = dropped;
                }
                _ => panic!("unknown opcode {:04X}", opcode),
            },
            //9XY0 - skip next if vX != vY
            0x9000 => {
                if self.registers[x] != self.registers[y] {
                    self.program_counter += 2;
                }
            }
            //ANNN - set index pointer
            0xA000 => self.index_pointer = nnn,
            //BNNN - jump to address + v0
            0xB000 => self.program_counter = nnn + self.registers[0] as u16,
            //CXNN - vX = random byte & NN
            0xC000 => self.registers[x] = self.random_byte() & nn,
            //DXYN - draw sprite of N bytes at (vX, vY), vF is the collision flag
            0xD000 => {
                let start = self.index_pointer as usize;
                let sprite = &self.memory[start..start + n as usize];
                let collision = self.display.draw_sprite(
                    self.registers[x] as usize % self.display.width,
                    self.registers[y] as usize % self.display.height,
                    sprite,
                );
                self.registers[0xF] = collision as u8;
            }
            0xE000 => match nn {
                //EX9E - skip next if key vX is pressed
                0x9E => {
                    if self.keyboard.is_pressed(self.registers[x]) {
                        self.program_counter += 2;
                    }
                }
                //EXA1 - skip next if key vX is not pressed
                0xA1 => {
                    if !self.keyboard.is_pressed(self.registers[x]) {
                        self.program_counter += 2;
                    }
                }
                _ => panic!("unknown opcode {:04X}", opcode),
            },
            0xF000 => match nn {
                //FX07 - vX = delay timer
                0x07 => self.registers[x] = self.delay_timer,
                //FX0A - wait for a key press (repeat the opcode until one arrives)
                0x0A => match self.keyboard.first_pressed() {
                    Some(key) => self.registers[x] = key,
                    None => self.program_counter -= 2,
                },
                //FX15 - delay timer = vX
                0x15 => self.delay_timer = self.registers[x],
                //FX18 - sound timer = vX
                0x18 => self.sound_timer = self.registers[x],
                //FX1E - index pointer += vX
                0x1E => {
                    self.index_pointer = self.index_pointer.wrapping_add(self.registers[x] as u16)
                }
                //FX29 - point index to the font sprite of digit vX
                0x29 => {
                    let digit = (self.registers[x] & 0xF) as u16;
                    self.index_pointer = memory::FONTSET_START_ADDRESS as u16 + digit * 5;
                }
                //FX33 - store vX as decimal digits (hundreds, tens, ones)
                0x33 => {
                    let value = self.registers[x];
                    let i = self.index_pointer as usize;
                    self.memory[i] = value / 100;
                    self.memory[i + 1] = (value / 10) % 10;
                    self.memory[i + 2] = value % 10;
                }
                //FX55 - store v0..vX into memory starting at index
                0x55 => {
                    let i = self.index_pointer as usize;
                    self.memory[i..=i + x].copy_from_slice(&self.registers[..=x]);
                }
                //FX65 - load v0..vX from memory starting at index
                0x65 => {
                    let i = self.index_pointer as usize;
                    self.registers[..=x].copy_from_slice(&self.memory[i..=i + x]);
                }
                _ => panic!("unknown opcode {:04X}", opcode),
            },
            _ => panic!("unknown opcode {:04X}", opcode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //load a program and run one cycle per opcode
    fn run(program: &[u8], cycles: usize) -> Chip8 {
        let mut chip8 = Chip8::new();
        chip8.load_program(program);
        for _ in 0..cycles {
            chip8.emulate_cycle();
        }
        chip8
    }

    #[test]
    fn set_and_add_register() {
        //6A05 - vA = 5, 7A03 - vA += 3
        let chip8 = run(&[0x6A, 0x05, 0x7A, 0x03], 2);
        assert_eq!(chip8.registers[0xA], 8);
    }

    #[test]
    fn add_with_carry_flag() {
        //v0 = 0xFF, v1 = 0x02, v0 += v1 -> 0x01 with carry
        let chip8 = run(&[0x60, 0xFF, 0x61, 0x02, 0x80, 0x14], 3);
        assert_eq!(chip8.registers[0], 0x01);
        assert_eq!(chip8.registers[0xF], 1);
    }

    #[test]
    fn call_and_return() {
        //2206 - call 0x206, 0000 padding, 00EE - return
        let chip8 = run(&[0x22, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0xEE], 2);
        assert_eq!(chip8.program_counter, 0x202);
    }

    #[test]
    fn draw_sets_collision_flag() {
        //draw the same font sprite twice at (0,0): all pixels erased -> collision
        let program = [
            0x60, 0x00, //v0 = 0
            0xF0, 0x29, //index = sprite of digit 0
            0xD0, 0x05, //draw 5 bytes at (v0, v0)
            0xD0, 0x05, //draw again on top
        ];
        let mut chip8 = run(&program, 3);
        assert_eq!(chip8.registers[0xF], 0);
        chip8.emulate_cycle();
        assert_eq!(chip8.registers[0xF], 1);
        assert!(chip8.display.pixel.iter().all(|&pixel| !pixel));
    }

    #[test]
    fn store_decimal_digits() {
        //v0 = 234, FX33 stores 2, 3, 4 starting at the index pointer
        let program = [
            0x60, 0xEA, //v0 = 234
            0xA3, 0x00, //index = 0x300
            0xF0, 0x33, //store decimal digits
        ];
        let chip8 = run(&program, 3);
        assert_eq!(chip8.memory[0x300..0x303], [2, 3, 4]);
    }
}
