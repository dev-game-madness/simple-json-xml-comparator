use serde_json::Value;
use similar::TextDiff;

use crate::i18n::{self, Note};

pub const MAX_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;

fn too_large(text: &str) -> Option<Issue> {
    if text.len() > MAX_DOCUMENT_BYTES {
        Some(Issue {
            line: 1,
            column: 1,
            message: i18n::tr(
                "Документ больше 8 МБ. Операция остановлена.",
                "Document is larger than 8 MB. Operation stopped.",
                "文档超过 8 MB。操作已停止。",
            )
            .into(),
        })
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Json,
    Xml,
    Empty,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    Eq,
    Del,
    Add,
    Chg,
    Skip,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub kind: RowKind,
    pub left_no: Option<usize>,
    pub left: String,
    pub right_no: Option<usize>,
    pub right: String,
}

#[derive(Clone, Debug)]
pub struct Diff {
    pub rows: Vec<Row>,
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
}

pub fn detect_kind(text: &str) -> Kind {
    let sample = text.trim_start_matches('\u{feff}').trim_start();
    if sample.is_empty() {
        Kind::Empty
    } else if sample.starts_with('<') {
        Kind::Xml
    } else {
        Kind::Json
    }
}

pub fn validate(text: &str) -> Result<Kind, Issue> {
    match detect_kind(text) {
        Kind::Empty => Err(Issue {
            line: 1,
            column: 1,
            message: i18n::tr(
                "Пусто — вставьте JSON, XML или откройте файл",
                "Empty — paste JSON or XML, or open a file",
                "空 — 请粘贴 JSON 或 XML，或打开文件",
            )
            .into(),
        }),
        Kind::Xml => match roxmltree::Document::parse(text) {
            Ok(_) => Ok(Kind::Xml),
            Err(err) => Err(xml_issue(&err)),
        },
        Kind::Json => match serde_json::from_str::<Value>(text) {
            Ok(_) => Ok(Kind::Json),
            Err(err) => Err(json_issue(&err)),
        },
    }
}

fn json_issue(err: &serde_json::Error) -> Issue {
    Issue {
        line: err.line(),
        column: err.column(),
        message: i18n::line_col(err.line(), err.column(), &err.to_string()),
    }
}

fn xml_issue(err: &roxmltree::Error) -> Issue {
    let pos = err.pos();
    let line = pos.row as usize;
    let column = pos.col as usize;
    Issue {
        line,
        column,
        message: i18n::xml_line(line, column, &err.to_string()),
    }
}

pub fn beautify(text: &str) -> Result<String, Issue> {
    if let Some(issue) = too_large(text) {
        return Err(issue);
    }
    match detect_kind(text) {
        Kind::Xml => beautify_xml(text),
        Kind::Json => {
            let value = parse_json_keep(text)?;
            let mut pretty = format_json(&value, 0);
            if !pretty.ends_with('\n') {
                pretty.push('\n');
            }
            Ok(pretty)
        }
        Kind::Empty => Err(Issue {
            line: 1,
            column: 1,
            message: i18n::tr("Нечего форматировать", "Nothing to format", "没有可格式化的内容").into(),
        }),
    }
}

pub fn compare(left: &str, right: &str, ignore_order: bool, only_changes: bool) -> Result<Diff, String> {
    if left.len() > MAX_DOCUMENT_BYTES || right.len() > MAX_DOCUMENT_BYTES {
        return Err(i18n::tr(
            "Документ больше 8 МБ. Сравнение остановлено.",
            "Document is larger than 8 MB. Compare stopped.",
            "文档超过 8 MB。比较已停止。",
        )
        .into());
    }
    let left_kind = detect_kind(left);
    let right_kind = detect_kind(right);
    if left_kind == Kind::Empty || right_kind == Kind::Empty {
        return Err(i18n::tr(
            "Оба окна должны содержать JSON или XML",
            "Both panes must contain JSON or XML",
            "两个窗口都必须包含 JSON 或 XML",
        )
        .into());
    }
    if left_kind != right_kind {
        return Err(i18n::formats_differ(kind_name(left_kind), kind_name(right_kind)));
    }
    let (left_lines, right_lines) = if left_kind == Kind::Xml {
        (
            xml_lines(left, ignore_order).map_err(|e| format!("A: {}", e.message))?,
            xml_lines(right, ignore_order).map_err(|e| format!("B: {}", e.message))?,
        )
    } else {
        (
            json_lines(left, ignore_order).map_err(|e| format!("A: {}", e.message))?,
            json_lines(right, ignore_order).map_err(|e| format!("B: {}", e.message))?,
        )
    };
    Ok(diff_lines(&left_lines, &right_lines, only_changes))
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Json => "JSON",
        Kind::Xml => "XML",
        Kind::Empty => i18n::tr("пусто", "empty", "空"),
    }
}

fn json_lines(text: &str, ignore_order: bool) -> Result<Vec<String>, Issue> {
    let mut value = parse_json_keep(text)?;
    sort_structure(&mut value, ignore_order);
    let pretty = format_json(&value, 0);
    Ok(pretty.lines().map(str::to_string).collect())
}

#[derive(Clone, Debug)]
enum Json {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

fn parse_json_keep(text: &str) -> Result<Json, Issue> {
    let mut parser = Parser { src: text, i: 0, depth: 0 };
    parser.skip();
    let value = parser.parse_value()?;
    parser.skip();
    if parser.i < parser.src.len() {
        return Err(parser.issue(i18n::tr("лишние данные после JSON", "extra data after JSON", "JSON 结束后还有多余内容")));
    }
    Ok(value)
}

struct Parser<'a> {
    src: &'a str,
    i: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn issue(&self, message: &str) -> Issue {
        let (line, column) = self.line_col();
        Issue {
            line,
            column,
            message: i18n::line_col(line, column, message),
        }
    }

