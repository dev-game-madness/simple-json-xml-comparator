#![windows_subsystem = "windows"]

mod crash;
mod engine;
mod i18n;
mod theme;

use engine::{Diff, Issue, Kind, RowKind};
use i18n::Lang;
use theme::Palette;

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Frame, Layout, Margin, Rect,
    ResizeDirection, RichText, ScrollArea, Sense, Stroke, TextEdit, TextStyle, Vec2, ViewportCommand,
};

const APP_NAME: &str = "Simple JSON & XML Comparator";
const APP_TITLE: &str = concat!("Simple JSON & XML Comparator v", env!("CARGO_PKG_VERSION"));
const VERSION_MARK: &str = concat!("v", env!("CARGO_PKG_VERSION"));

fn main() -> eframe::Result {
    crash::install();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_inner_size([1360.0, 880.0])
            .with_min_inner_size([1040.0, 680.0])
            .with_title(APP_TITLE),
        ..Default::default()
    };
    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(CompareApp::new(cc)))),
    )
}

enum LoadNote {
    TooBig,
    Failed(String),
}

struct Pane {
    path: String,
    text: String,
    title: String,
    side_a: bool,
    issue: Option<Issue>,
    kind: Option<Kind>,
    load_note: Option<LoadNote>,
}

impl Pane {
    fn new(side_a: bool) -> Self {
        Self {
            path: String::new(),
            text: String::new(),
            title: String::new(),
            side_a,
            issue: None,
            kind: None,
            load_note: None,
        }
    }

    fn display_title(&self) -> String {
        if self.title.is_empty() {
            if self.side_a {
                i18n::tr("Окно A", "Pane A", "窗口 A").to_string()
            } else {
                i18n::tr("Окно B", "Pane B", "窗口 B").to_string()
            }
        } else {
            self.title.clone()
        }
    }

    fn clear_content(&mut self) {
        self.path.clear();
        self.text.clear();
        self.title.clear();
        self.kind = None;
        self.issue = None;
        self.load_note = None;
        self.refresh_validation();
    }

    fn refresh_validation(&mut self) {
        if self.text.len() > 16 * 1024 * 1024 {
            self.kind = None;
            self.issue = Some(Issue {
                line: 1,
                column: 1,
                message: i18n::tr(
                    "Текст больше 16 МБ. Проверка остановлена.",
                    "Text is larger than 16 MB. Validation stopped.",
                    "文本超过 16 MB。已停止校验。",
                )
                .into(),
            });
            return;
        }
        let text = self.text.clone();
        match crash::guard(move || engine::validate(&text)) {
            Ok(Ok(kind)) => {
                self.kind = Some(kind);
                self.issue = None;
            }
            Ok(Err(issue)) => {
                self.kind = None;
                self.issue = Some(issue);
            }
            Err(message) => {
                self.kind = None;
                self.issue = Some(Issue {
                    line: 1,
                    column: 1,
                    message: i18n::with_detail("Сбой проверки:", "Validation failure:", "校验失败：", &message),
                });
            }
        }
    }
}

enum Phrase {
    Welcome,
    UiCrash,
    CompareBlocked,
    Cleared,
    Beautified,
    NothingToFormat,
    FormatCrash(String),
    Repaired(Vec<i18n::Note>),
    NothingToRepair,
    AlreadyValid,
    RepairCrash(String),
    Unescaped,
    NothingEscaped,
    UnescapeCrash(String),
    DocTooLarge,
    NoDiff,
    DiffCounts { removed: usize, added: usize, changed: usize },
    CompareFail(String),
    CompareCrash(String),
    Raw(String),
}

impl Phrase {
    fn render(&self) -> String {
        match self {
            Phrase::Welcome => i18n::tr(
                "Вставьте JSON или XML либо откройте файл.",
                "Paste JSON or XML, or open a file.",
                "请粘贴 JSON 或 XML，或打开文件。",
            )
            .into(),
            Phrase::UiCrash => i18n::tr(
                "Сбой интерфейса. Подробности в журнале рядом с программой.",
                "Interface failure. Details are in the log next to the program.",
                "界面发生故障。详情见程序旁边的日志。",
            )
            .into(),
            Phrase::CompareBlocked => i18n::tr(
                "Сравнение недоступно: в файле ошибка проверки.",
                "Compare is unavailable: a file failed validation.",
                "无法比较：文件未通过校验。",
            )
            .into(),
            Phrase::Cleared => i18n::tr("Окно очищено.", "Pane cleared.", "窗口已清空。").into(),
            Phrase::Beautified => i18n::tr(
                "Beautify: текст отформатирован.",
                "Beautify: text formatted.",
                "Beautify：文本已格式化。",
            )
            .into(),
            Phrase::NothingToFormat => i18n::tr("Нечего форматировать", "Nothing to format", "没有可格式化的内容").into(),
            Phrase::FormatCrash(message) => i18n::with_detail(
                "Сбой форматирования. Подробности в журнале рядом с программой.",
                "Format failure. Details are in the log next to the program.",
                "格式化失败。详情见程序旁边的日志。",
                message,
            ),
            Phrase::Repaired(notes) => format!(
                "{} {}",
                i18n::tr("Исправлено:", "Repaired:", "已修复："),
                i18n::join_notes(notes)
            ),
            Phrase::NothingToRepair => i18n::tr("Нечего исправлять", "Nothing to repair", "没有可修复的内容").into(),
            Phrase::AlreadyValid => i18n::tr(
                "Исправлять нечего: документ уже корректен.",
                "Nothing to repair: the document is already valid.",
                "无需修复：文档已经正确。",
            )
            .into(),
            Phrase::RepairCrash(message) => i18n::with_detail(
                "Сбой исправления. Подробности в журнале рядом с программой.",
                "Repair failure. Details are in the log next to the program.",
                "修复失败。详情见程序旁边的日志。",
                message,
            ),
            Phrase::Unescaped => i18n::tr(
                "Экранирование убрано: \\\" заменены на кавычки, строки с объектами раскрыты.",
                "Escapes removed: \\\" replaced with quotes, and object strings expanded.",
                "已去除转义：\\\" 已替换为引号，对象字符串已展开。",
            )
            .into(),
            Phrase::NothingEscaped => i18n::tr(
                "Экранированных кавычек и строк-объектов нет.",
                "No escaped quotes or object strings.",
                "没有转义引号或对象字符串。",
            )
            .into(),
            Phrase::UnescapeCrash(message) => i18n::with_detail(
                "Сбой снятия экранирования. Подробности в журнале рядом с программой.",
                "Unescape failure. Details are in the log next to the program.",
                "去除转义失败。详情见程序旁边的日志。",
                message,
            ),
            Phrase::DocTooLarge => i18n::tr(
                "Документ больше 8 МБ. Операция остановлена.",
                "Document is larger than 8 MB. Operation stopped.",
                "文档超过 8 MB。操作已停止。",
            )
            .into(),
            Phrase::NoDiff => i18n::tr(
                "Отличий нет. Порядок ключей JSON и атрибутов XML не учитывается.",
                "No differences. JSON key order and XML attribute order are ignored.",
                "没有差异。JSON 键顺序和 XML 属性顺序不计入比较。",
            )
            .into(),
            Phrase::DiffCounts { removed, added, changed } => i18n::diff_counts(*removed, *added, *changed),
            Phrase::CompareFail(message) | Phrase::Raw(message) => message.clone(),
            Phrase::CompareCrash(message) => i18n::with_detail(
                "Сбой сравнения. Подробности в журнале рядом с программой.",
                "Compare failure. Details are in the log next to the program.",
                "比较失败。详情见程序旁边的日志。",
                message,
            ),
        }
    }
}

