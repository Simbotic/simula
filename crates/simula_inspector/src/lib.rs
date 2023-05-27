use bevy::{prelude::*, window::PrimaryWindow};

pub use bevy_inspector_egui::{
    self,
    bevy_egui::{self, EguiContext, EguiContexts},
    egui,
};
pub use world::WorldInspectorPlugin;

mod world;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_egui::EguiPlugin)
            .add_plugin(bevy_inspector_egui::DefaultInspectorConfigPlugin)
            .insert_resource(Inspectors::default())
            .add_startup_system(setup_ui)
            .add_system(inspector_ui);
    }
}

fn setup_ui(mut contexts: EguiContexts) {
    const TITLE_FONT_NAME: &str = "MY_FONT_NAME";
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        TITLE_FONT_NAME.into(),
        egui::FontData::from_static(include_bytes!("../../../assets/fonts/DejaVuSans.ttf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, TITLE_FONT_NAME.into());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, TITLE_FONT_NAME.into());
    contexts.ctx_mut().set_fonts(fonts);

    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = 2.0.into();
    visuals.window_shadow.extrusion = 0.0;
    visuals.window_fill = egui::Color32::from_rgba_unmultiplied(52, 50, 55, 140);
    visuals.window_stroke = egui::Stroke::NONE;
    contexts.ctx_mut().set_visuals(visuals);
}

#[derive(Clone)]
pub struct Inspector {
    pub menu_ui: fn(&mut egui::Ui, &mut World),
    pub window_ui: fn(context: &mut egui::Context, &mut World),
}

#[derive(Default, Resource, Clone)]
pub struct Inspectors {
    pub inspectors: Vec<Inspector>,
}

impl Inspectors {
    pub fn add(&mut self, inspector: Inspector) {
        self.inspectors.push(inspector);
    }
}

fn inspector_ui(world: &mut World) {
    let mut context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    let inspectors = world
        .get_resource::<Inspectors>()
        .unwrap()
        .inspectors
        .clone();

    egui::TopBottomPanel::top("top").show(context.get_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            for inspector in inspectors.iter() {
                ui.separator();
                (inspector.menu_ui)(ui, world);
            }
        });
    });

    for inspector in inspectors.iter() {
        (inspector.window_ui)(context.get_mut(), world);
    }
}
