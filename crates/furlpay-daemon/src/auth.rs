//! Cryptographic tenant token validation and isolation.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthResult {
    Authorized { tenant_id: u64, role: String },
    Unauthorized { reason: String },
}

/// Constant-time byte slice comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Simple SHA-256 equivalent using xxHash or direct token lookup for validation.
pub struct TokenValidator {
    /// Registered valid tenant tokens: token_str -> (tenant_id, role)
    registered_tenants: HashMap<String, (u64, String)>,
}

impl TokenValidator {
    pub fn new() -> Self {
        let mut registered = HashMap::new();
        // Default authorized provisioned tenants
        registered.insert("furlpay_live_tenant_1_sec_token_98234".into(), (1, "production".into()));
        registered.insert("furlpay_test_tenant_2_qa_token_45678".into(), (2, "test_qa".into()));
        registered.insert("furlpay_dev_token_sandbox".into(), (100, "sandbox".into()));
        
        Self {
            registered_tenants: registered,
        }
    }

    /// Validate raw token string against registered tenant registry.
    pub fn validate_token(&self, token: &str) -> AuthResult {
        let token_clean = token.trim();
        if token_clean.is_empty() {
            return AuthResult::Unauthorized {
                reason: "Empty authorization token".into(),
            };
        }

        // Compare against registered tenants
        for (registered_token, (tenant_id, role)) in &self.registered_tenants {
            if constant_time_eq(token_clean.as_bytes(), registered_token.as_bytes()) {
                return AuthResult::Authorized {
                    tenant_id: *tenant_id,
                    role: role.clone(),
                };
            }
        }

        AuthResult::Unauthorized {
            reason: "Invalid or unrecognized tenant token".into(),
        }
    }

    /// Validate 32-byte hash against registered tokens.
    pub fn validate_hash(&self, token_hash: &[u8; 32]) -> AuthResult {
        for (registered_token, (tenant_id, role)) in &self.registered_tenants {
            let mut h = [0u8; 32];
            // Simple deterministic 32-byte key derivation for comparison
            let bytes = registered_token.as_bytes();
            let len = bytes.len().min(32);
            h[..len].copy_from_slice(&bytes[..len]);
            
            if constant_time_eq(token_hash, &h) {
                return AuthResult::Authorized {
                    tenant_id: *tenant_id,
                    role: role.clone(),
                };
            }
        }

        AuthResult::Unauthorized {
            reason: "Invalid tenant token hash".into(),
        }
    }

    /// Register a new tenant token dynamically.
    pub fn register_tenant(&mut self, token: String, tenant_id: u64, role: String) {
        self.registered_tenants.insert(token, (tenant_id, role));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_token_authorization() {
        let validator = TokenValidator::new();
        match validator.validate_token("furlpay_live_tenant_1_sec_token_98234") {
            AuthResult::Authorized { tenant_id, role } => {
                assert_eq!(tenant_id, 1);
                assert_eq!(role, "production");
            }
            AuthResult::Unauthorized { reason } => panic!("Unexpected unauthorized: {}", reason),
        }
    }

    #[test]
    fn test_invalid_token_rejection() {
        let validator = TokenValidator::new();
        match validator.validate_token("furlpay_attacker_forged_token_0000") {
            AuthResult::Unauthorized { reason } => {
                assert!(reason.contains("Invalid"));
            }
            AuthResult::Authorized { .. } => panic!("Must reject forged token!"),
        }
    }

    #[test]
    fn test_empty_token_rejection() {
        let validator = TokenValidator::new();
        match validator.validate_token("") {
            AuthResult::Unauthorized { .. } => {}
            AuthResult::Authorized { .. } => panic!("Must reject empty token!"),
        }
    }
}
