#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec, log, Symbol};
use stellar_insured_lib::Proposal;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    SlashingContract,
    Proposal(u64),
    ProposalCounter,
    VoterRecord(u64, Address),
    VotingPeriod,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteRecord {
    pub voter: Address,
    pub weight: i128,
    pub is_yes: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalStats {
    pub yes_votes: i128,
    pub no_votes: i128,
    pub total_votes: i128,
    pub status: Symbol,
}

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        slashing_contract: Address,
        voting_period: u64,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::SlashingContract, &slashing_contract);
        env.storage().instance().set(&DataKey::VotingPeriod, &voting_period);
        env.storage().instance().set(&DataKey::ProposalCounter, &0u64);
    }

    pub fn create_proposal(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        execution_data: String,
        threshold_percentage: u32,
    ) -> u64 {
        creator.require_auth();

        let mut counter: u64 = env.storage().instance().get(&DataKey::ProposalCounter).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&DataKey::ProposalCounter, &counter);

        let voting_period: u64 = env.storage().instance().get(&DataKey::VotingPeriod).unwrap();
        
        let proposal = Proposal {
            id: counter,
            title,
            description,
            execution_data,
            creator,
            expires_at: env.ledger().timestamp() + voting_period,
            threshold_percentage,
            yes_votes: 0,
            no_votes: 0,
            is_finalized: false,
            is_executed: false,
        };

        env.storage().persistent().set(&DataKey::Proposal(counter), &proposal);

        env.events().publish(
            (symbol_short!("gov"), symbol_short!("created")),
            counter,
        );

        counter
    }

    pub fn create_slashing_proposal(
        env: Env,
        creator: Address,
        target: Address,
        role: Symbol,
        reason: String,
        amount: i128,
        threshold: u32,
    ) -> u64 {
        creator.require_auth();
        
        let title = String::from_str(&env, "Slashing Proposal");
        let mut description = String::from_str(&env, "Slash ");
        // Simplified description construction
        
        // Execution data would be a serialized call to slashing contract
        let execution_data = String::from_str(&env, "slash_call"); 

        self::GovernanceContract::create_proposal(
            env,
            creator,
            title,
            description,
            execution_data,
            threshold
        )
    }

    pub fn vote(env: Env, voter: Address, proposal_id: u64, weight: i128, is_yes: bool) {
        voter.require_auth();

        let mut proposal: Proposal = env.storage().persistent().get(&DataKey::Proposal(proposal_id)).expect("Proposal not found");
        
        if env.ledger().timestamp() > proposal.expires_at {
            panic!("Voting period ended");
        }

        let record_key = DataKey::VoterRecord(proposal_id, voter.clone());
        if env.storage().persistent().has(&record_key) {
            panic!("Already voted");
        }

        if is_yes {
            proposal.yes_votes += weight;
        } else {
            proposal.no_votes += weight;
        }

        let record = VoteRecord {
            voter: voter.clone(),
            weight,
            is_yes,
            timestamp: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().persistent().set(&record_key, &record);

        env.events().publish(
            (symbol_short!("gov"), symbol_short!("vote")),
            (proposal_id, voter),
        );
    }

    pub fn finalize_proposal(env: Env, proposal_id: u64) {
        let mut proposal: Proposal = env.storage().persistent().get(&DataKey::Proposal(proposal_id)).expect("Proposal not found");
        
        if env.ledger().timestamp() <= proposal.expires_at {
            panic!("Voting period not yet ended");
        }

        proposal.is_finalized = true;
        env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (symbol_short!("gov"), symbol_short!("final")),
            proposal_id,
        );
    }

    pub fn execute_proposal(env: Env, proposal_id: u64) {
        let mut proposal: Proposal = env.storage().persistent().get(&DataKey::Proposal(proposal_id)).expect("Proposal not found");
        
        if !proposal.is_finalized {
            panic!("Proposal must be finalized first");
        }

        if proposal.is_executed {
            panic!("Already executed");
        }

        let total_votes = proposal.yes_votes + proposal.no_votes;
        if total_votes == 0 || (proposal.yes_votes * 100 / total_votes) < proposal.threshold_percentage as i128 {
            panic!("Threshold not met");
        }

        proposal.is_executed = true;
        env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);
        
        env.events().publish(
            (symbol_short!("gov"), symbol_short!("exec")),
            proposal_id,
        );
    }

    pub fn execute_slashing_proposal(env: Env, proposal_id: u64) {
        // In a real implementation, this would parse execution_data and call slashing contract
        self::GovernanceContract::execute_proposal(env, proposal_id);
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        env.storage().persistent().get(&DataKey::Proposal(proposal_id)).expect("Proposal not found")
    }

    pub fn get_active_proposals(env: Env) -> Vec<u64> {
        let counter: u64 = env.storage().instance().get(&DataKey::ProposalCounter).unwrap_or(0);
        let mut list = Vec::new(&env);
        let now = env.ledger().timestamp();
        for i in 1..=counter {
            if let Some(p) = env.storage().persistent().get::<DataKey, Proposal>(&DataKey::Proposal(i)) {
                if !p.is_finalized && now <= p.expires_at {
                    list.push_back(i);
                }
            }
        }
        list
    }

    pub fn get_proposal_stats(env: Env, proposal_id: u64) -> ProposalStats {
        let p: Proposal = env.storage().persistent().get(&DataKey::Proposal(proposal_id)).expect("Proposal not found");
        let now = env.ledger().timestamp();
        let status = if p.is_executed {
            symbol_short!("executed")
        } else if p.is_finalized {
            symbol_short!("finalized")
        } else if now > p.expires_at {
            symbol_short!("expired")
        } else {
            symbol_short!("active")
        };

        ProposalStats {
            yes_votes: p.yes_votes,
            no_votes: p.no_votes,
            total_votes: p.yes_votes + p.no_votes,
            status,
        }
    }

    pub fn get_all_proposals(env: Env) -> Vec<u64> {
        let counter: u64 = env.storage().instance().get(&DataKey::ProposalCounter).unwrap_or(0);
        let mut list = Vec::new(&env);
        for i in 1..=counter {
            list.push_back(i);
        }
        list
    }

    pub fn get_vote_record(env: Env, proposal_id: u64, voter: Address) -> Option<VoteRecord> {
        env.storage().persistent().get(&DataKey::VoterRecord(proposal_id, voter))
    }
}