    fn line_col(&self) -> (usize, usize) {
        let mut line = 1usize;
        let mut column = 1usize;
        for (index, ch) in self.src.char_indices() {
            if index >= self.i {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        (line, column)
    }

    fn skip(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.src[self.i..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.i += ch.len_utf8();
        Some(ch)
    }

    fn parse_value(&mut self) -> Result<Json, Issue> {
        self.skip();
        match self.peek() {
            Some('n') => self.consume_word("null").map(|_| Json::Null),
            Some('t') => self.consume_word("true").map(|_| Json::Bool(true)),
            Some('f') => self.consume_word("false").map(|_| Json::Bool(false)),
            Some('"') => Ok(Json::String(self.parse_string()?)),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some('-') | Some('0'..='9') => self.parse_number(),
            _ => Err(self.issue(i18n::tr("ожидалось значение", "expected a value", "需要值"))),
        }
    }

    fn consume_word(&mut self, word: &str) -> Result<(), Issue> {
        if self.src[self.i..].starts_with(word) {
            self.i += word.len();
            Ok(())
        } else {
            Err(self.issue(i18n::tr("ожидалось значение", "expected a value", "需要值")))
        }
    }

    fn parse_number(&mut self) -> Result<Json, Issue> {
        let start = self.i;
        if self.peek() == Some('-') {
            self.bump();
        }
        let mut saw_digit = false;
        while matches!(self.peek(), Some('0'..='9')) {
            saw_digit = true;
            self.bump();
        }
        if self.peek() == Some('.') {
            self.bump();
            while matches!(self.peek(), Some('0'..='9')) {
                saw_digit = true;
                self.bump();
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            self.bump();
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.bump();
            }
            while matches!(self.peek(), Some('0'..='9')) {
                saw_digit = true;
                self.bump();
            }
        }
        if !saw_digit {
            return Err(self.issue(i18n::tr("ожидалось число", "expected a number", "需要数字")));
        }
        Ok(Json::Number(self.src[start..self.i].to_string()))
    }

    fn parse_string(&mut self) -> Result<String, Issue> {
        if self.bump() != Some('"') {
            return Err(self.issue(i18n::tr("ожидалась строка", "expected a string", "需要字符串")));
        }
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return Err(self.issue(i18n::tr("строка не закрыта", "string is not closed", "字符串未闭合"))),
                Some('"') => return Ok(out),
                Some('\\') => {
                    let escaped = self.bump().ok_or_else(|| self.issue(i18n::tr("незаконченное экранирование", "incomplete escape", "转义未完成")))?;
                    match escaped {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000c}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let mut hex = String::new();
                            for _ in 0..4 {
                                let ch = self.bump().ok_or_else(|| self.issue(i18n::tr("неполное \\u", "incomplete \\u", "\\u 不完整")))?;
                                hex.push(ch);
                            }
                            let code = u32::from_str_radix(&hex, 16).map_err(|_| self.issue(i18n::tr("неверное \\u", "invalid \\u", "\\u 无效")))?;
                            out.push(char::from_u32(code).ok_or_else(|| self.issue(i18n::tr("неверный символ", "invalid character", "无效字符")))?);
                        }
                        _ => return Err(self.issue(i18n::tr("неверное экранирование", "invalid escape", "无效转义"))),
                    }
                }
                Some(ch) => out.push(ch),
            }
        }
    }

    fn enter(&mut self) -> Result<(), Issue> {
        if self.depth >= 96 {
            return Err(self.issue(i18n::tr("слишком глубокая вложенность", "nesting is too deep", "嵌套过深")));
        }
        self.depth += 1;
        Ok(())
    }

    fn parse_array(&mut self) -> Result<Json, Issue> {
        self.bump();
        self.enter()?;
        let parsed = self.parse_array_items();
        self.depth -= 1;
        parsed
    }

    fn parse_array_items(&mut self) -> Result<Json, Issue> {
        let mut items = Vec::new();
        self.skip();
        if self.peek() == Some(']') {
            self.bump();
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip();
            match self.bump() {
                Some(',') => {
                    self.skip();
                    if self.peek() == Some(']') {
                        return Err(self.issue(i18n::tr("лишняя запятая", "trailing comma", "多余的逗号")));
                    }
                }
                Some(']') => return Ok(Json::Array(items)),
                _ => return Err(self.issue(i18n::tr("ожидалась ]", "expected ]", "需要 ]"))),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Json, Issue> {
        self.bump();
        self.enter()?;
        let parsed = self.parse_object_items();
        self.depth -= 1;
        parsed
    }

    fn parse_object_items(&mut self) -> Result<Json, Issue> {
        let mut pairs = Vec::new();
        self.skip();
        if self.peek() == Some('}') {
            self.bump();
            return Ok(Json::Object(pairs));
        }
        loop {
            self.skip();
            if self.peek() != Some('"') {
                return Err(self.issue(i18n::tr("ожидалось имя свойства", "expected a property name", "需要属性名")));
            }
            let key = self.parse_string()?;
            self.skip();
            if self.bump() != Some(':') {
                return Err(self.issue(i18n::tr("ожидалось двоеточие", "expected a colon", "需要冒号")));
            }
            let value = self.parse_value()?;
            pairs.push((key, value));
            self.skip();
            match self.bump() {
                Some(',') => {
                    self.skip();
                    if self.peek() == Some('}') {
                        return Err(self.issue(i18n::tr("лишняя запятая", "trailing comma", "多余的逗号")));
                    }
                }
                Some('}') => return Ok(Json::Object(pairs)),
                _ => return Err(self.issue(i18n::tr("ожидалась }", "expected }", "需要 }"))),
            }
        }
    }
}

fn sort_structure(value: &mut Json, ignore_order: bool) {
    match value {
        Json::Object(pairs) => {
            for (_, child) in pairs.iter_mut() {
                sort_structure(child, ignore_order);
            }
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
        }
        Json::Array(items) => {
            for item in items.iter_mut() {
                sort_structure(item, ignore_order);
            }
            if ignore_order {
                items.sort_by(|a, b| format_compact(a).cmp(&format_compact(b)));
            }
        }
        _ => {}
    }
}

fn format_json(value: &Json, level: usize) -> String {
    let mut out = String::new();
    write_json(value, level, &mut out);
    out
}

fn write_json(value: &Json, level: usize, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Number(raw) => out.push_str(raw),
        Json::String(text) => out.push_str(&escape_json(text)),
        Json::Array(items) if items.is_empty() => out.push_str("[]"),
        Json::Object(pairs) if pairs.is_empty() => out.push_str("{}"),
        Json::Array(items) => {
            out.push_str("[\n");
            for (index, item) in items.iter().enumerate() {
                out.push_str(&"  ".repeat(level + 1));
                write_json(item, level + 1, out);
                if index + 1 != items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&"  ".repeat(level));
            out.push(']');
        }
        Json::Object(pairs) => {
            out.push_str("{\n");
            for (index, (key, child)) in pairs.iter().enumerate() {
                out.push_str(&"  ".repeat(level + 1));
                out.push_str(&escape_json(key));
                out.push_str(": ");
                write_json(child, level + 1, out);
                if index + 1 != pairs.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&"  ".repeat(level));
            out.push('}');
        }
    }
}

fn format_compact(value: &Json) -> String {
    match value {
        Json::Null => "null".into(),
        Json::Bool(true) => "true".into(),
        Json::Bool(false) => "false".into(),
        Json::Number(raw) => raw.clone(),
        Json::String(text) => escape_json(text),
        Json::Array(items) => {
            let body = items.iter().map(format_compact).collect::<Vec<_>>().join(",");
            format!("[{body}]")
        }
        Json::Object(pairs) => {
            let mut pairs = pairs.clone();
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            let body = pairs
                .iter()
                .map(|(key, child)| format!("{}:{}", escape_json(key), format_compact(child)))
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{body}}}")
        }
    }
}

