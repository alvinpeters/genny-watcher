use clap::builder::Str;
use rkyv::{Archive, Deserialize, Serialize};
use crate::types::fuel::{ElementOrShards, Fuel};
use super::coordinates::{UE4Coordinates};

pub(crate) trait TrackedStructure {
    fn coords(&self) -> UE4Coordinates;
}

pub(crate) trait Generator {
    fn range(&self) -> i32;
}

#[derive(Archive, Serialize, Deserialize)]
pub(crate) struct TekGenerator {
    id: u64,
    name: String,
    coordinates: UE4Coordinates,
    current_fuel: Fuel<ElementOrShards, 64800>,
}