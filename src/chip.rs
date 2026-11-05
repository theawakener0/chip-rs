use std::{u8, u16};

use rand::Rng;

pub struct Chip {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: [u16; 16],
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    display: [[bool; 64]; 32],
    keys: [bool; 16],
    draw_flag: bool,
    waiting_for_key: Option<usize>
}

impl Chip {
    pub fn new() -> Self {
        let mut chip = Chip {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            display: [[false; 64]; 32],
            keys: [false; 16],
            draw_flag: false,
            waiting_for_key: None
        };

        chip.memory[0x50..0x50 + FONTSET.len()].copy_from_slice(&FONTSET); 
        chip
    }
    pub fn load_rom(&mut self, data: &[u8]) {
        let start = 0x200;
        let end = start + data.len();
        if end > 4096 {
            panic!("ROM too large")
        }

        self.memory[start..end].copy_from_slice(data);
        self.pc = 0x200;
    }
    pub fn cycle(&mut self) {
        if let Some(reg) = self.waiting_for_key {
            for (k, pressed) in self.keys.iter().enumerate() {
                if *pressed {
                    self.v[reg] = k as u8;
                    self.waiting_for_key = None;
                    self.pc += 2;
                    break;
                }
            }
            return;
        }

        let opcode = self.fetch();
        
    }
    pub fn tick_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            print!("\x07")
        }
    }

    fn fetch(&mut self) -> u16 {
        let hi = self.memory[self.pc as usize] as u16;
        let lo = self.memory[self.pc as usize + 1] as u16;
        let opcode = (hi << 8) | lo;
        self.pc += 2;

        opcode
    }

    fn execute(&mut self, opcode: u16) {
        let x = ((opcode & 0xF00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;
    }
}

const FONTSET: [u8; 80] = [
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

