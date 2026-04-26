use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
pub enum DataKey {
    Config,
    Admin,
    Request(u64),
    History(Address),
    ChainInfo(u32),
    VerifiedTx(BytesN<32>),
    Operators,
    ReqCounter,
    TxCounter,
}

/// Maximum bridge history entries retained per account (prevents unbounded growth).
pub const MAX_HISTORY_ITEMS: u32 = 50;
