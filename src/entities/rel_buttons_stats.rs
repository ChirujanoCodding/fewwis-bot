//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "rel-buttons-stats")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub stats_id: i32,
    pub buttons_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::flags::Entity",
        from = "Column::ButtonsId",
        to = "super::flags::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Flags,
    #[sea_orm(
        belongs_to = "super::stats::Entity",
        from = "Column::StatsId",
        to = "super::stats::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Stats,
}

impl Related<super::flags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Flags.def()
    }
}

impl Related<super::stats::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Stats.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
