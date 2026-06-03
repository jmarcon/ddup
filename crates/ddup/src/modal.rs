//! Modal text.

use crate::app::Modal;

/// Returns modal text.
#[must_use]
pub fn modal_text(modal: &Modal) -> Option<String> {
    match modal {
        Modal::None => None,
        Modal::Help => Some(
            "Keys: q quit, Esc close help/quit, j/k or arrows move, mouse click/scroll\n\
             enter toggle, h close, l open, f view, s sort field, o/r sort order\n\
             d mark delete, m mark keep, u clear mark, F5/Ctrl+R re-scan, ? help\n\
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
