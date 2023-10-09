//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub post_id: u64,
    pub title: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::step::Entity")]
    Step,
}

impl Related<super::step::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Step.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}