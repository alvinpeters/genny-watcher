use foundationdb::{api::NetworkAutoStop, Database, Transaction};

pub(crate) struct FdbBackend {
    guard: NetworkAutoStop,
}
