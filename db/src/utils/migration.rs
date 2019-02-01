use diesel::{migration::RunMigrationsError, PgConnection};
use diesel_migrations::MigrationConnection;

#[derive(EmbedFileList)]
#[embed_dir = "./migrations"]
struct MigrationDirs;

pub fn has_pending_migrations(conn: &PgConnection) -> Result<bool, RunMigrationsError> {
    let migration_dirs = MigrationDirs::DIR_NAMES;

    let already_run = conn.previously_run_migration_versions()?;

    let migration_versions = migration_dirs
        .iter()
        .map(|d| d.split("_").collect::<Vec<&str>>()[0])
        .collect::<Vec<&str>>();

    let has_pending_migrations = migration_versions
        .iter()
        .any(|v| !already_run.contains(&v.to_string()));

    Ok(has_pending_migrations)
}
