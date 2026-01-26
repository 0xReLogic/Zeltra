//! `SeaORM` Entity for entities table
//!
//! Represents a legal or operational unit (company, subsidiary, branch, division) within an organization.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "entities"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Eq, Serialize, Deserialize)]
pub struct Model {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub legal_name: Option<String>,
    pub tax_id: Option<String>,
    pub entity_type: String,
    pub base_currency: String,
    pub is_active: bool,
    pub settings: Json,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    OrganizationId,
    Name,
    LegalName,
    TaxId,
    EntityType,
    BaseCurrency,
    IsActive,
    Settings,
    CreatedAt,
    UpdatedAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = Uuid;
    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Organizations,
    ChartOfAccounts,
    Transactions,
    LedgerEntries,
    Budgets,
    FiscalYears,
    AccrualSchedules,
    RevaluationLogs,
    IntercompanyMappingsSource,
    IntercompanyMappingsTarget,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Uuid.def(),
            Self::OrganizationId => ColumnType::Uuid.def(),
            Self::Name => ColumnType::String(StringLen::N(255u32)).def(),
            Self::LegalName => ColumnType::String(StringLen::N(255u32)).def().null(),
            Self::TaxId => ColumnType::String(StringLen::N(100u32)).def().null(),
            Self::EntityType => ColumnType::String(StringLen::N(50u32)).def(),
            Self::BaseCurrency => ColumnType::Char(Some(3u32)).def(),
            Self::IsActive => ColumnType::Boolean.def(),
            Self::Settings => ColumnType::JsonBinary.def(),
            Self::CreatedAt => ColumnType::TimestampWithTimeZone.def(),
            Self::UpdatedAt => ColumnType::TimestampWithTimeZone.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Organizations => Entity::belongs_to(super::organizations::Entity)
                .from(Column::OrganizationId)
                .to(super::organizations::Column::Id)
                .into(),
            Self::ChartOfAccounts => Entity::has_many(super::chart_of_accounts::Entity).into(),
            Self::Transactions => Entity::has_many(super::transactions::Entity).into(),
            Self::LedgerEntries => Entity::has_many(super::ledger_entries::Entity).into(),
            Self::Budgets => Entity::has_many(super::budgets::Entity).into(),
            Self::FiscalYears => Entity::has_many(super::fiscal_years::Entity).into(),
            Self::AccrualSchedules => Entity::has_many(super::accrual_schedules::Entity).into(),
            Self::RevaluationLogs => Entity::has_many(super::revaluation_logs::Entity).into(),
            Self::IntercompanyMappingsSource => {
                Entity::has_many(super::intercompany_mappings::Entity)
                    .from(Column::Id)
                    .to(super::intercompany_mappings::Column::SourceEntityId)
                    .into()
            }
            Self::IntercompanyMappingsTarget => {
                Entity::has_many(super::intercompany_mappings::Entity)
                    .from(Column::Id)
                    .to(super::intercompany_mappings::Column::TargetEntityId)
                    .into()
            }
        }
    }
}

impl Related<super::organizations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Organizations.def()
    }
}

impl Related<super::chart_of_accounts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChartOfAccounts.def()
    }
}

impl Related<super::transactions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transactions.def()
    }
}

impl Related<super::ledger_entries::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LedgerEntries.def()
    }
}

impl Related<super::budgets::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Budgets.def()
    }
}

impl Related<super::fiscal_years::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FiscalYears.def()
    }
}

impl Related<super::accrual_schedules::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccrualSchedules.def()
    }
}

impl Related<super::revaluation_logs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RevaluationLogs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
