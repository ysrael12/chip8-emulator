#![allow(non_camel_case_types)]

// Chip8 memory size constants
pub const MEMORY_SIZE: usize = 4096;
pub const START_ADDRESS: usize = 0x200;
pub const STACK_SIZE: usize = 16;
pub const FONTSET_START_ADDRESS: usize = 0x50;

pub type Memory = [u8; MEMORY_SIZE];

// routines and stack
pub struct Stack {
    stack: [u16; STACK_SIZE],
    pub top: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stack: [0; STACK_SIZE],
            top: 0,
        }
    }

    //save the return address before jumping into a routine
    pub fn push(&mut self, address: u16) {
        if self.top >= STACK_SIZE {
            panic!("stack overflow: more than {} nested calls", STACK_SIZE);
        }
        self.stack[self.top] = address;
        self.top += 1;
    }

    //recover the return address when the routine ends
    pub fn pop(&mut self) -> u16 {
        if self.top == 0 {
            panic!("stack underflow: return without a call");
        }
        self.top -= 1;
        self.stack[self.top]
    }
}

pub type registers = [u8; 16];

pub type index_pointer = u16;

pub type program_counter = u16;
