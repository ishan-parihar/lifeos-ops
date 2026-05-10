use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Tag, TagEnd};
use serde_json::{json, Value};

fn make_rich_text_item(content: &str, href: Option<&str>, bold: bool, italic: bool, strikethrough: bool, code: bool) -> Value {
    let mut item = json!({
        "type": "text", "text": { "content": content },
        "annotations": { "bold": bold, "italic": italic, "strikethrough": strikethrough, "underline": false, "code": code, "color": "default" },
        "href": null
    });
    if let Some(url) = href { item["href"] = json!(url); }
    item
}

fn make_block(block_type: &str, rich_text: Vec<Value>) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("object".to_string(), json!("block"));
    map.insert("type".to_string(), json!(block_type));
    map.insert(block_type.to_string(), json!({ "rich_text": rich_text }));
    Value::Object(map)
}

fn make_todo(rich_text: Vec<Value>, checked: bool) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("object".to_string(), json!("block"));
    map.insert("type".to_string(), json!("to_do"));
    map.insert("to_do".to_string(), json!({ "rich_text": rich_text, "checked": checked }));
    Value::Object(map)
}

fn make_code_block(language: &str, code: &str) -> Value {
    let lang = if language.is_empty() { "plain text" } else { language };
    let mut map = serde_json::Map::new();
    map.insert("object".to_string(), json!("block"));
    map.insert("type".to_string(), json!("code"));
    map.insert("code".to_string(), json!({ "rich_text": [{ "type": "text", "text": { "content": code }, "annotations": { "bold": false, "italic": false, "strikethrough": false, "underline": false, "code": false, "color": "default" }, "href": null }], "language": lang, "caption": [] }));
    Value::Object(map)
}

fn make_divider() -> Value { json!({ "object": "block", "type": "divider", "divider": {} }) }

fn make_image(url: &str) -> Value { json!({ "object": "block", "type": "image", "image": { "type": "external", "external": { "url": url } } }) }

fn make_table_row(cells: Vec<Vec<Value>>) -> Value { json!({ "object": "block", "type": "table_row", "table_row": { "cells": cells } }) }

struct Accum {
    bold: u32, italic: u32, strikethrough: u32, text_buf: String, href: Option<String>, items: Vec<Value>,
}

impl Accum {
    fn new() -> Self { Self { bold: 0, italic: 0, strikethrough: 0, text_buf: String::new(), href: None, items: Vec::new() } }
    fn flush(&mut self) {
        if !self.text_buf.is_empty() {
            let content = std::mem::take(&mut self.text_buf);
            self.items.push(make_rich_text_item(&content, self.href.as_deref(), self.bold > 0, self.italic > 0, self.strikethrough > 0, false));
        }
    }
    fn push_text(&mut self, text: &str) { self.text_buf.push_str(text); }
    fn push_code(&mut self, text: &str) { self.flush(); self.items.push(make_rich_text_item(text, self.href.as_deref(), self.bold > 0, self.italic > 0, self.strikethrough > 0, true)); }
    fn reset(&mut self) { self.bold = 0; self.italic = 0; self.strikethrough = 0; self.text_buf.clear(); self.href = None; self.items.clear(); }
    fn finish(&mut self) -> Vec<Value> { self.flush(); std::mem::take(&mut self.items) }
}

#[derive(Clone, Copy, PartialEq)]
enum ListKind { Unordered, Ordered }

