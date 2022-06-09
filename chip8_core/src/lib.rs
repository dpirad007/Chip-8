pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGISTERS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGISTERS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emulator = Emu {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGISTERS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emulator
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGISTERS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1
        }

        if self.st > 0 {
            if self.st == 1 {

                //Audio not implemented
            }

            self.st -= 1
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;

        /*
        Left shift higher byte by 8 so,
        high = 0001000 -> 0001000 00000000
        then bitwise or with lower
        high or low -> 0001000 00000000
                               10010000
        */

        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        /*
        We have a 16 bit opcode and we require the individual digits so,
        we do an "and" operation with the 4 bits which correspond to a hex digit

        op -> 1010 1111 1010 0000 (to get first digit we & with F (1111))
          (&) 1111 0000 0000 0000
              -------------------
              1010 0000 0000 0000

              Then right shift by remaining (for first digit that is 12, then, 8... so on)
        */

        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 12;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // 0000 - Nop
            (0, 0, 0, 0) => return,

            // 00E0 - Clear screen
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],

            // 00EE - Return from Subroutine
            (0, 0, 0xE, 0xE) => {
                let return_adress = self.pop();
                self.pc = return_adress;
            }

            // 1NNN - Jump
            (1, _, _, _) => {
                let last_three = op & 0xFFF;
                self.pc = last_three;
            }

            // 2NNN - Call Subroutine
            (2, _, _, _) => {
                let last_three = op & 0xFFF;
                self.push(self.pc);
                self.pc = last_three;
            }

            // 3XNN - Skip next if VX == NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            // 4XNN - Skip next if VX != NN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }

            // 5XY0 - Skip next if VX == VY
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2
                }
            }

            // 6XNN - VX = NN
            (6, _, _, _) => {
                let x = digit1 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            }

            // 7XNN - VX += NN
            (7, _, _, _) => {
                let x = digit1 as usize;
                let nn = (op & 0xFF) as u8;
                // could just use += right? weird
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }

            // 8XY0 - VX = VY
            (8, _, _, 0) => {
                let x = digit1 as usize;
                let y = digit2 as usize;
                self.v_reg[x] = self.v_reg[y]
            }

            (_, _, _, _) => unimplemented!("Unimplemented opcode {}", op),
        }
    }
}
