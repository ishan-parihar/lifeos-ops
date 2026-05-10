use crate::notion::types::*;
use serde_json::Value;

pub fn blocks_to_markdown(blocks: &[NotionBlock]) -> String {
    render_block_list(blocks, 0, true)
}

fn render_block_list(blocks: &[NotionBlock], indent: usize, top_level: bool) -> String {
    if blocks.is_empty() { return String::new(); }
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < blocks.len() {
        match blocks[i].block_type.as_str() {
            "bulleted_list_item" => {
                let start = i;
                while i < blocks.len() && blocks[i].block_type == "bulleted_list_item" { i += 1; }
                let items: Vec<String> = blocks[start..i].iter().map(|b| render_list_item(b, indent, "-", None)).collect();
                parts.push(items.join("\n"));
            }
            "numbered_list_item" => {
                let start = i;
                while i < blocks.len() && blocks[i].block_type == "numbered_list_item" { i += 1; }
                let items: Vec<String> = blocks[start..i].iter().enumerate().map(|(j, b)| render_list_item(b, indent, "", Some(j as u32 + 1))).collect();
                parts.push(items.join("\n"));
            }
            "table" => {
                let has_header = blocks[i].table.as_ref().and_then(|t| t.has_column_header).unwrap_or(false);
                let table_width = blocks[i].table.as_ref().and_then(|t| t.table_width).unwrap_or(0);
                i += 1;
                let mut rows: Vec<&TableRowContent> = Vec::new();
                while i < blocks.len() && blocks[i].block_type == "table_row" {
                    if let Some(ref row) = blocks[i].table_row { rows.push(row); }
                    i += 1;
                }
                let table = render_gfm_table(&rows, has_header, table_width as usize);
                if !table.is_empty() { parts.push(table); }
            }
            _ => {
                let rendered = render_single_block(&blocks[i], indent);
                if !rendered.is_empty() { parts.push(rendered); }
                i += 1;
            }
        }
    }
    let result = parts.join("\n\n");
    if top_level { result.trim_end().to_string() } else { result }
}

fn render_single_block(block: &NotionBlock, indent: usize) -> String {
    let is = indent_str(indent);
    match block.block_type.as_str() {
        "paragraph" => {
            if let Some(ref p) = block.paragraph {
                let text = rich_text_str(p.rich_text.as_ref());
                let mut result = format!("{}{}", is, text);
                if let Some(ref children) = p.children {
                    let child_text = render_block_list(children, indent, false);
                    if !child_text.is_empty() { result.push('\n'); result.push_str(&child_text); }
                }
                result
            } else { String::new() }
        }
        "heading_1" => {
            let text = block.heading_1.as_ref().and_then(|h| h.rich_text.as_ref()).map(|rt| render_rich_text(rt)).unwrap_or_default();
            if text.is_empty() { String::new() } else { format!("{}# {}", is, text) }
        }
        "heading_2" => {
            let text = block.heading_2.as_ref().and_then(|h| h.rich_text.as_ref()).map(|rt| render_rich_text(rt)).unwrap_or_default();
            if text.is_empty() { String::new() } else { format!("{}## {}", is, text) }
        }
        "heading_3" => {
            let text = block.heading_3.as_ref().and_then(|h| h.rich_text.as_ref()).map(|rt| render_rich_text(rt)).unwrap_or_default();
            if text.is_empty() { String::new() } else { format!("{}### {}", is, text) }
        }
        "to_do" => render_todo_item(block, indent),
        "code" => render_code_block(block, indent),
        "quote" => render_quote_block(block, indent),
        "callout" => render_callout_block(block, indent),
        "divider" => "---".to_string(),
        "child_page" => if let Some(ref cp) = block.child_page { format!("{}[[{}]]", is, cp.title) } else { String::new() },
        "child_database" => if let Some(ref cd) = block.child_database { format!("{}[[{}]]", is, cd.title) } else { String::new() },
        "image" => render_media_block(block, "image", &is),
        "video" => render_media_block(block, "video", &is),
        "bookmark" => render_bookmark_block(block, &is),
        "embed" => render_media_block(block, "embed", &is),
        "pdf" => render_media_block(block, "pdf", &is),
        "file" => render_file_block(block, &is),
        "equation" => if let Some(ref eq) = block.equation { format!("{is}$$\n{}\n$$", eq.expression) } else { String::new() },
        "column_list" | "column" => String::new(),
        "breadcrumb" => String::new(),
        "link_preview" => if let Some(ref lp) = block.link_preview { format!("{}[{}]({})", is, lp.url, lp.url) } else { String::new() },
        "template" => {
            if let Some(ref t) = block.template {
                let text = rich_text_str(t.rich_text.as_ref());
                let mut result = String::new();
                for line in text.lines() { result.push_str(&format!("{}> {}\n", is, line)); }
                if let Some(ref children) = t.children {
                    let child_text = render_block_list(children, indent, false);
                    if !child_text.is_empty() { for line in child_text.lines() { result.push_str(&format!("{}> {}\n", is, line)); } }
                }
                result.trim_end().to_string()
            } else { String::new() }
        }
        "synced_block" => {
            if let Some(ref sb) = block.synced_block {
                if let Some(ref children) = sb.children { render_block_list(children, indent, false) } else { String::new() }
            } else { String::new() }
        }
        "table_row" => String::new(),
        _ => String::new(),
    }
}