struct CompareApp {
    left: Pane,
    right: Pane,
    dark: bool,
    lang: Lang,
    ignore_order: bool,
    only_changes: bool,
    show_removed: bool,
    show_added: bool,
    show_fixed: bool,
    show_lines: bool,
    mode_diff: bool,
    diff: Option<Diff>,
    status: Phrase,
    fix_spans: Vec<(bool, usize, usize)>,
    diff_scroll: f32,
}

impl CompareApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_fonts(cc);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(Palette::new(true).visuals());
        let lang = i18n::load();
        i18n::set_current(lang);
        Self {
            left: Pane::new(true),
            right: Pane::new(false),
            dark: true,
            lang,
            ignore_order: false,
            only_changes: false,
            show_removed: true,
            show_added: true,
            show_fixed: true,
            show_lines: true,
            mode_diff: false,
            diff: None,
            status: Phrase::Welcome,
            fix_spans: Vec::new(),
            diff_scroll: 0.0,
        }
    }

    fn palette(&self) -> Palette {
        Palette::new(self.dark)
    }

    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.dark = !self.dark;
        ctx.set_visuals(self.palette().visuals());
    }

    fn apply_lang(&mut self, lang: Lang) {
        if self.lang == lang {
            return;
        }
        self.lang = lang;
        i18n::set_current(lang);
        i18n::save(lang);
        self.left.refresh_validation();
        self.right.refresh_validation();
        if self.mode_diff || matches!(self.status, Phrase::CompareFail(_)) {
            refresh_compare(self, false);
        }
    }
}

impl eframe::App for CompareApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.draw(ctx, frame)));
        if let Err(payload) = outcome {
            let message = panic_text(payload.as_ref());
            crash::record(&message, "перехвачено, работа продолжена");
            self.status = Phrase::UiCrash;
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.palette().page.to_normalized_gamma_f32()
    }
}

impl CompareApp {
    fn draw(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        i18n::set_current(self.lang);
        let palette = self.palette();
        ctx.set_visuals(palette.visuals());
        pad_buttons(ctx);
        window_resize_edges(ctx);
        title_bar(ctx, self, &palette);

        egui::TopBottomPanel::bottom("status")
            .exact_height(32.0)
            .frame(flat_frame(palette.page))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(16.0);
                    ui.label(RichText::new(self.status.render()).color(palette.muted).size(15.0));
                });
            });

        egui::CentralPanel::default()
            .frame(flat_frame(palette.page))
            .show(ctx, |ui| {
            toolbar(ui, self, &palette);
            ui.add_space(8.0);
            if let Some(message) = editors(ui, self, &palette) {
                self.status = message;
            }
        });
    }
}

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "неизвестная ошибка".to_string()
    }
}

fn install_face(fonts: &mut egui::FontDefinitions, name: &str, path: &str, index: u32) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let mut data = egui::FontData::from_owned(bytes);
    data.index = index;
    fonts.font_data.insert(name.to_owned(), std::sync::Arc::new(data));
    true
}

