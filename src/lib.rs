mod client;
mod cookie;
mod error;
mod response;

// public re-exports
pub type Result<T> = std::result::Result<T, error::Error>;
pub use client::new_client;
pub use error::Error;
pub use response::Response;
