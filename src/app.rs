use egui::{CentralPanel, Color32, Context, DragValue, Frame, Key, RichText, ScrollArea, TextBuffer, TextEdit, TextStyle, TopBottomPanel, Ui, vec2};
use egui_extras::{Size, StripBuilder};

use egui_phosphor::regular as icons;

use crate::{driver::GlobalState, dis};
use crate::dis::{Line, LineKind};

pub struct App {
    // todo: should probably keep everything in either App or GlobalState
    state: GlobalState,
    // this is separate to allow detecting when the bank value actually changed
    bank_value: u8,
    currently_edited_text: Option<String>,
    // used to deal with some jank in editing multiline comments
    move_comment_focus_to: Option<(usize, usize)>,
}

impl App {
    pub fn new(state: GlobalState) -> Self {
        Self { bank_value: state.bank, state, currently_edited_text: None, move_comment_focus_to: None }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("menu-bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                self.menu_bar(ui);
            });
        });
        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.toolbar(ui);
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(ui.spacing().item_spacing.y);
            self.editor(ui);
        });
    }
}

impl App {
    fn menu_bar(&mut self, ui: &mut Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Open ROM").clicked() {
                ui.close_menu();
                // TODO: ask for names
                self.state = GlobalState::new("smw.sfc", "rules.yml");
            }
            if ui.button("Save").clicked() {
                ui.close_menu();
                self.state.save();
            }

            ui.separator();

