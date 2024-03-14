use glow::HasContext;
use serde_derive::{Serialize, Deserialize};
use crate::dis;
use crate::rom::Rom;
use std::collections::HashMap;

/*
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
        let line_height = 20.0;
        ui.window("Status").build(|| {
            if let Some(c) = self.state.selection {
                ui.text(format!("Selection {:06X}-{:06X}", c[0],c[1]));
            }
            if let Some(c) = self.state.editing_comment {
                ui.text(format!("Comment {:06X}", c));
            }
            if ui.button("Save") {
                self.state.save();
            }
            if ui.input_int("Bank", &mut self.state.bank).chars_hexadecimal(true).display_format("%02X").enter_returns_true(true).build() {
                self.state.bank &= 0x1F;
                self.state.update_lines();
            }
        });
        ui.window("Disassembly").build(|| {
            let cursor = ui.cursor_screen_pos();
            ui.invisible_button("hexdump", [100.0, self.state.lines.len() as f32 * line_height+10.0]);
            let mut y = 0.0;
            let mut draw_list = ui.get_window_draw_list();
            if ui.is_window_focused() && ui.is_mouse_clicked(imgui::MouseButton::Left) {
                self.state.selection = None;
                self.state.editing_comment = None;
            }
            let mut editing_idx = None::<usize>;
            for (idx,i) in self.state.lines.iter().enumerate() {
                if y < ui.scroll_y() - line_height - 60.0 { y += line_height; continue; }
                if y > ui.scroll_y() + ui.window_size()[1] + 60.0 { break; }
                let mut pos = cursor;
                pos[1] += y;
                let mut buf = format!("{:06X} | ", i.pc);
                draw_list.add_text(pos, 0xFF808080, &buf);
                pos[0] += ui.calc_text_size(&buf)[0];
                let mut buf = &i.text;
                draw_list.add_text(pos, 0xFFFFFFFF, &buf);
                let size = ui.calc_text_size(&buf);
                let hover = ui.is_mouse_hovering_rect(pos, [pos[0]+size[0], pos[1]+size[1]]);
                if hover && ui.is_mouse_clicked(imgui::MouseButton::Left) {
                    editing_idx = Some(idx);
                }
                match i.kind {
                    dis::LineKind::Label => {
                        if hover && ui.is_mouse_clicked(imgui::MouseButton::Left) {
                            self.state.editing_label = Some(i.pc);
                            ui.open_popup("label_rename_menu");
                        }
                    },
                    dis::LineKind::Code => {
                        if hover && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
                            if let Some(target) = self.state.dis.entries[&i.pc].instr.jump_target(i.pc) {
                                // Scroll to the line
                                let mut y = 0.0;
                                if let Some(_line) = self.state.lines.iter().find(|c| {
                                    y += line_height;
                                    c.pc == target
                                }) {
                                    ui.set_scroll_y(y - 100.0);
                                }
                            }
                        }
                        if hover && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                            if ui.is_key_down(imgui::Key::LeftShift) {
                                if let Some(target) = self.state.dis.entries[&i.pc].instr.label_target(i.pc, i.pc>>16) {
                                    self.state.editing_label = Some(self.state.dis.normalize_addr(target));
                                    ui.open_popup("label_rename_menu");
                                }
                            } else {
                                self.state.editing_label = Some(i.pc);
                                ui.open_popup("code_popup_menu");
                            }
                        }
                    },
                    _ => {},
                }
                pos[0] = 512.0;
                if matches!(i.kind, dis::LineKind::Code) {
                    if self.state.editing_comment == Some(i.pc) {
                        editing_idx = Some(idx);
                        ui.set_cursor_pos([512.0, cursor[1] + y + ui.scroll_y()]);
                        ui.set_keyboard_focus_here();
                        ui.set_next_item_width(-1.0);
                        if ui.input_text(format!("##comment{}",i.pc), self.state.comments.entry(i.pc).or_insert(String::new())).build() {
                        }
                    } else {
                        let mut buf = format!("; {}", self.state.comments.get(&i.pc).map(|c| &**c).unwrap_or(""));
                        draw_list.add_text(pos, 0xFFA0A0A0, &buf);
                        let size = ui.calc_text_size(&buf);
                        let hover = ui.is_mouse_hovering_rect(pos, [pos[0]+size[0], pos[1]+size[1]]);
                        if hover && ui.is_mouse_clicked(imgui::MouseButton::Left) {
                            self.state.editing_comment = Some(i.pc);
                        }
                    }
                }
                y += line_height;
            }
            ui.popup("code_popup_menu", || {
                let pc = self.state.editing_label.unwrap();
                if ui.menu_item("Add label") {
                    let default = self.state.dis.get_label(pc);
                    let label = self.state.dis.label_names.entry(pc)
                        .or_insert(default);
                    self.state.update_lines();
                }
            });
            ui.popup("label_rename_menu", || {
                let pc = self.state.editing_label.unwrap();
                let default = self.state.dis.get_label(pc);
                let label = self.state.dis.label_names.entry(pc)
                    .or_insert(default);
                ui.set_keyboard_focus_here();
                if ui.input_text("##label_rename", label).enter_returns_true(true).build() {
                    self.state.update_lines();
                    ui.close_current_popup();
                }
            });
            if let Some(idx) = editing_idx {
                if ui.is_key_pressed(imgui::Key::Enter) {
                    self.state.editing_comment = None;
                }
                if ui.is_key_pressed(imgui::Key::UpArrow) {
                    for i in (0..idx).rev() {
                        if matches!(self.state.lines[i].kind, dis::LineKind::Code) {
                            self.state.editing_comment = Some(self.state.lines[i].pc);
                            break;
                        }
                    }
                }
                if ui.is_key_pressed(imgui::Key::DownArrow) {
                    for i in idx+1..self.state.lines.len() {
                        if matches!(self.state.lines[i].kind, dis::LineKind::Code) {
                            self.state.editing_comment = Some(self.state.lines[i].pc);
                            break;
                        }
                    }
                }
            }
            /*
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
            }*/
            /*
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
            });*/
        });
    }
}*/

