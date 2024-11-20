use std::ops::Deref;

use anyhow::Result;
use jwt_simple::{claims, prelude::*};

use crate::{models::User, AppError};

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_web";

pub struct EncodingKey(Ed25519KeyPair);
pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISS).with_audience(JWT_AUD);
        Ok(self.0.sign(claims)?)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    pub fn verify(&self, token: &str) -> Result<User, AppError> {
        let opts = VerificationOptions {
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUD])),
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISS])),
            ..Default::default()
        };

        let claims = self.verify_token::<User>(token, Some(opts))?;
        Ok(claims.custom)
    }
}

impl Deref for EncodingKey {
    type Target = Ed25519KeyPair;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for DecodingKey {
    type Target = Ed25519PublicKey;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn jwt_sign_verify_should_work() -> Result<()> {
        let encoding_pem = include_str!("../../pem/private.pem");
        let decoding_pem = include_str!("../../pem/public.pem");
        let ek = EncodingKey::load(encoding_pem)?;
        let sk = DecodingKey::load(decoding_pem)?;

        let user = User::new(1, "hildxd", "hildxd@qq.com");

        let token = ek.sign(user.clone())?;
        let user2 = sk.verify(&token)?;
        assert_eq!(user, user2);

        Ok(())
    }
}
