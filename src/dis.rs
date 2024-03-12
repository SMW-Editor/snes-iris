use crate::rom::Rom;
use crate::cpu::{self, Instruction, Mnemonic, CpuState};

use std::collections::{HashMap, HashSet};
use serde_derive::{Serialize, Deserialize};

pub struct Disassembler {
    pub rom: Rom,
    pub entries: HashMap<u32, Entry>,
    pub labels: HashSet<u32>,
    pub returns: HashSet<u32>,
    pub subroutines: HashMap<u32, Subroutine>,
    pub extra_rules: Vec<Rule>,
    pub label_names: HashMap<u32, String>,
}

#[derive(Clone,Debug)]
pub enum StackDataType {
    CpuState { state: CpuState, sr_state: SrCpuState },
    Bank(u8),
    Data,
    RetAddr,
}

#[derive(Clone)]
pub struct Entry {
    pub stack: Vec<StackDataType>,
    pub state: CpuState,
    pub instr: Instruction,
    pub subroutine: u32,
}
pub struct DataEntry {
    
}

#[derive(Copy,Clone,Default,Debug)]
pub struct SrCpuState {
    pub affect_m: Option<bool>,
    pub affect_x: Option<bool>,
}

#[derive(Copy,Clone,Default,Debug)]
pub struct Subroutine {
    pub sr_effect: SrCpuState,
    pub divergent: bool,
}

pub struct QueueEntry {
    pub pc: u32,
    pub stack: Vec<StackDataType>,
    pub state: CpuState,
    pub sr_state: SrCpuState,
}
#[derive(Deserialize, Serialize, Clone)]
pub enum Rule {
    JumpTable { pc: u32, size: u32, long: bool },
}

pub struct Line {
    pub pc: u32,
    pub len: usize,
    pub text: String,
    pub kind: LineKind
}
pub enum LineKind {
    Label,
    Code,
    Data,
    Spacing,
}