fn escape_json(text: &str) -> String {
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn diff_lines(left: &[String], right: &[String], only_changes: bool) -> Diff {
    let left_text = left.join("\n");
    let right_text = right.join("\n");
    let diff = TextDiff::from_lines(&left_text, &right_text);
    let mut rows = Vec::new();
    for op in diff.ops() {
        match op {
            similar::DiffOp::Equal { old_index, new_index, len } => {
                for offset in 0..*len {
                    rows.push(Row {
                        kind: RowKind::Eq,
                        left_no: Some(old_index + offset + 1),
                        left: left.get(old_index + offset).cloned().unwrap_or_default(),
                        right_no: Some(new_index + offset + 1),
                        right: right.get(new_index + offset).cloned().unwrap_or_default(),
                    });
                }
            }
            similar::DiffOp::Delete { old_index, old_len, .. } => {
                for offset in 0..*old_len {
                    rows.push(Row {
                        kind: RowKind::Del,
                        left_no: Some(old_index + offset + 1),
                        left: left.get(old_index + offset).cloned().unwrap_or_default(),
                        right_no: None,
                        right: String::new(),
                    });
                }
            }
            similar::DiffOp::Insert { new_index, new_len, .. } => {
                for offset in 0..*new_len {
                    rows.push(Row {
                        kind: RowKind::Add,
                        left_no: None,
                        left: String::new(),
                        right_no: Some(new_index + offset + 1),
                        right: right.get(new_index + offset).cloned().unwrap_or_default(),
                    });
                }
            }
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                let count = (*old_len).max(*new_len);
                for offset in 0..count {
                    let has_left = offset < *old_len;
                    let has_right = offset < *new_len;
                    if has_left && has_right {
                        rows.push(Row {
                            kind: RowKind::Chg,
                            left_no: Some(old_index + offset + 1),
                            left: left[old_index + offset].clone(),
                            right_no: Some(new_index + offset + 1),
                            right: right[new_index + offset].clone(),
                        });
                    } else if has_left {
                        rows.push(Row {
                            kind: RowKind::Del,
                            left_no: Some(old_index + offset + 1),
                            left: left[old_index + offset].clone(),
                            right_no: None,
                            right: String::new(),
                        });
                    } else {
                        rows.push(Row {
                            kind: RowKind::Add,
                            left_no: None,
                            left: String::new(),
                            right_no: Some(new_index + offset + 1),
                            right: right[new_index + offset].clone(),
                        });
                    }
                }
            }
        }
    }
    let added = rows.iter().filter(|row| row.kind == RowKind::Add).count();
    let removed = rows.iter().filter(|row| row.kind == RowKind::Del).count();
    let changed = rows.iter().filter(|row| row.kind == RowKind::Chg).count();
    if only_changes {
        rows = collapse(&rows, 3);
    }
    Diff { rows, added, removed, changed }
}