fn apply_fonts(cc: &eframe::CreationContext<'_>) {
    pad_buttons(&cc.egui_ctx);
    let mut fonts = egui::FontDefinitions::default();
    if install_face(&mut fonts, "segoe", r"C:\Windows\Fonts\segoeui.ttf", 0) {
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "segoe".to_owned());
    }
    if install_face(&mut fonts, "yahei", r"C:\Windows\Fonts\msyh.ttc", 0) {
        fonts.families.entry(FontFamily::Proportional).or_default().push("yahei".to_owned());
        fonts.families.entry(FontFamily::Monospace).or_default().push("yahei".to_owned());
    }
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeuib.ttf") {
        fonts.font_data.insert(
            "segoe-bold".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        fonts
            .families
            .entry(FontFamily::Name("bold".into()))
            .or_default()
            .insert(0, "segoe-bold".to_owned());
    }
    if install_face(&mut fonts, "yahei-bold", r"C:\Windows\Fonts\msyhbd.ttc", 0) {
        fonts.families.entry(FontFamily::Name("bold".into())).or_default().push("yahei-bold".to_owned());
    }
    cc.egui_ctx.set_fonts(fonts);
    let mut style = (*cc.egui_ctx.style()).clone();
    style.text_styles.insert(TextStyle::Monospace, FontId::new(15.0, FontFamily::Monospace));
    style.text_styles.insert(TextStyle::Body, FontId::new(16.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Button, FontId::new(15.0, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Small, FontId::new(15.0, FontFamily::Proportional));
    cc.egui_ctx.set_style(style);
}

fn pad_buttons(ctx: &egui::Context) {
    ctx.style_mut(|style| {
        style.spacing.button_padding = egui::vec2(16.0, 0.0);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    });
}

fn flat_frame(fill: Color32) -> Frame {
    Frame::new().fill(fill).stroke(Stroke::NONE).inner_margin(Margin::same(12))
}

fn title_bar(ctx: &egui::Context, app: &mut CompareApp, palette: &Palette) {
    egui::TopBottomPanel::top("window_chrome")
        .exact_height(48.0)
        .frame(flat_frame(palette.header).inner_margin(Margin::ZERO))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let reserved = 460.0;
            let drag_rect = Rect::from_min_max(rect.min, egui::pos2((rect.max.x - reserved).max(rect.min.x + 16.0), rect.max.y));
            let drag = ui.interact(drag_rect, ui.id().with("window_drag"), Sense::click_and_drag());
            if drag.drag_started() {
                ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            }
            if drag.double_clicked() {
                let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
                ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!maximized));
            }
            ui.painter().hline(rect.x_range(), rect.bottom(), Stroke::new(1.0, palette.line));
            let painter = ui.painter().with_clip_rect(drag_rect);
            let name_rect = painter.text(
                drag_rect.left_center() + egui::vec2(16.0, 0.0),
                Align2::LEFT_CENTER,
                APP_NAME,
                FontId::new(18.0, FontFamily::Name("bold".into())),
                palette.header_text,
            );
            painter.text(
                egui::pos2(name_rect.right() + 2.0, name_rect.top() - 3.0),
                Align2::LEFT_TOP,
                VERSION_MARK,
                FontId::new(11.0, FontFamily::Name("bold".into())),
                palette.header_text,
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if chrome_button(ui, Chrome::Close, palette) {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                }
                let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
                if chrome_button(ui, if maximized { Chrome::Restore } else { Chrome::Max }, palette) {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(!maximized));
                }
                if chrome_button(ui, Chrome::Min, palette) {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
                }
                ui.add_space(8.0);
                if theme_button(ui, app.dark, palette) {
                    app.toggle_theme(ctx);
                }
                ui.add_space(8.0);
                language_control(ui, app, palette);
            });
        });
}

#[derive(Clone, Copy)]
enum Chrome {
    Close,
    Min,
    Max,
    Restore,
}

fn chrome_button(ui: &mut egui::Ui, kind: Chrome, palette: &Palette) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(46.0, 48.0), Sense::click());
    let close = matches!(kind, Chrome::Close);
    let mark = Rect::from_center_size(rect.center(), Vec2::splat(32.0));
    let fill = if response.hovered() {
        if close {
            hex_color("#B3261E")
        } else if palette.dark {
            hex_color("#243444")
        } else {
            hex_color("#D5DEE8")
        }
    } else {
        palette.chrome_button
    };
    ui.painter().rect_filled(mark, 8.0, fill);
    let color = if close && response.hovered() { Color32::WHITE } else { palette.header_text };
    let stroke = Stroke::new(1.6, color);
    let center = rect.center();
    let arm = 5.5;
    match kind {
        Chrome::Close => {
            ui.painter().line_segment([center + egui::vec2(-arm, -arm), center + egui::vec2(arm, arm)], stroke);
            ui.painter().line_segment([center + egui::vec2(arm, -arm), center + egui::vec2(-arm, arm)], stroke);
        }
        Chrome::Min => {
            ui.painter().line_segment([center + egui::vec2(-arm, 0.0), center + egui::vec2(arm, 0.0)], stroke);
        }
        Chrome::Max => {
            let mark = Rect::from_center_size(center, Vec2::splat(arm * 2.0));
            ui.painter().rect_stroke(mark, 0.0, stroke, egui::StrokeKind::Inside);
        }
        Chrome::Restore => {
            let back = Rect::from_min_size(center + egui::vec2(-arm + 2.0, -arm), Vec2::splat(arm * 1.7));
            let front = Rect::from_min_size(center + egui::vec2(-arm, -arm + 3.0), Vec2::splat(arm * 1.7));
            ui.painter().rect_stroke(back, 0.0, stroke, egui::StrokeKind::Inside);
            ui.painter().rect_stroke(front, 0.0, stroke, egui::StrokeKind::Inside);
        }
    }
    response.clicked()
}

fn window_resize_edges(ctx: &egui::Context) {
    let thickness = 5.0;
    let screen = ctx.screen_rect();
    let edges = [
        ("n", ResizeDirection::North, Rect::from_min_max(screen.min, egui::pos2(screen.max.x, screen.min.y + thickness))),
        ("s", ResizeDirection::South, Rect::from_min_max(egui::pos2(screen.min.x, screen.max.y - thickness), screen.max)),
        ("w", ResizeDirection::West, Rect::from_min_max(screen.min, egui::pos2(screen.min.x + thickness, screen.max.y))),
        ("e", ResizeDirection::East, Rect::from_min_max(egui::pos2(screen.max.x - thickness, screen.min.y), screen.max)),
    ];
    for (name, direction, rect) in edges {
        egui::Area::new(egui::Id::new(("resize", name)))
            .order(egui::Order::Foreground)
            .fixed_pos(rect.min)
            .show(ctx, |ui| {
                let response = ui.interact(rect, ui.id().with(name), Sense::drag());
                if response.drag_started() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::BeginResize(direction));
                }
            });
    }
}

fn hex_color(value: &str) -> Color32 {
    let value = value.trim_start_matches('#');
    let number = u32::from_str_radix(value, 16).unwrap_or(0);
    Color32::from_rgb((number >> 16) as u8, (number >> 8) as u8, number as u8)
}

