use rand::Rng;
use std::io::BufReader;

const RAM_SIZE: usize = 4096; // 4KB
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
pub const SCREEN_WIDTH: usize = 64; // chip8 standard width resulution => 64
pub const SCREEN_HEIGHT: usize = 32; // chip8 standard height resulution => 32
const START_ADDR: u16 = 0x200; // 512'th index, from where the program starts

// commonly used characters
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Emulator {
    program_counter: u16, // keep track of the current program instruction
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_registers: [u8; NUM_REGISTERS], // used by the game because its faster than reading from RAM
    i_register: u16, // used for indexing into RAM reads and writes
    stack_pointer: u16, // keeps track of the top of the stack
    stack: [u16; STACK_SIZE], // works on the "Lat in, first out" principe
    keys: [bool; NUM_KEYS], // keeps track of which keys are pressed
    delay_timer: u8, // used as a timer, performing an action when it hits 0
    sound_timer: u8, // counts down every cycle, emitting a noise when it hits 0
}

impl Emulator {
    pub fn new() -> Self {
        let mut new_emulator = Self {
            program_counter: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_registers: [0; NUM_REGISTERS],
            i_register: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        };

        // load the defualt characters into ram
        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emulator

    }

    pub fn reset() -> Self {
        Self::new()
    }

