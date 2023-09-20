use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(Flags::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Flags::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Flags::FirstAttempt).integer().not_null())
                    .col(ColumnDef::new(Flags::SecondAttempt).integer().not_null())
                    .col(ColumnDef::new(Flags::ThirdAttempt).integer().not_null())
                    .col(ColumnDef::new(Flags::Wrong).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Flags::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Flags {
    Table,
    Id,
    FirstAttempt,
    SecondAttempt,
    ThirdAttempt,
    Wrong,
}