fn toolbar(ui: &mut egui::Ui, app: &mut CompareApp, palette: &Palette) {
    const ROW: f32 = 40.0;
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = ROW;
        ui.spacing_mut().button_padding = egui::vec2(16.0, 0.0);
        ui.style_mut().override_font_id = Some(FontId::proportional(15.0));

        ui.horizontal(|ui| {
            if state_button(ui, i18n::tr("Сравнение", "Compare", "比较"), app.mode_diff, true, palette, ROW) {
                toggle_compare(app);
            }
            if ghost_button(ui, i18n::tr("Поменять местами", "Swap", "交换"), palette, ROW) {
                std::mem::swap(&mut app.left, &mut app.right);
                for (left, _, _) in &mut app.fix_spans {
                    *left = !*left;
                }
                if app.mode_diff {
                    refresh_compare(app, true);
                }
            }
            if ghost_button(ui, i18n::tr("Отчёт HTML", "HTML report", "HTML 报告"), palette, ROW) {
                save_report(app);
            }
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if state_button(ui, i18n::tr("Упорядочить структуру", "Order structure", "整理结构"), app.ignore_order, app.mode_diff, palette, ROW) {
                app.ignore_order = !app.ignore_order;
                if app.mode_diff {
                    refresh_compare(app, true);
                }
            }
            if state_button(ui, i18n::tr("Только отличия", "Only differences", "仅差异"), app.only_changes, app.mode_diff, palette, ROW) {
                app.only_changes = !app.only_changes;
                if app.mode_diff {
                    refresh_compare(app, true);
                }
            }
            if state_button(ui, i18n::tr("Номера строк", "Line numbers", "行号"), app.show_lines, true, palette, ROW) {
                app.show_lines = !app.show_lines;
            }
            ui.add_space(16.0);
            chip(ui, &mut app.show_removed, i18n::tr("удалено", "removed", "已删除"), palette.removed, palette.removed_text, palette, ROW);
            chip(ui, &mut app.show_added, i18n::tr("добавлено", "added", "已添加"), palette.added, palette.added_text, palette, ROW);
            // chip(ui, &mut app.show_fixed, "правка", palette.fixed, palette.fixed_text, palette, ROW);
        });
    });
}

fn toggle_compare(app: &mut CompareApp) {
    if app.mode_diff {
        app.mode_diff = false;
        return;
    }
    app.left.refresh_validation();
    app.right.refresh_validation();
    if app.left.issue.is_some() || app.right.issue.is_some() || app.left.load_note.is_some() || app.right.load_note.is_some() {
        app.status = Phrase::CompareBlocked;
        return;
    }
    refresh_compare(app, true);
}

fn editors(ui: &mut egui::Ui, app: &mut CompareApp, palette: &Palette) -> Option<Phrase> {
    let mut note = None;
    let height = ui.available_height();
    let editing = !app.mode_diff;
    let show_lines = app.show_lines;
    let rows = app.diff.as_ref().map(|diff| diff.rows.clone());
    ui.set_min_height(height);
    ui.columns(2, |columns| {
        columns[0].set_min_height(height);
        columns[1].set_min_height(height);
        if let Some(message) = editor_column(
            &mut columns[0],
            &mut app.left,
            &mut app.fix_spans,
            &mut app.diff_scroll,
            true,
            editing,
            rows.as_deref(),
            app.show_removed,
            app.show_added,
            app.show_fixed,
            show_lines,
            palette,
        ) {
            note = Some(message);
        }
        if let Some(message) = editor_column(
            &mut columns[1],
            &mut app.right,
            &mut app.fix_spans,
            &mut app.diff_scroll,
            false,
            editing,
            rows.as_deref(),
            app.show_removed,
            app.show_added,
            app.show_fixed,
            show_lines,
            palette,
        ) {
            note = Some(message);
        }
    });
    note
}

fn editor_column(
    ui: &mut egui::Ui,
    pane: &mut Pane,
    fix_spans: &mut Vec<(bool, usize, usize)>,
    diff_scroll: &mut f32,
    is_left: bool,
    editing: bool,
    rows: Option<&[engine::Row]>,
    show_removed: bool,
    show_added: bool,
    show_fixed: bool,
    show_lines: bool,
    palette: &Palette,
) -> Option<Phrase> {
    let mut note = None;
    Frame::new()
        .fill(palette.page)
        .stroke(egui::Stroke::new(1.0, palette.control))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add_enabled(editing, pane_button(i18n::tr("Обзор", "Browse", "浏览"), palette)).clicked() {
                    if let Some(path) = pick_file() {
                        load_into(pane, &path);
                        fix_spans.retain(|(left, _, _)| *left != is_left);
                    }
                }
                ui.add_space(8.0);
                let rest = ui.available_size();
                ui.allocate_ui_with_layout(Vec2::new(rest.x, 40.0), Layout::right_to_left(Align::Center), |ui| {
                    if ui.add_enabled(editing, pane_button(i18n::tr("Очистить", "Clear", "清空"), palette)).clicked() {
                        pane.clear_content();
                        fix_spans.retain(|(left, _, _)| *left != is_left);
                        note = Some(Phrase::Cleared);
                    }
                    let title = pane.display_title();
                    ui.add_sized(
                        Vec2::new(ui.available_width(), 40.0),
                        egui::Label::new(RichText::new(title).strong().size(15.0).color(palette.text))
                            .truncate()
                            .halign(Align::LEFT),
                    );
                });
            });
            ui.horizontal(|ui| {
                if ui.add_enabled(editing, pane_button("Beautify", palette)).clicked() {
                    let text = pane.text.clone();
                    note = Some(match crash::guard(move || engine::beautify(&text)) {
                        Ok(Ok(pretty)) => {
                            pane.text = pretty;
                            pane.load_note = None;
                            pane.refresh_validation();
                            Phrase::Beautified
                        }
                        Ok(Err(issue)) => phrase_from_issue(&issue),
                        Err(message) => Phrase::FormatCrash(message),
                    });
                }
                if ui.add_enabled(editing, pane_button(i18n::tr("Исправить*", "Repair*", "修复*"), palette)).clicked() {
                    let text = pane.text.clone();
                    note = Some(match crash::guard(move || engine::repair(&text)) {
                        Ok(Ok((updated, notes))) if updated != pane.text && !notes.is_empty() => {
                            pane.text = updated;
                            pane.load_note = None;
                            pane.refresh_validation();
                            Phrase::Repaired(notes)
                        }
                        Ok(Ok(_)) => Phrase::AlreadyValid,
                        Ok(Err(issue)) => phrase_from_issue(&issue),
                        Err(message) => Phrase::RepairCrash(message),
                    });
                }
                if ui.add_enabled(editing, pane_button(i18n::tr("Убрать экранирование", "Unescape", "去除转义"), palette)).clicked() {
                    if pane.text.len() > engine::MAX_DOCUMENT_BYTES {
                        note = Some(Phrase::DocTooLarge);
                    } else {
                        let text = pane.text.clone();
                        note = Some(match crash::guard(move || engine::unescape_quotes(&text)) {
                            Ok((updated, spans)) if updated != pane.text => {
                                pane.text = updated;
                                pane.load_note = None;
                                pane.refresh_validation();
                                fix_spans.retain(|(left, _, _)| *left != is_left);
                                fix_spans.extend(spans.into_iter().map(|(start, end)| (is_left, start, end)));
                                Phrase::Unescaped
                            }
                            Ok(_) => Phrase::NothingEscaped,
                            Err(message) => Phrase::UnescapeCrash(message),
                        });
                    }
                }
            });
            validation_line(ui, pane, palette);
            let size = ui.available_size();
            // Подсветка диапазонов «правка» отключена вместе с чипом.
            // let spans: Vec<(usize, usize)> = if show_fixed {
            //     fix_spans.iter().filter(|(left, _, _)| *left == is_left).map(|(_, start, end)| (*start, *end)).collect()
            // } else {
            //     Vec::new()
            // };
            let _show_fixed = show_fixed;
            let document = if editing {
                ViewText {
                    text: pane.text.clone(),
                    lines: Vec::new(),
                    spans: Vec::new(),
                }
            } else {
                diff_document(rows.unwrap_or(&[]), is_left, show_removed, show_added, palette)
            };
            let mut shown = if editing { String::new() } else { document.text.clone() };
            let buffer = if editing { &mut pane.text } else { &mut shown };
            let changed = code_view(
                ui,
                buffer,
                editing,
                show_lines,
                &document.lines,
                &document.spans,
                if is_left { "code-left" } else { "code-right" },
                if editing { None } else { Some(diff_scroll) },
                palette,
                size,
            );
            if editing && changed {
                pane.load_note = None;
                pane.refresh_validation();
                fix_spans.retain(|(left, _, _)| *left != is_left);
            }
        });
    note
}

