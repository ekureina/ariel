use sea_orm_migration::prelude::*;

mod m_20250502_000001_initial;
mod m_20250506_000001_add_fandoms;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m_20250502_000001_initial::Migration),
            Box::new(m_20250506_000001_add_fandoms::Migration),
        ]
    }
}
