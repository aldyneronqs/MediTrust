# MediTrust

> **On-chain laboratory result verification powered by Stellar USDC and Soroban**

---

## One-Line Description

MediTrust lets patients in Metro Manila and Central Luzon pay laboratory fees with Stellar USDC and receive a QR-linked, tamper-proof result record stored on Soroban — so hospitals, employers, and schools can verify authenticity in seconds.

---

## Problem

Patients at small diagnostic clinics in Metro Manila and Pampanga face two compounding problems:

1. **Delayed results** — clinic staff manually validate payment receipts before releasing results, causing long wait times.
2. **Document fraud** — paper and PDF laboratory results can be forged, yet third-party verifiers (employers, schools, hospitals) have no fast way to confirm authenticity.

---

## Solution

1. Patient scans a laboratory QR code and pays the fee in **Stellar USDC** directly from a mobile wallet.
2. The laboratory confirms payment and uploads the SHA-256 hash of the result document.
3. A **Soroban smart contract** stores the payment status, result hash, lab address, and patient address on-chain permanently.
4. A QR code containing the `record_id` is generated and given to the patient.
5. Any verifier (doctor, HR officer, school registrar) scans the QR, re-hashes the document, and calls `verify_record` — getting an instant **Authentic / Tampered / Not Found** response.

---

## Stellar Features Used

| Feature | How It Is Used |
|---|---|
| **Stellar USDC** | Patient pays laboratory fee; fast, near-zero-cost settlement |
| **Soroban Smart Contract** | Stores immutable payment record + document hash |
| **Trustlines** | Patient wallet establishes USDC trustline before payment |

---

## Vision and Purpose

Healthcare document fraud costs Philippine patients and institutions millions of pesos per year through re-testing, delayed hiring, and fraudulent clearances. MediTrust replaces manual paper validation with cryptographic proof — giving SME laboratories a competitive edge while protecting patients and third-party verifiers. Long term, the protocol can expand to drug/medicine batch verification and cross-border medical record sharing within ASEAN.

---

## Target Users

| Segment | Location | Why They Care |
|---|---|---|
| Small diagnostic labs | Metro Manila, Pampanga | Faster payment settlement, tamper-proof records attract patients |
| Clinic patients | NCR, Central Luzon | No more waiting; single QR proves payment and result |
| Employers / HR | Nationwide | Instant pre-employment medical verification |
| Schools / universities | Nationwide | Verified medical clearances without calling clinics |

---

## Core MVP Transaction Flow

```
Patient                  Laboratory               Soroban Contract         Verifier
   │                         │                          │                      │
   │── scan lab QR ─────────▶│                          │                      │
   │                         │                          │                      │
   │── pay USDC (Stellar) ──▶│                          │                      │
   │                         │── record_payment_and_result() ──────────────────│
   │                         │     (lab, record_id, patient,                   │
   │                         │      result_hash, amount, test_name)            │
   │                         │                          │                      │
   │◀── QR code (record_id) ─│                          │                      │
   │                         │                          │                      │
   │                         │                          │◀─ verify_record() ───│
   │                         │                          │   (record_id, hash)  │
   │                         │                          │── Authentic ────────▶│
```

Demo time: **under 2 minutes**

---

## Project Timeline

| Phase | Tasks | Duration |
|---|---|---|
| **Day 1** | Soroban contract (`lib.rs`), unit tests, Cargo.toml | 4 h |
| **Day 2** | Deploy to testnet, build Next.js / React front-end (QR payment + scan pages) | 5 h |
| **Day 3** | Integrate Freighter wallet, USDC trustline flow, QR generation | 4 h |
| **Day 4** | Verifier page, end-to-end demo recording, README polish | 3 h |

---

## Prerequisites

| Tool | Version |
|---|---|
| Rust | ≥ 1.78 (`rustup update stable`) |
| `wasm32-unknown-unknown` target | `rustup target add wasm32-unknown-unknown` |
| Soroban CLI | ≥ 22.0.0 — `cargo install --locked soroban-cli` |
| Node.js (front-end) | ≥ 20 |

---

## Build

```bash
# Clone the repo
git clone https://github.com/your-org/meditrust
cd meditrust

# Compile to Wasm
soroban contract build
# Output: target/wasm32-unknown-unknown/release/meditrust.wasm
```

---

## Test

```bash
cargo test
# Runs all 5 unit tests against the local Soroban environment
```

Expected output:

```
running 5 tests
test tests::test_happy_path_record_and_verify          ... ok
test tests::test_unregistered_lab_is_rejected          ... ok
test tests::test_storage_state_after_record            ... ok
test tests::test_tampered_document_returns_tampered    ... ok
test tests::test_duplicate_record_id_rejected          ... ok

test result: ok. 5 passed; 0 failed
```

---

## Deploy to Testnet

```bash
# 1. Configure testnet identity (run once)
soroban keys generate --global alice --network testnet
soroban keys fund alice --network testnet

# 2. Deploy the contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/meditrust.wasm \
  --source alice \
  --network testnet

# Note the returned CONTRACT_ID — you will need it for every invocation.
export CONTRACT_ID=<returned_contract_id>

# 3. Initialize the contract
soroban contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- initialize \
  --admin $(soroban keys address alice)
```

---

## Sample CLI Invocations

### Register a laboratory

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- register_lab \
  --admin $(soroban keys address alice) \
  --lab GABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234
```

### Record payment and result (called by the laboratory)

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source lab-identity \
  --network testnet \
  -- record_payment_and_result \
  --lab    GABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234 \
  --record_id  0101010101010101010101010101010101010101010101010101010101010101 \
  --patient    GPATIENTA1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF12 \
  --result_hash ABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABAB \
  --amount_paid 500000000 \
  --test_name "CBC + Urinalysis"
```

### Verify a record (called by employer / school)

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source verifier-identity \
  --network testnet \
  -- verify_record \
  --record_id   0101010101010101010101010101010101010101010101010101010101010101 \
  --document_hash ABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABAB
# Returns: "Authentic"
```

### Fetch full record (patient portal)

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source alice \
  --network testnet \
  -- get_record \
  --record_id 0101010101010101010101010101010101010101010101010101010101010101
```

---

## Repository Structure

```
meditrust/
├── Cargo.toml          # Package manifest and Soroban SDK dependency
├── README.md           # This file
└── src/
    ├── lib.rs          # Soroban smart contract (all on-chain logic)
    └── test.rs         # 5 unit tests
```

---

## Why This Wins

- **Real users, real money**: USDC payments replace manual receipt validation at actual Philippine diagnostic clinics.
- **Stellar-native**: Fast finality (~5 s) and sub-cent fees make micro-payments per lab result economically viable; Soroban stores the immutable hash that traditional databases cannot provide.
- **Demo-able**: The full flow — pay → store → scan → verify — runs in under 2 minutes with Freighter on testnet.
- **Local economy impact**: Addresses healthcare document fraud in a region (NCR + Central Luzon) with millions of annual lab tests.

---

## Deployment Reference

- Stellar Testnet Horizon: `https://horizon-testnet.stellar.org`
- Stellar Testnet RPC: `https://soroban-testnet.stellar.org`
- Freighter Wallet: https://www.freighter.app
- Soroban Docs: https://developers.stellar.org/docs/build/smart-contracts

---

## License

MIT © 2026 MediTrust Contributors