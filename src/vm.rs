#![allow(non_snake_case)]

use std::fs::File;
use std::io::prelude::*;

pub struct VM {
    I: usize,
    V: [u8; 16],
    fb: [u64; 32],
    pc: usize,
    sp: usize,
    ram: [u8; 0x1000],
    next: usize,
    delay: u8,
    sound: u8,
    stack: [usize; 12],
    keypad: [bool; 16],
    opcode: usize,
    operands: Operands,
}

impl VM {
    pub fn new(rom_file: &str) -> Self {
        let mut ram = init_ram([0; 0x1000]);

        load_rom(rom_file, &mut ram[0x200..]).unwrap();

        Self {
            I: 0,
            V: [0; 16],
            fb: [0; 32],
            pc: 512,
            sp: 0,
            ram: ram,
            next: 514,
            delay: 0,
            sound: 0,
            stack: [0; 12],
            keypad: [false; 16],
            opcode: 0,
            operands: Operands::from(0),
        }
    }

    pub fn get_fb(&self) -> [u64; 32] {
        self.fb
    }

    pub fn update_keys(&mut self, keys: [bool; 16]) {
        self.keypad = keys;
    }

    pub fn cycle(&mut self) {
        self.opcode = self.get_opcode();
        self.operands = Operands::from(self.opcode);
        self.next = self.pc + 2;

        //println!("Executing: {:X}", self.opcode);

        match self.opcode {
            0x00E0 => self.x00E0(),
            0x00EE => self.x00EE(),
            0x1000..=0x1FFF => self.x1NNN(),
            0x2000..=0x2FFF => self.x2NNN(),
            0x3000..=0x3FFF => self.x3XNN(),
            0x4000..=0x4FFF => self.x4XNN(),
            0x5000..=0x5FFF if self.opcode & 0x0F == 0x00 => self.x5XY0(),
            0x6000..=0x6FFF => self.x6XNN(),
            0x7000..=0x7FFF => self.x7XNN(),
            0x8000..=0x8FFF if self.opcode & 0x0F == 0x00 => self.x8XY0(),
            0x8000..=0x8FFF if self.opcode & 0x0F == 0x01 => self.x8XY1(),
            0x8000..=0x8FFF if self.opcode & 0x0F == 0x02 => self.x8XY2(),
            0x8000..=0x8FFF if self.opcode & 0x0F == 0x03 => self.x8XY3(),
            0x8000..=0x8FFF if self.opcode & 0x0F == 0x04 => self.x8XY4(),
            0x8000..=0x8FFF if self.opcode & 0x0F == 0x05 => self.x8XY5(),
            0x8000..=0x8FFF if self.opcode & 0x0F == 0x06 => self.x8XY6(),
            0x8000..=0x8FFF if self.opcode & 0x0F == 0x07 => self.x8XY7(),
            0x8000..=0x8FFF if self.opcode & 0x0F == 0x0E => self.x8XYE(),
            0x9000..=0x9FFF if self.opcode & 0x0F == 0x00 => self.x9XY0(),
            0xA000..=0xAFFF => self.xANNN(),
            0xB000..=0xBFFF => self.xBNNN(),
            0xC000..=0xCFFF => self.xCXNN(),
            0xD000..=0xDFFF => self.xDXYN(),
            0xE000..=0xEFFF if self.opcode & 0xFF == 0x9E => self.xEX9E(),
            0xE000..=0xEFFF if self.opcode & 0xFF == 0xA1 => self.xEXA1(),
            0xF000..=0xFFFF if self.opcode & 0xFF == 0x07 => self.xFX07(),
            0xF000..=0xFFFF if self.opcode & 0xFF == 0x0A => self.xFX0A(),
            0xF000..=0xFFFF if self.opcode & 0xFF == 0x15 => self.xFX15(),
            0xF000..=0xFFFF if self.opcode & 0xFF == 0x18 => self.xFX18(),
            0xF000..=0xFFFF if self.opcode & 0xFF == 0x1E => self.xFX1E(),
            0xF000..=0xFFFF if self.opcode & 0xFF == 0x29 => self.xFX29(),
            0xF000..=0xFFFF if self.opcode & 0xFF == 0x33 => self.xFX33(),
            0xF000..=0xFFFF if self.opcode & 0xFF == 0x55 => self.xFX55(),
            0xF000..=0xFFFF if self.opcode & 0xFF == 0x65 => self.xFX65(),
            _ => panic!("Unknown opcode: {:X}", self.opcode),
        };

        self.pc = self.next & 0xFFF;
    }

