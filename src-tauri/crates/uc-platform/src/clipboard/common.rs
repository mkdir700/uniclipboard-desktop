use anyhow::{anyhow, ensure, Result};
use clipboard_rs::{common::RustImage, Clipboard, ContentFormat};
use uc_core::clipboard::{MimeType, SystemClipboardRepresentation, SystemClipboardSnapshot};
use uc_core::ids::RepresentationId;

pub struct CommonClipboardImpl;

fn map_clipboard_err<T>(
    result: std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>,
) -> Result<T> {
    result.map_err(|e| anyhow!(e))
}

impl CommonClipboardImpl {
    pub fn read_snapshot(
        ctx: &mut clipboard_rs::ClipboardContext,
    ) -> Result<SystemClipboardSnapshot> {
        let available = map_clipboard_err(ctx.available_formats())?;

        let mut reps = Vec::new();

        if ctx.has(ContentFormat::Text) {
            if let std::result::Result::Ok(text) = ctx.get_text() {
                reps.push(SystemClipboardRepresentation {
                    id: RepresentationId::new(),
                    format_id: "text".into(),
                    mime: Some(MimeType::text_plain()),
                    bytes: text.into_bytes(),
                });
            }
        }

        if ctx.has(ContentFormat::Rtf) {
            if let std::result::Result::Ok(rtf) = ctx.get_rich_text() {
                reps.push(SystemClipboardRepresentation {
                    id: RepresentationId::new(),
                    format_id: "rtf".into(),
                    mime: Some(MimeType("text/rtf".to_string())),
                    bytes: rtf.into_bytes(),
                });
            }
        }

        if ctx.has(ContentFormat::Html) {
            if let std::result::Result::Ok(html) = ctx.get_html() {
                reps.push(SystemClipboardRepresentation {
                    id: RepresentationId::new(),
                    format_id: "html".into(),
                    mime: Some(MimeType::text_html()),
                    bytes: html.into_bytes(),
                });
            }
        }

        if ctx.has(ContentFormat::Files) {
            if let std::result::Result::Ok(files) = ctx.get_files() {
                reps.push(SystemClipboardRepresentation {
                    id: RepresentationId::new(),
                    format_id: "files".into(),
                    mime: Some(MimeType("text/uri-list".to_string())),
                    bytes: files.join("\n").into_bytes(),
                });
            }
        }

        if ctx.has(ContentFormat::Image) {
            if let std::result::Result::Ok(img) = ctx.get_image() {
                if let std::result::Result::Ok(png) = img.to_png() {
                    reps.push(SystemClipboardRepresentation {
                        id: RepresentationId::new(),
                        format_id: "image".into(),
                        mime: Some(MimeType("image/png".to_string())),
                        bytes: png.get_bytes().to_vec(),
                    });
                }
            }
        }

        // raw fallback
        use std::collections::HashSet;
        let seen: HashSet<String> = reps.iter().map(|r| r.format_id.to_string()).collect();

        for format_id in available {
            if seen.contains(&format_id) {
                continue;
            }
            if let std::result::Result::Ok(buf) = ctx.get_buffer(&format_id) {
                reps.push(SystemClipboardRepresentation {
                    id: RepresentationId::new(),
                    format_id: format_id.into(),
                    mime: None,
                    bytes: buf,
                });
            }
        }

        Ok(SystemClipboardSnapshot {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            representations: reps,
        })
    }

    pub fn write_snapshot(
        ctx: &mut clipboard_rs::ClipboardContext,
        snapshot: SystemClipboardSnapshot,
    ) -> Result<()> {
        ensure!(
            snapshot.representations.len() == 1,
            "platform::write expects exactly ONE representation"
        );

        let rep = &snapshot.representations[0];

        match rep.mime.as_ref().map(|m| m.as_str()) {
            Some("text/plain") => {
                map_clipboard_err(ctx.set_text(String::from_utf8(rep.bytes.clone())?))?;
            }
            Some("text/rtf") => {
                map_clipboard_err(ctx.set_rich_text(String::from_utf8(rep.bytes.clone())?))?;
            }
            Some("text/html") => {
                map_clipboard_err(ctx.set_html(String::from_utf8(rep.bytes.clone())?))?;
            }
            Some("text/uri-list") | Some("file/uri-list") => {
                let files = String::from_utf8(rep.bytes.clone())?
                    .lines()
                    .map(|s| s.to_string())
                    .collect();
                map_clipboard_err(ctx.set_files(files))?;
            }
            Some("image/png") => {
                let img =
                    clipboard_rs::RustImageData::from_bytes(&rep.bytes).map_err(|e| anyhow!(e))?;
                map_clipboard_err(ctx.set_image(img))?;
            }
            _ => {
                map_clipboard_err(ctx.set_buffer(&rep.format_id, rep.bytes.clone()))?;
            }
        }

        Ok(())
    }
}
