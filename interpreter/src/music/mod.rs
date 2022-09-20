mod build_library;
mod library;
mod tag;

pub use build_library::build_library_from_files;
#[cfg(feature = "sql")]
pub use build_library::build_library_from_sqlite;
pub use library::Library;