fn collapse(rows: &[Row], context: usize) -> Vec<Row> {
    if rows.iter().all(|row| row.kind == RowKind::Eq) {
        return vec![Row {
            kind: RowKind::Skip,
            left_no: None,
            left: i18n::tr("Файлы совпадают", "Files match", "文件相同").into(),
            right_no: None,
            right: i18n::tr("Файлы совпадают", "Files match", "文件相同").into(),
        }];
    }
    let mut keep = vec![false; rows.len()];
    for (index, row) in rows.iter().enumerate() {
        if row.kind == RowKind::Eq {
            continue;
        }
        let start = index.saturating_sub(context);
        let end = (index + context + 1).min(rows.len());
        for flag in &mut keep[start..end] {
            *flag = true;
        }
    }
    let mut out = Vec::new();
    let mut index = 0;
    while index < rows.len() {
        if keep[index] {
            out.push(rows[index].clone());
            index += 1;
            continue;
        }
        let start = index;
        while index < rows.len() && !keep[index] {
            index += 1;
        }
        let message = i18n::hidden_lines(index - start);
        out.push(Row {
            kind: RowKind::Skip,
            left_no: None,
            left: message.clone(),
            right_no: None,
            right: message,
        });
    }
    out
}

pub fn char_diff(left: &str, right: &str) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    if left.len() > 4000 || right.len() > 4000 {
        return (Vec::new(), Vec::new());
    }
    let diff = TextDiff::from_chars(left, right);
    let mut left_spans = Vec::new();
    let mut right_spans = Vec::new();
    for op in diff.ops() {
        match op {
            similar::DiffOp::Equal { .. } => {}
            similar::DiffOp::Delete { old_index, old_len, .. } => {
                if *old_len > 0 {
                    left_spans.push((*old_index, old_index + old_len));
                }
            }
            similar::DiffOp::Insert { new_index, new_len, .. } => {
                if *new_len > 0 {
                    right_spans.push((*new_index, new_index + new_len));
                }
            }
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                if *old_len > 0 {
                    left_spans.push((*old_index, old_index + old_len));
                }
                if *new_len > 0 {
                    right_spans.push((*new_index, new_index + new_len));
                }
            }
        }
    }
    (left_spans, right_spans)
}

fn beautify_xml(text: &str) -> Result<String, Issue> {
    write_xml(text, false, false)
}

fn xml_lines(text: &str, ignore_order: bool) -> Result<Vec<String>, Issue> {
    let rendered = write_xml(text, true, ignore_order)?;
    Ok(rendered.lines().map(str::to_string).collect())
}

fn write_xml(text: &str, sort_attrs: bool, ignore_order: bool) -> Result<String, Issue> {
    let doc = roxmltree::Document::parse(text).map_err(|err| xml_issue(&err))?;
    let mut out = String::new();
    if !emit_element(doc.root_element(), sort_attrs, ignore_order, &mut out, 0) {
        return Err(Issue {
            line: 1,
            column: 1,
            message: i18n::xml_line(1, 1, i18n::tr("слишком глубокая вложенность", "nesting is too deep", "嵌套过深")),
        });
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

fn emit_element(
    node: roxmltree::Node,
    sort_attrs: bool,
    ignore_order: bool,
    out: &mut String,
    level: usize,
) -> bool {
    if level >= 96 {
        return false;
    }
    let indent = "  ".repeat(level);
    let name = node.tag_name().name();
    let mut attrs: Vec<(String, String)> = node
        .attributes()
        .map(|attr| (attr.name().to_string(), attr.value().to_string()))
        .collect();
    if sort_attrs {
        attrs.sort_by(|a, b| a.0.cmp(&b.0));
    }
    let mut children: Vec<_> = node.children().filter(|child| child.is_element()).collect();
    if ignore_order {
        children.sort_by(|a, b| a.tag_name().name().cmp(b.tag_name().name()));
    }
    let text = node
        .children()
        .filter(|child| child.is_text())
        .filter_map(|child| child.text())
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    out.push_str(&indent);
    out.push('<');
    out.push_str(name);
    for (key, value) in &attrs {
        out.push(' ');
        out.push_str(key);
        out.push_str("=\"");
        out.push_str(&xml_escape(value, true));
        out.push('"');
    }
    if children.is_empty() && text.is_empty() {
        out.push_str("/>\n");
        return true;
    }
    if children.is_empty() {
        out.push('>');
        out.push_str(&xml_escape(&text, false));
        out.push_str("</");
        out.push_str(name);
        out.push_str(">\n");
        return true;
    }
    out.push_str(">\n");
    if !text.is_empty() {
        out.push_str(&indent);
        out.push_str("  ");
        out.push_str(&xml_escape(&text, false));
        out.push('\n');
    }
    for child in children {
        if !emit_element(child, sort_attrs, ignore_order, out, level + 1) {
            return false;
        }
    }
    out.push_str(&indent);
    out.push_str("</");
    out.push_str(name);
    out.push_str(">\n");
    true
}

fn xml_escape(value: &str, attribute: bool) -> String {
    let mut escaped = value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    if attribute {
        escaped = escaped.replace('"', "&quot;");
    }
    escaped
}

pub fn unescape_quotes(text: &str) -> (String, Vec<(usize, usize)>) {
    if text.len() > MAX_DOCUMENT_BYTES {
        return (text.to_string(), Vec::new());
    }
    let mut output = String::new();
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\\' && index + 1 < chars.len() && chars[index + 1] == '"' {
            let start = output.chars().count();
            output.push('"');
            spans.push((start, start + 1));
            index += 2;
            continue;
        }
        if chars[index] == '"' {
            if let Some(closing) = wrapped_container(&chars, index) {
                let inner: String = chars[index + 1..closing].iter().collect();
                let piece_start = output.chars().count();
                append_unescaped(&inner, &mut output, &mut spans);
                let piece = inner.replace("\\\"", "\"");
                let left = piece.trim_start();
                if left.starts_with('{') || left.starts_with('[') {
                    let brace = piece_start + piece.chars().count() - left.chars().count();
                    spans.push((brace, brace + 1));
                }
                let right = piece.trim_end();
                if right.ends_with('}') || right.ends_with(']') {
                    let brace = piece_start + right.chars().count() - 1;
                    spans.push((brace, brace + 1));
                }
                index = closing + 1;
                continue;
            }
        }
        output.push(chars[index]);
        index += 1;
    }
    if output == text {
        spans.clear();
    }
    (output, spans)
}

