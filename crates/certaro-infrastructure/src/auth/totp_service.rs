use certaro_application::ports::auth::TotpPort;
use totp_rs::{Algorithm, Secret, TOTP};

pub struct TotpService;

impl TotpService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TotpService {
    fn default() -> Self {
        Self::new()
    }
}

impl TotpPort for TotpService {
    fn generate_secret(&self) -> String {
        Secret::generate_secret().to_encoded().to_string()
    }

    fn verify_code(&self, secret: &str, code: &str) -> bool {
        let code = code.trim();
        if code.len() != 6 {
            return false;
        }
        if let Ok(secret_bytes) = Secret::Encoded(secret.to_string()).to_bytes() {
            if let Ok(totp) = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret_bytes,
            ) {
                return totp.check_current(code).unwrap_or(false);
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_generation_and_verification() {
        let service = TotpService::new();
        let secret = service.generate_secret();
        assert!(!secret.is_empty());

        let secret_bytes = Secret::Encoded(secret.clone()).to_bytes().unwrap();
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
        )
        .unwrap();

        let current_code = totp.generate_current().unwrap();
        assert!(service.verify_code(&secret, &current_code));
        assert!(!service.verify_code(&secret, "000000"));
    }
}
