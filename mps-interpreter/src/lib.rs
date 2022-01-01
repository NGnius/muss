//! All necessary components to interpret and run a MPS script.
//!
//! MpsInterpretor uses a non-standard Iterator implementation,
//! so it is recommended to use MpsRunner to execute a script.
//! Since MPS is centered around iterators, script execution is also done by iterating.
//!
//! MpsInterpretor is misspelt to emphasise that it behaves strangely:
//! after every MPS statement, a None item is returned even when the script is not complete.
//! MpsRunner wraps MpsInterpretor so that this behaviour is hidden when iterating.
//!
//! ```
//! use std::io::Cursor;
//! use mps_interpreter::*;
//!
//! let cursor = Cursor::new(
//! "files(folder=`~/Music/`, recursive=true)" // retrieve all files from Music folder
//! );
//!
//! let interpreter = MpsRunner::with_stream(cursor);
//!
//! // warning: my library has ~3800 songs, so this outputs too much information to be useful.
//! for result in interpreter {
//!     match result {
//!         Ok(item) => println!("Got song `{}` (file: `{}`)", item.title, item.filename),
//!         Err(e) => panic!("Got error while executing: {}", e),
//!     }
//! }
//! ```
//!

mod context;
mod interpretor;
pub mod lang;
#[cfg(feature = "music_library")]
pub mod music;
mod music_item;
pub mod processing;
mod runner;
pub mod tokens;

pub use context::MpsContext;
pub use interpretor::{interpretor, MpsInterpretor};
pub use music_item::MpsMusicItem;
pub use runner::MpsRunner;

#[cfg(test)]
mod tests {}