fn append_unescaped(chunk: &str, output: &mut String, spans: &mut Vec<(usize, usize)>) {
    let chars: Vec<char> = chunk.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\\' && index + 1 < chars.len() && chars[index + 1] == '"' {
            let start = output.chars().count();
            output.push('"');
            spans.push((start, start + 1));
            index += 2;
        } else {
            output.push(chars[index]);
            index += 1;
        }
    }
}

fn wrapped_container(chars: &[char], quote: usize) -> Option<usize> {
    if !is_value_quote(chars, quote) {
        return None;
    }
    let body = skip_ws(chars, quote + 1);
    if body >= chars.len() || (chars[body] != '{' && chars[body] != '[') {
        return None;
    }
    let end = matching_bracket(chars, body)?;
    let closing = skip_ws(chars, end + 1);
    if closing >= chars.len() || chars[closing] != '"' || is_escaped(chars, closing) {
        return None;
    }
    Some(closing)
}

fn is_value_quote(chars: &[char], index: usize) -> bool {
    let mut cursor = index as isize - 1;
    while cursor >= 0 && matches!(chars[cursor as usize], ' ' | '\t' | '\r' | '\n') {
        cursor -= 1;
    }
    cursor < 0 || matches!(chars[cursor as usize], ':' | ',' | '[')
}

fn skip_ws(chars: &[char], mut index: usize) -> usize {
    while index < chars.len() && matches!(chars[index], ' ' | '\t' | '\r' | '\n') {
        index += 1;
    }
    index
}

fn is_escaped(chars: &[char], index: usize) -> bool {
    let mut slashes = 0usize;
    let mut cursor = index as isize - 1;
    while cursor >= 0 && chars[cursor as usize] == '\\' {
        slashes += 1;
        cursor -= 1;
    }
    slashes % 2 == 1
}

