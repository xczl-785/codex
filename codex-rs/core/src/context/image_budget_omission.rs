use super::ContextualUserFragment;
use codex_protocol::models::ContentItemKind;

pub(crate) struct ImageBudgetOmission;

impl ContextualUserFragment for ImageBudgetOmission {
    fn content_kind(&self) -> ContentItemKind {
        ContentItemKind("images.request_budget_omission".to_owned())
    }

    fn role(&self) -> &'static str {
        "user"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        ("", "")
    }

    fn body(&self) -> String {
        "[image omitted from this request by the local inline-image budget; reopen the original image or a smaller crop if its visual details are needed]".to_owned()
    }
}
