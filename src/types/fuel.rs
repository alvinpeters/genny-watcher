use std::time::Duration;
use rkyv::{Archive, Deserialize, Serialize};

pub(crate) trait FuelItem {
    fn lasts_until(&self, duration_secs: u64) -> Duration;
}

/// Power duration in seconds
#[derive(Archive, Deserialize, Serialize)]
pub(crate) struct Fuel<I: FuelItem, const DURATION_SECS: u64> {
    fuel: I,
    last_filled: u64,
}

#[derive(Archive, Deserialize, Serialize)]
pub(crate) struct ElementOrShards {
    raw_element: RawElement,
    element_shards: ElementShard
}

/// DURATION_SECS refer to a single raw element
impl FuelItem for ElementOrShards where {
    fn lasts_until(&self, duration_secs: u64) -> Duration  {
        todo!()
    }
}

#[derive(Archive, Deserialize, Serialize)]
struct RawElement {
    count: u32
}

impl FuelItem for RawElement where {
    fn lasts_until(&self, duration_secs: u64) -> Duration  {
        Duration::from_secs(duration_secs * self.count as u64)
    }
}

#[derive(Archive, Deserialize, Serialize)]
struct ElementShard {
    count: u32
}

impl FuelItem for ElementShard {
    fn lasts_until(&self, duration_secs: u64) -> Duration {
        Duration::from_secs(duration_secs * self.count as u64)
    }
}

#[derive(Archive, Deserialize, Serialize)]
pub(crate) struct Gasoline {
    count: u32,
}