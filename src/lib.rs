//! PNGTuber Lite: overlay transparente con VAD por RMS.
//!
//! La biblioteca expone la composición (`app`, `window`), los features y las
//! piezas compartidas. El binario (`src/main.rs`) solo arranca la aplicación.

pub mod app;
pub mod features;
pub mod shared;
pub mod win32;
pub mod window;
