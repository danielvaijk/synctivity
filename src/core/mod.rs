mod repo;
mod user;

const SYNC_REPO_NAME: &str = "synctivity";

pub use self::repo::copy::CopyRepo;
pub use self::repo::sync::SyncRepo;

pub use self::user::author::Author;
