use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m_20250502_000001_initial"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    #[allow(clippy::too_many_lines)]
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    // Store u64s as Strings, since SQLite can't store u64 (only i64)
                    .col(ColumnDef::new(Users::DiscordId).string_len(20).not_null())
                    .col(ColumnDef::new(Users::BirthdayMonth).integer().null())
                    .col(ColumnDef::new(Users::BirthdayDay).integer().null())
                    .col(ColumnDef::new(Users::LastLoveSpam).timestamp().null())
                    // https://data.iana.org/time-zones/theory.html#naming
                    .col(ColumnDef::new(Users::TimeZone).string_len(29).null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(FicPlatforms::Table)
                    .col(
                        ColumnDef::new(FicPlatforms::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(FicPlatforms::Name)
                            .string_len(50)
                            .not_null()
                            .unique_key(),
                    )
                    // Name of the Emoji
                    .col(ColumnDef::new(FicPlatforms::Emoji).string_len(30).null())
                    .to_owned(),
            )
            .await?;

        self.install_default_fic_platforms(manager).await?;

        manager
            .create_table(
                Table::create()
                    .table(Fics::Table)
                    .col(
                        ColumnDef::new(Fics::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Fics::AuthorUserId).integer().not_null())
                    .col(ColumnDef::new(Fics::Title).string_len(200).not_null())
                    .col(ColumnDef::new(Fics::PhoenixSongBook).integer().null())
                    .col(ColumnDef::new(Fics::PhoenixSongChapter).integer().null())
                    .col(
                        ColumnDef::new(Fics::IsPhoenixRocktail)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("Fics_AuthorUserId_Users_Id")
                            .from(Fics::Table, Fics::AuthorUserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Urls::Table)
                    /*.col(
                        ColumnDef::new(Urls::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )*/
                    .col(ColumnDef::new(Urls::Url).string_len(1024).not_null())
                    .col(ColumnDef::new(Urls::FicId).integer().not_null())
                    .col(ColumnDef::new(Urls::PlatformId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("Urls_FicId_Fics_id")
                            .from(Urls::Table, Urls::FicId)
                            .to(Fics::Table, Fics::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("Urls_PlatformId_FicPlatforms_id")
                            .from(Urls::Table, Urls::PlatformId)
                            .to(FicPlatforms::Table, FicPlatforms::Id),
                    )
                    .primary_key(Index::create().col(Urls::FicId).col(Urls::PlatformId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WritingPrompts::Table)
                    .col(
                        ColumnDef::new(WritingPrompts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WritingPrompts::Type)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WritingPrompts::Value)
                            .string_len(30)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        self.install_default_writing_prompts(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(FicPlatforms::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Fics::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(WritingPrompts::Table).to_owned())
            .await
    }
}

impl Migration {
    async fn install_default_fic_platforms(
        &self,
        manager: &SchemaManager<'_>,
    ) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(FicPlatforms::Table)
                    .columns([FicPlatforms::Name, FicPlatforms::Emoji])
                    .values_from_panic(vec![
                        ["Archive of Our Own".into(), "ao3".into()],
                        ["Fanfiction.net".into(), "ffn".into()],
                        ["FicWad".into(), "ficwad".into()],
                        ["FimFiction".into(), "fimfiction".into()],
                        ["Inkitt".into(), "inkitt".into()],
                        ["itch.io".into(), None::<String>.into()],
                        ["MediaMiner".into(), "mediaminer".into()],
                        ["Neobook".into(), "neobook".into()],
                        ["Questionable Questing".into(), "qq".into()],
                        ["QuoteV".into(), "quotev".into()],
                        ["Royal Road".into(), "rr".into()],
                        ["Scribble Hub".into(), "scribblehub".into()],
                        ["Spacebattles".into(), "spacebattles".into()],
                        ["Sufficient Velocity".into(), "sv".into()],
                        ["Tapas".into(), "tapas".into()],
                        ["Wattpad".into(), "wattpad".into()],
                        ["WebNovel".into(), "webnovel".into()],
                    ])
                    .to_owned(),
            )
            .await
    }

    async fn install_default_writing_prompts(
        &self,
        manager: &SchemaManager<'_>,
    ) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(WritingPrompts::Table)
                    .columns([WritingPrompts::Type, WritingPrompts::Value])
                    .values_from_panic(vec![
                        ["pov".into(), "Outsider".into()],
                        ["pov".into(), "Minor Character".into()],
                        ["pov".into(), "Antagonist".into()],
                        ["pov".into(), "Protagonist".into()],
                        ["pov".into(), "Alternating".into()],
                        ["pov".into(), "Omniscient Outsider".into()],
                        ["conflict".into(), "Vs. Man".into()],
                        ["conflict".into(), "Vs. Technology".into()],
                        ["conflict".into(), "Vs. Nature".into()],
                        ["conflict".into(), "Vs. Society".into()],
                        ["conflict".into(), "Vs. Self".into()],
                        ["conflict".into(), "Vs. Supernatural".into()],
                        ["time".into(), "Present Day".into()],
                        ["time".into(), "Pre-Modern".into()],
                        ["time".into(), "Ancient History".into()],
                        ["time".into(), "Alternative History".into()],
                        ["time".into(), "Medieval History".into()],
                        ["time".into(), "Future".into()],
                        ["theme".into(), "Justice".into()],
                        ["theme".into(), "Love".into()],
                        ["theme".into(), "Hope".into()],
                        ["theme".into(), "Identity".into()],
                        ["theme".into(), "Power".into()],
                        ["theme".into(), "Loneliness".into()],
                        ["place".into(), "Rural".into()],
                        ["place".into(), "Your Hometown".into()],
                        ["place".into(), "Foreign Country".into()],
                        ["place".into(), "Urban".into()],
                        ["place".into(), "Dystopian".into()],
                        ["place".into(), "Fantasy Setting".into()],
                        ["character_identity".into(), "Woman".into()],
                        ["character_identity".into(), "Man".into()],
                        ["character_identity".into(), "Non-Binary".into()],
                        ["character_identity".into(), "Trans Man".into()],
                        ["character_identity".into(), "Trans Woman".into()],
                        ["character_identity".into(), "Genderless".into()],
                        ["character_identity".into(), "Child".into()],
                        ["character_identity".into(), "Senior".into()],
                        ["character_trait".into(), "Empathetic".into()],
                        ["character_trait".into(), "Optimistic".into()],
                        ["character_trait".into(), "Curious".into()],
                        ["character_trait".into(), "Charismatic".into()],
                        ["character_trait".into(), "Witty".into()],
                        ["character_trait".into(), "Prudent".into()],
                        ["character_trait".into(), "Sexual".into()],
                        ["character_trait".into(), "Insecure".into()],
                        ["character_trait".into(), "Impatient".into()],
                        ["character_trait".into(), "Stubborn".into()],
                        ["character_trait".into(), "Arrogant".into()],
                        ["character_trait".into(), "Greedy".into()],
                        ["character_trait".into(), "World-Weary".into()],
                        ["character_trait".into(), "Martyr Complex".into()],
                        ["character_background".into(), "Uneducated".into()],
                        ["character_background".into(), "Shape-shifter".into()],
                        ["character_background".into(), "Minority".into()],
                        ["character_background".into(), "Middle Class".into()],
                        ["character_background".into(), "Privileged".into()],
                        ["character_background".into(), "Immigrant".into()],
                        ["character_background".into(), "Orphan".into()],
                    ])
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum Urls {
    Table,
    //Id,
    Url,
    FicId,
    PlatformId,
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    DiscordId,
    BirthdayMonth,
    BirthdayDay,
    LastLoveSpam,
    TimeZone,
}

#[derive(Iden)]
pub enum FicPlatforms {
    Table,
    Id,
    Name,
    Emoji,
}

#[derive(Iden)]
pub enum Fics {
    Table,
    Id,
    AuthorUserId,
    Title,
    PhoenixSongBook,
    PhoenixSongChapter,
    IsPhoenixRocktail,
}

#[derive(Iden)]
pub enum WritingPrompts {
    Table,
    Id,
    Type,
    Value,
}
