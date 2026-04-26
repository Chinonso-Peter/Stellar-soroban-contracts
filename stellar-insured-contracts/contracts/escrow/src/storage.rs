use soroban_sdk::{contracttype, Address};

use crate::types::ApprovalType;

#[contracttype]
pub enum DataKey {
    Escrow(u64),
    EscrowCount,
    Admin,
    Paused,
    MultiSig(u64),
    Signature(u64, ApprovalType, Address),
    SigCount(u64, ApprovalType),
}