fn text_font() -> FontId {
    FontId::monospace(15.0)
}

fn number_font() -> FontId {
    FontId::monospace(13.0)
}

fn paint_spans(
    text: &str,
    spans: &[(usize, usize)],
    color: Color32,
    background: Color32,
    line_height: Option<f32>,
    job: &mut egui::text::LayoutJob,
) {
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    while index < chars.len() {
        let marked = spans.iter().any(|(start, end)| index >= *start && index < *end);
        let mut end = index + 1;
        while end < chars.len() && spans.iter().any(|(start, stop)| end >= *start && end < *stop) == marked {
            end += 1;
        }
        let slice: String = chars[index..end].iter().collect();
        job.append(
            &slice,
            0.0,
            egui::TextFormat {
                font_id: text_font(),
                line_height,
                color,
                background: if marked { background } else { Color32::TRANSPARENT },
                ..Default::default()
            },
        );
        index = end;
    }
}

fn paint_editor_lines(ui: &egui::Ui, galley: &egui::Galley, galley_pos: egui::Pos2, editor_left: f32, color: Color32, clip: Rect) {
    let painter = ui.painter().with_clip_rect(clip);
    let font = number_font();
    let mut line = 1usize;
    let mut at_line_start = true;
    for row in &galley.rows {
        if at_line_start {
            let rect = row.rect().translate(galley_pos.to_vec2());
            if rect.intersects(clip) {
                painter.text(
                    egui::pos2(editor_left - 8.0, rect.center().y),
                    Align2::RIGHT_CENTER,
                    line.to_string(),
                    font.clone(),
                    color,
                );
            }
            line += 1;
        }
        at_line_start = row.ends_with_newline;
    }
}

struct LinePaint {
    fg: Color32,
    bg: Option<Color32>,
}

struct ViewText {
    text: String,
    lines: Vec<LinePaint>,
    spans: Vec<(usize, usize)>,
}

fn diff_document(rows: &[engine::Row], is_left: bool, show_removed: bool, show_added: bool, palette: &Palette) -> ViewText {
    let mut text = String::new();
    let mut lines = Vec::new();
    let mut spans = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            text.push('\n');
        }
        let line = if is_left { &row.left } else { &row.right };
        let marked = match row.kind {
            RowKind::Del if is_left && show_removed => true,
            RowKind::Add if !is_left && show_added => true,
            RowKind::Chg if is_left && show_removed => true,
            RowKind::Chg if !is_left && show_added => true,
            _ => false,
        };
        let (bg, fg) = if marked {
            if is_left {
                (Some(palette.removed), palette.removed_text)
            } else {
                (Some(palette.added), palette.added_text)
            }
        } else if row.kind == RowKind::Skip {
            (Some(palette.header), palette.muted)
        } else {
            (None, palette.text)
        };
        if row.kind == RowKind::Chg && marked {
            let (left_spans, right_spans) = engine::char_diff(&row.left, &row.right);
            let local = if is_left { left_spans } else { right_spans };
            let base = text.chars().count();
            spans.extend(local.into_iter().map(|(start, end)| (base + start, base + end)));
        }
        lines.push(LinePaint { fg, bg });
        text.push_str(line);
    }
    ViewText { text, lines, spans }
}

fn layout_document(
    ui: &egui::Ui,
    text: &str,
    wrap: f32,
    lines: &[LinePaint],
    spans: &[(usize, usize)],
    fallback: Color32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap;
    if text.is_empty() {
        job.append(
            " ",
            0.0,
            egui::TextFormat {
                font_id: text_font(),
                color: Color32::TRANSPARENT,
                ..Default::default()
            },
        );
        return ui.fonts(|fonts| fonts.layout_job(job));
    }
    let span_bg = Color32::from_white_alpha(48);
    if lines.is_empty() {
        paint_spans(text, spans, fallback, span_bg, None, &mut job);
    } else {
        let parts: Vec<&str> = text.split('\n').collect();
        let mut offset = 0usize;
        for (index, line) in parts.iter().enumerate() {
            let fg = lines.get(index).map(|style| style.fg).unwrap_or(fallback);
            let len = line.chars().count();
            let local: Vec<(usize, usize)> = spans
                .iter()
                .filter_map(|(start, end)| {
                    if *end <= offset || *start >= offset + len {
                        None
                    } else {
                        Some((start.saturating_sub(offset), end.saturating_sub(offset).min(len)))
                    }
                })
                .collect();
            paint_spans(line, &local, fg, span_bg, None, &mut job);
            offset += len;
            if index + 1 < parts.len() {
                job.append(
                    "\n",
                    0.0,
                    egui::TextFormat {
                        font_id: text_font(),
                        color: fg,
                        ..Default::default()
                    },
                );
                offset += 1;
            }
        }
    }
    ui.fonts(|fonts| fonts.layout_job(job))
}

