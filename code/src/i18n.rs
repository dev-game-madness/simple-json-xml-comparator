use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Ru = 0,
    En = 1,
    Zh = 2,
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
            Lang::Zh => "zh",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Lang::Ru => "RU",
            Lang::En => "ENG",
            Lang::Zh => "CN",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "ru" => Some(Lang::Ru),
            "en" => Some(Lang::En),
            "zh" => Some(Lang::Zh),
            _ => None,
        }
    }

    fn from_code(code: u8) -> Self {
        match code {
            1 => Lang::En,
            2 => Lang::Zh,
            _ => Lang::Ru,
        }
    }
}

pub fn current() -> Lang {
    Lang::from_code(CURRENT.load(Ordering::Relaxed))
}

pub fn set_current(lang: Lang) {
    CURRENT.store(lang as u8, Ordering::Relaxed);
}

pub fn t(lang: Lang, ru: &'static str, en: &'static str, zh: &'static str) -> &'static str {
    match lang {
        Lang::Ru => ru,
        Lang::En => en,
        Lang::Zh => zh,
    }
}

pub fn tr(ru: &'static str, en: &'static str, zh: &'static str) -> &'static str {
    t(current(), ru, en, zh)
}

pub fn load() -> Lang {
    std::fs::read_to_string(lang_path())
        .ok()
        .and_then(|text| Lang::parse(&text))
        .unwrap_or(Lang::Ru)
}

pub fn save(lang: Lang) {
    let _ = std::fs::write(lang_path(), lang.code());
}

fn lang_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|dir| dir.join("sjxc-lang.txt")))
        .unwrap_or_else(|| PathBuf::from("sjxc-lang.txt"))
}

pub fn line_col(line: usize, column: usize, detail: &str) -> String {
    match current() {
        Lang::Ru => format!("Строка {line}, столбец {column}: {detail}"),
        Lang::En => format!("Line {line}, column {column}: {detail}"),
        Lang::Zh => format!("第 {line} 行，第 {column} 列：{detail}"),
    }
}

pub fn xml_line(line: usize, column: usize, detail: &str) -> String {
    match current() {
        Lang::Ru => format!("XML, строка {line}, столбец {column}: {detail}"),
        Lang::En => format!("XML, line {line}, column {column}: {detail}"),
        Lang::Zh => format!("XML，第 {line} 行，第 {column} 列：{detail}"),
    }
}

pub fn formats_differ(left: &str, right: &str) -> String {
    match current() {
        Lang::Ru => format!("Разные форматы: слева {left}, справа {right}"),
        Lang::En => format!("Different formats: {left} on the left, {right} on the right"),
        Lang::Zh => format!("格式不同：左侧为 {left}，右侧为 {right}"),
    }
}

pub fn hidden_lines(count: usize) -> String {
    match current() {
        Lang::Ru => format!("··· без изменений: {count} строк ···"),
        Lang::En => format!("··· unchanged: {count} lines ···"),
        Lang::Zh => format!("··· 未更改：{count} 行 ···"),
    }
}

pub fn diff_counts(removed: usize, added: usize, changed: usize) -> String {
    match current() {
        Lang::Ru => format!("Удалено строк: {removed}. Добавлено: {added}. Изменено: {changed}."),
        Lang::En => format!("Lines removed: {removed}. Added: {added}. Changed: {changed}."),
        Lang::Zh => format!("删除行：{removed}。添加：{added}。更改：{changed}。"),
    }
}

pub fn html_summary(removed: usize, added: usize, changed: usize) -> String {
    match current() {
        Lang::Ru => format!("Удалено: {removed}. Добавлено: {added}. Изменено: {changed}."),
        Lang::En => format!("Removed: {removed}. Added: {added}. Changed: {changed}."),
        Lang::Zh => format!("删除：{removed}。添加：{added}。更改：{changed}。"),
    }
}

