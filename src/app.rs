use egui::{CentralPanel, Context, DragValue, Frame, ScrollArea, TopBottomPanel, Ui};

use egui_phosphor::regular as icons;

use crate::{driver::GlobalState, dis};

pub struct App {
    // todo: should probably keep everything in either App or GlobalState
    state: GlobalState,
    // this is separate to allow detecting when the bank value actually changed
    bank_value: u8,
}

impl App {
    pub fn new(state: GlobalState) -> Self {
        Self { bank_value: state.bank, state }
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
            // Frame::menu(ui.style()).show(ui, |ui| {
            //     ui.horizontal(|ui| {
            //         self.toolbar(ui);
            //     });
            // });
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
            let text_style = egui::TextStyle::Monospace;
            let row_height = ui.text_style_height(&text_style);
            let num_rows = self.state.lines.len();
            let font_id = text_style.resolve(ui.style());
            let char_width = ui.fonts(|fonts| fonts.glyph_width(&font_id, 'x'));
            egui::ScrollArea::vertical().auto_shrink(false).show_rows(ui, row_height, num_rows, |ui, row_range| {
                // contents of the editor
                for i in row_range {
                    let l = &self.state.lines[i];
                    let comment = if let dis::LineKind::Code = l.kind {
                        self.state.comments.get(&l.pc).map(|l| "; ".to_owned() + l).unwrap_or("".to_owned())
                    } else { String::new() };
                    // idk why but it seems like i need to wrap the stripbuilder inside a
                    // Horizontal here lol
                    ui.horizontal(|ui| {
                        egui_extras::StripBuilder::new(ui)
                            .size(egui_extras::Size::exact(8. * char_width))
                            .size(egui_extras::Size::exact(40. * char_width))
                            .size(egui_extras::Size::remainder())
                            .horizontal(|mut strip| {
                                strip.cell(|ui| { ui.monospace(format!("{:06X}", l.pc)); });
                                strip.cell(|ui| {
                                    let txt = egui::RichText::new(l.text.trim_end()).monospace().color(egui::Color32::WHITE);
                                    ui.label(txt);
                                });
                                strip.cell(|ui| { ui.monospace(comment); });
                            });
                    });
                }
            });
        });
    }
}