fn paint_line_backgrounds(ui: &egui::Ui, origin: egui::Pos2, top: f32, galley: &egui::Galley, lines: &[LinePaint], width: f32) {
    let mut line = 0usize;
    for row in &galley.rows {
        if let Some(color) = lines.get(line).and_then(|style| style.bg) {
            let y = origin.y + top + row.pos.y;
            ui.painter().rect_filled(Rect::from_min_size(egui::pos2(origin.x, y), Vec2::new(width, row.rect().height())), 0.0, color);
        }
        if row.ends_with_newline {
            line += 1;
        }
    }
}

fn code_view(
    ui: &mut egui::Ui,
    text: &mut String,
    editable: bool,
    show_lines: bool,
    lines: &[LinePaint],
    spans: &[(usize, usize)],
    id: &'static str,
    mut scroll: Option<&mut f32>,
    palette: &Palette,
    viewport: Vec2,
) -> bool {
    let (rect, _) = ui.allocate_exact_size(viewport, Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(6), palette.input);
    let mut changed = false;
    let offset = scroll.as_deref().copied();
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        let mut area = ScrollArea::vertical().id_salt(id).auto_shrink([false, false]);
        if let Some(offset) = offset {
            area = area.vertical_scroll_offset(offset);
        }
        let output = area.show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            ui.horizontal(|ui| {
                if show_lines {
                    ui.add_space(48.0);
                }
                let column = ui.available_width().max(24.0);
                let margin = Margin::symmetric(8, 4);
                let origin = ui.cursor().min;
                let wrap = (column - margin.sum().x).max(24.0);
                let preview = layout_document(ui, text, wrap, lines, spans, palette.text);
                paint_line_backgrounds(ui, origin, margin.sum().y / 2.0, &preview, lines, column);
                let mut layouter = |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, wrap: f32| {
                    layout_document(ui, buffer.as_str(), wrap, lines, spans, palette.text)
                };
                let shown = if editable {
                    TextEdit::multiline(text)
                        .id_salt(id)
                        .font(egui::FontSelection::FontId(text_font()))
                        .desired_width(wrap)
                        .min_size(Vec2::new(wrap, (viewport.y - margin.sum().y).max(40.0)))
                        .margin(margin)
                        .code_editor()
                        .text_color(palette.text)
                        .frame(false)
                        .layouter(&mut layouter)
                        .show(ui)
                } else {
                    let mut frozen: &str = text.as_str();
                    TextEdit::multiline(&mut frozen)
                        .id_salt(id)
                        .font(egui::FontSelection::FontId(text_font()))
                        .desired_width(wrap)
                        .min_size(Vec2::new(wrap, (viewport.y - margin.sum().y).max(40.0)))
                        .margin(margin)
                        .code_editor()
                        .text_color(palette.text)
                        .frame(false)
                        .layouter(&mut layouter)
                        .show(ui)
                };
                if show_lines {
                    paint_editor_lines(ui, &shown.galley, shown.galley_pos, shown.response.rect.left(), palette.gutter, rect);
                }
                changed = editable && shown.response.changed();
            });
        });
        if let Some(scroll) = scroll.as_deref_mut() {
            let pointer = ui.input(|input| input.pointer.hover_pos());
            if pointer.is_some_and(|pos| output.inner_rect.contains(pos)) {
                *scroll = output.state.offset.y;
            }
        }
    });
    changed
}

fn phrase_from_issue(issue: &Issue) -> Phrase {
    let nothing_format = i18n::tr("Нечего форматировать", "Nothing to format", "没有可格式化的内容");
    let nothing_repair = i18n::tr("Нечего исправлять", "Nothing to repair", "没有可修复的内容");
    let too_big = i18n::tr(
        "Документ больше 8 МБ. Операция остановлена.",
        "Document is larger than 8 MB. Operation stopped.",
        "文档超过 8 MB。操作已停止。",
    );
    if issue.message == nothing_format {
        Phrase::NothingToFormat
    } else if issue.message == nothing_repair {
        Phrase::NothingToRepair
    } else if issue.message == too_big {
        Phrase::DocTooLarge
    } else {
        Phrase::Raw(issue.message.clone())
    }
}

fn validation_line(ui: &mut egui::Ui, pane: &Pane, palette: &Palette) {
    let (text, color) = if let Some(note) = &pane.load_note {
        let text = match note {
            LoadNote::TooBig => i18n::tr(
                "Файл больше 16 МБ, он не открыт.",
                "File is larger than 16 MB and was not opened.",
                "文件超过 16 MB，未打开。",
            )
            .to_string(),
            LoadNote::Failed(err) => i18n::with_detail("Не удалось прочитать файл:", "Could not read the file:", "无法读取文件：", err),
        };
        (text, palette.removed_text)
    } else if pane.text.trim().is_empty() {
        (
            i18n::tr(
                "Пусто — вставьте JSON, XML или откройте файл",
                "Empty — paste JSON or XML, or open a file",
                "空 — 请粘贴 JSON 或 XML，或打开文件",
            )
            .to_string(),
            palette.muted,
        )
    } else if let Some(issue) = &pane.issue {
        (issue.message.clone(), palette.removed_text)
    } else {
        let name = match pane.kind {
            Some(Kind::Xml) => i18n::tr("XML корректен", "XML is valid", "XML 正确"),
            _ => i18n::tr("JSON корректен", "JSON is valid", "JSON 正确"),
        };
        (name.to_string(), palette.added_text)
    };
    let response = ui.add(egui::Label::new(RichText::new(text).color(color).size(15.0)).sense(egui::Sense::click()));
    if response.clicked() {
        if let Some(issue) = &pane.issue {
            // Cursor jump is handled by focusing the editor; the line is shown in the message.
            let _ = issue.line;
        }
    }
}

