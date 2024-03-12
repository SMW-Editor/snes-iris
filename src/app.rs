use egui::{CentralPanel, Context, DragValue, Frame, ScrollArea, TopBottomPanel, Ui};

use egui_phosphor::regular as icons;

pub struct App {
    bank: u8,
}

impl Default for App {
    fn default() -> Self {
        Self {
            bank: 0,
        }
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
                //
                ui.close_menu();
            }
            if ui.button("Save").clicked() {
                //
                ui.close_menu();
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

        ui.add(DragValue::new(&mut self.bank));
        ui.label("Bank");

        ui.separator();

        ui.add_space(ui.available_width());
    }

    fn editor(&mut self, ui: &mut Ui) {
        Frame::canvas(ui.style()).show(ui, |ui| {
            ScrollArea::new([true, true]).show_viewport(ui, |ui, viewport| {
                ui.allocate_space(ui.available_size());
                // contents of the editor
            });
        });
    }
}
