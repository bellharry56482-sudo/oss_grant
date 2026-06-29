# oss_grant

## Project Title
oss_grant — On-chain Open-Source Grant Tracker on Stellar Soroban

## Project Description
Open-source maintainers often struggle to receive transparent, milestone-based funding from sponsors, while sponsors struggle to verify that the work they paid for was actually delivered. `oss_grant` is a Soroban smart contract that records the entire grant lifecycle on Stellar — application, funding commitment, milestone evidence submission, sponsor verification, and per-milestone payout release — so both sides can audit every step on-chain.

## Project Vision
Our long-term vision is to make open-source funding **provably accountable** and globally accessible. We want any maintainer in the world — from a student building a Stellar SDK plugin to a core Linux contributor — to be able to apply for a grant, deliver milestones with hashed evidence (commit hashes, IPFS CIDs, signed builds), and receive sponsor-released funds without intermediaries. By anchoring the workflow on Stellar's fast, low-fee network, `oss_grant` aims to become the default settlement layer for OSS bounties, foundation grants, and DAO-funded development.

## Key Features
- **Maintainer-driven applications** — A maintainer registers a `grant_id`, a `project_hash` (e.g. repo commit), and how many milestones the grant will cover via `apply(...)`.
- **Sponsor-locked funding** — A sponsor calls `fund(...)` to bind themselves to the grant, lock in the total amount, and become the only address allowed to verify and release milestones.
- **Hash-anchored milestone submissions** — Maintainers attach a 32-byte `evidence_hash` per milestone via `submit_milestone(...)`, creating a tamper-proof on-chain record of what was delivered.
- **Two-step verify + release flow** — `verify_milestone(...)` separates sponsor approval from payout, and `release(...)` then unlocks an equal share of the grant (`total_amount / milestone_count`) and updates the released accounting.
- **Auditable status view** — Anyone can call `get_status(...)` / `get_released(...)` to inspect lifecycle state (APPLIED → FUNDED → IN_PROGRESS → COMPLETED) without trusting an off-chain dashboard.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** work dApp — see `contracts/oss_grant/src/lib.rs` for the full oss_grant business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CBNM6QCV2WZVGNKZAQTM7Q5UPHYCHMXCVBBLIILCFG6NBUP2XFLSQMFT`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/7b1c5ae3f39bd9e12fc40f38779a016bb6410b336a087aec5ec4a5043a9727b8`

## Future Scope
- **Real asset settlement** — Integrate the Stellar Asset Contract (SAC) so `fund` actually escrows USDC/XLM into the contract and `release` performs the on-chain transfer to the maintainer.
- **Multi-sponsor / quadratic funding pools** — Allow several sponsors to co-fund the same `grant_id`, with weighted matching pools inspired by Gitcoin's quadratic funding.
- **Independent reviewer role** — Add a neutral reviewer Address that can dispute or co-sign milestone verification, reducing sponsor unilateral power.
- **Vesting & slashing** — Time-locked milestone releases, plus slashing rules that return funds to the sponsor if deadlines lapse.
- **Frontend dApp + Freighter integration** — A Next.js dashboard where maintainers track applications, sponsors approve milestones with one click, and the public browses funded OSS projects.
- **GitHub oracle attestations** — A signed off-chain oracle that automatically posts the latest commit hash of a project as the `evidence_hash`, removing manual submission friction.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `oss_grant` (work)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
