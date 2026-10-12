pub mod port;
pub mod statefs;

pub use port::{ConfigStorageMut, ConfigStoragePort};
pub use statefs::StateFsStorageAdapter;
