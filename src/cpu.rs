#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum Mnemonic {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI,
    BNE, BPL, BRA, BRK, BRL, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, COP, CPX, CPY, DEC,
    DEX, DEY, EOR, INC, INX, INY, JML, JMP,
    JSL, JSR, LDA, LDX, LDY, LSR, MVN, MVP,
    NOP, ORA, PEA, PEI, PER, PHA, PHB, PHD,
    PHK, PHP, PHX, PHY, PLA, PLB, PLD, PLP,
    PLX, PLY, REP, ROL, ROR, RTI, RTL, RTS,
    SBC, SEC, SED, SEI, SEP, STA, STP, STX,
    STY, STZ, TAX, TAY, TCD, TCS, TDC, TRB,
    TSB, TSC, TSX, TXA, TXS, TXY, TYA, TYX,
    WAI, WDM, XBA, XCE,
}

use Mnemonic::*;

pub static INSTR: [Mnemonic; 256] = [
//  0    1    2    3    4    5    6    7       8    9    A    B    C    D    E    F
    BRK, ORA, COP, ORA, TSB, ORA, ASL, ORA,    PHP, ORA, ASL, PHD, TSB, ORA, ASL, ORA,      // 0
    BPL, ORA, ORA, ORA, TRB, ORA, ASL, ORA,    CLC, ORA, INC, TCS, TRB, ORA, ASL, ORA,      // 1
    JSR, AND, JSL, AND, BIT, AND, ROL, AND,    PLP, AND, ROL, PLD, BIT, AND, ROL, AND,      // 2
    BMI, AND, AND, AND, BIT, AND, ROL, AND,    SEC, AND, DEC, TSC, BIT, AND, ROL, AND,      // 3
    RTI, EOR, WDM, EOR, MVP, EOR, LSR, EOR,    PHA, EOR, LSR, PHK, JMP, EOR, LSR, EOR,      // 4
    BVC, EOR, EOR, EOR, MVN, EOR, LSR, EOR,    CLI, EOR, PHY, TCD, JML, EOR, LSR, EOR,      // 5
    RTS, ADC, PER, ADC, STZ, ADC, ROR, ADC,    PLA, ADC, ROR, RTL, JMP, ADC, ROR, ADC,      // 6
    BVS, ADC, ADC, ADC, STZ, ADC, ROR, ADC,    SEI, ADC, PLY, TDC, JMP, ADC, ROR, ADC,      // 7
    BRA, STA, BRL, STA, STY, STA, STX, STA,    DEY, BIT, TXA, PHB, STY, STA, STX, STA,      // 8
    BCC, STA, STA, STA, STY, STA, STX, STA,    TYA, STA, TXS, TXY, STZ, STA, STZ, STA,      // 9
    LDY, LDA, LDX, LDA, LDY, LDA, LDX, LDA,    TAY, LDA, TAX, PLB, LDY, LDA, LDX, LDA,      // A
    BCS, LDA, LDA, LDA, LDY, LDA, LDX, LDA,    CLV, LDA, TSX, TYX, LDY, LDA, LDX, LDA,      // B
    CPY, CMP, REP, CMP, CPY, CMP, DEC, CMP,    INY, CMP, DEX, WAI, CPY, CMP, DEC, CMP,      // C
    BNE, CMP, CMP, CMP, PEI, CMP, DEC, CMP,    CLD, CMP, PHX, STP, JMP, CMP, DEC, CMP,      // D
    CPX, SBC, SEP, SBC, CPX, SBC, INC, SBC,    INX, SBC, NOP, XBA, CPX, SBC, INC, SBC,      // E
    BEQ, SBC, SBC, SBC, PEA, SBC, INC, SBC,    SED, SBC, PLX, XCE, JSR, SBC, INC, SBC,      // F
];

