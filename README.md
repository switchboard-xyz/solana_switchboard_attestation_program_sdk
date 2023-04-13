# solana_switchboard_attestation_program_sdk

This is the solana program module to be used along with the Switchboard Attestation Client

Use this module to verify attestions match a certain enclave measurement and the associated signer.

```
let sgx_quote_account_info = <QUOTE_ACCOUNT_INFO>;
let mr_enclave = "IN0WD0ApAbKcAFBHK6xYS5QyToq7oJdnIVRJyq6brdM=".to_string();
let quote_data = sgx_quote_account_info.try_borrow_data().unwrap();
// TODO: check discriminator
let quote = bytemuck::from_bytes::<QuoteAccountData>(data[8..mem::size_of::<QuoteAccountData>() + 8].as_ref());
quote.check_measurement(mr_enclave, sgx_quote_account_info.key, Clock::get()).unwrap();
msg!("Attestation for key {:?} verified. Key is bound to {}, sgx_quote_account_info.key, mr_enclave);
```