fn render_list_item(block: &NotionBlock, indent: usize, marker: &str, number: Option<u32>) -> String {
    let is = indent_str(indent);
    let prefix = if let Some(n) = number { format!("{}{}.", is, n) } else { format!("{}{}", is, marker) };
    let content = get_list_item_text(block);
    let mut result = format!("{} {}", prefix, content);
    let children = get_list_item_children(block);
    if let Some(ref ch) = children {
        let child_text = render_block_list(ch, indent + 1, false);
        if !child_text.is_empty() { result.push('\n'); result.push_str(&child_text); }
    }
    result
}

fn render_todo_item(block: &NotionBlock, indent: usize) -> String {
    let is = indent_str(indent);
    if let Some(ref todo) = block.to_do {
        let checked = todo.checked.unwrap_or(false);
        let checkbox = if checked { "[x]" } else { "[ ]" };
        let text = rich_text_str(todo.rich_text.as_ref());
        let mut result = format!("{}- {} {}", is, checkbox, text);
        if let Some(ref children) = todo.children {
            let child_text = render_block_list(children, indent + 1, false);
            if !child_text.is_empty() { result.push('\n'); result.push_str(&child_text); }
        }
        result
    } else { String::new() }
}

fn render_code_block(block: &NotionBlock, indent: usize) -> String {
    if let Some(ref code) = block.code {
        let is = indent_str(indent);
        let language = code.language.as_deref().unwrap_or("");
        let text = rich_text_str(code.rich_text.as_ref());
        if text.is_empty() { return String::new(); }
        format!("{}```{}\n{}\n{}```", is, language, text, is)
    } else { String::new() }
}

fn render_quote_block(block: &NotionBlock, indent: usize) -> String {
    let is = indent_str(indent);
    let mut result = String::new();
    if let Some(ref q) = block.quote {
        let text = rich_text_str(q.rich_text.as_ref());
        for line in text.lines() { result.push_str(&format!("{}> {}\n", is, line)); }
        if let Some(ref children) = q.children {
            let child_text = render_block_list(children, indent, false);
            if !child_text.is_empty() { for line in child_text.lines() { result.push_str(&format!("{}> {}\n", is, line)); } }
        }
    }
    result.trim_end().to_string()
}

