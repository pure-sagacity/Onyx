use diesel::{SqliteConnection, prelude::*};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

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

// This macro finds your global `migrations/` directory at compile time
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

type DBResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

impl Database {
    /// This function establishes a connection to the SQLite database and runs any pending migrations.
    pub fn establish_connection(url: &str) -> DBResult<SqliteConnection> {
        let mut conn = SqliteConnection::establish(url)?;

        let _ = &conn.run_pending_migrations(MIGRATIONS)?;

        Ok(conn)
    }

    /// This function creates a new project based on name and environment.
    /// It additionally takes a description.
    /// It checks if a project with the same name and environment already exists before creating a new one.
    pub fn create_project(
        &self,
        new_name: String,
        new_environment: String,
        description: Option<String>,
    ) -> DBResult<models::Project> {
        let conn = &mut Database::establish_connection(&self.url)?;

        // Check if a project with the same name and environment already exists
        let existing_project = secrets
            .filter(name.eq(new_name.to_string()))
            .filter(environment.eq(new_environment.to_string()))
            .first::<Secret>(conn)
            .optional()?;

        if existing_project.is_some() {
            return Err(format!(
                "A project with the name '{}' and environment '{}' already exists.",
                new_name, new_environment
            )
            .into());
        }

        let new_project = models::Project {
            id: uuid::Uuid::new_v4().to_string(),
            name: new_name.to_string(),
            description,
            created_at: None,
        };

        diesel::insert_into(crate::schema::projects::table)
            .values(&new_project)
            .execute(conn)?;

        Ok(new_project)
    }

    /// This function will get a project's metadata based on the project ID.
    pub fn get_project_metadata(
        &self,
        project_id_to_match: &str,
    ) -> DBResult<Option<(String, String)>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let secret = secrets
            .filter(project_id.eq(project_id_to_match.to_string()))
            .select((project_id, environment))
            .first::<(String, String)>(conn)
            .optional()?;

        Ok(secret)
    }

    /// This function gets all secrets from the database. This is primarily used for debugging and testing purposes.
    pub fn get_all(&self) -> DBResult<Vec<models::Secret>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let all_secrets = secrets.load(conn)?;

        Ok(all_secrets)
    }

    /// This function gets all secrets for a specific project ID.
    pub fn get_secrets(&self, other_project_id: &str) -> DBResult<Vec<models::Secret>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let project_secrets = secrets
            .filter(project_id.eq(other_project_id))
            .select(Secret::as_select())
            .load(conn)?;

        Ok(project_secrets)
    }

    /// This function adds a new secret to the database.
    pub fn add_secret(
        &self,
        new_name: String,
        new_encrypted_value: String,
        new_nonce: String,
        new_project_id: String,
        new_environment: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    /// This function checks if a secret with the given name already exists in the database.
    pub fn secret_exists(&self, secret_name: &str) -> DBResult<bool> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let count = secrets
            .filter(name.eq(secret_name.to_string()))
            .count()
            .execute(conn)?;

        Ok(count > 0)
    }

    /// This function retrieves a secret from the database based on the specified field and value.
    pub fn get_secret_by(
        &self,
        method: SecretField,
        project_id_to_match: &str,
        value_to_match: &str,
    ) -> DBResult<Option<Secret>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let secret = match method {
            SecretField::Name => secrets
                .filter(name.eq(value_to_match.to_string()))
                .filter(project_id.eq(project_id_to_match))
                .first::<Secret>(conn)
                .optional()?,
            SecretField::Value => secrets
                .filter(value.eq(value_to_match.to_string()))
                .filter(project_id.eq(project_id_to_match))
                .first::<Secret>(conn)
                .optional()?,
            SecretField::Nonce => secrets
                .filter(nonce.eq(value_to_match.to_string()))
                .filter(project_id.eq(project_id_to_match))
                .first::<Secret>(conn)
                .optional()?,
            SecretField::ProjectId => secrets
                .filter(project_id.eq(value_to_match.to_string()))
                .filter(project_id.eq(project_id_to_match))
                .first::<Secret>(conn)
                .optional()?,
            SecretField::Environment => secrets
                .filter(environment.eq(value_to_match.to_string()))
                .filter(project_id.eq(project_id_to_match))
                .first::<Secret>(conn)
                .optional()?,
        };

        Ok(secret)
    }

    /// This function retrieves a secret from the database based on its ID.
    pub fn get_secret_id(&self, secret_id: i32) -> DBResult<Option<Secret>> {
        let conn = &mut Database::establish_connection(&self.url)?;

        let secret = secrets
            .filter(id.eq(secret_id))
            .first::<Secret>(conn)
            .optional()?;

        Ok(secret)
    }

    /// This function updates an existing secret in the database with a new encrypted value and nonce.
    pub fn set_secret(
        &self,
        secret_id: i32,
        new_encrypted_value: String,
        new_nonce: String,
    ) -> DBResult<()> {
        let conn = &mut Database::establish_connection(&self.url)?;

        diesel::update(secrets.filter(id.eq(secret_id)))
            .set((value.eq(new_encrypted_value), nonce.eq(new_nonce)))
            .execute(conn)?;

        Ok(())
    }

    /// This function deletes a secret from the database based on its ID.
    pub fn delete_secret(&self, secret_id: i32) -> DBResult<()> {
        let conn = &mut Database::establish_connection(&self.url)?;

        diesel::delete(secrets.filter(id.eq(secret_id))).execute(conn)?;

        Ok(())
    }
}
