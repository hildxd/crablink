use std::mem;

use crate::AppError;

use super::{CreateUser, User, VerifyUser};
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::PgPool;

impl User {
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        sqlx::query_as("SELECT id, fullname, email FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn create(user: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
        let password_hash = hash_password(&user.password)?;
        let user = sqlx::query_as(
            r#"
            INSERT INTO users (email, fullname, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, fullname, email, created_at
            "#,
        )
        .bind(&user.email)
        .bind(&user.fullname)
        .bind(&password_hash)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    pub async fn verify(dto: &VerifyUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = Self::find_by_email(&dto.email, pool).await?;
        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid = verify_password(&dto.password, &password_hash.unwrap_or_default())?;
                match is_valid {
                    true => Ok(Some(user)),
                    false => Ok(None),
                }
            }
            None => Ok(None),
        }
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    let argon2 = Argon2::default();
    let is_valid = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use crate::test::create_test_pool;

    use super::*;

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "password";
        let hash = hash_password(password)?;
        assert_eq!(hash.len(), 97);
        assert!(verify_password(password, &hash)?);
        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let db = create_test_pool().await?;
        let user = CreateUser {
            email: "a@b.com".to_string(),
            fullname: "a b".to_string(),
            password: "password".to_string(),
        };
        let ret = User::create(&user, &db).await?;
        assert_eq!(user.email, ret.email);
        assert_eq!(user.fullname, ret.fullname);
        assert!(ret.id > 0);
        Ok(())
    }
}
