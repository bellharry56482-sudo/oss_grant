#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol};

/// Lifecycle status codes for a grant.
/// 0 = APPLIED, 1 = FUNDED, 2 = IN_PROGRESS, 3 = COMPLETED.
#[contracttype]
#[derive(Clone)]
pub struct Grant {
    pub maintainer: Address,
    pub sponsor: Option<Address>,
    pub project_hash: BytesN<32>,
    pub total_amount: i128,
    pub released_amount: i128,
    pub milestone_count: u32,
    pub funded: bool,
    pub status: u32,
}

/// On-chain record of a single milestone deliverable.
#[contracttype]
#[derive(Clone)]
pub struct Milestone {
    pub id: u32,
    pub evidence_hash: BytesN<32>,
    pub submitted: bool,
    pub verified: bool,
    pub released: bool,
}

/// Persistent storage keys for grants and their milestones.
#[contracttype]
pub enum DataKey {
    Grant(Symbol),
    Milestone(Symbol, u32),
}

#[contract]
pub struct OssGrant;

#[contractimpl]
impl OssGrant {
    /// Maintainer applies for a new open-source grant.
    /// Records the project content hash (e.g. repo commit) and the
    /// number of milestone slots that will be tracked on-chain.
    pub fn apply(
        env: Env,
        maintainer: Address,
        grant_id: Symbol,
        project_hash: BytesN<32>,
        milestones: u32,
    ) {
        maintainer.require_auth();

        if milestones == 0 {
            panic!("at least one milestone is required");
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::Grant(grant_id.clone()))
        {
            panic!("grant_id already used");
        }

        let grant = Grant {
            maintainer,
            sponsor: None,
            project_hash,
            total_amount: 0,
            released_amount: 0,
            milestone_count: milestones,
            funded: false,
            status: 0, // APPLIED
        };

        env.storage()
            .persistent()
            .set(&DataKey::Grant(grant_id), &grant);
    }

    /// Sponsor commits a funding amount to an existing grant.
    /// The amount is recorded on-chain and locks the grant against re-funding.
    /// No real XLM is moved — this contract tracks accounting only.
    pub fn fund(env: Env, sponsor: Address, grant_id: Symbol, amount: i128) {
        sponsor.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let key = DataKey::Grant(grant_id.clone());
        let mut grant: Grant = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("grant not found"));

        if grant.funded {
            panic!("grant already funded");
        }

        grant.sponsor = Some(sponsor);
        grant.total_amount = amount;
        grant.funded = true;
        grant.status = 1; // FUNDED

        env.storage().persistent().set(&key, &grant);
    }

    /// Maintainer submits proof-of-work for a milestone by attaching
    /// an evidence hash (e.g. PR merge commit, IPFS CID).
    pub fn submit_milestone(
        env: Env,
        maintainer: Address,
        grant_id: Symbol,
        milestone_id: u32,
        evidence_hash: BytesN<32>,
    ) {
        maintainer.require_auth();

        let grant_key = DataKey::Grant(grant_id.clone());
        let mut grant: Grant = env
            .storage()
            .persistent()
            .get(&grant_key)
            .unwrap_or_else(|| panic!("grant not found"));

        if grant.maintainer != maintainer {
            panic!("only registered maintainer can submit");
        }
        if !grant.funded {
            panic!("grant not yet funded");
        }
        if milestone_id >= grant.milestone_count {
            panic!("milestone_id out of range");
        }

        let m_key = DataKey::Milestone(grant_id.clone(), milestone_id);
        if let Some(existing) = env.storage().persistent().get::<_, Milestone>(&m_key) {
            if existing.released {
                panic!("milestone already released");
            }
        }

        let milestone = Milestone {
            id: milestone_id,
            evidence_hash,
            submitted: true,
            verified: false,
            released: false,
        };
        env.storage().persistent().set(&m_key, &milestone);

        grant.status = 2; // IN_PROGRESS
        env.storage().persistent().set(&grant_key, &grant);
    }

    /// Sponsor reviews the submitted evidence and approves the milestone.
    /// Verification is the prerequisite for releasing payment.
    pub fn verify_milestone(env: Env, sponsor: Address, grant_id: Symbol, milestone_id: u32) {
        sponsor.require_auth();

        let grant: Grant = env
            .storage()
            .persistent()
            .get(&DataKey::Grant(grant_id.clone()))
            .unwrap_or_else(|| panic!("grant not found"));

        match grant.sponsor {
            Some(ref s) if s == &sponsor => (),
            _ => panic!("only the funding sponsor can verify"),
        }

        let m_key = DataKey::Milestone(grant_id, milestone_id);
        let mut milestone: Milestone = env
            .storage()
            .persistent()
            .get(&m_key)
            .unwrap_or_else(|| panic!("milestone not submitted"));

        if !milestone.submitted {
            panic!("milestone not submitted");
        }
        if milestone.verified {
            panic!("milestone already verified");
        }

        milestone.verified = true;
        env.storage().persistent().set(&m_key, &milestone);
    }

    /// Sponsor releases the per-milestone payout slot for a verified milestone.
    /// Returns the i128 amount released (= total_amount / milestone_count).
    /// No real XLM is transferred; this records the released accounting entry.
    pub fn release(env: Env, sponsor: Address, grant_id: Symbol, milestone_id: u32) -> i128 {
        sponsor.require_auth();

        let grant_key = DataKey::Grant(grant_id.clone());
        let mut grant: Grant = env
            .storage()
            .persistent()
            .get(&grant_key)
            .unwrap_or_else(|| panic!("grant not found"));

        match grant.sponsor {
            Some(ref s) if s == &sponsor => (),
            _ => panic!("only the funding sponsor can release"),
        }

        let m_key = DataKey::Milestone(grant_id.clone(), milestone_id);
        let mut milestone: Milestone = env
            .storage()
            .persistent()
            .get(&m_key)
            .unwrap_or_else(|| panic!("milestone not found"));

        if !milestone.verified {
            panic!("milestone not verified yet");
        }
        if milestone.released {
            panic!("milestone payout already released");
        }

        let per_milestone = grant.total_amount / (grant.milestone_count as i128);
        milestone.released = true;
        grant.released_amount += per_milestone;

        if grant.released_amount >= grant.total_amount {
            grant.status = 3; // COMPLETED
        }

        env.storage().persistent().set(&m_key, &milestone);
        env.storage().persistent().set(&grant_key, &grant);

        per_milestone
    }

    /// View: returns the lifecycle status code of a grant.
    /// 0 = APPLIED, 1 = FUNDED, 2 = IN_PROGRESS, 3 = COMPLETED.
    pub fn get_status(env: Env, grant_id: Symbol) -> u32 {
        let grant: Grant = env
            .storage()
            .persistent()
            .get(&DataKey::Grant(grant_id))
            .unwrap_or_else(|| panic!("grant not found"));
        grant.status
    }

    /// View: returns the cumulative amount released so far for a grant.
    pub fn get_released(env: Env, grant_id: Symbol) -> i128 {
        let grant: Grant = env
            .storage()
            .persistent()
            .get(&DataKey::Grant(grant_id))
            .unwrap_or_else(|| panic!("grant not found"));
        grant.released_amount
    }
}