    pub fn tick(&mut self) {
        if self.delay > 0 {
            self.delay -= 1;
        }

        if self.sound > 0 {
            self.sound -= 1;
        }
    }

    pub fn is_drawing(&self) -> bool {
        (0xD000..=0xDFFF).contains(&self.opcode)
    }

    fn get_opcode(&self) -> usize {
        let pc = self.pc;
        let left = self.ram[pc];
        let right = self.ram[(pc + 1) & 0xFFF];

        u16::from_be_bytes([left, right]) as usize
    }

    fn push(&mut self, x: usize) {
        self.stack[self.sp] = x;
        self.sp += 1;
        self.sp &= 0o7;
    }

    fn pop(&mut self) -> usize {
        self.sp = usize::wrapping_sub(self.sp, 1) & 0o7;

        self.stack[self.sp]
    }

    fn x00E0(&mut self) {
        self.fb = [0; 32];
    }

    fn x00EE(&mut self) {
        let x = self.pop();
        self.next = x;
    }

    fn x1NNN(&mut self) {
        self.next = self.operands.NNN;
    }

    fn x2NNN(&mut self) {
        self.push(self.pc + 2);
        self.x1NNN();
    }

    fn x3XNN(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let NN = operands.NN;

        if self.V[X] == NN {
            self.next += 2;
        }
    }

    fn x4XNN(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let NN = operands.NN;

        if self.V[X] != NN {
            self.next += 2;
        }
    }

    fn x5XY0(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        if self.V[X] == self.V[Y] {
            self.next += 2;
        }
    }

    fn x6XNN(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let NN = operands.NN;

        self.V[X] = NN;
    }

    fn x7XNN(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let NN = operands.NN;

        self.V[X] = u8::wrapping_add(self.V[X], NN);
    }

    fn x8XY0(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        self.V[X] = self.V[Y];
    }

    fn x8XY1(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        self.V[X] |= self.V[Y];
    }

    fn x8XY2(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        self.V[X] &= self.V[Y];
    }

    fn x8XY3(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        self.V[X] ^= self.V[Y];
    }

    fn x8XY4(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        if let Some(sum) = u8::checked_add(self.V[X], self.V[Y]) {
            self.V[X] = sum;
            self.V[0xF] = 0;
        } else {
            self.V[X] = u8::wrapping_add(self.V[X], self.V[Y]);
            self.V[0xF] = 1;
        }
    }

    fn x8XY5(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        if let Some(diff) = u8::checked_sub(self.V[X], self.V[Y]) {
            self.V[X] = diff;
            self.V[0xF] = 1;
        } else {
            self.V[X] = u8::wrapping_sub(self.V[X], self.V[Y]);
            self.V[0xF] = 0;
        }
    }

    #[cfg(feature = "quirky_shift")]
    fn x8XY6(&mut self) {
        let operands = self.operands;
        let X = operands.X;

        self.V[0xF] = self.V[X] & 1;
        self.V[X] = self.V[X] >> 1;
    }

    #[cfg(not(feature = "quirky_shift"))]
    fn x8XY6(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        self.V[0xF] = self.V[Y] & 1;
        self.V[Y] = self.V[Y] >> 1;
        self.V[X] = self.V[Y];
    }

    fn x8XY7(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        if let Some(diff) = u8::checked_sub(self.V[Y], self.V[X]) {
            self.V[X] = diff;
            self.V[0xF] = 1;
        } else {
            self.V[X] = u8::wrapping_sub(self.V[Y], self.V[X]);
            self.V[0xF] = 0;
        }
    }

