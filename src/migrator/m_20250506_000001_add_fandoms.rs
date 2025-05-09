use sea_orm_migration::prelude::*;

use super::m_20250502_000001_initial::Fics;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m_20250506_000001_add_fandoms"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Fandoms::Table)
                    .col(
                        ColumnDef::new(Fandoms::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Fandoms::Name).string_len(30).not_null())
                    .to_owned(),
            )
            .await?;

        self.install_default_fandoms(manager).await?;

        // FicsFandoms Linking Table
        manager
            .create_table(
                Table::create()
                    .table(FicsFandoms::Table)
                    .col(ColumnDef::new(FicsFandoms::FicId).integer().not_null())
                    .col(ColumnDef::new(FicsFandoms::FandomId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FicsFandoms_FicId_Fics_Id")
                            .from(FicsFandoms::Table, FicsFandoms::FicId)
                            .to(Fics::Table, Fics::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FicsFandoms_FandomId_Fandom_Id")
                            .from(FicsFandoms::Table, FicsFandoms::FandomId)
                            .to(Fandoms::Table, Fandoms::Id),
                    )
                    .primary_key(
                        Index::create()
                            .col(FicsFandoms::FicId)
                            .col(FicsFandoms::FandomId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Fandoms::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(FicsFandoms::Table).to_owned())
            .await
    }
}

impl Migration {
    async fn install_default_fandoms(&self, manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(Fandoms::Table)
                    .columns([Fandoms::Name])
                    // Common Fandoms used by Speakeasy Writers Commonly
                    .values_from_panic(vec![
                        ["Ranma 1/2".into()],
                        ["Original Work".into()],
                        ["Troublevers".into()],
                        ["Star Trek".into()],
                        ["My Hero Academia".into()],
                        ["DuckTales".into()],
                        ["Rifts".into()],
                        ["She-Ra".into()],
                        ["Ascendance of a Bookworm".into()],
                        ["Steven Universe".into()],
                        ["Pretty Guardian Sailor Moon".into()],
                        ["Warhammer 40K".into()],
                    ])
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum Fandoms {
    Table,
    Id,
    Name,
}

#[derive(Iden)]
pub enum FicsFandoms {
    Table,
    FicId,
    FandomId,
}