impl Disassembler {
    pub fn new(rom: Rom) -> Self {
        Self {
            rom,
            entries: HashMap::new(),
            labels: HashSet::new(),
            returns: HashSet::new(),
            subroutines: HashMap::new(),
            extra_rules: vec![],
            label_names: HashMap::new(),
        }
    }
    pub fn process_rules<'a>(&mut self, rules: impl IntoIterator<Item=&'a Rule>) {
        //self.process(QueueEntry { pc: 0xCCE0, stack: vec![], sr_state: Default::default(), state: CpuState { m: true, x: true } });
        let start = self.rom.load_u16(0xFFFC) as u32;
        let irq = self.rom.load_u16(0xFFEE) as u32;
        let nmi = self.rom.load_u16(0xFFEA) as u32;
        self.process(QueueEntry { pc: start, stack: vec![], sr_state: Default::default(), state: CpuState { m: true, x: true } });
        self.process(QueueEntry { pc: irq, stack: vec![], sr_state: Default::default(), state: CpuState { m: true, x: true } });
        self.process(QueueEntry { pc: nmi, stack: vec![], sr_state: Default::default(), state: CpuState { m: true, x: true } });
        let mut jt = HashSet::new();
        for i in rules.into_iter() { match i {
            Rule::JumpTable { pc, size, long } => {
                jt.insert(pc);
                let width = if *long { 3 } else { 2 };
                for i in 0..*size {
                    let addr = pc + 4 + i*width;
                    let addr = if *long {
                        self.rom.load_u24(addr)
                    } else {
                        self.rom.load_u16(addr) as u32 | (pc & 0xFF0000)
                    };
                    //println!("Doing jt {:06X} - {:06X}", pc, addr);
                    self.process(QueueEntry { pc: addr, stack: vec![StackDataType::RetAddr; width as usize], sr_state: Default::default(), state: cpu::CpuState { m: true, x: true }});
                    //self.xrefs.insert(addr, vec![]);
                }
            }
        } }
    }
    pub fn print_bank(&self, bank: u32) -> Vec<Line> {
        let mut lines = vec![];
        let mut rpc = 0x8000;
        while rpc < 0x10000 {
            let pc = rpc + (bank << 16);
            use std::fmt::Write;
            if self.labels.contains(&pc) || self.label_names.contains_key(&pc) {
                let mut out = String::new();
                writeln!(out, "{}:", self.get_label(pc));
                lines.push(Line { pc, len: 0, text: out, kind: LineKind::Label });
            }
            if let Some(i) = &self.entries.get(&pc) {
                let mut out = String::new();
                write!(out, "    ");
                let old_len = out.len();
                if let Some(c) = i.instr.jump_target(pc) {
                    i.instr.display(Some(&self.get_label(c)), &mut out);
                } else if let Some(c) = i.instr.label_target(pc, pc >> 16) {
                    if matches!(i.instr.mode, cpu::Mode::Imm) {
                        i.instr.display(None, &mut out);
                    } else {
                        i.instr.display(Some(&self.get_data_label(c)), &mut out);
                    }
                } else {
                    i.instr.display(None, &mut out);
                }
                /*write!(out, "{}", " ".repeat(48_usize.saturating_sub(out.len()-old_len)));
                writeln!(out, "; {:06X} | {}{} | {}",
                    pc,
                    if i.state.m { "M" } else { "m" },
                    if i.state.x { "X" } else { "x" },
                    i.stack.len()
                );*/
                lines.push(Line { pc, len: i.instr.size, text: out, kind: LineKind::Code });
                if i.instr.divergent() {
                    lines.push(Line { pc: pc+i.instr.size as u32 + 1, len: 0, text: "".into(), kind: LineKind::Spacing });
                }
                rpc += (i.instr.size + 1) as u32;
            } else {
                lines.push(Line { pc, len: 1, text: format!("    db ${:02X}", self.rom.load(pc)), kind: LineKind::Data });
                rpc += 1;
            }
        }
        lines
    }
    pub fn process(&mut self, entry: QueueEntry) -> &Subroutine {
        let orig_pc = entry.pc;
        if self.subroutines.contains_key(&orig_pc) {
            return self.subroutines.get(&orig_pc).unwrap()
        }
        if orig_pc & 0x8000 != 0x8000 || orig_pc & 0x7FFFFF > 0x400000 {
            return self.subroutines.entry(orig_pc).or_insert(Subroutine { sr_effect: Default::default(), divergent: false });
        }
        let mut queue = vec![entry];
        let mut sr_effect = SrCpuState { affect_m: None, affect_x: None };
        // this should really be in rules, not hardcoded
        let mut divergent = false;
        if orig_pc == 0x86DF || orig_pc == 0x86FA { divergent = true; }
        'outer: while let Some(QueueEntry { mut pc, mut stack, mut state, mut sr_state }) = queue.pop() {
            if pc & 0x8000 != 0x8000 || pc & 0x7FFFFF > 0x400000 {
                continue;
            }
            self.labels.insert(pc);
            while let Some((size, instr)) = cpu::parse_instr(self.rom.slice(pc), state) {
                use Mnemonic::*;
                'inner: for i in pc..pc+size as u32 {
                    if let Some(c) = self.entries.get(&i) {
                        // TODO: figure out sr_effect
                        if i == pc {
                            if c.subroutine == orig_pc {
                                // do nothing
                            } else if let Some(sub) = self.subroutines.get(&c.subroutine) {
                                if let Some(c) = sub.sr_effect.affect_m { sr_effect.affect_m = Some(c); state.m = c; }
                                if let Some(c) = sub.sr_effect.affect_x { sr_effect.affect_x = Some(c); state.x = c; }
                                self.subroutines.insert(orig_pc, sub.clone());
                            } else {
                                // do nothing
                            }
                        } else {
                            eprintln!("WARN: bad instr at {:06X}", pc);
                        }
                        continue 'outer;
                    }
                }
                /*println!("{:06X} {:06X} {}{} {} {:?}", orig_pc, pc, 
                    if state.m { "M" } else { "m" },
                    if state.x { "X" } else { "x" },
                    instr, stack);*/
                self.entries.insert(pc, Entry { stack: stack.clone(), state, instr, subroutine: orig_pc });
                instr.apply_flags(&mut state);
                instr.apply_flags_opt(&mut sr_state.affect_m, &mut sr_state.affect_x);
                self.apply_instr(&instr, &mut state, &mut sr_state, &mut stack);
                if matches!(instr.mnemonic, RTS|RTL) {
                    self.returns.insert(pc);
                    sr_effect = sr_state;
                    divergent = false;
                }
                if let Some(target) = instr.jump_addr(pc) {
                    queue.push(QueueEntry { pc: target, stack: stack.clone(), state, sr_state });
                } else if let Some(target) = instr.jump_target(pc) {
                    let size = if instr.mnemonic == JSL { 3 } else { 2 };
                    let sr = self.process(QueueEntry { pc: target, stack: vec![StackDataType::RetAddr; size], state, sr_state: Default::default() });
                    if let Some(c) = sr.sr_effect.affect_m { sr_state.affect_m = Some(c); state.m = c; }
                    if let Some(c) = sr.sr_effect.affect_x { sr_state.affect_x = Some(c); state.x = c; }
                    if sr.divergent {
                        divergent = false;
                        self.extra_rules.push(Rule::JumpTable { pc, size: 0, long: target == 0x0086FA });
                        break;
                    }
                }
                if instr.divergent() { break; }

                pc += size as u32;
            }
        }
        let sr = Subroutine { sr_effect, divergent };
        //println!("${:06X} ;{:?}", orig_pc, sr);
        self.subroutines.entry(orig_pc).or_insert(sr)
    }
    pub fn get_label(&self, addr: u32) -> String {
        if let Some(v) = self.label_names.get(&addr) {
            v.to_string()
        } else if self.subroutines.contains_key(&addr) {
            format!("sub_{:06X}", addr)
        } else if self.returns.contains(&addr) {
            format!("ret_{:06X}", addr)
        } else {
            format!("loc_{:06X}", addr)
        }
    }
    pub fn normalize_addr(&self, mut addr: u32) -> u32 {
        if addr & 0xFFFF < 0x2000 && (addr >> 16) & 0x7F < 0x40 {
            addr = (addr & 0xFFFF) | 0x7E0000;
        }
        if addr & 0xFFFF >= 0x2000 && addr & 0xFFFF < 0x8000 {
            addr = addr & 0xFFFF;
        }
        addr
    }
    pub fn get_data_label(&self, mut addr: u32) -> String {
        addr = self.normalize_addr(addr);
        if let Some(v) = self.label_names.get(&addr) {
            v.to_string()
        } else if addr >= 0x7E2000 {
            format!("wram_{:06X}", addr)
        } else if addr >= 0x7E0000 {
            format!("wram_{:04X}", addr & 0xFFFF)
        } else if addr >= 0x700000 {
            format!("sram_{:06X}", addr)
        } else if addr & 0xFFFF >= 0x2000 && addr & 0xFFFF < 0x8000 {
            format!("reg_{:04X}", addr & 0xFFFF)
        } else {
            format!("data_{:06X}", addr)
        }
    }
    pub fn apply_instr(
        &mut self,
        instr: &Instruction,
        state: &mut CpuState,
        sr_state: &mut SrCpuState,
        stack: &mut Vec<StackDataType>
    ) {
        use Mnemonic::*;
        match instr.mnemonic {
            PHP => stack.push(StackDataType::CpuState { state: *state, sr_state: *sr_state }),
            PHA => if state.m {
                stack.push(StackDataType::Data);
            } else {
                stack.push(StackDataType::Data);
                stack.push(StackDataType::Data);
            }
            PHX|PHY => if state.x {
                stack.push(StackDataType::Data);
            } else {
                stack.push(StackDataType::Data);
                stack.push(StackDataType::Data);
            }
            PEA|PER|PEI|PHD => {
                stack.push(StackDataType::Data);
                stack.push(StackDataType::Data);
            }
            PHK|PHB => stack.push(StackDataType::Data),
            PLP => match stack.pop() {
                Some(StackDataType::CpuState { state: s, sr_state: r }) => { *state = s; *sr_state = r },
                _ => eprintln!("uh oh bad PLP"),
            },
            PLA => if state.m {
                stack.pop();
            } else {
                stack.pop();
                stack.pop();
            }
            PLX|PLY => if state.x {
                stack.pop();
            } else {
                stack.pop();
                stack.pop();
            }
            PLD => {
                stack.pop();
                stack.pop();
            }
            PLB => { stack.pop(); },
            _ => {}
        }
    }
}


