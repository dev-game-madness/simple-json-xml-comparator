use eframe::egui::{Color32, CornerRadius, Stroke, Visuals};

#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub page: Color32,
    pub panel: Color32,
    pub header: Color32,
    pub header_text: Color32,
    pub input: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub gutter: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub inactive: Color32,
    pub inactive_text: Color32,
    pub line: Color32,
    pub control: Color32,
    pub chrome_button: Color32,
    pub theme_fill: Color32,
    pub selection: Color32,
    pub removed: Color32,
    pub removed_text: Color32,
    pub added: Color32,
    pub added_text: Color32,
    // Цвета чипа «правка». Чип и подсветка сейчас отключены, поля сохранены.
    #[allow(dead_code)]
    pub fixed: Color32,
    #[allow(dead_code)]
    pub fixed_text: Color32,
}

impl Palette {
    pub fn new(dark: bool) -> Self {
        if dark {
            Self {
                dark: true,
                page: hex("#2C3E50"),
                panel: hex("#34495E"),
                header: hex("#243444"),
                header_text: hex("#FFFFFF"),
                input: hex("#1E2D3A"),
                text: hex("#FFFFFF"),
                muted: hex("#C5D0DA"),
                gutter: hex("#A6B5C2"),
                accent: hex("#4A9FE0"),
                on_accent: hex("#153247"),
                inactive: hex("#1A2733"),
                inactive_text: hex("#8FA0B0"),
                line: hex("#5C7084"),
                control: hex("#8293A4"),
                chrome_button: hex("#1A2733"),
                theme_fill: hex("#E5EBF1"),
                selection: hex("#7BD0F1"),
                removed: hex("#4A2422"),
                removed_text: hex("#F2B8B5"),
                added: hex("#1A4554"),
                added_text: hex("#7BD0F1"),
                fixed: hex("#4A341C"),
                fixed_text: hex("#FF9933"),
            }
        } else {
            Self {
                dark: false,
                page: hex("#F9F9F9"),
                panel: hex("#E5EBF1"),
                header: hex("#F9F9F9"),
                header_text: hex("#333333"),
                input: hex("#E5EBF1"),
                text: hex("#333333"),
                muted: hex("#5E6A75"),
                gutter: hex("#4D4D4D"),
                accent: hex("#005CA9"),
                on_accent: hex("#FFFFFF"),
                inactive: hex("#E5EBF1"),
                inactive_text: hex("#7B8794"),
                line: hex("#E0E0E0"),
                control: hex("#71859A"),
                chrome_button: hex("#E5EBF1"),
                theme_fill: hex("#2C3E50"),
                selection: hex("#0077B8"),
                removed: hex("#F8E4E2"),
                removed_text: hex("#B3261E"),
                added: hex("#D4F1FB"),
                added_text: hex("#1B6F8A"),
                fixed: hex("#FFF0E0"),
                fixed_text: hex("#8A4E00"),
            }
        }
    }

    pub fn visuals(&self) -> Visuals {
        let mut visuals = if self.dark { Visuals::dark() } else { Visuals::light() };
        visuals.window_fill = self.page;
        visuals.panel_fill = self.page;
        visuals.extreme_bg_color = self.input;
        visuals.faint_bg_color = self.panel;
        visuals.code_bg_color = self.input;
        visuals.override_text_color = Some(self.text);
        visuals.hyperlink_color = self.accent;
        visuals.window_stroke = Stroke::NONE;
        visuals.widgets.noninteractive.bg_fill = self.page;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text);
        visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
        visuals.widgets.inactive.bg_fill = self.input;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.control);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
        visuals.widgets.hovered.bg_fill = self.panel;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.control);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.on_accent);
        visuals.widgets.active.corner_radius = CornerRadius::same(8);
        visuals.widgets.open.corner_radius = CornerRadius::same(8);
        visuals.selection.bg_fill = self.selection.gamma_multiply(0.28);
        visuals.selection.stroke = Stroke::new(1.0, self.selection);
        visuals
    }
}

fn hex(value: &str) -> Color32 {
    let value = value.trim_start_matches('#');
    let number = u32::from_str_radix(value, 16).unwrap_or(0);
    Color32::from_rgb((number >> 16) as u8, (number >> 8) as u8, number as u8)
}
