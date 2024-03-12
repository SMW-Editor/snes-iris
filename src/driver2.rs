use glow::HasContext;
use serde_derive::{Serialize, Deserialize};

pub struct Driver {
    pub state: GlobalState,
}

impl Driver {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        Self {
            state: GlobalState::new(gl),
        }
    }
    pub unsafe fn draw(&mut self, gl: &glow::Context, ui: &mut imgui::Ui) {
        gl.clear_color(0.1,0.1,0.1,1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        gl.enable(glow::BLEND);
        gl.blend_equation_separate(glow::FUNC_ADD, glow::FUNC_ADD);
        gl.blend_func_separate(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA, glow::ONE, glow::ONE_MINUS_SRC_ALPHA);

        ui.dockspace_over_main_viewport();
        let line_height = 16.0;
        ui.window("Status").build(|| {
            if let Some(c) = self.state.selection {
                ui.text(format!("Selection {:06X}-{:06X}", c[0],c[1]));
            }
        });
        ui.window("Disassembly").build(|| {
            let cursor = ui.cursor_screen_pos();
            ui.invisible_button("hexdump", [100.0, self.state.lines.len() as f32 * line_height]);
            let mut iter = self.state.lines.iter();
            let mut next = iter.next();
            let mut y = 0.0;
            let mut draw_list = ui.get_window_draw_list();
            if ui.is_window_focused() && ui.is_mouse_clicked(imgui::MouseButton::Left) {
                self.state.selection = None;
            }
            while let Some(cur) = next {
                next = iter.next();
                if y >= ui.scroll_y() - line_height {
                    use std::fmt::Write;
                    if y > ui.scroll_y() + ui.window_size()[1] { break; }
                    let mut pos = cursor;
                    pos[1] += y;
                    let mut buf = format!("{:06X} | ", cur.0);
                    draw_list.add_text(pos, 0xFF808080, &buf);
                    pos[0] += ui.calc_text_size(&buf)[0];
                    let mut end = next.as_ref().map(|c| *c.0).unwrap_or(0xFFFFFF);
                    if cur.0 >> 16 != end >> 16 {
                        end = (cur.0 & 0xFF0000) + 0x10000;
                    }
                    let mut iter = match cur.1 {
                        LineType::String => Box::new({
                            (*cur.0..end).map(|i| {
                                (i, format!("{}", self.state.rom.load(i) as char), 1u32)
                            })
                        }) as Box<dyn Iterator<Item=(u32,String,u32)>>,
                        LineType::Data { size, .. } => Box::new({
                            let size = *size as u32;
                            let rom = &self.state.rom;
                            (*cur.0..end).step_by(size as _).map(move |i| {
                                let mut val = 0u32;
                                for c in 0..size {
                                    val |= (rom.load(i+c) as u32) << (c*8);
                                }
                                let comma = if i + size < end { ", " } else { "" };
                                (i, format!("${:0size$X}{}", val, comma, size=size as usize*2), size)
                            })
                        }),
                        LineType::Code { data, .. } => Box::new({
                            Some((*cur.0, data.clone(), end-*cur.0)).into_iter()
                        }),
                        _ => Box::new({
                            (*cur.0..end).map(|i| (i, format!("{:02X} ", self.state.rom.load(i)), 1))
                        })
                    };
                    let (st,fin) = match cur.1 {
                        LineType::String => ("db \"", "\""),
                        LineType::Data { size: 1, .. } => ("db ", ""),
                        LineType::Data { size: 2, .. } => ("dw ", ""),
                        LineType::Data { size: 3, .. } => ("dl ", ""),
                        LineType::Data { size: 4, .. } => ("dd ", ""),
                        _ => ("", "")
                    };
                    draw_list.add_text(pos, 0xFFFFA080, &st);
                    pos[0] += ui.calc_text_size(&st)[0];
                    for (i,buf,data_size) in iter.take(64) {
                        let size = ui.calc_text_size(&buf);
                        let hover = ui.is_mouse_hovering_rect(pos, [pos[0]+size[0], pos[1]+size[1]]);
                        if hover && ui.is_window_focused() {
                            if ui.is_mouse_clicked(imgui::MouseButton::Left) {
                                self.state.selection = Some([i,i+data_size]);
                            }
                            if ui.is_mouse_down(imgui::MouseButton::Left) {
                                if let Some(c) = &mut self.state.selection {
                                    c[1] = i+data_size;
                                }
                            }
                        }
                        let mut selected = false;
                        if let Some(mut c) = self.state.selection {
                            c.sort();
                            if (c[0]..c[1]).contains(&i) {
                                selected = true;
                            }
                        }
                        if selected {
                            draw_list.add_rect(pos, [pos[0]+size[0], pos[1]+size[1]], 0xFF804020).filled(true).build();
                        }
                        let color = if hover { 0xFFFFFFFFu32 } else { 0xFFE0E0E0 };
                        draw_list.add_text(pos, color, &buf);
                        pos[0] += size[0];
                    }
                    draw_list.add_text(pos, 0xFFFFA080, &fin);
                }
                y += line_height;
            }
            if ui.is_mouse_clicked(imgui::MouseButton::Right) {
                ui.open_popup("hexdump_ctx_menu");
            }
            ui.popup("hexdump_ctx_menu", || {
                if ui.menu_item("Disassemble") {
                    self.state.disassemble(self.state.selection.unwrap()[0]);
                }
                if ui.menu_item("Unknown") {
                    self.state.update_selection(LineType::Unknown);
                }
                if ui.menu_item("Bytes") {
                    self.state.update_selection(LineType::Data { size: 1, labeled: false });
                }
                if ui.menu_item("Words") {
                    self.state.selection_chunks(2);
                    self.state.update_selection(LineType::Data { size: 2, labeled: false });
                }
                if ui.menu_item("Word ptrs") {
                    self.state.selection_chunks(2);
                    self.state.update_selection(LineType::Data { size: 2, labeled: true });
                }
                if ui.menu_item("Longs") {
                    self.state.selection_chunks(2);
                    self.state.update_selection(LineType::Data { size: 3, labeled: false });
                }
                if ui.menu_item("Long ptrs") {
                    self.state.selection_chunks(2);
                    self.state.update_selection(LineType::Data { size: 3, labeled: true });
                }
                if ui.menu_item("String") {
                    self.state.update_selection(LineType::String);
                }
            });
        });
    }
}

