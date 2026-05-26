#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, BytesN, Env, String,
};

// ---------------------------------------------------------------------------
// Storage key enums
// ---------------------------------------------------------------------------

/// Keys used in persistent + instance storage
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Lab record indexed by a unique record ID (BytesN<32>)
    LabRecord(BytesN<32>),
    /// Whether a given address is a registered laboratory
    RegisteredLab(Address),
    /// Admin / contract owner
    Admin,
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Status of the payment associated with a lab record
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum PaymentStatus {
    Pending,
    Confirmed,
}

/// Status of the verification when someone scans the QR
#[contracttype]
#[derive(Clone, PartialEq, Debug)]
pub enum VerificationResult {
    Authentic,
    Tampered,
    NotFound,
}

/// Core record stored on-chain for every laboratory result
#[contracttype]
#[derive(Clone, Debug)]
pub struct LabRecord {
    /// Address of the laboratory that created this record
    pub lab: Address,
    /// Address of the patient who paid
    pub patient: Address,
    /// Keccak / SHA-256 hash of the result document (computed off-chain, stored here)
    pub result_hash: BytesN<32>,
    /// Amount paid in USDC stroops (7-decimal USDC: 1 USDC = 1_000_000_0)
    pub amount_paid: i128,
    /// Human-readable test name, e.g. "CBC + Urinalysis"
    pub test_name: String,
    /// Unix timestamp of payment confirmation
    pub timestamp: u64,
    /// Payment status
    pub payment_status: PaymentStatus,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct MediTrustContract;

#[contractimpl]
impl MediTrustContract {
    // -----------------------------------------------------------------------
    // Admin / Setup
    // -----------------------------------------------------------------------

    /// Initialize the contract. Must be called once by the deployer.
    /// `admin` becomes the only address that can register laboratories.
    pub fn initialize(env: Env, admin: Address) {
        // Prevent re-initialization
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Register a laboratory so it is allowed to create lab records.
    /// Only the admin can call this.
    pub fn register_lab(env: Env, admin: Address, lab: Address) {
        admin.require_auth();

        // Verify caller is the stored admin
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        if admin != stored_admin {
            panic!("unauthorized: caller is not admin");
        }

        env.storage()
            .persistent()
            .set(&DataKey::RegisteredLab(lab), &true);
    }

    // -----------------------------------------------------------------------
    // Core MVP Flow
    // -----------------------------------------------------------------------

    /// Step 1 + 2 + 3 combined (MVP demo step):
    /// Called by the laboratory after the patient's USDC payment is confirmed.
    ///
    /// The Stellar USDC transfer itself happens on the Stellar network (the
    /// dApp front-end performs the payment and records the tx hash off-chain).
    /// This function records the *outcome* of that payment and the result hash
    /// on Soroban, creating the trustless on-chain record.
    ///
    /// `record_id`   – unique 32-byte ID for this record (e.g. UUID v4 bytes)
    /// `patient`     – patient's Stellar address
    /// `result_hash` – SHA-256 of the PDF / result document
    /// `amount_paid` – USDC amount in stroops
    /// `test_name`   – human-readable test description
    pub fn record_payment_and_result(
        env: Env,
        lab: Address,
        record_id: BytesN<32>,
        patient: Address,
        result_hash: BytesN<32>,
        amount_paid: i128,
        test_name: String,
    ) {
        // Require the laboratory to sign this transaction
        lab.require_auth();

        // Only registered labs may create records
        let is_registered: bool = env
            .storage()
            .persistent()
            .get(&DataKey::RegisteredLab(lab.clone()))
            .unwrap_or(false);
        if !is_registered {
            panic!("unauthorized: lab is not registered");
        }

        // Prevent duplicate records with the same ID
        if env
            .storage()
            .persistent()
            .has(&DataKey::LabRecord(record_id.clone()))
        {
            panic!("record already exists");
        }

        // Validate payment is positive
        if amount_paid <= 0 {
            panic!("amount_paid must be positive");
        }

        let record = LabRecord {
            lab,
            patient,
            result_hash,
            amount_paid,
            test_name,
            timestamp: env.ledger().timestamp(),
            payment_status: PaymentStatus::Confirmed,
        };

        // Persist indefinitely (persistent storage survives archival cycles)
        env.storage()
            .persistent()
            .set(&DataKey::LabRecord(record_id), &record);
    }

    // -----------------------------------------------------------------------
    // Verification (QR scan)
    // -----------------------------------------------------------------------

    /// Called when a doctor, employer, or school scans the patient's QR code.
    ///
    /// `record_id`        – embedded in the QR code
    /// `document_hash`    – SHA-256 the verifier computes from the document file
    ///
    /// Returns `VerificationResult::Authentic` if the hashes match and payment
    /// is confirmed, `Tampered` if hashes differ, or `NotFound` if the record
    /// does not exist.
    pub fn verify_record(
        env: Env,
        record_id: BytesN<32>,
        document_hash: BytesN<32>,
    ) -> VerificationResult {
        let maybe_record: Option<LabRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::LabRecord(record_id));

        match maybe_record {
            None => VerificationResult::NotFound,
            Some(record) => {
                // Payment must be confirmed
                if record.payment_status != PaymentStatus::Confirmed {
                    return VerificationResult::Tampered;
                }
                // Document hash must match what the lab submitted
                if record.result_hash == document_hash {
                    VerificationResult::Authentic
                } else {
                    VerificationResult::Tampered
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Read helpers
    // -----------------------------------------------------------------------

    /// Fetch the full record for a given record ID (used by the patient portal).
    pub fn get_record(env: Env, record_id: BytesN<32>) -> Option<LabRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::LabRecord(record_id))
    }

    /// Check whether an address is a registered laboratory.
    pub fn is_registered_lab(env: Env, lab: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::RegisteredLab(lab))
            .unwrap_or(false)
    }
}