            if ui.button("Exit").clicked() {
                //
                ui.close_menu();
            }
        });

        ui.menu_button("Edit", |ui| {
            if ui.button("Copy").clicked() {
                // copy selection in code
                ui.close_menu();
            }
            if ui.button("Cut").clicked() {
                // copy and delete selection in code
                ui.close_menu();
            }
            if ui.button("Paste").clicked() {
                // paste text at current caret position
                ui.close_menu();
            }

            ui.separator();

            if ui.button("Undo").clicked() {
                // undo last action
                ui.close_menu();
            }
            if ui.button("Redo").clicked() {
                // redo previously undone action
                ui.close_menu();
            }

            ui.separator();

            if ui.button("Find").clicked() {
                // open search bar
            }

            if ui.button("Replace").clicked() {
                // open search bar
            }

            if ui.button("Go to address...").clicked() {
                // open popup with text box to provide ROM address and navigate to the address in code upon pressing enter
            }
        });
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        if ui.button(icons::FLOPPY_DISK).clicked() {
            // save
            self.state.save();
        }

        ui.separator();

        let a = ui.add(DragValue::new(&mut self.bank_value));
        let b = ui.label("Bank");
        a.labelled_by(b.id);

        if self.bank_value != self.state.bank {
            self.state.bank = self.bank_value;
            self.state.update_lines();
        }

        ui.separator();

        ui.add_space(ui.available_width());
    }

    fn editor(&mut self, ui: &mut Ui) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            let text_style = TextStyle::Monospace;
            let row_height = ui.text_style_height(&text_style);
            let num_rows = self.state.display_lines.len();
            let font_id = text_style.resolve(ui.style());
            let char_width = ui.fonts(|fonts| fonts.glyph_width(&font_id, 'x'));
            ScrollArea::vertical().auto_shrink(false).show_rows(ui, row_height, num_rows, |ui, row_range| {
                // contents of the editor
                for i in row_range {
                    ui.horizontal(|ui| {
                        StripBuilder::new(ui)
                            .size(Size::exact(12. * char_width))
                            .size(Size::exact(40. * char_width))
                            .size(Size::remainder())
                            .horizontal(|mut strip| {
                                let disp_line = self.state.display_lines[i].clone();
                                let line_pc = self.state.lines[disp_line.which_line].pc;
                                let line_kind = self.state.lines[disp_line.which_line].kind.clone();
                                strip.cell(|ui| {
                                    //ui.monospace(format!("{},{},{:X}", disp_line.which_line, disp_line.line_offset, line_pc));
                                    if !matches!(line_kind, LineKind::Spacing) && disp_line.line_offset == 0 {
                                        ui.monospace(format!("{:06X}", line_pc));
                                    }
                                });
                                strip.cell(|ui| {
                                    match &line_kind {
                                        LineKind::Label(lbl) => {
                                            if disp_line.line_offset == 0 {
                                                // todo: HOW do we handle the : here lol
                                                let mut lbl = lbl.clone() + ":";
                                                if TextEdit::singleline(&mut lbl)
                                                    .frame(false)
                                                    .font(TextStyle::Monospace)
                                                    .desired_width(f32::INFINITY)
                                                    .margin(vec2(0.0, 0.0))
                                                    .show(ui)
                                                    .response
                                                    .changed()
                                                {
                                                    self.state.update_lines();
                                                }
                                            }
                                        }
                                        LineKind::Code(txt) => {
                                            if disp_line.line_offset == 0 {
                                                ui.monospace(RichText::new(txt.trim_end()).color(Color32::WHITE));
                                            }
                                        }
                                        LineKind::ManualAsm(lines) => {
                                            if disp_line.line_offset < lines.len() {
                                                // TODO: figure out edit semantics for this
                                                ui.monospace(RichText::new(&lines[disp_line.line_offset]).color(Color32::YELLOW));
                                            }
                                        }
                                        LineKind::Spacing => {}
                                    }
                                });
                                strip.cell(|ui| {
                                    let mut comment = disp_line.comment.clone();
                                    // we need to test for the old value here, not the new one
                                    let comment_is_empty = comment.is_empty();

                                    ui.monospace("; ");
                                    let resp = TextEdit::singleline(&mut comment)
                                        .frame(false)
                                        .font(TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                        .margin(vec2(0.0, 0.0))
                                        .show(ui)
                                        .response;

                                    if self.move_comment_focus_to.is_some_and(|(i, j)| i == disp_line.which_line && j == disp_line.line_offset) {
                                        self.move_comment_focus_to = None;
                                        resp.request_focus();
                                    }

                                    let this_comments = if let LineKind::Label(lbl) = line_kind {
                                        self.state.label_comments.entry(lbl).or_default()
                                    } else {
                                        self.state.asm_comments.entry(line_pc).or_default()
                                    };
                                    if resp.has_focus() && comment_is_empty && ui.input(|i| i.key_pressed(egui::Key::Backspace)) {
                                        //println!("removal!");
                                        if disp_line.line_offset < this_comments.len() {
                                            this_comments.remove(disp_line.line_offset);
                                        }
                                        self.state.update_lines();
                                        resp.surrender_focus();
                                        // move focus to the previous line
                                        self.move_comment_focus_to = Some((disp_line.which_line, disp_line.line_offset.saturating_sub(1)));
                                    // checking on lost_focus here because the enter press will
                                    // actually unfocus the textarea itself
                                    } else if resp.lost_focus() && ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Enter)) {
                                        //println!("add!");
                                        if this_comments.len() <= disp_line.line_offset {
                                            this_comments.resize(disp_line.line_offset + 1, "".into());
                                        }
                                        this_comments.insert(disp_line.line_offset+1, "".into());
                                        self.state.update_lines();
                                        resp.surrender_focus();
                                        self.move_comment_focus_to = Some((disp_line.which_line, disp_line.line_offset + 1));
                                    } else if resp.changed() {
                                        if this_comments.len() <= disp_line.line_offset {
                                            this_comments.resize(disp_line.line_offset + 1, "".into());
                                        }
                                        this_comments[disp_line.line_offset] = comment;
                                        // doing a whole update_lines on this on every single edit
                                        // feels a bit excessive? but whatever
                                        self.state.update_lines();
                                    }
                                });
                            });
                    });
                }
            });
        });
    }
}
