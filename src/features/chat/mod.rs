//! Feature chat: maqueta visual de la conversacion.
//!
//! `state.rs` guarda el modelo puro y testeable; `view.rs` solo lo dibuja.

mod state;
mod view;

pub use state::{ChatState, Message, Role};
pub use view::ChatView;
