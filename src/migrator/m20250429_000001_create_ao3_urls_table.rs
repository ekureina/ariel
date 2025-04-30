use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20250429_000001_create_ao3_urls_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Ao3Urls::Table)
                    .col(
                        ColumnDef::new(Ao3Urls::Id)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Ao3Urls::Url)
                            .string_len(512)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Ao3Urls::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Ao3Urls {
    Table,
    Id,
    Url,
}
