//! IPC layer — JSON-RPC wire types, read/write helpers, and the dispatcher.

pub mod dispatcher;
mod mod_types;

// Re-export wire types at the ipc:: level for convenience.
pub use mod_types::{read_request, write_response, Request, Response, RpcError};
