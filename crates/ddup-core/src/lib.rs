//! ddup-core: engine de detecção de duplicatas (diretórios e arquivos).

pub mod error;
pub mod types;

pub use error::{CoreError, Result};
pub use types::{DirHash, FileHash};