fn state_button(ui: &mut egui::Ui, text: &str, active: bool, enabled: bool, palette: &Palette, height: f32) -> bool {
    let fill = if active { palette.accent } else { palette.inactive };
    let color = if active { palette.on_accent } else { palette.inactive_text };
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(text).color(color).size(15.0))
            .fill(fill)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(10))
            .min_size(Vec2::new(0.0, height)),
    )
    .clicked()
}

fn theme_button(ui: &mut egui::Ui, dark: bool, palette: &Palette) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::click());
    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, CornerRadius::same(10), palette.theme_fill);
        let (uri, bytes, tint): (&str, &[u8], Color32) = if dark {
            ("bytes://theme-dark.png", include_bytes!("../dark.png"), Color32::BLACK)
        } else {
            ("bytes://theme-light.png", include_bytes!("../light.png"), Color32::WHITE)
        };
        let icon = Rect::from_center_size(rect.center(), Vec2::splat(20.0));
        egui::Image::from_bytes(uri, bytes).tint(tint).paint_at(ui, icon);
    }
    response.clicked()
}

const FLAG_W: f32 = 16.0;
const FLAG_H: f32 = 11.0;

fn language_control(ui: &mut egui::Ui, app: &mut CompareApp, palette: &Palette) {
    let response = lang_menu_button(ui, app.lang, palette);
    egui::Popup::menu(&response)
        .width(response.rect.width().max(120.0))
        .show(|ui| {
            ui.set_min_width(response.rect.width().max(120.0));
            ui.spacing_mut().item_spacing.y = 2.0;
            for lang in [Lang::Ru, Lang::En, Lang::Zh] {
                if lang_choice(ui, lang, lang == app.lang, palette) {
                    app.apply_lang(lang);
                    ui.close();
                }
            }
        });
    ui.label(RichText::new(i18n::tr("Язык", "Language", "语言")).color(palette.header_text).size(15.0));
}

fn lang_menu_button(ui: &mut egui::Ui, lang: Lang, palette: &Palette) -> egui::Response {
    let font = FontId::new(14.0, FontFamily::Proportional);
    let text_width = ui.fonts(|fonts| fonts.layout_no_wrap(lang.badge().to_owned(), font.clone(), palette.header_text).rect.width());
    let width = 8.0 + text_width + 6.0 + FLAG_W + 8.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 36.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let fill = if response.hovered() { palette.panel } else { palette.chrome_button };
        ui.painter().rect_filled(rect, CornerRadius::same(10), fill);
        ui.painter().text(
            egui::pos2(rect.left() + 8.0, rect.center().y),
            Align2::LEFT_CENTER,
            lang.badge(),
            font,
            palette.header_text,
        );
        let flag = Rect::from_min_size(
            egui::pos2(rect.right() - 8.0 - FLAG_W, rect.center().y - FLAG_H / 2.0),
            Vec2::new(FLAG_W, FLAG_H),
        );
        paint_flag(ui.painter(), flag, lang);
    }
    response
}

fn lang_choice(ui: &mut egui::Ui, lang: Lang, selected: bool, palette: &Palette) -> bool {
    let width = ui.available_width().max(120.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 30.0), Sense::click());
    if ui.is_rect_visible(rect) {
        if selected || response.hovered() {
            ui.painter().rect_filled(rect, CornerRadius::same(6), palette.panel);
        }
        ui.painter().text(
            egui::pos2(rect.left() + 8.0, rect.center().y),
            Align2::LEFT_CENTER,
            lang.badge(),
            FontId::new(14.0, FontFamily::Proportional),
            palette.text,
        );
        let flag = Rect::from_min_size(
            egui::pos2(rect.right() - 8.0 - FLAG_W, rect.center().y - FLAG_H / 2.0),
            Vec2::new(FLAG_W, FLAG_H),
        );
        paint_flag(ui.painter(), flag, lang);
    }
    response.clicked()
}

fn paint_flag(painter: &egui::Painter, rect: Rect, lang: Lang) {
    let painter = painter.with_clip_rect(rect);
    match lang {
        Lang::Ru => paint_russia(&painter, rect),
        Lang::En => paint_britain(&painter, rect),
        Lang::Zh => paint_china(&painter, rect),
    }
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_rgb(30, 36, 44)), egui::StrokeKind::Inside);
}

fn paint_russia(painter: &egui::Painter, rect: Rect) {
    let stripe = rect.height() / 3.0;
    let colors = [Color32::WHITE, Color32::from_rgb(0, 57, 166), Color32::from_rgb(213, 43, 30)];
    for (index, color) in colors.iter().enumerate() {
        let top = rect.top() + index as f32 * stripe;
        let height = if index == 2 { rect.bottom() - top } else { stripe };
        painter.rect_filled(Rect::from_min_size(egui::pos2(rect.left(), top), Vec2::new(rect.width(), height)), 0.0, *color);
    }
}

fn paint_britain(painter: &egui::Painter, rect: Rect) {
    let blue = Color32::from_rgb(1, 33, 105);
    let red = Color32::from_rgb(200, 16, 46);
    painter.rect_filled(rect, 0.0, blue);
    let white_diag = Stroke::new((rect.height() * 0.34).max(2.0), Color32::WHITE);
    let red_diag = Stroke::new((rect.height() * 0.16).max(1.0), red);
    painter.line(vec![rect.left_top(), rect.right_bottom()], white_diag);
    painter.line(vec![rect.right_top(), rect.left_bottom()], white_diag);
    painter.line(vec![rect.left_top(), rect.right_bottom()], red_diag);
    painter.line(vec![rect.right_top(), rect.left_bottom()], red_diag);
    let thick = (rect.height() * 0.36).max(2.0);
    let thin = (rect.height() * 0.2).max(1.2);
    let center = rect.center();
    painter.rect_filled(Rect::from_center_size(center, Vec2::new(rect.width(), thick)), 0.0, Color32::WHITE);
    painter.rect_filled(Rect::from_center_size(center, Vec2::new(thick, rect.height())), 0.0, Color32::WHITE);
    painter.rect_filled(Rect::from_center_size(center, Vec2::new(rect.width(), thin)), 0.0, red);
    painter.rect_filled(Rect::from_center_size(center, Vec2::new(thin, rect.height())), 0.0, red);
}

