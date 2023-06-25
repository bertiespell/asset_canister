use ic_cdk::export::candid::{CandidType, Deserialize};

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub struct CanisterInfo {
    pub heap_memory_size: u64,
    pub memory_size: u64,
    pub cycles: u64,
}