    pub fn tick(&mut self) {
        let operation = self.fetch();

        self.execute(operation);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn execute(&mut self, operation: u16) {
        let digit1 = (operation & 0xF000) >> 12;
        let digit2 = (operation & 0x0F00) >> 8;
        let digit3 = (operation & 0x00F0) >> 4;
        let digit4 = operation & 0x000F;

        // match opcodes
        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return,
            // 00E0 => clear display
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // 00EE => return from a subroutine
            (0, 0, 0xE, 0xE) => {
                // get current stack pointer
                let subroutine_address = self.pop();
                // get back to the current stack pointer
                self.program_counter = subroutine_address;
            },
            // 1NNN => jump to an address NNN
            (1,_,_,_) => {
                let nnn = operation & 0xFFF;
                self.program_counter = nnn;
            },
            // 2NNN => calls subroutine at NNN
            (2,_,_,_) => {
                let nnn = operation & 0xFFF;
                // return to current step after
                self.push(self.program_counter);
                // go to address
                self.program_counter = nnn
            },
            // 3XNN => skip next operation if vX == NN
            (3,_,_,_) => {
                let x: usize = digit2 as usize;
                let nn = (operation & 0xFF) as u8;

                if self.v_registers[x] == nn {
                    // skip 1 operation
                    self.program_counter += 2;
                }
            },
            // 4XNN => skip next operation if vX != NN
            (4,_,_,_) => {
                let x = digit2 as usize;
                let nn = (operation & 0xFF) as u8;
                if self.v_registers[x] != nn {
                    // skip 1 operation
                    self.program_counter += 2;
                }
            },
            // 5XY0 => skip next operation if vX == vY
            (5,_,_,0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_registers[x] == self.v_registers[y] {
                    // skip next operation
                    self.program_counter += 2;
                }
            },
            // 6XNN => set vX to NN
            (6,_,_,_) => {
                let x = digit2 as usize;
                let nn = (operation & 0xFF) as u8;

                self.v_registers[x] = nn;
            },
            // 7XNN => add vX to nn
            (7,_,_,_) => {
                let x = digit2 as usize;
                let nn = (operation & 0xFF) as u8;

                // use .wrapping_add instead if .add because nn may overflow resulting in panic (crash)
                self.v_registers[x] = self.v_registers[x].wrapping_add(nn);
            },
            // 8XY0 => sets vX to value of xY
            (8,_,_,0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_registers[x] = self.v_registers[y];
            },
            // 8XY1 => sets vX to the result of vX |= vY
            (8,_,_,1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_registers[x] |= self.v_registers[y];
            },
            // 8XY2 => sets vX to the result of vX &= vY
            (8,_,_,2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_registers[x] &= self.v_registers[y];
            },
            // 8XY3 => sets vX to the result of vX ^= vY
            (8,_,_,3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_registers[x] ^= self.v_registers[y];
            },
            // 8XY4 => Adds vY to vX, sets vF
            (8,_,_,4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_x,overflowed) = self.v_registers[x].overflowing_add(self.v_registers[y]);
                self.v_registers[x] = new_x;

                if overflowed {
                    self.v_registers[0xF] = 1
                } else {
                    self.v_registers[0xF] = 0
                }
            },
             // 8XY5 => subtracts vY from vX, sets vF
            (8,_,_,5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (nex_x,overflowed) = self.v_registers[x].overflowing_sub(self.v_registers[y]);
                self.v_registers[x] = nex_x;

                if overflowed {
                    self.v_registers[0xF] = 0;
                } else {
                    self.v_registers[0xF] = 1;
                }
            },
             // 8XY6 => shifts vX one bit to the right, and sets vF
            (8,_,_,6) => {
                let x = digit2 as usize;
                let least_significant_bit = self.v_registers[x] & 1;

                self.v_registers[x] >>= 1;
                self.v_registers[0xF] = least_significant_bit;
            },
            // 8XY7 => subtracts vX from vY, sets vF
            (8,_,_,7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (nex_x,overflowed) = self.v_registers[y].overflowing_sub(self.v_registers[x]);
                self.v_registers[x] = nex_x;

                if overflowed {
                    self.v_registers[0xF] = 0;
                } else {
                    self.v_registers[0xF] = 1;
                }
            },
            // 8XYE => shifts vX one bit to the left, and sets vF
            (8,_,_,0xE) => {
                let x = digit2 as usize;
                let most_significat_bit = (self.v_registers[x] >> 7) & 1;

                self.v_registers[x] <<= 1;
                self.v_registers[0xF] = most_significat_bit;
            },
            // 9XY0 => skip next option if vX != vY
            (9,_,_,0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_registers[x] != self.v_registers[y] {
                    self.program_counter += 2;
                }
            },
            // ANNN => sets i to nnn
            (0xA,_,_,_) => {
                let nnn = operation & 0xFFF;
                self.i_register = nnn;
            },
            // BNNN => jump to the address of nnn + v[0]
            (0xB,_,_,_) => {
                let nnn: u16 = operation & 0xFFF;
                self.program_counter = nnn + (self.v_registers[0] as u16);
            },
            // CXNN => set vx to a random value masked (bitwise AND) with NN
            (0xC,_,_,_) => {
                let x = digit2 as usize;
                let nn = operation & 0xFF;
                let random_number:u8 = rand::thread_rng().gen();

                self.v_registers[x] = random_number & (nn as u8);
            },
            // DXYN => Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction. As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen
            (0xD,_,_,_) => {
                // get cords
                let x_cord = self.v_registers[digit2 as usize] as u16;
                let y_cord = self.v_registers[digit3 as usize] as u16;

                // number of rows is the last digit
                let rows = digit4;
                // track the flipped flag
                let mut flipped = false;

                for row in 0..rows {
                    let row_address = self.i_register + row as u16;
                    let pixels = self.ram[row_address as usize];

                    // 0..8 because each sprite width is 8px
                    for col in 0..8 {
                        // get current pixel's bit, flip if its a 1, do nothing if its a 0
                        // cant exactly understand how the logic below works (hard copy)
                        let is_flipped = pixels & (0b1000_0000 >> col) != 0;
                        if is_flipped {
                            // redraw
                            let x = (x_cord + col) as usize % SCREEN_WIDTH;
                            let y = (y_cord + row) as usize % SCREEN_HEIGHT;

                            // Get our pixel's index for our 1D screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_registers[0xF] = 1;
                } else {
                    self.v_registers[0xF] = 0;
                }

            },
            // EX9E => skip on key press
            (0xE,_,9,0xE) => {
                let x = digit2 as usize;
                let key_pressed = self.keys[self.v_registers[x] as usize];

                if key_pressed {
                    self.program_counter += 2;
                }
            },
            // EXA1 => skip if key is not pressed
            (0xE,_,0xA,1) => {
                let x = digit2 as usize;
                let key_pressed = self.keys[self.v_registers[x] as usize];

                if !key_pressed {
                    self.program_counter += 2;
                }
            },
            // FX07 => sets delay timer
            (0xF,_,0,7) => {
                let x = digit2 as usize;
                self.v_registers[x] = self.delay_timer;
            },
            // FX0A => wait for key press
            (0xF,_,0,0xA) => {
                let x = digit2 as usize;
                let mut key_pressed = false;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_registers[x] = i as u8;
                        key_pressed = true;
                        break;
                    }
                }

                // using this flag because in a loop our code would not be able to process a key press, turning into infinity loop (from guide)
                if !key_pressed {
                    // retry opcode
                    self.program_counter -= 2;
                }
            },
            // FX15 => set delay timer to vX
            (0xF,_,1,5) => {
                let x = digit2 as usize;
                self.delay_timer = self.v_registers[x];
            },
            // FX18 => set sound timer to vX
            (0xF,_,1,8) => {
                let x = digit2 as usize;
                self.sound_timer = self.v_registers[x];
            },
            // FX1E => adds vX to I
            (0xF,_,1,0xE) => {
                let x = digit2 as usize;
                self.i_register = self.i_register.wrapping_add(self.v_registers[x] as u16);
            },
            // FX29 => sets I to font address
            (0xF,_,2,9) => {
                let x = digit2 as usize;
                let character_position = self.v_registers[x] as u16;

                // times 5 because each font is 5 bytes each
                self.i_register = character_position * 5;
            },
            // FX33 => Stores the binary-coded decimal representation of VX, with the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2
            (0xF,_,3,3) => {
                let x = digit2 as usize;
                let v_x = self.v_registers[x] as f32;

                let hundreds = (v_x / 100.0).floor();
                let tens = ((v_x / 10.0) % 10.0).floor();
                let ones = (v_x % 10.0).floor();

                self.ram[self.i_register as usize] = hundreds as u8;
                self.ram[(self.i_register + 1) as usize] = tens as u8;
                self.ram[(self.i_register + 2) as usize] = ones as u8;
            },
            // FX55 => Stores from V0 to VX (including VX) in memory, starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified
            (0xF,_,5,5) => {
                let x = digit2 as usize;
                let offset = self.i_register as usize;

                // ..= (including vX)
                for i in 0..=x {
                    // store in memory (ram)
                    self.ram[offset + i] = self.v_registers[i];
                }
            },
            // FX65 => Fills from V0 to VX (including VX) with values from memory, starting at address I. The offset from I is increased by 1 for each value read, but I itself is left unmodified
            (0xF,_,6,5) => {
                let x = digit2 as usize;
                let offset = self.i_register as usize;

                // ..= (including vX)
                for i in 0..=x {
                    // store in memory (ram)
                    self.v_registers[i] = self.ram[offset + i];
                }
            },
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", operation),
        }
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -=1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                self.beep();
            }
            self.sound_timer -=1;
        }

    }

    fn beep(&mut self) {
        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();

        let current_dir =  std::env::current_dir().unwrap();
        let sound_relative_path = String::from("chip8_core/sounds/beep.wav");
        let sound_absoulte_path = format!("{}/{sound_relative_path}",current_dir.display());

        let file = match std::fs::File::open(sound_absoulte_path.to_string()) {
            Ok(file) => file,
            Err(_) => {
                println!("Unexpected Error opening the sound file");
                return;
            }
        };

        sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
        sink.set_speed(8.0);
        sink.sleep_until_end();
    }

    fn fetch(&mut self) -> u16 {
        // get current operation take 2 because each ram item is 8 bytes
        let higher_byte = self.ram[self.program_counter as usize] as u16;
        let lower_byte = self.ram[(self.program_counter + 1) as usize] as u16;

        // update program position
        self.program_counter += 2;

        let operation = (higher_byte << 8) | lower_byte; // Combines the bytes into one

        // return operation
        operation

    }

    fn push(&mut self, val: u16) {
        self.stack[self.stack_pointer as usize] = val;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }
}
