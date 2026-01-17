use super::model::{SelectionPolicyVersion, SelectionTarget};
use crate::{
    clipboard::{
        ClipboardSelection, ObservedClipboardRepresentation, PolicyError, SystemClipboardSnapshot,
    },
    ids::RepresentationId,
    ports::SelectRepresentationPolicyPort,
};
use std::cmp::Ordering;

/// v1 策略：稳定、可解释、保守
///
/// v1 的核心：
/// - UI Preview 优先简洁预览：files > plain > image > rich > uri > unknown
/// - Default Paste 优先保留格式：files > rich > plain > image > uri > unknown
/// - stable sort: score desc, size asc, format_id asc, id asc
#[derive(Debug, Default)]
pub struct SelectRepresentationPolicyV1;

impl SelectRepresentationPolicyV1 {
    pub fn new() -> Self {
        Self
    }

    fn is_usable(rep: &ObservedClipboardRepresentation) -> bool {
        if rep.size_bytes() <= 0 {
            return false;
        }
        true
    }

    fn classify(rep: &ObservedClipboardRepresentation) -> RepKind {
        // 注意：v1 刻意不引入平台特例，只基于 mime_type + 少量 format_id 兜底
        if rep.mime.is_none() {
            return RepKind::Unknown;
        }

        let mime = rep.mime.as_ref().unwrap();

        // 文件列表（常见：text/uri-list）
        if mime.eq_ignore_ascii_case("text/uri-list") || mime.starts_with("text/uri-list") {
            return RepKind::FileList;
        }

        // 图片（image/*）
        if mime.starts_with("image/") {
            return RepKind::Image;
        }

        // 富文本（html/rtf）
        if mime.eq_ignore_ascii_case("text/html") || mime.eq_ignore_ascii_case("text/rtf") {
            return RepKind::RichText;
        }

        // 纯文本（text/plain 或其他 text/*）
        if mime.eq_ignore_ascii_case("text/plain") || mime.starts_with("text/") {
            return RepKind::PlainText;
        }

        // URI（有些平台会给 text/x-uri / application/x-url 等；v1 只做轻量识别）
        if mime.contains("uri") || mime.contains("url") {
            return RepKind::Uri;
        }

        // format_id 兜底（非常保守）
        // 例如某些实现会把文件列表映射到 format_id="files"
        if rep.format_id.eq_ignore_ascii_case("files") || rep.format_id.contains("uri-list") {
            return RepKind::FileList;
        }

        RepKind::Unknown
    }

    fn score(kind: RepKind, target: SelectionTarget) -> i32 {
        match (target, kind) {
            // UiPreview: PlainText 优先（简洁预览），其次 Image，最后 RichText
            (SelectionTarget::UiPreview, RepKind::FileList) => 100,
            (SelectionTarget::UiPreview, RepKind::PlainText) => 90,
            (SelectionTarget::UiPreview, RepKind::Image) => 80,
            (SelectionTarget::UiPreview, RepKind::RichText) => 70,
            (SelectionTarget::UiPreview, RepKind::Uri) => 60,
            (SelectionTarget::UiPreview, RepKind::Unknown) => 10,

            // DefaultPaste: RichText 优先（保留格式），其次 PlainText（兼容性），最后 Image
            (SelectionTarget::DefaultPaste, RepKind::FileList) => 100,
            (SelectionTarget::DefaultPaste, RepKind::RichText) => 90,
            (SelectionTarget::DefaultPaste, RepKind::PlainText) => 80,
            (SelectionTarget::DefaultPaste, RepKind::Image) => 70,
            (SelectionTarget::DefaultPaste, RepKind::Uri) => 60,
            (SelectionTarget::DefaultPaste, RepKind::Unknown) => 10,
        }
    }

    fn select_one<'a>(
        snapshot: &'a SystemClipboardSnapshot,
        target: SelectionTarget,
    ) -> Option<&'a ObservedClipboardRepresentation> {
        let mut reps: Vec<&ObservedClipboardRepresentation> = snapshot
            .representations
            .iter()
            .filter(|r| Self::is_usable(r))
            .collect();

        if reps.is_empty() {
            return None;
        }

        reps.sort_by(|a, b| {
            let ka = Self::classify(a);
            let kb = Self::classify(b);

            // 1) 分数：desc
            let sa = Self::score(ka, target);
            let sb = Self::score(kb, target);
            match sb.cmp(&sa) {
                Ordering::Equal => {}
                ord => return ord,
            }

            // 2) size：asc（更轻更不容易卡 UI；paste 也更稳）
            match a.size_bytes().cmp(&b.size_bytes()) {
                Ordering::Equal => {}
                ord => return ord,
            }

            // 3) format_id：asc（保证稳定）
            match a.format_id.cmp(&b.format_id) {
                Ordering::Equal => {}
                ord => return ord,
            }

            // 4) id：asc（最终稳定）
            a.id.cmp(&b.id)
        });

        reps.into_iter().next()
    }
}

impl SelectRepresentationPolicyPort for SelectRepresentationPolicyV1 {
    fn select(
        &self,
        snapshot: &SystemClipboardSnapshot,
    ) -> Result<ClipboardSelection, PolicyError> {
        let preview = Self::select_one(snapshot, SelectionTarget::UiPreview)
            .ok_or(PolicyError::NoUsableRepresentation)?;

        let paste = Self::select_one(snapshot, SelectionTarget::DefaultPaste)
            .ok_or(PolicyError::NoUsableRepresentation)?;

        // 收集所有可用的 representations
        let usable_reps: Vec<&ObservedClipboardRepresentation> = snapshot
            .representations
            .iter()
            .filter(|r| Self::is_usable(r))
            .collect();

        // 找出除 primary 之外的其他 representation IDs
        let secondary_rep_ids: Vec<RepresentationId> = usable_reps
            .iter()
            .filter(|r| r.id != paste.id)
            .map(|r| r.id.clone())
            .collect();

        // v1：primary = paste，secondary 包含其他所有可用的 representations
        Ok(ClipboardSelection {
            primary_rep_id: paste.id.clone(),
            preview_rep_id: preview.id.clone(),
            paste_rep_id: paste.id.clone(),
            secondary_rep_ids,
            policy_version: SelectionPolicyVersion::V1,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepKind {
    FileList,
    Image,
    RichText,
    PlainText,
    Uri,
    Unknown,
}
