//! Modal text.

use crate::app::Modal;

/// Returns modal text.
#[must_use]
pub fn modal_text(modal: &Modal) -> Option<String> {
    match modal {
        Modal::None => None,
        Modal::Help => Some(
            "Navigation\n\
             j/k or arrows       Move selection\n\
             PgUp/PgDown         Move by page\n\
             Enter               Open or close group\n\
             h/l                 Close or open group\n\
             Mouse               Click selection or scroll\n\
             \n\
             Selection marks\n\
             Space, d, x         Toggle delete mark [x]\n\
             K                   Toggle keep mark [K]\n\
             u or Backspace      Clear mark\n\
             a                   Apply marked delete/keep actions\n\
             \n\
             Actions\n\
             Delete              Delete selected item now\n\
             m                   Move selected item\n\
             o                   Open in file manager\n\
             c                   Copy full path\n\
             \n\
             Views and scan\n\
             f                   Cycle Dirs / Files Smart / Files Flat\n\
             s/r                 Change sort field / order\n\
             F5 or Ctrl+R        Re-scan\n\
             i                   Toggle Nerd Font icons\n\
             q or Esc            Exit\n\
             \n\
             Smart mode\n\
             Smart hides duplicate files already covered by duplicate directories.\n\
             Use Flat when you need every duplicate file listed."
                .to_owned(),
        ),
        Modal::ConfirmDelete { .. } => Some("Confirm delete? Y/N".to_owned()),
        Modal::MovePrompt { input, .. } => Some(format!("Move to: {input}")),
        Modal::Error { message } => Some(message.clone()),
    }
}

/// Returns modal title.
#[must_use]
pub fn modal_title(modal: &Modal) -> &'static str {
    match modal {
        Modal::None => "",
        Modal::Help => "Help",
        Modal::ConfirmDelete { .. } => "Confirm delete",
        Modal::MovePrompt { .. } => "Move item",
        Modal::Error { .. } => "Error",
    }
}
