//! Transport-size protection for request copies, never persisted session history.

use crate::context::ContextualUserFragment;
use crate::context::ImageBudgetOmission;
use crate::context::is_contextual_user_fragment;
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputContentItem;
use codex_protocol::models::ResponseItem;

const MAX_REQUEST_INLINE_IMAGE_BYTES: usize = 12 * 1024 * 1024;
const MAX_RETAINED_INLINE_IMAGES: usize = 8;
const MAX_PROTECTED_LATEST_USER_IMAGES: usize = 4;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ImageBudgetStats {
    pub(crate) original_bytes: usize,
    pub(crate) retained_bytes: usize,
    pub(crate) retained_images: usize,
    pub(crate) omitted_images: usize,
}

enum ImageSlot<'a> {
    Message(&'a mut ContentItem),
    Tool(&'a mut FunctionCallOutputContentItem),
}

fn inline_image_bytes(url: &str) -> Option<usize> {
    let (header, payload) = url.split_once(',')?;
    (header.get(..11)?.eq_ignore_ascii_case("data:image/")
        && header.rsplit(';').next()?.eq_ignore_ascii_case("base64")
        && !payload.is_empty())
    .then_some(url.len())
}

/// Prefer up to four images in the last real user message, then newest images.
/// Priority never overrides either hard limit, including for a single oversized image.
pub(crate) fn apply_inline_image_byte_budget(input: &mut [ResponseItem]) -> ImageBudgetStats {
    let latest_user = input.iter().rposition(|item| {
        matches!(item, ResponseItem::Message { role, content, .. }
            if role == "user" && !content.iter().all(is_contextual_user_fragment))
    });
    let mut slots = Vec::new();
    let mut protected = 0;
    for (index, item) in input.iter_mut().enumerate().rev() {
        match item {
            ResponseItem::Message { content, .. } => {
                for part in content.iter_mut().rev() {
                    if let ContentItem::InputImage { image_url, .. } = part
                        && let Some(bytes) = inline_image_bytes(image_url)
                    {
                        let priority = Some(index) == latest_user
                            && protected < MAX_PROTECTED_LATEST_USER_IMAGES;
                        protected += usize::from(priority);
                        slots.push((priority, bytes, ImageSlot::Message(part)));
                    }
                }
            }
            ResponseItem::FunctionCallOutput { output, .. }
            | ResponseItem::CustomToolCallOutput { output, .. } => {
                if let Some(content) = output.content_items_mut() {
                    for part in content.iter_mut().rev() {
                        if let FunctionCallOutputContentItem::InputImage { image_url, .. } = part
                            && let Some(bytes) = inline_image_bytes(image_url)
                        {
                            slots.push((false, bytes, ImageSlot::Tool(part)));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    // Stable sorting preserves newest-first order within each priority group.
    slots.sort_by_key(|(priority, ..)| !priority);
    let mut stats = ImageBudgetStats::default();
    for (_, bytes, slot) in slots {
        stats.original_bytes = stats.original_bytes.saturating_add(bytes);
        if bytes <= MAX_REQUEST_INLINE_IMAGE_BYTES - stats.retained_bytes
            && stats.retained_images < MAX_RETAINED_INLINE_IMAGES
        {
            stats.retained_bytes += bytes;
            stats.retained_images += 1;
        } else {
            let text = ImageBudgetOmission.body();
            match slot {
                ImageSlot::Message(part) => *part = ContentItem::InputText { text },
                ImageSlot::Tool(part) => *part = FunctionCallOutputContentItem::InputText { text },
            }
            stats.omitted_images += 1;
        }
    }
    stats
}

#[cfg(test)]
#[path = "request_image_budget_tests.rs"]
mod tests;
