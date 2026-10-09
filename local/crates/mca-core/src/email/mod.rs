mod body;
mod compose;
mod draft;
mod html;

pub use compose::{compose, subject_line};
pub use draft::{EmailDraft, EmailInput};
pub use html::escape as escape_html;