pub fn with_detail(prefix_ru: &'static str, prefix_en: &'static str, prefix_zh: &'static str, detail: &str) -> String {
    format!("{} {detail}", tr(prefix_ru, prefix_en, prefix_zh))
}

#[derive(Clone, Debug)]
pub enum Note {
    RemovedComma,
    RemovedComment,
    SingleQuotes,
    QuotedIdent(String),
    ReplacedLiteral { ident: String, replacement: String },
    InsertedBracket,
    InsertedComma,
    RemovedCommaTail,
    ClosedString,
    AddedBrackets(String),
    ClosedTag(String),
    AddedGt,
    AddedCloseTags(String),
}

impl Note {
    pub fn render(&self) -> String {
        match self {
            Note::RemovedComma => tr("удалена лишняя запятая", "removed an extra comma", "已删除多余逗号").into(),
            Note::RemovedComment => tr("удалён комментарий", "removed a comment", "已删除注释").into(),
            Note::SingleQuotes => tr(
                "одинарные кавычки заменены на двойные",
                "single quotes replaced with double quotes",
                "单引号已替换为双引号",
            )
            .into(),
            Note::QuotedIdent(ident) => match current() {
                Lang::Ru => format!("имя «{ident}» взято в кавычки"),
                Lang::En => format!("name “{ident}” was quoted"),
                Lang::Zh => format!("名称“{ident}”已加上引号"),
            },
            Note::ReplacedLiteral { ident, replacement } => match current() {
                Lang::Ru => format!("{ident} заменено на {replacement}"),
                Lang::En => format!("{ident} replaced with {replacement}"),
                Lang::Zh => format!("{ident} 已替换为 {replacement}"),
            },
            Note::InsertedBracket => tr("вставлена ]", "inserted ]", "已插入 ]").into(),
            Note::InsertedComma => tr("вставлена запятая", "inserted a comma", "已插入逗号").into(),
            Note::RemovedCommaTail => tr("удалена лишняя запятая в конце", "removed a trailing comma", "已删除末尾多余逗号").into(),
            Note::ClosedString => tr("закрыта строка кавычкой", "closed a string with a quote", "已用引号闭合字符串").into(),
            Note::AddedBrackets(closers) => match current() {
                Lang::Ru => format!("добавлены скобки {closers}"),
                Lang::En => format!("added brackets {closers}"),
                Lang::Zh => format!("已添加括号 {closers}"),
            },
            Note::ClosedTag(name) => match current() {
                Lang::Ru => format!("закрыт тег {name}"),
                Lang::En => format!("closed tag {name}"),
                Lang::Zh => format!("已闭合标签 {name}"),
            },
            Note::AddedGt => tr("добавлен символ >", "added >", "已添加 >").into(),
            Note::AddedCloseTags(tags) => match current() {
                Lang::Ru => format!("добавлены закрывающие теги {tags}"),
                Lang::En => format!("added closing tags {tags}"),
                Lang::Zh => format!("已添加结束标签 {tags}"),
            },
        }
    }
}

pub fn join_notes(notes: &[Note]) -> String {
    notes.iter().map(Note::render).collect::<Vec<_>>().join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_codes_round_trip() {
        assert_eq!(Lang::parse("ru"), Some(Lang::Ru));
        assert_eq!(Lang::parse(" en "), Some(Lang::En));
        assert_eq!(Lang::parse("zh"), Some(Lang::Zh));
        assert_eq!(Lang::parse("fr"), None);
        assert_eq!(Lang::Ru.code(), "ru");
        assert_eq!(Lang::En.badge(), "ENG");
        assert_eq!(Lang::Zh.badge(), "CN");
        assert_eq!(t(Lang::En, "Язык", "Language", "语言"), "Language");
        assert_eq!(t(Lang::Zh, "Язык", "Language", "语言"), "语言");
        assert_eq!(t(Lang::Ru, "Язык", "Language", "语言"), "Язык");
    }
}