    #[cfg(feature = "quirky_shift")]
    fn x8XYE(&mut self) {
        let operands = self.operands;
        let X = operands.X;

        self.V[0xF] = self.V[X] >> 7;
        self.V[X] = self.V[X] << 1;
    }

    #[cfg(not(feature = "quirky_shift"))]
    fn x8XYE(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        self.V[0xF] = self.V[Y] >> 7;
        self.V[Y] = self.V[Y] << 1;
        self.V[X] = self.V[Y];
    }

    fn x9XY0(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;

        if self.V[X] != self.V[Y] {
            self.next += 2;
        }
    }

    fn xANNN(&mut self) {
        self.I = self.operands.NNN;
    }

    fn xBNNN(&mut self) {
        self.next = self.operands.NNN + self.V[0] as usize;
    }

    fn xCXNN(&mut self) {
        self.V[self.operands.X] = rand::random::<u8>() & self.operands.NN;
    }

    fn xDXYN(&mut self) {
        let operands = self.operands;
        let X = operands.X;
        let Y = operands.Y;
        let N = operands.N;

        let x = self.V[X] % 64;
        let y = self.V[Y] % 32;

        let height = std::cmp::min(N as u8, 32 - y);

        let mut unset = false;

        for i in 0..height {
            let old = self.fb[(y + i) as usize];

            self.fb[(y + i) as usize] ^=
                (self.ram[(self.I + i as usize) & 0xFFF] as u64) << 56 >> x;

            if old & self.fb[(y + i) as usize] < old {
                unset = true;
            }
        }

        self.V[0xF] = if unset { 1 } else { 0 };
    }

    fn xEX9E(&mut self) {
        let X = self.operands.X;

        if self.keypad[self.V[X] as usize % 16] {
            self.next += 2;
        }
    }

    fn xEXA1(&mut self) {
        let X = self.operands.X;

        if !self.keypad[self.V[X] as usize % 16] {
            self.next += 2;
        }
    }

    fn xFX07(&mut self) {
        let X = self.operands.X;

        self.V[X] = self.delay;
    }

    fn xFX0A(&mut self) {
        let X = self.operands.X;

        for key in 0x0..=0xF {
            if self.keypad[key] {
                self.V[X] = key as u8;
                return;
            }
        }

        self.next = usize::wrapping_sub(self.next, 2);
    }

    fn xFX15(&mut self) {
        let X = self.operands.X;

        self.delay = self.V[X];
    }

    fn xFX18(&mut self) {
        let X = self.operands.X;

        self.sound = self.V[X];
    }

    fn xFX1E(&mut self) {
        let X = self.operands.X;

        self.I += self.V[X] as usize;
        self.I &= 0xFFF;
    }

    fn xFX29(&mut self) {
        let X = self.operands.X;

        self.I = self.V[X] as usize * 5;
    }

    fn xFX33(&mut self) {
        let X = self.operands.X;

        self.ram[(self.I + 2) & 0xFFF] = self.V[X] % 10;
        self.ram[(self.I + 1) & 0xFFF] = (self.V[X] / 10) % 10;
        self.ram[self.I] = (self.V[X] / 100) % 10;
    }

    #[cfg(feature = "quirky_restore")]
    fn xFX55(&mut self) {
        let X = self.operands.X;

        for i in 0..=X {
            self.ram[self.I + i] = self.V[i];
        }
    }

    #[cfg(not(feature = "quirky_restore"))]
    fn xFX55(&mut self) {
        let X = self.operands.X;

        for i in 0..=X {
            self.ram[self.I + i] = self.V[i];
        }

        self.I += X + 1;
        self.I &= 0xFFF;
    }

    #[cfg(feature = "quirky_restore")]
    fn xFX65(&mut self) {
        let X = self.operands.X;

        for i in 0..=X {
            self.V[i] = self.ram[self.I + i];
        }
    }

    #[cfg(not(feature = "quirky_restore"))]
    fn xFX65(&mut self) {
        let X = self.operands.X;

        for i in 0..=X {
            self.V[i] = self.ram[self.I + i];
        }

        self.I += X + 1;
        self.I &= 0xFFF;
    }
}

