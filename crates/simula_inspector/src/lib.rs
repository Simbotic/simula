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
        app.add_plugins((
            bevy_egui::EguiPlugin,
            bevy_inspector_egui::DefaultInspectorConfigPlugin,
        ))
        .insert_resource(Inspectors::default())
        .add_systems(Startup, setup_ui)
        .add_systems(Update, inspector_ui);
    }
}

fn setup_ui(mut contexts: EguiContexts) {
    const INSPECTOR_MONO_FONT: &str = "INSPECTOR_MONO_FONT";
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        INSPECTOR_MONO_FONT.into(),
        egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/JetBrainsMono-ExtraLight.ttf"
        )),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, INSPECTOR_MONO_FONT.into());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, INSPECTOR_MONO_FONT.into());
    contexts.ctx_mut().set_fonts(fonts);

    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = 2.0.into();
    visuals.window_shadow.extrusion = 0.0;
    // visuals.window_fill = egui::Color32::from_rgba_unmultiplied(52, 50, 55, 200);
    visuals.window_fill = egui::Color32::from_rgb(32, 30, 35);
    visuals.window_stroke = egui::Stroke::NONE;
    visuals.override_text_color = Some(egui::Color32::from_rgb(200, 200, 200));
    contexts.ctx_mut().set_visuals(visuals);

    let mut style = egui::Style::default();
    if let Some(text_style) = style.text_styles.get_mut(&egui::TextStyle::Body) {
        text_style.family = egui::FontFamily::Monospace;
        text_style.size = 12.0;
    }
    if let Some(text_style) = style.text_styles.get_mut(&egui::TextStyle::Button) {
        text_style.family = egui::FontFamily::Monospace;
        text_style.size = 12.0;
    }
    if let Some(text_style) = style.text_styles.get_mut(&egui::TextStyle::Heading) {
        text_style.family = egui::FontFamily::Monospace;
        text_style.size = 12.0;
    }
    if let Some(text_style) = style.text_styles.get_mut(&egui::TextStyle::Monospace) {
        text_style.family = egui::FontFamily::Monospace;
        text_style.size = 12.0;
    }
    if let Some(text_style) = style.text_styles.get_mut(&egui::TextStyle::Small) {
        text_style.family = egui::FontFamily::Monospace;
        text_style.size = 10.0;
    }
    contexts.ctx_mut().set_style(style);
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

    egui::TopBottomPanel::top("main-inspector-top").show(context.get_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            if ui.add(egui::Button::new("🌀").frame(false)).clicked() {
                println!("Button clicked!");
            }
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