pub static MODES: [Mode; 256] = [
//  0    1    2    3    4    5    6    7       8    9    A    B    C    D    E    F
    Imp, Idx, Imm, Sr,  Dp,  Dp,  Dp,  Idl,    Imp, Imm, Imp, Imp, Abs, Abs, Abs, Abl,      // 0
    Rel, Idy, Idp, Isy, Dp,  Dpx, Dpx, Ily,    Imp, Aby, Imp, Imp, Abs, Abx, Abx, Alx,      // 1
    Abs, Idx, Abl, Sr,  Dp,  Dp,  Dp,  Idl,    Imp, Imm, Imp, Imp, Abs, Abs, Abs, Abl,      // 2
    Rel, Idy, Idp, Isy, Dpx, Dpx, Dpx, Ily,    Imp, Aby, Imp, Imp, Abx, Abx, Abx, Alx,      // 3
    Imp, Idx, Imm, Sr,  Bm,  Dp,  Dp,  Idl,    Imp, Imm, Imp, Imp, Abs, Abs, Abs, Abl,      // 4
    Rel, Idy, Idp, Isy, Bm,  Dpx, Dpx, Ily,    Imp, Aby, Imp, Imp, Abl, Abx, Abx, Alx,      // 5
    Imp, Idx, Rll, Sr,  Dp,  Dp,  Dp,  Idl,    Imp, Imm, Imp, Imp, Ind, Abs, Abs, Abl,      // 6
    Rel, Idy, Idp, Isy, Dpx, Dpx, Dpx, Ily,    Imp, Aby, Imp, Imp, Iax, Abx, Abx, Alx,      // 7
    Rel, Idx, Rll, Sr,  Dp,  Dp,  Dp,  Idl,    Imp, Imm, Imp, Imp, Abs, Abs, Abs, Abl,      // 8
    Rel, Idy, Idp, Isy, Dpx, Dpx, Dpy, Ily,    Imp, Aby, Imp, Imp, Abs, Abx, Abx, Alx,      // 9
    Imm, Idx, Imm, Sr,  Dp,  Dp,  Dp,  Idl,    Imp, Imm, Imp, Imp, Abs, Abs, Abs, Abl,      // A
    Rel, Idy, Idp, Isy, Dpx, Dpx, Dpy, Ily,    Imp, Aby, Imp, Imp, Abx, Abx, Aby, Alx,      // B
    Imm, Idx, Imm, Sr,  Dp,  Dp,  Dp,  Idl,    Imp, Imm, Imp, Imp, Abs, Abs, Abs, Abl,      // C
    Rel, Idy, Idp, Isy, Idp, Dpx, Dpx, Ily,    Imp, Aby, Imp, Imp, Ial, Abx, Abx, Alx,      // D
    Imm, Idx, Imm, Sr,  Dp,  Dp,  Dp,  Idl,    Imp, Imm, Imp, Imp, Abs, Abs, Abs, Abl,      // E
    Rel, Idy, Idp, Isy, Abs, Dpx, Dpx, Ily,    Imp, Aby, Imp, Imp, Iax, Abx, Abx, Alx,      // F
];

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum Mode {
    Imp, Imm, Sr,
    Dp, Dpx, Dpy,
    Idp, Idx, Idy, Idl, Ily, Isy,
    Abs, Abx, Aby, Abl, Alx,
    Ind, Iax, Ial,
    Rel, Rll,
    Bm
}
use Mode::*;

pub enum Register { A, XY }

pub fn affects(mnem: Mnemonic) -> Option<Register> {
    match mnem {
        ADC | SBC | CMP | AND | EOR | ORA | BIT | LDA | STA => Some(Register::A),
        CPX | CPY | LDX | LDY | STX | STY => Some(Register::XY),
        _ => None
    }
}


