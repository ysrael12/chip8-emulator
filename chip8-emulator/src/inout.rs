//constants of display
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const KEY_COUNT: usize = 16;

//fontset
pub const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Display {
    pub width: usize,
    pub height: usize,
    pub pixel: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pub modify_display: bool,
}

impl Display {
    pub fn new() -> Self {
        Self {
            width: DISPLAY_WIDTH,
            height: DISPLAY_HEIGHT,
            pixel: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            modify_display: false,
        }
    }

    //opcode 00E0 clears the screen
    pub fn clear(&mut self) {
        self.pixel = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        self.modify_display = true;
    }

    //draw a sprite with xor, returns true if any pixel was erased (collision)
    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;
        for (row, byte) in sprite.iter().enumerate() {
            for bit in 0..8 {
                if byte & (0x80 >> bit) != 0 {
                    //sprites wrap around the screen edges
                    let px = (x + bit) % self.width;
                    let py = (y + row) % self.height;
                    let index = py * self.width + px;
                    if self.pixel[index] {
                        collision = true;
                    }
                    self.pixel[index] ^= true;
                }
            }
        }
        self.modify_display = true;
        collision
    }
}

pub struct Keyboard {
    pub key: [bool; KEY_COUNT],
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            key: [false; KEY_COUNT],
        }
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.key[key as usize]
    }

    //returns the first pressed key, if any (used by opcode FX0A)
    pub fn first_pressed(&self) -> Option<u8> {
        self.key.iter().position(|&pressed| pressed).map(|i| i as u8)
    }
}

pub type Timer = u8;

//windows system beep, no audio library needed
#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn Beep(frequency: u32, duration_ms: u32) -> i32;
}

//blocks the current thread while beeping, call it from a separate thread
pub fn beep(duration_ms: u32) {
    #[cfg(windows)]
    unsafe {
        Beep(440, duration_ms);
    }
    #[cfg(not(windows))]
    let _ = duration_ms; //no sound on other platforms
}
