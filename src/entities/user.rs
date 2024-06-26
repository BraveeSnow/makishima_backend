//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "User")]
pub struct Model {
    #[sea_orm(column_name = "Id", primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(column_name = "DiscordToken", unique)]
    pub discord_token: String,
    #[sea_orm(column_name = "AnilistId", unique)]
    pub anilist_id: Option<u32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::anilist::Entity",
        from = "Column::AnilistId",
        to = "super::anilist::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Anilist,
}

impl Related<super::anilist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Anilist.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