#[derive(Clone, Copy)]
struct Operands {
    X: usize,
    Y: usize,
    N: usize,
    NN: u8,
    NNN: usize,
}

impl Operands {
    const fn from(opcode: usize) -> Self {
        Self {
            X: (opcode >> 8) & 0xF,
            Y: (opcode >> 4) & 0xF,
            N: (opcode >> 0) & 0xF,
            NN: opcode as u8,
            NNN: opcode & 0xFFF,
        }
    }
}

fn load_rom(rom_file: &str, rom: &mut [u8]) -> std::io::Result<()> {
    File::open(rom_file)?.read(rom)?;

    Ok(())
}

const fn init_ram(mut ram: [u8; 0x1000]) -> [u8; 0x1000] {
    // 0
    ram[00] = 0b11110000;
    ram[01] = 0b10010000;
    ram[02] = 0b10010000;
    ram[03] = 0b10010000;
    ram[04] = 0b11110000;

    // 1
    ram[05] = 0b00100000;
    ram[06] = 0b01100000;
    ram[07] = 0b00100000;
    ram[08] = 0b00100000;
    ram[09] = 0b01110000;

    // 2
    ram[10] = 0b11110000;
    ram[11] = 0b00010000;
    ram[12] = 0b11110000;
    ram[13] = 0b10000000;
    ram[14] = 0b11110000;

    // 3
    ram[15] = 0b11110000;
    ram[16] = 0b00010000;
    ram[17] = 0b11110000;
    ram[18] = 0b00010000;
    ram[19] = 0b11110000;

    // 4
    ram[20] = 0b10010000;
    ram[21] = 0b10010000;
    ram[22] = 0b11110000;
    ram[23] = 0b00010000;
    ram[24] = 0b00010000;

    // 5
    ram[25] = 0b11110000;
    ram[26] = 0b10000000;
    ram[27] = 0b11110000;
    ram[28] = 0b00010000;
    ram[29] = 0b11110000;

    // 6
    ram[30] = 0b11110000;
    ram[31] = 0b10000000;
    ram[32] = 0b11110000;
    ram[33] = 0b10010000;
    ram[34] = 0b11110000;

    // 7
    ram[35] = 0b11110000;
    ram[36] = 0b00010000;
    ram[37] = 0b00100000;
    ram[38] = 0b01000000;
    ram[39] = 0b01000000;

    // 8
    ram[40] = 0b11110000;
    ram[41] = 0b10010000;
    ram[42] = 0b11110000;
    ram[43] = 0b10010000;
    ram[44] = 0b11110000;

    // 9
    ram[45] = 0b11110000;
    ram[46] = 0b10010000;
    ram[47] = 0b11110000;
    ram[48] = 0b00010000;
    ram[49] = 0b11110000;

    // A
    ram[50] = 0b11110000;
    ram[51] = 0b10010000;
    ram[52] = 0b11110000;
    ram[53] = 0b10010000;
    ram[54] = 0b10010000;

    // B
    ram[55] = 0b11100000;
    ram[56] = 0b10010000;
    ram[57] = 0b11100000;
    ram[58] = 0b10010000;
    ram[59] = 0b11100000;

    // C
    ram[60] = 0b11110000;
    ram[61] = 0b10000000;
    ram[62] = 0b10000000;
    ram[63] = 0b10000000;
    ram[64] = 0b11110000;

    // D
    ram[65] = 0b11100000;
    ram[66] = 0b10010000;
    ram[67] = 0b10010000;
    ram[68] = 0b10010000;
    ram[69] = 0b11100000;

    // E
    ram[70] = 0b11110000;
    ram[71] = 0b10000000;
    ram[72] = 0b11110000;
    ram[73] = 0b10000000;
    ram[74] = 0b11110000;

    // F
    ram[75] = 0b11110000;
    ram[76] = 0b10000000;
    ram[77] = 0b11110000;
    ram[78] = 0b10000000;
    ram[79] = 0b10000000;

    ram
}
