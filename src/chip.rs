use rand::RngExt;

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
        self.execute(opcode);
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

    // ---- opcode ----

    fn op_00e0(&mut self) {
        self.display = [[false; 64]; 32];
        self.draw_flag = true;
    }
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
    }
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = nnn;
    }
    fn op_3xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.pc += 2;
        }
    }
    fn op_4xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.pc += 2;
        }
    }
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }
    fn op_6xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }
    fn op_7xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = self.v[x].wrapping_add(nn);
    }
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
    }
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
    }
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
    }
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = sum;
        self.v[0xF] = if carry { 1 } else { 0 };
    }
    fn op_8xy5(&mut self, x: usize, y: usize) {
        self.v[0xF] = if self.v[x] >= self.v[y] { 1 } else { 0 };
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
    }
    fn op_8xy6(&mut self, x: usize) {
        self.v[0xF] = self.v[x] & 1;
        self.v[x] >>= 1;
    }
    fn op_8xy7(&mut self, x: usize, y: usize) {
        self.v[0xF] = if self.v[y] >= self.v[x] { 1 } else { 0 };
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
    }
    fn op_8xye(&mut self, x: usize) {
        self.v[0xF] = (self.v[x] >> 7) & 1;
        self.v[x] <<= 1;
    }
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }
    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }
    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = nnn + self.v[0] as u16;
    }
    fn op_cxnn(&mut self, x: usize, nn: u8) {
        let mut rng = rand::rng();
        self.v[x] = rng.random();
        self.v[x] &= nn;
    }
    fn op_dxyn(&mut self, x: usize, y: usize, height: u8) {
        let vx = self.v[x] as usize % 64;
        let vy = self.v[y] as usize % 32;
        self.v[0xF] = 0;

        for row in 0..height as usize {
            if vy + row >= 32 {
                break;
            }
            let sprite = self.memory[self.i as usize + row];
            for col in 0..8 {
                if vx + col >= 64 {
                    break;
                }
                let bit = (sprite >> (7 - col)) & 1;
                if bit == 1 {
                    let pixel = &mut self.display[vy + row][vx + col];
                    if *pixel {
                        self.v[0xF] = 1;
                    }
                    *pixel ^= true;
                }
            }
        }
        self.draw_flag = true;
    }
    fn op_ex9e(&mut self, x: usize) {
        if self.keys[self.v[x] as usize] {
            self.pc += 2;
        }
    }
    fn op_exa1(&mut self, x: usize) {
        if !self.keys[self.v[x] as usize] {
            self.pc += 2;
        }
    }
    fn op_fx07(&mut self, x: usize) {
        self.v[x] = self.delay_timer;
    }
    fn op_fx0a(&mut self, x: usize) {
        self.waiting_for_key = Some(x);
        self.pc -= 2;
    }
    fn op_fx15(&mut self, x: usize) {
        self.delay_timer = self.v[x];
    }
    fn op_fx18(&mut self, x: usize) {
        self.sound_timer = self.v[x];
    }
    fn op_fx1e(&mut self, x: usize) {
        self.i = self.i.wrapping_add(self.v[x] as u16);
    }
    fn op_fx29(&mut self, x: usize) {
        self.i = 0x50 + (self.v[x] as u16 * 5);
    }
    fn op_fx33(&mut self, x: usize) {
        let val = self.v[x];
        self.memory[self.i as usize] = val / 100;
        self.memory[self.i as usize + 1] = (val / 10) % 10;
        self.memory[self.i as usize + 2] = val % 10;
    }
    fn op_fx55(&mut self, x: usize) {
        for i in 0..=x {
            self.memory[self.i as usize + i] = self.v[i];
        }
        self.i += (x + 1) as u16;
    }
    fn op_fx65(&mut self, x: usize) {
        for i in 0..=x {
             self.v[i] = self.memory[self.i as usize + i];
        }
        self.i += (x + 1) as u16;
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