pub struct GlobalState {
    pub rom: Rom,
    pub dis: dis::Disassembler,
    pub rules: Vec<dis::Rule>,
    pub selection: Option<[u32;2]>,
    pub lines: Vec<dis::Line>,
    pub comments: HashMap<u32, String>,
    pub editing_comment: Option<u32>,
    pub editing_label: Option<u32>,
    pub bank: u8,
    // should be PathBuf probably
    pub rules_filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct SavedData {
    rules: Vec<dis::Rule>,
    comments: HashMap<u32, String>,
    label_names: HashMap<u32, String>,
}

impl GlobalState {
    pub fn new(rom_fname: &str, rules_fname: &str) -> Self {
        let rom = crate::rom::Rom::new(std::fs::read(rom_fname).unwrap(), crate::rom::Mapper::LoRom);
        let mut dis = dis::Disassembler::new(rom.clone());
        let data: SavedData = serde_yaml::from_slice(&std::fs::read(rules_fname).unwrap()).unwrap();
        dis.label_names = data.label_names;
        dis.process_rules(data.rules.iter());
        let lines = dis.print_bank(0);
        Self {
            rom,
            dis,
            rules: data.rules,
            selection: None,
            lines,
            editing_comment: None,
            editing_label: None,
            comments: data.comments,
            bank: 0,
            rules_filename: rules_fname.to_string(),
        }
    }
    pub fn save(&mut self) {
        self.comments.retain(|_k, v| {
            v.len() > 0
        });
        let b = serde_yaml::to_string(&SavedData {
            rules: self.rules.clone(),
            comments: self.comments.clone(),
            label_names: self.dis.label_names.clone(),
        }).unwrap();
        // TODO: error reporting
        std::fs::write(&self.rules_filename, &b).unwrap();
    }
    pub fn update_lines(&mut self) {
        self.lines = self.dis.print_bank(self.bank as _);
    }
    /*
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
    }*/
}
