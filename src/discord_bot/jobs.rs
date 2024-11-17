use crate::types::coordinates::ArkMap;
use rkyv::{Archive, Deserialize, Serialize};
use serenity::all::MessageId;
use std::future::Future;

#[derive(Archive, Serialize, Deserialize)]
struct GeneratorList {
    server: String,
    map: ArkMap,
    list_id: Vec<u8>,
}

#[derive(Archive, Serialize, Deserialize)]
#[repr(u16)]
enum JobAction {
    UpdateTimers() = 0,
}

#[derive(Archive, Serialize, Deserialize)]
struct Job {
    action: JobAction,
}