fn matching_bracket(chars: &[char], start: usize) -> Option<usize> {
    let mut stack = vec![chars[start]];
    let mut index = start + 1;
    let mut in_string = false;
    while index < chars.len() {
        let char = chars[index];
        if in_string {
            if char == '"' && !is_escaped(chars, index) {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if char == '"' && !is_escaped(chars, index) {
            in_string = true;
        } else if char == '{' || char == '[' {
            stack.push(char);
        } else if char == '}' || char == ']' {
            let expected = if char == '}' { '{' } else { '[' };
            if stack.last().copied() != Some(expected) {
                return None;
            }
            stack.pop();
            if stack.is_empty() {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

pub fn repair(text: &str) -> Result<(String, Vec<Note>), Issue> {
    if let Some(issue) = too_large(text) {
        return Err(issue);
    }
    match detect_kind(text) {
        Kind::Xml => repair_xml(text),
        Kind::Json => Ok(repair_json(text)),
        Kind::Empty => Err(Issue {
            line: 1,
            column: 1,
            message: i18n::tr("Нечего исправлять", "Nothing to repair", "没有可修复的内容").into(),
        }),
    }
}

fn repair_json(text: &str) -> (String, Vec<Note>) {
    let mut current = text.trim_start_matches('\u{feff}').to_string();
    let mut notes = Vec::new();
    let mut seen = std::collections::HashSet::new();
    seen.insert(current.clone());
    for _ in 0..40 {
        if serde_json::from_str::<Value>(&current).is_ok() {
            return (current, notes);
        }
        let err = match serde_json::from_str::<Value>(&current) {
            Ok(_) => return (current, notes),
            Err(err) => err,
        };
        let Some((updated, new_notes)) = one_json_fix(&current, &err) else {
            break;
        };
        if updated == current || !seen.insert(updated.clone()) {
            break;
        }
        current = updated;
        notes.extend(new_notes);
    }
    (current, notes)
}

fn one_json_fix(text: &str, err: &serde_json::Error) -> Option<(String, Vec<Note>)> {
    let message = err.to_string().to_lowercase();
    let pos = index_at(text, err.line(), err.column()).min(text.len());
    if message.contains("trailing comma") {
        return remove_previous_comma(text, pos);
    }
    if text[pos..].starts_with("//") || text[pos..].starts_with("/*") || text[pos..].starts_with('#') {
        return strip_comment(text, pos);
    }
    if text[pos..].starts_with('\'') {
        return convert_single_quoted(text, pos);
    }
    if message.contains("key must be a string") || message.contains("expected ident") {
        if let Some(fixed) = quote_ident(text, pos) {
            return Some(fixed);
        }
    }
    if message.contains("expected value") {
        if let Some(fixed) = replace_literal(text, pos) {
            return Some(fixed);
        }
    }
    if message.contains("trailing comma") || message.contains("expected value") {
        if let Some(fixed) = remove_previous_comma(text, pos) {
            return Some(fixed);
        }
    }
    if pos >= text.trim_end().len() || text[pos..].trim().is_empty() || message.contains("eof") || message.contains("expected `}`") || message.contains("expected `]`") {
        if let Some(closed) = close_json_tail(text) {
            return Some(closed);
        }
    }
    if message.contains("expected `,`") || message.contains("comma") {
        if let Some(fixed) = insert_comma_or_bracket(text, pos) {
            return Some(fixed);
        }
    }
    close_json_tail(text)
}

fn index_at(text: &str, line: usize, column: usize) -> usize {
    let mut current_line = 1usize;
    let mut current_col = 1usize;
    for (index, ch) in text.char_indices() {
        if current_line == line && current_col == column {
            return index;
        }
        if ch == '\n' {
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
    }
    text.len()
}

fn remove_previous_comma(text: &str, pos: usize) -> Option<(String, Vec<Note>)> {
    let bytes = text.as_bytes();
    let mut cursor = pos.min(bytes.len());
    while cursor > 0 && bytes[cursor - 1].is_ascii_whitespace() {
        cursor -= 1;
    }
    if cursor > 0 && bytes[cursor - 1] == b',' {
        let mut updated = String::new();
        updated.push_str(&text[..cursor - 1]);
        updated.push_str(&text[cursor..]);
        return Some((updated, vec![Note::RemovedComma]));
    }
    None
}

fn strip_comment(text: &str, pos: usize) -> Option<(String, Vec<Note>)> {
    if text[pos..].starts_with("//") || text[pos..].starts_with('#') {
        let end = text[pos..].find('\n').map(|offset| pos + offset).unwrap_or(text.len());
        let mut updated = String::new();
        updated.push_str(&text[..pos]);
        updated.push_str(&text[end..]);
        return Some((updated, vec![Note::RemovedComment]));
    }
    if text[pos..].starts_with("/*") {
        let end = text[pos + 2..]
            .find("*/")
            .map(|offset| pos + 2 + offset + 2)
            .unwrap_or(text.len());
        let mut updated = String::new();
        updated.push_str(&text[..pos]);
        updated.push_str(&text[end..]);
        return Some((updated, vec![Note::RemovedComment]));
    }
    None
}

fn convert_single_quoted(text: &str, pos: usize) -> Option<(String, Vec<Note>)> {
    let chars: Vec<char> = text.chars().collect();
    let start = text[..pos].chars().count();
    if chars.get(start) != Some(&'\'') {
        return None;
    }
    let mut collected = String::new();
    let mut cursor = start + 1;
    while cursor < chars.len() {
        if chars[cursor] == '\\' && cursor + 1 < chars.len() {
            collected.push(chars[cursor + 1]);
            cursor += 2;
            continue;
        }
        if chars[cursor] == '\'' {
            let encoded = serde_json::to_string(&collected).ok()?;
            let prefix: String = chars[..start].iter().collect();
            let suffix: String = chars[cursor + 1..].iter().collect();
            return Some((prefix + &encoded + &suffix, vec![Note::SingleQuotes]));
        }
        collected.push(chars[cursor]);
        cursor += 1;
    }
    None
}

fn quote_ident(text: &str, pos: usize) -> Option<(String, Vec<Note>)> {
    let rest = &text[pos..];
    let ident: String = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '$')
        .collect();
    if ident.is_empty() {
        return None;
    }
    let encoded = serde_json::to_string(&ident).ok()?;
    let mut updated = String::new();
    updated.push_str(&text[..pos]);
    updated.push_str(&encoded);
    updated.push_str(&text[pos + ident.len()..]);
    Some((updated, vec![Note::QuotedIdent(ident)]))
}

fn replace_literal(text: &str, pos: usize) -> Option<(String, Vec<Note>)> {
    let rest = &text[pos..];
    let ident: String = rest.chars().take_while(|ch| ch.is_ascii_alphabetic()).collect();
    let replacement = match ident.as_str() {
        "True" | "TRUE" => "true",
        "False" | "FALSE" => "false",
        "None" | "NULL" | "undefined" => "null",
        _ => return None,
    };
    let mut updated = String::new();
    updated.push_str(&text[..pos]);
    updated.push_str(replacement);
    updated.push_str(&text[pos + ident.len()..]);
    Some((updated, vec![Note::ReplacedLiteral { ident, replacement: replacement.into() }]))
}

fn insert_comma_or_bracket(text: &str, pos: usize) -> Option<(String, Vec<Note>)> {
    let chars: Vec<char> = text.chars().collect();
    let at = text[..pos.min(text.len())].chars().count();
    if at < chars.len() && chars[at] == ':' {
        let token = previous_token_start(&chars, at);
        let prefix: String = chars[..token].iter().collect();
        let suffix: String = chars[token..].iter().collect();
        return Some((format!("{prefix}]{suffix}"), vec![Note::InsertedBracket]));
    }
    if at < chars.len() && (chars[at] == '}' || chars[at] == ']') {
        return None;
    }
    let mut updated = String::new();
    updated.push_str(&text[..pos.min(text.len())]);
    updated.push(',');
    updated.push_str(&text[pos.min(text.len())..]);
    Some((updated, vec![Note::InsertedComma]))
}

fn previous_token_start(chars: &[char], index: usize) -> usize {
    let mut end = index as isize - 1;
    while end >= 0 && matches!(chars[end as usize], ' ' | '\t' | '\r' | '\n') {
        end -= 1;
    }
    if end < 0 {
        return index;
    }
    if chars[end as usize] == '"' {
        let mut cursor = end - 1;
        while cursor >= 0 {
            if chars[cursor as usize] == '"' && !is_escaped(chars, cursor as usize) {
                return cursor as usize;
            }
            cursor -= 1;
        }
    }
    end as usize
}

fn close_json_tail(text: &str) -> Option<(String, Vec<Note>)> {
    let mut body = text.trim_end().to_string();
    let mut notes = Vec::new();
    let (mut stack, mut in_string) = scan_json(&body);
    if !in_string && body.ends_with(',') {
        body.pop();
        body = body.trim_end().to_string();
        notes.push(Note::RemovedCommaTail);
        (stack, in_string) = scan_json(&body);
    }
    if in_string {
        body.push('"');
        notes.push(Note::ClosedString);
        (stack, _) = scan_json(&body);
    }
    if !stack.is_empty() {
        let closers: String = stack.iter().rev().map(|kind| if *kind == '{' { '}' } else { ']' }).collect();
        notes.push(Note::AddedBrackets(closers.clone()));
        body.push_str(&closers);
    }
    if notes.is_empty() {
        return None;
    }
    Some((body, notes))
}

fn scan_json(text: &str) -> (Vec<char>, bool) {
    let mut stack = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    for char in text.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if char == '\\' {
                escaped = true;
            } else if char == '"' {
                in_string = false;
            }
            continue;
        }
        match char {
            '"' => in_string = true,
            '{' | '[' => stack.push(char),
            '}' | ']' => {
                let expected = if char == '}' { '{' } else { '[' };
                if stack.last() == Some(&expected) {
                    stack.pop();
                }
            }
            _ => {}
        }
    }
    (stack, in_string)
}

fn repair_xml(text: &str) -> Result<(String, Vec<Note>), Issue> {
    if roxmltree::Document::parse(text).is_ok() {
        return Ok((text.to_string(), Vec::new()));
    }
    let mut current = text.to_string();
    let mut notes = Vec::new();
    for _ in 0..20 {
        let Some((updated, note)) = close_xml_tags(&current) else {
            break;
        };
        if updated == current {
            break;
        }
        current = updated;
        notes.push(note);
        if roxmltree::Document::parse(&current).is_ok() {
            return Ok((current, notes));
        }
    }
    if notes.is_empty() {
        return match roxmltree::Document::parse(text) {
            Err(err) => Err(xml_issue(&err)),
            Ok(_) => Ok((current, notes)),
        };
    }
    Ok((current, notes))
}

fn close_xml_tags(text: &str) -> Option<(String, Note)> {
    let chars: Vec<char> = text.chars().collect();
    let mut stack: Vec<String> = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index..].starts_with(&['<', '!', '-', '-']) {
            if let Some(end) = find_seq(&chars, index + 4, &['-', '-', '>']) {
                index = end + 3;
                continue;
            }
            index = chars.len();
            continue;
        }
        if chars[index..].starts_with(&['<', '?', ]) {
            if let Some(end) = find_seq(&chars, index + 2, &['?', '>']) {
                index = end + 2;
                continue;
            }
        }
        if chars[index] != '<' {
            index += 1;
            continue;
        }
        if chars.get(index + 1) == Some(&'/') {
            let mut cursor = index + 2;
            while cursor < chars.len() && !matches!(chars[cursor], ' ' | '\t' | '\n' | '\r' | '>') {
                cursor += 1;
            }
            let name: String = chars[index + 2..cursor].iter().collect();
            if stack.last().map(String::as_str) == Some(name.as_str()) {
                stack.pop();
            } else if let Some(expected) = stack.last().cloned() {
                let prefix: String = chars[..index].iter().collect();
                let suffix: String = chars[index..].iter().collect();
                return Some((format!("{prefix}</{expected}>{suffix}"), Note::ClosedTag(expected)));
            }
            index = chars[cursor..].iter().position(|ch| *ch == '>').map(|offset| cursor + offset + 1).unwrap_or(chars.len());
            continue;
        }
        let mut cursor = index + 1;
        while cursor < chars.len() && !matches!(chars[cursor], ' ' | '\t' | '\n' | '\r' | '>' | '/') {
            cursor += 1;
        }
        let name: String = chars[index + 1..cursor].iter().collect();
        let end = chars[cursor..].iter().position(|ch| *ch == '>');
        let Some(end_offset) = end else {
            let current: String = chars.iter().collect();
            return Some((format!("{current}>"), Note::AddedGt));
        };
        let end = cursor + end_offset;
        let mut closer = end as isize - 1;
        while closer > cursor as isize && matches!(chars[closer as usize], ' ' | '\t') {
            closer -= 1;
        }
        if !name.is_empty() && chars.get(closer as usize) != Some(&'/') {
            stack.push(name);
        }
        index = end + 1;
    }
    if stack.is_empty() {
        return None;
    }
    let current: String = chars.iter().collect();
    let closers: String = stack.iter().rev().map(|name| format!("</{name}>")).collect();
    let note = Note::AddedCloseTags(stack.iter().rev().map(|name| format!("</{name}>")).collect::<Vec<_>>().join(" "));
    Some((current + &closers, note))
}

fn find_seq(chars: &[char], from: usize, seq: &[char]) -> Option<usize> {
    chars[from..]
        .windows(seq.len())
        .position(|window| window == seq)
        .map(|offset| from + offset)
}

pub fn report_html(title_left: &str, title_right: &str, diff: &Diff) -> String {
    let mut body = String::new();
    for row in &diff.rows {
        let class = match row.kind {
            RowKind::Del => "del",
            RowKind::Add => "add",
            RowKind::Chg => "chg",
            RowKind::Skip => "skip",
            RowKind::Eq => "eq",
        };
        body.push_str(&format!(
            "<tr class=\"{class}\"><td class=\"no\">{}</td><td class=\"left\">{}</td><td class=\"no\">{}</td><td class=\"right\">{}</td></tr>",
            row.left_no.map(|n| n.to_string()).unwrap_or_default(),
            html_escape(&row.left),
            row.right_no.map(|n| n.to_string()).unwrap_or_default(),
            html_escape(&row.right)
        ));
    }
    format!(
        r#"<!DOCTYPE html><html lang="{lang}"><head><meta charset="utf-8"><title>{heading}</title>
<style>
body {{ margin: 24px; font-family: "Segoe UI", sans-serif; background:#313338; color:#f2f3f5; }}
table {{ border-collapse: collapse; width: 100%; font-family: Consolas, monospace; font-size: 13px; }}
td {{ vertical-align: top; white-space: pre-wrap; padding: 0 8px; }}
td.no {{ color:#949ba4; text-align:right; width:3em; }}
tr.del td.left {{ background:#5c2b2e; }}
tr.add td.right {{ background:#1f4d32; }}
tr.chg td.left {{ background:#5c2b2e; }}
tr.chg td.right {{ background:#1f4d32; }}
tr.skip td {{ background:#2b2d31; color:#b5bac1; text-align:center; }}
</style></head><body>
<h1>{heading}</h1>
<p>{summary}</p>
<table><thead><tr><th></th><th>{left}</th><th></th><th>{right}</th></tr></thead><tbody>{body}</tbody></table>
</body></html>"#,
        lang = i18n::current().code(),
        heading = i18n::tr("Сравнение", "Comparison", "比较"),
        summary = i18n::html_summary(diff.removed, diff.added, diff.changed),
        left = html_escape(title_left),
        right = html_escape(title_right),
        body = body
    )
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_keys_are_kept() {
        let text = r#"{
  "user": {"role": "admin", "name": "Анна", "city": "Казань"},
  "user": {"role": "admin", "name": "Анна", "city": "Казань"},
  "user": {"role": "admin", "name": "Анна", "city": "Казань"}
}"#;
        let pretty = beautify(text).unwrap();
        assert_eq!(pretty.matches("\"user\"").count(), 3);
        let diff = compare(text, &pretty, false, false).unwrap();
        assert_eq!(diff.added + diff.removed + diff.changed, 0);
    }

    #[test]
    fn key_order_is_not_a_difference() {
        let left = r#"{"b":1,"a":{"z":[1,2],"y":"привет"}}"#;
        let right = r#"{"a":{"y":"привет","z":[1,2]},"b":1}"#;
        let diff = compare(left, right, false, false).unwrap();
        assert_eq!(diff.added + diff.removed + diff.changed, 0);
    }

    #[test]
    fn array_order_can_be_ignored() {
        let left = r#"[{"id":2,"name":"b"},{"id":1,"name":"a"}]"#;
        let right = r#"[{"name":"a","id":1},{"name":"b","id":2}]"#;
        let kept = compare(left, right, false, false).unwrap();
        assert!(kept.added + kept.removed + kept.changed > 0);
        let ignored = compare(left, right, true, false).unwrap();
        assert_eq!(ignored.added + ignored.removed + ignored.changed, 0);
    }

    #[test]
    fn beautify_keeps_key_order() {
        let pretty = beautify(r#"{"b":1,"a":2}"#).unwrap();
        assert!(pretty.find("\"b\"").unwrap() < pretty.find("\"a\"").unwrap());
    }

    #[test]
    fn repairs_missing_brace_and_comma() {
        let (fixed, notes) = repair_json(r#"{"a": {"b": 1}"#);
        assert!(!notes.is_empty(), "{fixed}");
        assert_eq!(serde_json::from_str::<Value>(&fixed).unwrap()["a"]["b"], 1);

        let (fixed, _) = repair_json("[1 2]");
        assert_eq!(serde_json::from_str::<Vec<i32>>(&fixed).unwrap(), vec![1, 2]);

        let (fixed, _) = repair_json(r#"{a: 1}"#);
        assert_eq!(serde_json::from_str::<Value>(&fixed).unwrap()["a"], 1);

        let (fixed, _) = repair_json(r#"{"a": 1,}"#);
        assert_eq!(serde_json::from_str::<Value>(&fixed).unwrap()["a"], 1);
    }

    #[test]
    fn unescape_embedded_object() {
        let source = r##"{
  "user": "{
    \"role\": \"admin\",
    \"name\": \"Анна\",
    \"city\": \"Казань\"}"
}"##;
        let (revealed, _) = unescape_quotes(source);
        let value: Value = serde_json::from_str(&revealed).expect(&revealed);
        assert_eq!(value["user"]["city"], "Казань");
        assert_eq!(value["user"]["role"], "admin");
        let (inline, _) = unescape_quotes(r#""user": "{\"role\": \"admin\"}""#);
        assert_eq!(inline, r#""user": {"role": "admin"}"#);
    }

    #[test]
    fn xml_attribute_order_is_ignored() {
        let left = r#"<user role="admin" id="1"><city>Казань</city></user>"#;
        let right = r#"<user id="1" role="admin"><city>Казань</city></user>"#;
        let diff = compare(left, right, false, false).unwrap();
        assert_eq!(diff.added + diff.removed + diff.changed, 0);
        let changed = compare(
            left,
            r#"<user role="editor" id="1"><city>Москва</city></user>"#,
            false,
            false,
        )
        .unwrap();
        assert!(changed.added + changed.removed + changed.changed > 0);
    }

    #[test]
    fn repairs_unclosed_xml() {
        let (fixed, notes) = repair_xml("<root><child>текст").unwrap();
        assert!(!notes.is_empty());
        let doc = roxmltree::Document::parse(&fixed).unwrap();
        assert_eq!(doc.descendants().find(|n| n.has_tag_name("child")).unwrap().text(), Some("текст"));
    }
}
