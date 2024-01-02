use clash_api::clash::Clash;
use eframe::egui::{FontDefinitions, ScrollArea, Separator};
// use eframe::epaint::ahash::HashMap;
use eframe::{egui::RichText, epaint::Color32, run_native};
use eframe::{
    egui::{self, Button, Layout},
    egui::{Context, Label},
    App, NativeOptions,
};
use tokio::runtime::Handle;
use tokio::task::block_in_place;

use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Index;
// use std::cell::RefCell;
use std::process;
// use std::sync::Arc;
pub const PADDING: f32 = 5.0;
#[derive(Default)]
pub struct MyApp {
    window_options: NativeOptions,
    // clash: Arc<RefCell<Clash>>,
    clash: Clash,
    passwd: String,
    base_url: String,
    selected_node: HashMap<String, String>,
}
impl App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.config_fonts(ctx);
        self.render_top_bar(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                self.render_access_site(ui);
                ScrollArea::vertical().show(ui, |ui| {
                    self.render_policy_group_site(ui);
                })
            });
        });
    }
}
impl MyApp {
    pub fn new() -> Self {
        Self::default()
    }
    fn config_fonts(&self, ctx: &Context) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(
            "fira_nerd_font".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../fonts/FiraCodeNerdFontMono-Regular.ttf"
            )),
        );
        font_def.font_data.insert(
            "noto_emoji_font".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/NotoEmoji-Regular.ttf")),
        );
        font_def.font_data.insert(
            "noto_serif_cjk_cf_sc".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/NotoSerifCJK-VF.ttc")),
        );
        font_def
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push("noto_serif_cjk_cf_sc".to_owned());
        font_def
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push("noto_emoji_font".to_owned());
        font_def
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "fira_nerd_font".to_owned());
        ctx.set_fonts(font_def);
    }
    fn render_top_bar(&self, ctx: &Context) {
        egui::TopBottomPanel::top("my_top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = ui.add(Button::new("X"));
                    if close_btn.clicked() {
                        process::exit(0);
                    }
                })
            });
        });
    }
    fn render_access_site(&mut self, ui: &mut eframe::egui::Ui) {
        ui.vertical_centered(|vui| {
            vui.horizontal(|hui| {
                hui.label("Address");
                hui.text_edit_singleline(&mut self.base_url);
            });
            vui.horizontal(|hui| {
                hui.label("Password");
                hui.text_edit_singleline(&mut self.passwd);
                let btm = hui.add(Button::new("Connect"));
                if btm.clicked() {
                    block_in_place(|| {
                        self.clash = Handle::current()
                            .block_on(Clash::new(&self.base_url, &self.passwd))
                            .unwrap_or_default();
                        self.clash.group_index.iter().for_each(|group| {
                            self.selected_node.insert(
                                group.clone(),
                                self.clash.proxies.get(group).unwrap().now.clone().unwrap(),
                            );
                        });
                        Handle::current()
                            .block_on(self.clash.query_group_delay("GLOBAL"))
                            .unwrap_or(HashMap::new());
                    })
                };
            });
        });
    }
    fn render_policy_group_site(&mut self, ui: &mut eframe::egui::Ui) {
        ScrollArea::new([false, false]).show(ui, |ui| {
            for group in &self.clash.clone().group_index {
                let group_delay = self
                    .clash
                    .delays
                    .get(&self.clash.proxies.get(group).unwrap().now.clone().unwrap())
                    .unwrap_or(&0);
                let group_delay_color = if *group_delay > 400 || *group_delay == 0 {
                    Color32::RED
                } else {
                    Color32::GREEN
                };
                let _ = egui::ComboBox::from_label(
                    RichText::new(format!("{} {}", group, group_delay)).color(group_delay_color),
                )
                .selected_text(RichText::new(
                    self.selected_node.get_mut(group).unwrap().clone(),
                ))
                .show_ui(ui, |ui| {
                    let mut selected_now_clone = self.selected_node.get_mut(group).unwrap().clone();
                    for node in &self
                        .clash
                        .clone()
                        .proxies
                        .get(group)
                        .unwrap()
                        .all
                        .clone()
                        .unwrap()
                    {
                        let sel_val = ui.selectable_value(
                            &mut selected_now_clone,
                            node.to_string(),
                            node.to_string(),
                        );
                        ui.add(Label::new(RichText::new(
                            self.clash.delays.get(node).unwrap_or(&0).to_string(),
                        )));
                        //Checking the status of every selectable_value, if changed, clash_api will be called
                        if sel_val.changed() {
                            block_in_place(|| {
                                Handle::current()
                                    .block_on(self.clash.select_proxy(group, &selected_now_clone))
                                    .unwrap();
                            });
                        }
                    }
                    self.selected_node.insert(group.clone(), selected_now_clone);
                });
                let btm = ui.add(Button::new("Ping"));
                if btm.clicked() {
                    block_in_place(|| {
                        Handle::current().block_on(self.clash.query_group_delay(group))
                    });
                }
            }
        });
    }
}
