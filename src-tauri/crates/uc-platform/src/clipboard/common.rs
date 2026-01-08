use anyhow::*;
use clipboard_rs::Clipboard;
use uc_core::clipboard::{ClipboardContent, RawClipboardRepresentation, RawClipboardSnapshot};

pub struct CommonClipboardImpl;

impl CommonClipboardImpl {
    pub fn read_snapshot(ctx: &mut clipboard_rs::ClipboardContext) -> Result<RawClipboardSnapshot> {
        let available = ctx.available_formats()?;

        let mut reps = Vec::new();

        if ctx.has(ContentFormat::Text) {
            if let Ok(text) = ctx.get_text() {
                reps.push(RawClipboardRepresentation {
                    format_id: "text".into(),
                    mime: Some("text/plain".into()),
                    bytes: text.into_bytes(),
                });
            }
        }

        if ctx.has(ContentFormat::Rtf) {
            if let Ok(rtf) = ctx.get_rich_text() {
                reps.push(RawClipboardRepresentation {
                    format_id: "rtf".into(),
                    mime: Some("text/rtf".into()),
                    bytes: rtf.into_bytes(),
                });
            }
        }

        if ctx.has(ContentFormat::Html) {
            if let Ok(html) = ctx.get_html() {
                reps.push(RawClipboardRepresentation {
                    format_id: "html".into(),
                    mime: Some("text/html".into()),
                    bytes: html.into_bytes(),
                });
            }
        }

        if ctx.has(ContentFormat::Files) {
            if let Ok(files) = ctx.get_files() {
                reps.push(RawClipboardRepresentation {
                    format_id: "files".into(),
                    mime: Some("text/uri-list".into()),
                    bytes: files.join("\n").into_bytes(),
                });
            }
        }

        if ctx.has(ContentFormat::Image) {
            if let Ok(img) = ctx.get_image() {
                if let Ok(png) = img.to_png() {
                    reps.push(RawClipboardRepresentation {
                        format_id: "image".into(),
                        mime: Some("image/png".into()),
                        bytes: png.get_bytes().to_vec(),
                    });
                }
            }
        }

        // raw fallback
        use std::collections::HashSet;
        let mut seen: HashSet<String> = reps.iter().map(|r| r.format_id.clone()).collect();

        for format_id in available {
            if seen.contains(&format_id) {
                continue;
            }
            if let Ok(buf) = ctx.get_buffer(&format_id) {
                reps.push(RawClipboardRepresentation {
                    format_id,
                    mime: None,
                    bytes: buf,
                });
            }
        }

        Ok(RawClipboardSnapshot {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            representations: reps,
        })
    }

    pub fn write_snapshot(
        ctx: &mut clipboard_rs::ClipboardContext,
        snapshot: RawClipboardSnapshot,
    ) -> Result<()> {
        ensure!(
            snapshot.representations.len() == 1,
            "platform::write expects exactly ONE representation"
        );

        let rep = &snapshot.representations[0];

        match rep.mime.as_deref() {
            Some("text/plain") => {
                ctx.set_text(String::from_utf8(rep.bytes.clone())?)?;
            }
            Some("text/rtf") => {
                ctx.set_rich_text(String::from_utf8(rep.bytes.clone())?)?;
            }
            Some("text/html") => {
                ctx.set_html(String::from_utf8(rep.bytes.clone())?)?;
            }
            Some("text/uri-list") | Some("file/uri-list") => {
                let files = String::from_utf8(rep.bytes.clone())?
                    .lines()
                    .map(|s| s.to_string())
                    .collect();
                ctx.set_files(files)?;
            }
            Some("image/png") => {
                let img = clipboard_rs::RustImageData::from_png(rep.bytes.clone())?;
                ctx.set_image(img)?;
            }
            _ => {
                ctx.set_buffer(&rep.format_id, rep.bytes.clone())?;
            }
        }

        Ok(())
    }

    pub fn write_content(
        ctx: &mut clipboard_rs::ClipboardContext,
        content: &ClipboardContent,
    ) -> Result<()> {
        ensure!(
            content.items.len() == 1,
            "platform::write expects exactly ONE representation"
        );
        let item = content
            .items
            .first()
            .ok_or_else(|| anyhow!("No item found"))?;

        match item.mime.as_str() {
            "text/plain" => ctx.set_text(String::from_utf8(bytes)?)?,
            "text/rtf" => ctx.set_rich_text(String::from_utf8(bytes)?)?,
            "text/html" => ctx.set_html(String::from_utf8(bytes)?)?,
            "text/uri-list" | "file/uri-list" => {
                let files = String::from_utf8(bytes)?
                    .lines()
                    .map(|s| s.to_string())
                    .collect();
                ctx.set_files(files)?;
            }
            "image/png" => {
                let img = clipboard_rs::RustImageData::from_png(bytes)?;
                ctx.set_image(img)?;
            }
            _ => {
                ctx.set_buffer(mime, bytes)?;
            }
        }
        Ok(())
    }
}
