mod client;
mod cookie;
mod error;
mod response;

// public re-exports
pub type Result<T> = std::result::Result<T, error::Error>;
pub use crate::client::new_client;
pub use crate::client::new_client_with_reqwest;
pub use crate::client::Client;
pub use crate::cookie::Cookie;
pub use crate::error::Error;
pub use crate::response::Response;