fn render_callout_block(block: &NotionBlock, indent: usize) -> String {
    let is = indent_str(indent);
    let mut result = String::new();
    if let Some(ref c) = block.callout {
        let icon = extract_icon_str(&c.icon);
        let text = rich_text_str(c.rich_text.as_ref());
        let first_line = if icon.is_empty() { format!("{}> {}", is, text) } else { format!("{}> **{}** {}", is, icon, text) };
        result.push_str(&first_line);
        if let Some(ref children) = c.children {
            let child_text = render_block_list(children, indent, false);
            if !child_text.is_empty() { result.push('\n'); for line in child_text.lines() { result.push_str(&format!("\n{}> {}", is, line)); } }
        }
    }
    result
}

fn render_media_block(block: &NotionBlock, block_type: &str, is: &str) -> String {
    match block_type {
        "image" => {
            if let Some(ref img) = block.image {
                match get_media_url(img.image.as_ref(), img.external.as_ref()) {
                    Some(url) => {
                        let caption = rich_text_str(block.paragraph.as_ref().and_then(|p| p.rich_text.as_ref()));
                        let alt = if caption.is_empty() { "" } else { &caption };
                        format!("{}![{}]({})", is, alt, url)
                    }
                    None => String::new(),
                }
            } else { String::new() }
        }
        "embed" => {
            if let Some(ref em) = block.embed {
                match get_media_url(em.image.as_ref(), em.external.as_ref()) { Some(url) => format!("{}![embed]({})", is, url), None => String::new() }
            } else { String::new() }
        }
        "video" => {
            let url = block.video.as_ref().and_then(|v| v.video.as_ref().map(|f| f.url.clone())).or_else(|| block.video.as_ref().and_then(|v| v.external.as_ref().map(|e| e.url.clone())));
            match url { Some(u) => format!("{}[video: {}]", is, u), None => String::new() }
        }
        "pdf" => {
            let url = block.pdf.as_ref().and_then(|p| p.image.as_ref().map(|f| f.url.clone())).or_else(|| block.pdf.as_ref().and_then(|p| p.external.as_ref().map(|e| e.url.clone())));
            match url { Some(u) => format!("{}[pdf: {}]", is, u), None => String::new() }
        }
        _ => String::new(),
    }
}

fn render_file_block(block: &NotionBlock, is: &str) -> String {
    if let Some(ref file) = block.file {
        let url = file.external.as_ref().map(|e| e.url.clone()).or_else(|| file.file.as_ref().map(|f| f.url.clone()));
        match url { Some(u) => format!("{}[file]({})", is, u), None => String::new() }
    } else { String::new() }
}

fn render_bookmark_block(block: &NotionBlock, is: &str) -> String {
    if let Some(ref bm) = block.bookmark {
        let caption = bm.caption.as_ref().map(|c| render_rich_text(c)).unwrap_or_default();
        let text = if caption.is_empty() { &bm.url } else { return format!("{}[{}]({})", is, caption, bm.url); };
        format!("{}[{}]({})", is, text, bm.url)
    } else { String::new() }
}

fn render_gfm_table(rows: &[&TableRowContent], has_header: bool, _width: usize) -> String {
    if rows.is_empty() { return String::new(); }
    let col_count = rows.iter().map(|r| r.cells.as_ref().map(|c| c.len()).unwrap_or(0)).max().unwrap_or(0);
    if col_count == 0 { return String::new(); }
    let mut out = String::new();
    let format_row = |cells: &[Vec<RichTextItem>], count: usize| -> String {
        let cell_strs: Vec<String> = cells.iter().map(|cell| render_rich_text(cell)).collect();
        let mut padded: Vec<&str> = cell_strs.iter().map(|s| s.as_str()).collect();
        while padded.len() < count { padded.push(""); }
        format!("| {} |", padded.join(" | "))
    };
    for (idx, row) in rows.iter().enumerate() {
        let cells = row.cells.as_deref().unwrap_or(&[]);
        out.push_str(&format_row(cells, col_count));
        out.push('\n');
        if has_header && idx == 0 {
            let sep: Vec<String> = std::iter::repeat("---".to_string()).take(col_count).collect();
            out.push_str(&format!("| {} |\n", sep.join(" | ")));
        }
    }
    out.trim_end().to_string()
}

