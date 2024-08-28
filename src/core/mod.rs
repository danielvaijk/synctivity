pub mod copy;
pub mod sync;

pub use copy::CopyRepo;
pub use sync::SyncRepo;

const SYNC_REPO_NAME: &str = "synctivity";
