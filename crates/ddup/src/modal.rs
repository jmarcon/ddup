//! Modal text.

use crate::app::Modal;

/// Returns modal text.
#[must_use]
pub fn modal_text(modal: &Modal) -> Option<String> {
    match modal {
        Modal::None => None,
        Modal::Help => Some(
            "Keys: q quit, Esc close help/quit, j/k or arrows move, mouse click/scroll\n\
             enter toggle, h close, l open group, f view, s sort field, r sort order\n\
             d delete, m move, o open, x mark delete, p mark keep, u clear mark, a apply marks\n\
             F5/Ctrl+R re-scan, ? help\n\
             What is Smart mode?\n\
             Smart suppresses duplicate files inside duplicated folders.\n\
             Use Flat when every duplicate file matters."
                .to_owned(),
        ),
        Modal::ConfirmDelete { .. } => Some("Confirm delete? Y/N".to_owned()),
        Modal::MovePrompt { input, .. } => Some(format!("Move to: {input}")),
        Modal::Error { message } => Some(message.clone()),
    }
}
