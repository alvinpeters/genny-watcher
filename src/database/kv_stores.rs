mod backends;
pub(super) mod types;

pub(crate) enum KvStore {
    #[cfg(feature = "fdb")]
    FoundationDB(backends::fdb::FdbBackend),
    #[cfg(feature = "rdb")]
    RocksDB(backends::rdb::RdbStore),
}