pub struct GlobalState {
    rom: crate::rom::Rom,
    dis: crate::dis::Disassembler,
    lines: std::collections::BTreeMap<u32, LineType>,
    selection: Option<[u32;2]>,
}

#[derive(Clone, Debug)]
pub enum LineType {
    Unknown,
    Data { size: u8, labeled: bool },
    Code { data: String, labeled: bool },
    ExternalFile,
    String,
    Custom(String),
    Padding,
}


impl GlobalState {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let rom = crate::rom::Rom::new(std::fs::read("patched.smc").unwrap()[0x200..].to_vec(), crate::rom::Mapper::LoRom);
        let dis = crate::dis::Disassembler::new(rom.clone());
        let mut lines = std::collections::BTreeMap::new();
        for b in 0x00..0x20 {
            for i in (0x8000..0xFFFF).step_by(16) {
                lines.insert((b<<16)+i, LineType::Unknown);
            }
        }
        //let blocks = dis.process(0x0FF900);
        Self {
            rom,
            dis,
            lines,
            selection: None,
        }
    }
    fn disassemble(&mut self, start: u32) {
        self.dis.process(start);
        for i in self.dis.blocks().to_vec() {
            for i in i {
                println!("{:06X}", i.pc);
                self.update_with(LineType::Code { data: i.instr.to_string(), labeled: false }, [i.pc, i.pc+i.instr.size as u32+1]);
            }
        }
    }
    fn selection_chunks(&mut self, chunks: u32) {
        if let Some(c) = &mut self.selection {
            c.sort();
            println!("{}", c[1]);
            c[1] = c[0] + (c[1] - c[0]) / chunks * chunks;
            println!("{}", c[1]);
            if c[0] == c[1] { self.selection = None; }
        }
    }
    fn update_with(&mut self, kind: LineType, mut c: [u32;2]) {
        c.sort();
        let mut r = self.lines.range(..c[0]).rev()
            .next().map(|c| c.1.clone()).unwrap_or(LineType::Unknown);
        println!("{:?}", r);
        for i in self.lines.range(c[0]..c[1]).map(|c| *c.0).collect::<Vec<_>>() {
            if c[0] != i {
                r = self.lines.remove(&i).unwrap();
                println!("{:?}", r);
            }
        }
        self.lines.insert(c[0], kind);
        if !self.lines.contains_key(&c[1]) && c[1] & 0x8000 != 0 {
            self.lines.insert(c[1], LineType::Unknown);
        }
    }
    fn update_selection(&mut self, kind: LineType) {
        if let Some(c) = self.selection {
            self.update_with(kind, c);
        }
    }
}
