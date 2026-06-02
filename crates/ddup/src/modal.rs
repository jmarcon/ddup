//! Modal text.

use crate::app::Modal;

/// Returns modal text.
#[must_use]
pub fn modal_text(modal: &Modal) -> Option<String> {
    match modal {
        Modal::None => None,
        Modal::Help => Some(
            "Keys: q quit, j/k move, enter expand, f view, s sort, r order\n\
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