fn render_rich_text(items: &[RichTextItem]) -> String {
    let mut output = String::new();
    for item in items {
        let plain = item.plain_text.as_deref().unwrap_or("");
        if item.rt_type == "equation" {
            if let Some(ref eq) = item.equation { output.push_str(&format!("${}$", eq.expression)); } else { output.push_str(plain); }
            continue;
        }
        if item.rt_type == "mention" { render_mention(&item, plain, &mut output); continue; }
        let formatted = if let Some(ref ann) = item.annotations { render_annotated_text(plain, ann) } else { plain.to_string() };
        let link_url = item.href.as_deref().and_then(|u| if u.is_empty() { None } else { Some(u.to_string()) }).or_else(|| item.text.as_ref().and_then(|t| t.link.as_ref().map(|l| l.url.clone())));
        if let Some(url) = link_url { output.push_str(&format!("[{}]({})", formatted, url)); } else { output.push_str(&formatted); }
    }
    output
}

fn render_mention(item: &RichTextItem, plain: &str, output: &mut String) {
    if let Some(ref mention) = item.mention {
        match mention.mention_type.as_str() {
            "user" => output.push_str(&format!("@{}", plain)),
            "page" | "database" => { let title = plain.trim(); if title.is_empty() { output.push_str(plain); } else { output.push_str(&format!("[[{}]]", title)); } }
            _ => output.push_str(plain),
        }
    } else { output.push_str(plain); }
}

fn render_annotated_text(text: &str, ann: &Annotations) -> String {
    if text.is_empty() { return String::new(); }
    if ann.code.unwrap_or(false) { return format!("`{}`", text); }
    let mut result = text.to_string();
    if ann.strikethrough.unwrap_or(false) { result = format!("~~{}~~", result); }
    if ann.italic.unwrap_or(false) { result = format!("*{}*", result); }
    if ann.bold.unwrap_or(false) { result = format!("**{}**", result); }
    if ann.underline.unwrap_or(false) { result = format!("<u>{}</u>", result); }
    if let Some(ref color) = ann.color { if color != "default" && !color.is_empty() { result = format!("<span style=\"color: {}\">{}</span>", color, result); } }
    result
}

fn indent_str(indent: usize) -> String { if indent == 0 { String::new() } else { " ".repeat(indent * 2) } }

fn rich_text_str(rich_text: Option<&Vec<RichTextItem>>) -> String { match rich_text { Some(items) if !items.is_empty() => render_rich_text(items), _ => String::new() } }

fn get_list_item_text(block: &NotionBlock) -> String {
    if let Some(ref b) = block.bulleted_list_item { return rich_text_str(b.rich_text.as_ref()); }
    if let Some(ref n) = block.numbered_list_item { return rich_text_str(n.rich_text.as_ref()); }
    String::new()
}

fn get_list_item_children(block: &NotionBlock) -> Option<&Vec<NotionBlock>> {
    if let Some(ref b) = block.bulleted_list_item { return b.children.as_ref(); }
    if let Some(ref n) = block.numbered_list_item { return n.children.as_ref(); }
    None
}

fn get_media_url(file_url: Option<&FileUrl>, external: Option<&ExternalFile>) -> Option<String> {
    file_url.map(|f| f.url.clone()).or_else(|| external.map(|e| e.url.clone()))
}

fn extract_icon_str(icon: &Option<Value>) -> String {
    match icon {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Object(obj)) => {
            if let Some(Value::String(emoji)) = obj.get("emoji") { return emoji.clone(); }
            if let Some(Value::Object(ext)) = obj.get("external") { if let Some(Value::String(url)) = ext.get("url") { return url.clone(); } }
            if let Some(Value::String(typ)) = obj.get("type") { if typ == "emoji" { if let Some(Value::String(emoji)) = obj.get("emoji") { return emoji.clone(); } } }
            String::new()
        }
        _ => String::new(),
    }
}