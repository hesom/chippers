use std::{fs, path::Path};

use rand::Rng;

pub struct Chip8 {
    pub gfx: [u8; 64 * 32],
    memory: Memory,
    stack: [u16; 16],
    sp: u16,
    v: [u8; 16],
    i: u16,
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
    pub key: [u8; 16],
    pub draw_flag: bool,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            gfx: [0; 64 * 32],
            memory: Memory::new(),
            stack: [0; 16],
            sp: 0,
            v: [0; 16],
            i: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            key: [0; 16],
            draw_flag: false,
        }
    }

    pub fn cycle(&mut self) {
        let opcode = (self.memory.read(self.pc) as u16) << 8 | self.memory.read(self.pc + 1) as u16;

        if opcode == 0x0000 {
            return;
        }

        match opcode & 0xF000 {
            0x0000 => {
                match opcode & 0x00FF {
                    0x00E0 => {
                        // Clear screen
                        self.gfx = [0; 64 * 32];
                        self.draw_flag = true;
                        self.pc += 2;
                    }
                    0x00EE => {
                        // Return from subroutine
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }
                    _ => panic!("Not implemented"),
                }
            }
            0x1000 => {
                // Jump to address
                self.pc = opcode & 0x0FFF;
            }
            0x2000 => {
                self.stack[self.sp as usize] = self.pc + 2;
                self.sp += 1;
                self.pc = opcode & 0x0FFF;
            }
            0x3000 => {
                let x = (opcode & 0x0F00) >> 8;
                let val = (opcode & 0x00FF) as u8;
                if self.v[x as usize] == val {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // Skip instruction
                let x = (opcode & 0x0F00) >> 8;
                let val = (opcode & 0x00FF) as u8;
                if self.v[x as usize] != val {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x5000 => {
                // Skip instruction
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // Sets VX to NN
                let x = (opcode & 0x0F00) >> 8;
                let val = (opcode & 0x00FF) as u8;
                self.v[x as usize] = val;
                self.pc += 2;
            }
            0x7000 => {
                // Adds NN to VX without carry
                let x = (opcode & 0x0F00) >> 8;
                let val = (opcode & 0x00FF) as u8;
                self.v[x as usize] = self.v[x as usize].wrapping_add(val);
                self.pc += 2;
            }
            0x8000 => {
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;

                match opcode & 0x000F {
                    0x0000 => {
                        self.v[x as usize] = self.v[y as usize];
                        self.pc += 2;
                    }
                    0x0001 => {
                        self.v[x as usize] |= self.v[y as usize];
                        self.pc += 2;
                    }
                    0x0002 => {
                        self.v[x as usize] &= self.v[y as usize];
                        self.pc += 2;
                    }
                    0x0003 => {
                        self.v[x as usize] ^= self.v[y as usize];
                        self.pc += 2;
                    }
                    0x0004 => {
                        let prev = self.v[x as usize];
                        self.v[x as usize] = self.v[x as usize].wrapping_add(self.v[y as usize]);
                        if self.v[x as usize] < prev {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.pc += 2;
                    }
                    0x0005 => {
                        let carry = if self.v[x as usize] >= self.v[y as usize] {
                            1
                        } else {
                            0
                        };
                        self.v[x as usize] = self.v[x as usize].wrapping_sub(self.v[y as usize]);
                        self.v[0xF] = carry;
                        self.pc += 2;
                    }
                    0x0006 => {
                        let lsb = self.v[y as usize] & 0x01;
                        self.v[x as usize] = self.v[y as usize] >> 1;
                        self.v[0xF] = lsb;
                        self.pc += 2;
                    }
                    0x0007 => {
                        let carry = if self.v[y as usize] >= self.v[x as usize] {
                            1
                        } else {
                            0
                        };
                        self.v[x as usize] = self.v[y as usize].wrapping_sub(self.v[x as usize]);
                        self.v[0xF] = carry;
                        self.pc += 2;
                    }
                    0x000E => {
                        let msb = self.v[y as usize] >> 7;
                        self.v[x as usize] = self.v[y as usize] << 1;
                        self.v[0xF] = msb;
                        self.pc += 2;
                    }
                    _ => panic!("Not implemented"),
                }
            }
            0x9000 => {
                // Skips next instruction based on test
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0xA000 => {
                // Sets I to address of NNN
                self.i = opcode & 0x0FFF;
                self.pc += 2;
            }
            0xB000 => {
                // Jumps to address of NNN plus V0
                self.pc = opcode & 0x0FFF + self.v[0x0] as u16;
            }
            0xC000 => {
                // Random number operation
                let x = (opcode & 0x0F00) >> 8;
                let val = (opcode & 0x00FF) as u8;
                let mut rng = rand::thread_rng();
                let r = rng.gen_range(0..=255);

                self.v[x as usize] = val & r;
                self.pc += 2;
            }
            0xD000 => {
                // Draws a sprite
                let x = (opcode & 0x0F00) >> 8;
                let y = (opcode & 0x00F0) >> 4;
                let n = (opcode & 0x000F) as u16;
                let x = self.v[x as usize] as u16;
                let y = self.v[y as usize] as u16;
                self.v[0xF] = 0;

                for y_line in 0..n {
                    let pixel = self.memory.read(self.i + y_line as u16);
                    for x_line in 0..8 {
                        if pixel & (0x80 >> x_line) != 0 {
                            if self.gfx[(x + x_line + ((y + y_line) * 64)) as usize] == 1 {
                                self.v[0xF] = 1;
                            }
                            self.gfx[(x + x_line + ((y + y_line) * 64)) as usize] ^= 1;
                        }
                    }
                }
                self.draw_flag = true;
                self.pc += 2;
            }
            0xE000 => {
                let x = (opcode & 0x0F00) >> 8;
                match opcode & 0x00FF {
                    0x009E => {
                        if self.key[self.v[x as usize] as usize] != 0 {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => {
                        if self.key[self.v[x as usize] as usize] == 0 {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    _ => panic!("Not implemented"),
                }
            }
            0xF000 => {
                let x = (opcode & 0x0F00) >> 8;
                match opcode & 0x00FF {
                    0x0007 => {
                        self.v[x as usize] = self.delay_timer;
                        self.pc += 2;
                    }
                    0x000A => {
                        self.v[x as usize] = 1;
                        self.pc += 2;
                        panic!("Implement blocking")
                    }
                    0x0015 => {
                        self.delay_timer = self.v[x as usize];
                        self.pc += 2;
                    }
                    0x0018 => {
                        self.sound_timer = self.v[x as usize];
                        self.pc += 2;
                    }
                    0x001E => {
                        self.i = self.i.wrapping_add(self.v[x as usize] as u16);
                        self.pc += 2;
                    } // Add to I
                    0x0029 => {
                        self.i = 5 * (self.v[x as usize] as u16);
                        self.pc += 2;
                    }
                    0x0033 => {
                        self.memory.write(self.i, self.v[x as usize] / 100);
                        self.memory
                            .write(self.i + 1, (self.v[x as usize] / 10) % 10);
                        self.memory
                            .write(self.i + 2, (self.v[x as usize] % 100) % 10);
                        self.pc += 2;
                    }
                    0x0055 => {
                        for p in 0..=x {
                            self.memory.write(self.i + p, self.v[p as usize]);
                        }
                        self.pc += 2;
                    }
                    0x0065 => {
                        for p in 0..=x {
                            self.v[p as usize] = self.memory.read(self.i + p);
                        }
                        self.pc += 2;
                    }
                    _ => panic!("Not implemented"),
                }
            }
            _ => (),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn load_raw(&mut self, rom: &Vec<u16>) {
        let mut p = 0;
        for i in 0..rom.len() {
            let opcode = rom[i];
            let left = ((opcode & 0xFF00) >> 8) as u8;
            let right = (opcode & 0x00FF) as u8;
            self.memory.write(0x200 + p, left);
            self.memory.write(0x200 + p + 1, right);
            p += 2;
        }
    }

    pub fn load_rom<P>(&mut self, path: P)
    where
        P: AsRef<Path>,
    {
        let rom = fs::read(path.as_ref()).expect("ROM should be readable");
        for i in 0..rom.len() {
            self.memory.write(0x200 + i as u16, rom[i]);
        }
    }
}

pub struct Memory {
    data: [u8; 0x1000],
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = Self { data: [0; 0x1000] };

        let fontset: [u8; 80] = [
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

        for i in 0..80 {
            memory.write(i, fontset[i as usize]);
        }

        memory
    }

    pub fn read(&self, loc: u16) -> u8 {
        self.data[loc as usize]
    }

    pub fn write(&mut self, loc: u16, val: u8) {
        self.data[loc as usize] = val;
    }
}
