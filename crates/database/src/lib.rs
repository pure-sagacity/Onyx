use diesel::{SqliteConnection, prelude::*};

pub mod models;
pub mod schema;

use crate::models::Secret;

// Select Type
pub enum SecretField {
    Name,
    Value,
    Nonce,
    ProjectId,
    Environment,
}

pub struct Database {
    pub url: String,
}

use self::schema::secrets::dsl::*;

impl Database {
    pub fn establish_connection(url: &str) -> Result<SqliteConnection, Box<dyn std::error::Error>> {
        let conn = SqliteConnection::establish(url)?;
        Ok(conn)
    }

    pub fn get_secrets(
        &self,
        other_project_id: &str,
    ) -> Result<Vec<models::Secret>, Box<dyn std::error::Error>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let project_secrets = secrets
            .filter(project_id.eq(other_project_id))
            .select(Secret::as_select())
            .load(conn)?;

        Ok(project_secrets)
    }

    pub fn add_secret(
        &self,
        new_name: &str,
        new_encrypted_value: &str,
        new_nonce: &str,
        new_project_id: &str,
        new_environment: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let new_secret = Secret {
            id: None,
            project_id: new_project_id.to_string(),
            name: new_name.to_string(),
            environment: new_environment.to_string(),
            value: new_encrypted_value.to_string(),
            nonce: new_nonce.to_string(),
            created_at: None,
        };

        diesel::insert_into(secrets)
            .values(new_secret)
            .execute(conn)?;

        Ok(())
    }

    pub fn secret_exists(&self, secret_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let count = secrets
            .filter(name.eq(secret_name.to_string()))
            .count()
            .execute(conn)?;

        Ok(count > 0)
    }

    fn get_secret_by(
        &self,
        method: SecretField,
        value_to_match: &str,
    ) -> Result<Option<Secret>, Box<dyn std::error::Error>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let secret = match method {
            SecretField::Name => secrets
                .filter(name.eq(value_to_match.to_string()))
                .first::<Secret>(conn)
                .optional()?,
            SecretField::Value => secrets
                .filter(value.eq(value_to_match.to_string()))
                .first::<Secret>(conn)
                .optional()?,
            SecretField::Nonce => secrets
                .filter(nonce.eq(value_to_match.to_string()))
                .first::<Secret>(conn)
                .optional()?,
            SecretField::ProjectId => secrets
                .filter(project_id.eq(value_to_match.to_string()))
                .first::<Secret>(conn)
                .optional()?,
            SecretField::Environment => secrets
                .filter(environment.eq(value_to_match.to_string()))
                .first::<Secret>(conn)
                .optional()?,
        };

        Ok(secret)
    }

    fn get_secret_id(&self, secret_id: i32) -> Result<Option<Secret>, Box<dyn std::error::Error>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let secret = secrets
            .filter(id.eq(secret_id))
            .first::<Secret>(conn)
            .optional()?;

        Ok(secret)
    }

    pub fn set_secret(
        &self,
        secret_id: i32,
        new_encrypted_value: &str,
        new_nonce: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        diesel::update(secrets.filter(id.eq(secret_id)))
            .set((value.eq(new_encrypted_value), nonce.eq(new_nonce)))
            .execute(conn)?;

        Ok(())
    }

    pub fn delete_secret(&self, secret_id: i32) -> Result<(), Box<dyn std::error::Error>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        diesel::delete(secrets.filter(id.eq(secret_id))).execute(conn)?;

        Ok(())
    }
}
