#![allow(dead_code)]
use anchor_lang::prelude::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as b64};
use sgx_quote::Quote;
use sha2::{Digest, Sha256};
use solana_program::pubkey;
use std::result::Result;

const PID: Pubkey = pubkey!("Hxfwq7cxss4Ef9iDvaLb617dhageGyNWbDLLrg2sdQgT");

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum VerificationStatus {
    VerificationPending = 1 << 0,
    VerificationFailure = 1 << 1,
    VerificationSuccess = 1 << 2,
    VerificationOverride = 1 << 3,
}
#[repr(packed)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct QuoteAccountData {
    pub node: Pubkey,
    pub node_authority: Pubkey,
    pub queue: Pubkey,
    pub quote_buffer: [u8; 8192],
    pub quote_len: u32,
    pub is_ready: bool,
    pub verification_status: u8,
    pub verification_timestamp: i64,
    pub valid_until: i64,
    pub _ebuf: [u8; 1024],
}
impl QuoteAccountData {
    pub fn parsed(&self) -> Result<Quote, SwitchboardError> {
        if !self.is_ready {
            return Err(SwitchboardError::InvalidQuoteError.into());
        }
        let quote = Quote::parse(&self.quote_buffer[..self.quote_len as usize])
            .map_err(|_| SwitchboardError::InvalidQuoteError)?;
        Ok(quote)
    }

    pub fn is_valid(&self, clock: &Clock) -> bool {
        if !self.is_ready {
            return false;
        }
        if self.verification_status == VerificationStatus::VerificationPending as u8 {
            return false;
        }
        if self.verification_status == VerificationStatus::VerificationFailure as u8 {
            return false;
        }
        if self.valid_until < clock.unix_timestamp {
            return false;
        }
        if self.verification_status == VerificationStatus::VerificationOverride as u8 {
            return true;
        }
        return true;
    }

    pub fn check_measurement(&self, expected: &String, enclave_key: &Pubkey, clock: &Clock) -> Result<(), SwitchboardError> {
        let expected = b64.decode(expected).map_err(|_| SwitchboardError::InvalidMeasurement)?;
        let expected: [u8; 32] = expected.try_into().map_err(|_| SwitchboardError::InvalidMeasurement)?;
        if !self.is_valid(clock) {
            return Err(SwitchboardError::InvalidQuote);
        }
        let quote = self.parsed()?;
        if quote.isv_report.mrenclave != expected {
            return Err(SwitchboardError::InvalidMrEnclave);
        }
        let mut hasher = Sha256::new();
        hasher.update(&enclave_key.to_bytes());
        let hash_result = &hasher.finalize()[..32];
        if quote.isv_report.report_data[..32] != *hash_result {
            return Err(SwitchboardError::InvalidEnclaveKey);
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq)]
pub enum SwitchboardError {
    InvalidQuoteError,
    InvalidMeasurement,
    InvalidMrEnclave,
    InvalidQuote,
    InvalidEnclaveKey,
}