pub fn markdown_to_blocks(markdown: &str) -> Vec<Value> {
    let mut blocks: Vec<Value> = Vec::new();
    let mut accum = Accum::new();
    let mut in_blockquote = false;
    let mut quote_accum = Accum::new();
    let mut list_kind: Option<ListKind> = None;
    let mut in_item = false;
    let mut item_checked: Option<bool> = None;
    let mut in_code_block = false;
    let mut code_language = String::new();
    let mut code_text = String::new();
    let mut in_table = false;
    let mut in_table_row = false;
    let mut table_rows: Vec<Vec<Vec<Value>>> = Vec::new();
    let mut current_row: Vec<Vec<Value>> = Vec::new();
    let mut cell_accum: Option<Accum> = None;

    macro_rules! for_each_accum {
        ($apply:expr) => {{
            if in_table && in_table_row {
                if let Some(ref mut ca) = cell_accum { ca.flush(); $apply(ca); }
            } else if in_blockquote {
                quote_accum.flush(); $apply(&mut quote_accum);
            } else {
                accum.flush(); $apply(&mut accum);
            }
        }};
    }

    let parser = pulldown_cmark::Parser::new(markdown);
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => { in_code_block = true; code_language = match kind { CodeBlockKind::Fenced(info) => info.to_string(), CodeBlockKind::Indented => String::new() }; code_text.clear(); }
            Event::End(TagEnd::CodeBlock) => { in_code_block = false; let trimmed = code_text.trim_end_matches('\n'); blocks.push(make_code_block(&code_language, trimmed)); }
            Event::Start(Tag::Paragraph) => { if !in_table && !in_blockquote && !in_item { accum.reset(); } }
            Event::End(TagEnd::Paragraph) => { if !in_table && !in_blockquote && !in_item { let rt = accum.finish(); if !rt.is_empty() { blocks.push(make_block("paragraph", rt)); } } }
            Event::Start(Tag::Heading { .. }) => { accum.reset(); }
            Event::End(TagEnd::Heading(level)) => {
                let rt = accum.finish();
                if !rt.is_empty() {
                    let type_name = match level { HeadingLevel::H1 => "heading_1", HeadingLevel::H2 => "heading_2", HeadingLevel::H3 => "heading_3", _ => "heading_3" };
                    blocks.push(make_block(type_name, rt));
                }
            }
            Event::Start(Tag::List(Some(_))) => { list_kind = Some(ListKind::Ordered); }
            Event::Start(Tag::List(None)) => { list_kind = Some(ListKind::Unordered); }
            Event::End(TagEnd::List(_)) => { list_kind = None; }
            Event::Start(Tag::Item) => { in_item = true; item_checked = None; accum.reset(); }
            Event::End(TagEnd::Item) => {
                in_item = false;
                let rt = accum.finish();
                if rt.is_empty() { continue; }
                match list_kind {
                    Some(ListKind::Unordered) if item_checked.is_some() => { blocks.push(make_todo(rt, item_checked.unwrap_or(false))); }
                    Some(ListKind::Unordered) => { blocks.push(make_block("bulleted_list_item", rt)); }
                    Some(ListKind::Ordered) => { blocks.push(make_block("numbered_list_item", rt)); }
                    None => { blocks.push(make_block("paragraph", rt)); }
                }
            }
            Event::TaskListMarker(checked) => { item_checked = Some(checked); }
            Event::Start(Tag::BlockQuote(_)) => { in_blockquote = true; quote_accum.reset(); }
            Event::End(TagEnd::BlockQuote) => { in_blockquote = false; let rt = quote_accum.finish(); if !rt.is_empty() { blocks.push(make_block("quote", rt)); } }
            Event::Rule => { blocks.push(make_divider()); }
            Event::Start(Tag::Image { dest_url, .. }) => { blocks.push(make_image(&dest_url)); }
            Event::Start(Tag::Table(_)) => { in_table = true; table_rows.clear(); }
            Event::End(TagEnd::Table) => {
                in_table = false;
                let width = table_rows.first().map_or(0, |r| r.len() as i64);
                let mut children: Vec<Value> = Vec::new();
                for row_cells in &table_rows { children.push(make_table_row(row_cells.clone())); }
                let mut map = serde_json::Map::new();
                map.insert("object".to_string(), json!("block"));
                map.insert("type".to_string(), json!("table"));
                map.insert("table".to_string(), json!({ "table_width": width, "has_column_header": false, "has_row_header": false, "children": children }));
                blocks.push(Value::Object(map));
            }
            Event::Start(Tag::TableHead) => {}
            Event::End(TagEnd::TableHead) => {}
            Event::Start(Tag::TableRow) => { in_table_row = true; current_row.clear(); }
            Event::End(TagEnd::TableRow) => { in_table_row = false; table_rows.push(std::mem::take(&mut current_row)); }
            Event::Start(Tag::TableCell) => { cell_accum = Some(Accum::new()); }
            Event::End(TagEnd::TableCell) => { if let Some(mut ca) = cell_accum.take() { let rt = ca.finish(); current_row.push(rt); } }
            Event::Start(Tag::Emphasis) => { for_each_accum!(|a: &mut Accum| a.italic += 1); }
            Event::End(TagEnd::Emphasis) => { for_each_accum!(|a: &mut Accum| { a.italic = a.italic.saturating_sub(1); }); }
            Event::Start(Tag::Strong) => { for_each_accum!(|a: &mut Accum| a.bold += 1); }
            Event::End(TagEnd::Strong) => { for_each_accum!(|a: &mut Accum| { a.bold = a.bold.saturating_sub(1); }); }
            Event::Start(Tag::Strikethrough) => { for_each_accum!(|a: &mut Accum| a.strikethrough += 1); }
            Event::End(TagEnd::Strikethrough) => { for_each_accum!(|a: &mut Accum| { a.strikethrough = a.strikethrough.saturating_sub(1); }); }
            Event::Start(Tag::Link { dest_url, .. }) => { let url = dest_url.to_string(); for_each_accum!(|a: &mut Accum| a.href = Some(url.clone())); }
            Event::End(TagEnd::Link) => { for_each_accum!(|a: &mut Accum| a.href = None); }
            Event::Text(text) | Event::Html(text) | Event::InlineHtml(text) => {
                let s: &str = text.as_ref();
                if in_code_block { code_text.push_str(s); }
                else if in_table && in_table_row { if let Some(ref mut ca) = cell_accum { ca.push_text(s); } }
                else if in_blockquote { quote_accum.push_text(s); }
                else { accum.push_text(s); }
            }
            Event::Code(text) => {
                let s: &str = text.as_ref();
                if in_table && in_table_row { if let Some(ref mut ca) = cell_accum { ca.push_code(s); } }
                else if in_blockquote { quote_accum.push_code(s); }
                else { accum.push_code(s); }
            }
            Event::InlineMath(text) | Event::DisplayMath(text) => {
                let s: &str = text.as_ref();
                if in_table && in_table_row { if let Some(ref mut ca) = cell_accum { ca.push_text(s); } }
                else if in_blockquote { quote_accum.push_text(s); }
                else { accum.push_text(s); }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_table && in_table_row { if let Some(ref mut ca) = cell_accum { ca.push_text("\n"); } }
                else if in_blockquote { quote_accum.push_text("\n"); }
                else { accum.push_text("\n"); }
            }
            Event::FootnoteReference(_) => {}
            Event::Start(Tag::FootnoteDefinition(_)) => {}
            Event::End(TagEnd::FootnoteDefinition) => {}
            Event::Start(Tag::MetadataBlock(_)) => {}
            Event::End(TagEnd::MetadataBlock(_)) => {}
            Event::Start(_) => {}
            Event::End(_) => {}
        }
    }
    blocks
}