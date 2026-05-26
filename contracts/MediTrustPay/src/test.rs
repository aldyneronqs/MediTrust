#[cfg(test)]
mod tests {
    use soroban_sdk::{
        testutils::Address as _,
        Address, BytesN, Env, String,
    };

    use crate::{MediTrustContract, MediTrustContractClient, PaymentStatus, VerificationResult};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Deploy the contract, initialize it with `admin`, register `lab`,
    /// and return (client, admin, lab, patient).
    fn setup() -> (
        MediTrustContractClient<'static>,
        Address,
        Address,
        Address,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, MediTrustContract);
        let client = MediTrustContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let lab = Address::generate(&env);
        let patient = Address::generate(&env);

        client.initialize(&admin);
        client.register_lab(&admin, &lab);

        (client, admin, lab, patient)
    }

    /// Create a deterministic 32-byte array filled with `byte`.
    fn make_bytes32(env: &Env, byte: u8) -> BytesN<32> {
        BytesN::from_array(env, &[byte; 32])
    }

    // -----------------------------------------------------------------------
    // Test 1 – Happy path: full MVP transaction executes end-to-end
    // -----------------------------------------------------------------------
    #[test]
    fn test_happy_path_record_and_verify() {
        let (client, _admin, lab, patient) = setup();
        let env = client.env.clone();

        let record_id = make_bytes32(&env, 0x01);
        let result_hash = make_bytes32(&env, 0xAB);
        let amount_paid: i128 = 50_000_0000; // 50 USDC (7-decimal)
        let test_name = String::from_str(&env, "CBC + Urinalysis");

        // Laboratory records payment and result
        client.record_payment_and_result(
            &lab,
            &record_id,
            &patient,
            &result_hash,
            &amount_paid,
            &test_name,
        );

        // Verifier (employer/school) scans QR and submits the same document hash
        let result = client.verify_record(&record_id, &result_hash);
        assert_eq!(result, VerificationResult::Authentic);
    }

    // -----------------------------------------------------------------------
    // Test 2 – Edge case: unregistered lab cannot create a record
    // -----------------------------------------------------------------------
    #[test]
    #[should_panic(expected = "unauthorized: lab is not registered")]
    fn test_unregistered_lab_is_rejected() {
        let (client, _admin, _lab, patient) = setup();
        let env = client.env.clone();

        // A completely new address that was never registered
        let rogue_lab = Address::generate(&env);
        let record_id = make_bytes32(&env, 0x02);
        let result_hash = make_bytes32(&env, 0xCC);

        client.record_payment_and_result(
            &rogue_lab,
            &record_id,
            &patient,
            &result_hash,
            &50_000_0000_i128,
            &String::from_str(&env, "X-Ray"),
        );
    }

    // -----------------------------------------------------------------------
    // Test 3 – State verification: storage reflects correct values post-write
    // -----------------------------------------------------------------------
    #[test]
    fn test_storage_state_after_record() {
        let (client, _admin, lab, patient) = setup();
        let env = client.env.clone();

        let record_id = make_bytes32(&env, 0x03);
        let result_hash = make_bytes32(&env, 0xDE);
        let amount_paid: i128 = 25_000_0000; // 25 USDC
        let test_name = String::from_str(&env, "Blood Chemistry");

        client.record_payment_and_result(
            &lab,
            &record_id,
            &patient,
            &result_hash,
            &amount_paid,
            &test_name,
        );

        let stored = client.get_record(&record_id).unwrap();

        assert_eq!(stored.lab, lab);
        assert_eq!(stored.patient, patient);
        assert_eq!(stored.result_hash, result_hash);
        assert_eq!(stored.amount_paid, amount_paid);
        assert_eq!(stored.payment_status, PaymentStatus::Confirmed);
    }

    // -----------------------------------------------------------------------
    // Test 4 – Tampered document hash returns Tampered result
    // -----------------------------------------------------------------------
    #[test]
    fn test_tampered_document_returns_tampered() {
        let (client, _admin, lab, patient) = setup();
        let env = client.env.clone();

        let record_id = make_bytes32(&env, 0x04);
        let original_hash = make_bytes32(&env, 0x11);
        let tampered_hash = make_bytes32(&env, 0xFF); // different bytes

        client.record_payment_and_result(
            &lab,
            &record_id,
            &patient,
            &original_hash,
            &100_000_0000_i128,
            &String::from_str(&env, "Hepatitis Panel"),
        );

        // Verifier submits a hash that does NOT match the stored one
        let result = client.verify_record(&record_id, &tampered_hash);
        assert_eq!(result, VerificationResult::Tampered);
    }

    // -----------------------------------------------------------------------
    // Test 5 – Duplicate record_id is rejected
    // -----------------------------------------------------------------------
    #[test]
    #[should_panic(expected = "record already exists")]
    fn test_duplicate_record_id_rejected() {
        let (client, _admin, lab, patient) = setup();
        let env = client.env.clone();

        let record_id = make_bytes32(&env, 0x05);
        let result_hash = make_bytes32(&env, 0x22);

        // First submission – should succeed
        client.record_payment_and_result(
            &lab,
            &record_id,
            &patient,
            &result_hash,
            &75_000_0000_i128,
            &String::from_str(&env, "COVID-19 PCR"),
        );

        // Second submission with the same record_id – must panic
        client.record_payment_and_result(
            &lab,
            &record_id,
            &patient,
            &result_hash,
            &75_000_0000_i128,
            &String::from_str(&env, "COVID-19 PCR"),
        );
    }
}