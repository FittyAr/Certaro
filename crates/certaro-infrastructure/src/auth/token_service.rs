use certaro_application::ports::auth::TokenPort;
use rand::RngCore;
use sha2::{Digest, Sha256};

pub struct Sha256TokenService;

impl Sha256TokenService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sha256TokenService {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenPort for Sha256TokenService {
    fn generate_token(&self) -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        result.iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_hashing() {
        let service = Sha256TokenService::new();
        let token = service.generate_token();
        assert_eq!(token.len(), 64);
        let hash1 = service.hash_token(&token);
        let hash2 = service.hash_token(&token);
        assert_eq!(hash1, hash2);
        assert_ne!(token, hash1);
    }
}
