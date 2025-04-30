use sea_orm_migration::prelude::*;

use super::m20250429_000001_create_ao3_urls_table::Ao3Urls;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20250429_000002_create_songs_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Songs::Table)
                    .col(
                        ColumnDef::new(Songs::Name)
                            .string_len(50)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Songs::Aliases).string_len(512))
                    .col(
                        ColumnDef::new(Songs::Ao3Id)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-songs-ao3_id")
                            .from(Songs::Table, Songs::Ao3Id)
                            .to(Ao3Urls::Table, Ao3Urls::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Songs::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Songs {
    Table,
    Name,
    Aliases,
    Ao3Id,
}