/*
pub struct Disassembler {
    rom: Rom,
    blocks: Vec<Vec<BasicBlockEntry>>,
    rules: Vec<Rule>,
    block_addrs: HashSet<u32>,
    subroutines: HashMap<u32, Subroutine>,
    labels: HashMap<String, u32>,
    returns: HashSet<u32>,
    xrefs: HashMap<u32, Vec<u32>>,
}


impl Disassembler {
    pub fn new(rom: Rom) -> Self {
        Self {
            rom,
            blocks: vec![],
            block_addrs: HashSet::new(),
            subroutines: HashSet::new(),
            returns: HashSet::new(),
            labels: HashMap::new(),
            xrefs: HashMap::new(),
            rules: vec![],
        }
    }
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
    pub fn blocks(&self) -> &[Vec<BasicBlockEntry>] {
        &self.blocks
    }
    pub fn process_vectors(&mut self) {
        let start = self.rom.load_u16(0xFFFC) as u32;
        let irq = self.rom.load_u16(0xFFEE) as u32;
        let nmi = self.rom.load_u16(0xFFEA) as u32;

        let mut queue = vec![
            QueueEntry { pc: start, stack: vec![], state: cpu::CpuState { m: true, x: true } },
            QueueEntry { pc: irq, stack: vec![], state: cpu::CpuState { m: true, x: true } },
            QueueEntry { pc: nmi, stack: vec![], state: cpu::CpuState { m: true, x: true } },
        ];
        self.xrefs.insert(start, vec![]);
        self.xrefs.insert(nmi, vec![]);
        self.xrefs.insert(irq, vec![]);
        self.labels.insert("VectorStart".into(), start);
        self.labels.insert("VectorNmi".into(), nmi);
        self.labels.insert("VectorIrq".into(), irq);

        for i in self.rules.iter() { match i {
            Rule::DivergentJump(sr) => { self.subroutines.entry(*sr).divergent = true; },
            Rule::JumpTable { pc, size, long } => {
                let width = if *long { 3 } else { 2 };
                for i in 0..*size {
                    let addr = pc + 4 + i*width;
                    eprintln!("{:06X}", addr);
                    let addr = if *long {
                        self.rom.load_u24(addr)
                    } else {
                        self.rom.load_u16(addr) as u32 | (pc & 0xFF0000)
                    };
                    eprintln!("{:06X}", addr);
                    queue.push(QueueEntry { pc: addr, stack: vec![StackDataType::RetAddr; width as usize], state: cpu::CpuState { m: true, x: true }});
                    self.xrefs.insert(addr, vec![]);
                }
            },
            _ => {}
        } }

        self.process(queue)
    }
    pub fn get_label(&self, addr: u32) -> String {
        if let Some((k,_)) = self.labels.iter().find(|(k,v)| {
            **v == addr
        }) {
            k.to_string()
        } else if self.subroutines.contains(&addr) {
            format!("sub_{:06X}", addr)
        } else if self.returns.contains(&addr) {
            format!("ret_{:06X}", addr)
        } else {
            format!("loc_{:06X}", addr)
        }
    }
    pub fn process(&mut self, mut queue: Vec<QueueEntry>) {
        //let mut divergent_srs = HashSet::new();
        'outer: while let Some(QueueEntry { mut pc, mut stack, mut state }) = queue.pop() {
            if pc & 0x8000 != 0x8000 || pc & 0x7FFFFF > 0x400000 { continue; }
            let orig_pc = pc;
            // see if we have any existing blocks
            for block in self.blocks.iter_mut() {
                if let Some(c) = block.iter().position(|i| i.pc == pc) {
                    if c != 0 {
                        let other = block.split_off(c);
                        self.blocks.push(other);
                        self.block_addrs.insert(pc);
                    }
                    continue 'outer;
                }
            }
            let mut block = vec![];
            while let Some((size, instr)) = cpu::parse_instr(self.rom.slice(pc), state) {
                if self.block_addrs.contains(&pc) {
                    break;
                }
                block.push(BasicBlockEntry { pc, stack: stack.clone(), state, instr });
                if let Some(target) = instr.jump_addr(pc) {
                    self.xrefs.entry(target).or_insert(vec![]).push(pc);
                    queue.push(QueueEntry { pc: target, stack: stack.clone(), state });
                } else if let Some(target) = instr.jump_target(pc) {
                    self.xrefs.entry(target).or_insert(vec![]).push(pc);
                    let size = if instr.mnemonic == JSL { 3 } else { 2 };
                    queue.push(QueueEntry { pc: target, stack: vec![StackDataType::RetAddr; size], state });
                    self.subroutines.insert(target);
                    if self.divergent_jumps.contains(&target) { break; }
                }
                if matches!(instr.mnemonic, RTS|RTL) {
                    self.returns.insert(pc);
                }
                pc += size as u32;
                use Mnemonic::*;
                match instr.mnemonic {
                    PHP => stack.push(StackDataType::CpuState(state)),
                    PHA => if state.m {
                        stack.push(StackDataType::Data);
                    } else {
                        stack.push(StackDataType::Data);
                        stack.push(StackDataType::Data);
                    }
                    PHX|PHY => if state.x {
                        stack.push(StackDataType::Data);
                    } else {
                        stack.push(StackDataType::Data);
                        stack.push(StackDataType::Data);
                    }
                    PEA|PER|PEI|PHD => {
                        stack.push(StackDataType::Data);
                        stack.push(StackDataType::Data);
                    }
                    PHK|PHB => stack.push(StackDataType::Data),
                    PLP => match stack.pop() {
                        Some(StackDataType::CpuState(s)) => state = s,
                        _ => eprintln!("uh oh bad PLP"),
                    },
                    PLA => if state.m {
                        stack.pop();
                    } else {
                        stack.pop();
                        stack.pop();
                    }
                    PLX|PLY => if state.x {
                        stack.pop();
                    } else {
                        stack.pop();
                        stack.pop();
                    }
                    PLD => {
                        stack.pop();
                        stack.pop();
                    }
                    PLB => { stack.pop(); },
                    _ => {}
                }
                instr.apply_flags(&mut state);
                if instr.divergent() { break; }
                if instr.branch() {
                    queue.push(QueueEntry { pc, stack: stack.clone(), state });
                    break;
                }
            }
            self.blocks.push(block);
            self.block_addrs.insert(orig_pc);
        }
        self.blocks.sort_by_key(|c| c.first().map(|c| c.pc));
        let mut out = String::new();
        for (idx,i) in self.blocks.iter().enumerate() {
            use std::fmt::Write;
            let pc = i.first().unwrap().pc;
            if let Some(c) = self.xrefs.get(&pc) {
                writeln!(out, "{}:  ; XREFS: {:06X?}", self.get_label(pc), c);
            }
            for i in i {
                write!(out, ";[[{:06X} | {}{} | {}]] ", i.pc,
                    if i.state.m { "M" } else { "m" },
                    if i.state.x { "X" } else { "x" },
                    i.stack.len(),
                );
                if let Some(c) = i.instr.jump_target(i.pc) {
                    i.instr.display(Some(&self.get_label(c)), &mut out);
                } else {
                    i.instr.display(None, &mut out);
                }
                writeln!(out);
            }
        }
        println!("{}", out);
    }
}
*/