impl Mode {
    pub fn format_item<X: std::fmt::Display, W: std::fmt::Write>(self, arg: X, mut fmt: W) -> std::fmt::Result {
        use std::fmt::Write;
        match self {
            Imp => write!(fmt, ""),
            Imm => write!(fmt, " #{}", arg),
            Sr => write!(fmt, " {},s", arg),
            Dp|Abs|Abl => write!(fmt, " {}", arg),
            Dpx|Abx|Alx => write!(fmt, " {},x", arg),
            Dpy|Aby => write!(fmt, " {},y", arg),
            Idp|Ind => write!(fmt, " ({})", arg),
            Idx|Iax => write!(fmt, " ({},x)", arg),
            Idy => write!(fmt, " ({}),y", arg),
            Idl|Ial => write!(fmt, " [{}]", arg),
            Ily => write!(fmt, " [{}],y", arg),
            Isy => write!(fmt, " ({},s),y", arg),
            Rel|Rll => write!(fmt, " {}", arg),
            Bm => write!(fmt, " {}", arg)
        }
    }
    pub fn size(self) -> usize {
        match self {
            Imp => 0,
            Imm => panic!("Attempted to take size of Imm mode"),
            Sr|Dp|Dpx|Dpy|Idp|Idx|Idy|Idl|Ily|Isy|Rel => 1,
            Abs|Abx|Aby|Ind|Iax|Ial|Rll|Bm => 2,
            Abl|Alx => 3
        }
    }
    pub fn size_spec(self, size: usize) -> &'static str {
        match self {
            Imm if size == 1 => ".b",
            Imm => ".w",
            Dp|Dpx|Dpy|Idp|Idx|Idy|Idl|Ily|Isy => ".b",
            Abs|Abx|Aby|Ind|Iax|Ial => ".w",
            Abl|Alx => ".l",
            _ => ""
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Instruction {
    pub mnemonic: Mnemonic,
    pub mode: Mode,
    pub size: usize,
    pub argument: u32
}

#[derive(Debug, Copy, Clone)]
pub struct CpuState {
    pub m: bool,
    pub x: bool,
}

pub fn parse_instr(input: &[u8], CpuState { m, x }: CpuState) -> Option<(usize, Instruction)> {
    let opcode = *input.get(0)?;
    let mnemonic = INSTR[opcode as usize];
    let mode = MODES[opcode as usize];
    let size = if mode == Mode::Imm {
        match affects(mnemonic) {
            Some(Register::A)  => if m { 1 } else { 2 },
            Some(Register::XY) => if x { 1 } else { 2 },
            None => 1
        }
    } else { mode.size() };
    let mut argument = [0; 4];
    argument[..size].copy_from_slice(&input.get(1..size+1)?);
    let argument = u32::from_le_bytes(argument);
    let instr = Instruction {
        mnemonic, mode, size, argument
    };
    Some((size+1, instr))
}

impl Instruction {
    pub fn display<W: std::fmt::Write>(self, label: Option<&str>, mut fmt: W) -> std::fmt::Result {
        write!(fmt, "{}", format!("{:?}", self.mnemonic).to_ascii_lowercase())?;
        write!(fmt, "{}", self.mode.size_spec(self.size))?;
        if let Some(s) = label {
            self.mode.format_item(s, fmt)
        } else {
            self.mode.format_item(format_args!("${:0size$X}", self.argument, size=self.size*2), fmt)
        }
    }
    pub fn unlikely(&self) -> bool {
        matches!(self.mnemonic, BRK|COP|BRL)
    }
    pub fn divergent(&self) -> bool {
        matches!(self.mnemonic, BRK|COP|JMP|JML|RTS|RTL|RTI|BRA|BRL)
    }
    pub fn branch(&self) -> bool {
        matches!(self.mnemonic, BCC|BCS|BEQ|BNE|BMI|BPL|BVC|BVS)
    }
    pub fn apply_flags(&self, CpuState { m, x }: &mut CpuState) {
        match self.mnemonic {
            REP => {
                *m &= self.argument & 0x20 == 0;
                *x &= self.argument & 0x10 == 0;
            }
            SEP => {
                *m |= self.argument & 0x20 != 0;
                *x |= self.argument & 0x10 != 0;
            },
            _ => {}
        }
    }
    pub fn apply_flags_opt(&self, m: &mut Option<bool>, x: &mut Option<bool>) {
        match self.mnemonic {
            REP => {
                if self.argument & 0x20 != 0 { *m = Some(false); }
                if self.argument & 0x10 != 0 { *x = Some(false); }
            }
            SEP => {
                if self.argument & 0x20 != 0 { *m = Some(true); }
                if self.argument & 0x10 != 0 { *x = Some(true); }
            },
            _ => {}
        }
    }
    pub fn label_target(&self, addr: u32, dbr: u32) -> Option<u32> {
        match self.mnemonic {
            JMP|JSR => return Some(self.argument | (addr & 0xFF0000)),
            MVN|MVP => return None,
            _ => {}
        }
        Some(match self.mode {
            Sr|Bm => self.argument,
            Imp => return None,
            Dp|Dpx|Dpy|Idp|Idx|Idy|Idl|Ily|Isy => self.argument,
            Abs|Abx|Aby|Ind|Iax|Ial|Imm => self.argument | (dbr << 16),
            Abl|Alx => self.argument,
            Rel => addr + 2 + self.argument as i8 as u32,
            Rll => addr + 3 + self.argument as i8 as u32,
        })
    }
    pub fn jump_addr(&self, addr: u32) -> Option<u32> {
        Some(match self.mnemonic {
            JMP if self.mode == Abs => (addr & 0xFF0000) | self.argument,
            JML if self.mode == Abl => self.argument,
            BRA|BCC|BCS|BEQ|BMI|BNE|BPL|BVC|BVS => {
                (addr + 2).wrapping_add(self.argument as i8 as u32)
            },
            BRL => {
                (addr + 3).wrapping_add(self.argument as i16 as u32)
            },
            _ => return None
        })
    }
    pub fn jsr_addr(&self, addr: u32) -> Option<u32> {
        Some(match self.mnemonic {
            JSR if self.mode == Abs => (addr & 0xFF0000) | self.argument,
            JSL if self.mode == Abl => self.argument,
            _ => return None
        })
    }
    pub fn jump_target(&self, addr: u32) -> Option<u32> {
        Some(match self.mnemonic {
            JMP|JSR if self.mode == Abs => (addr & 0xFF0000) | self.argument,
            JML|JSL if self.mode == Abl => self.argument,
            BRA|BCC|BCS|BEQ|BMI|BNE|BPL|BVC|BVS => {
                (addr + 2).wrapping_add(self.argument as i8 as u32)
            },
            BRL => {
                (addr + 3).wrapping_add(self.argument as i16 as u32)
            },
            _ => return None
        })
    }
    pub fn size(&self) -> usize { self.size }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.display(None, f)
    }
}