fn paint_china(painter: &egui::Painter, rect: Rect) {
    let red = Color32::from_rgb(222, 41, 16);
    let yellow = Color32::from_rgb(255, 222, 0);
    painter.rect_filled(rect, 0.0, red);
    let sx = rect.width() / 30.0;
    let sy = rect.height() / 20.0;
    let at = |x: f32, y: f32| egui::pos2(rect.left() + x * sx, rect.top() + y * sy);
    let large = at(5.0, 5.0);
    let large_r = (3.2 * sx).max(2.4);
    paint_star(painter, large, large_r, -std::f32::consts::FRAC_PI_2, yellow);
    let small_r = (large_r * 0.38).max(1.15);
    for (x, y) in [(10.0, 2.0), (12.0, 4.0), (12.0, 7.0), (10.0, 9.0)] {
        let center = at(x, y);
        let dir = large - center;
        paint_star(painter, center, small_r, dir.y.atan2(dir.x), yellow);
    }
}

fn paint_star(painter: &egui::Painter, center: egui::Pos2, outer: f32, rotation: f32, color: Color32) {
    let inner = outer * 0.382;
    let mut points = Vec::with_capacity(10);
    for index in 0..10 {
        let radius = if index % 2 == 0 { outer } else { inner };
        let angle = rotation + index as f32 * std::f32::consts::PI / 5.0;
        points.push(center + egui::vec2(angle.cos() * radius, angle.sin() * radius));
    }
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}

fn ghost_button(ui: &mut egui::Ui, text: &str, palette: &Palette, height: f32) -> bool {
    ui.add(
        egui::Button::new(RichText::new(text).color(palette.text).size(15.0))
            .fill(palette.input)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(10))
            .min_size(Vec2::new(0.0, height)),
    )
    .clicked()
}

fn pane_button<'a>(text: &'a str, palette: &Palette) -> egui::Button<'a> {
    egui::Button::new(RichText::new(text).color(palette.text).size(15.0))
        .fill(palette.input)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(10))
        .min_size(Vec2::new(0.0, 40.0))
}

fn chip(ui: &mut egui::Ui, enabled: &mut bool, text: &str, fill: Color32, color: Color32, palette: &Palette, height: f32) {
    let (button_fill, button_text) = if *enabled {
        (fill, color)
    } else {
        (palette.inactive, palette.inactive_text)
    };
    let clicked = ui
        .add(
            egui::Button::new(RichText::new(text).color(button_text).size(15.0))
                .fill(button_fill)
                .stroke(Stroke::NONE)
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(0.0, height)),
        )
        .clicked();
    if clicked {
        *enabled = !*enabled;
    }
}

fn pick_file() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .add_filter(i18n::tr("JSON и XML", "JSON and XML", "JSON 和 XML"), &["json", "xml"])
        .add_filter("JSON", &["json"])
        .add_filter("XML", &["xml"])
        .pick_file()
}

fn load_into(pane: &mut Pane, path: &std::path::Path) {
    match std::fs::metadata(path).map(|meta| meta.len()) {
        Ok(size) if size > 16 * 1024 * 1024 => {
            pane.load_note = Some(LoadNote::TooBig);
            return;
        }
        Err(err) => {
            pane.load_note = Some(LoadNote::Failed(err.to_string()));
            return;
        }
        Ok(_) => {}
    }
    match std::fs::read(path) {
        Ok(bytes) => {
            pane.load_note = None;
            pane.text = String::from_utf8(bytes).unwrap_or_else(|err| String::from_utf8_lossy(err.as_bytes()).into_owned());
            pane.path = path.display().to_string();
            pane.title = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_else(|| i18n::tr("Файл", "File", "文件"))
                .to_string();
            pane.refresh_validation();
        }
        Err(err) => {
            pane.load_note = Some(LoadNote::Failed(err.to_string()));
        }
    }
}

fn refresh_compare(app: &mut CompareApp, dialog: bool) {
    let left = app.left.text.clone();
    let right = app.right.text.clone();
    let ignore_order = app.ignore_order;
    let only_changes = app.only_changes;
    let compared = crash::guard(move || engine::compare(&left, &right, ignore_order, only_changes));
    match compared {
        Ok(Ok(diff)) => {
            let total = diff.added + diff.removed + diff.changed;
            app.status = if total == 0 {
                Phrase::NoDiff
            } else {
                Phrase::DiffCounts { removed: diff.removed, added: diff.added, changed: diff.changed }
            };
            if dialog || app.mode_diff {
                app.diff = Some(diff);
            }
            if dialog {
                app.mode_diff = true;
            }
        }
        Ok(Err(message)) => {
            app.status = Phrase::CompareFail(message.clone());
            if dialog {
                rfd::MessageDialog::new()
                    .set_title(i18n::tr("Ошибка", "Error", "错误"))
                    .set_description(&message)
                    .show();
            }
        }
        Err(message) => {
            app.status = Phrase::CompareCrash(message);
        }
    }
}

fn save_report(app: &CompareApp) {
    let Some(diff) = &app.diff else {
        rfd::MessageDialog::new()
            .set_title(i18n::tr("Отчёт", "Report", "报告"))
            .set_description(i18n::tr(
                "Сначала сравните два документа.",
                "Compare two documents first.",
                "请先比较两个文档。",
            ))
            .show();
        return;
    };
    let Some(path) = rfd::FileDialog::new()
        .add_filter("HTML", &["html"])
        .set_file_name("json-diff.html")
        .save_file()
    else {
        return;
    };
    let left_title = app.left.title.clone();
    let right_title = app.right.title.clone();
    let diff = diff.clone();
    let html = match crash::guard(move || engine::report_html(&left_title, &right_title, &diff)) {
        Ok(html) => html,
        Err(message) => {
            rfd::MessageDialog::new()
                .set_title(i18n::tr("Ошибка", "Error", "错误"))
                .set_description(&i18n::with_detail(
                    "Сбой отчёта. Подробности в журнале рядом с программой.",
                    "Report failure. Details are in the log next to the program.",
                    "报告失败。详情见程序旁边的日志。",
                    &message,
                ))
                .show();
            return;
        }
    };
    if let Err(err) = std::fs::write(&path, html) {
        rfd::MessageDialog::new()
            .set_title(i18n::tr("Ошибка", "Error", "错误"))
            .set_description(&err.to_string())
            .show();
    }
}
