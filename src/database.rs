use crate::database::kv_stores::types::AsKey;
use crate::Result;

mod models;

#[cfg(feature = "kv_stores")]
mod kv_stores;

enum Database {

}

pub(crate) trait DbModel: Sized {
    const KEYSPACE: &'static [u8];
    type Key: AsKey;

    async fn get(key: Self::Key) -> Result<Self>;
}
