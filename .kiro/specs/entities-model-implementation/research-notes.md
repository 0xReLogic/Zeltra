# Research Notes: SeaORM and Rust Database Patterns (2025-2026)

## Date: 2026-01-25

## SeaORM 2.0 Key Changes (January 2026)

### Breaking Changes
- **ExprTrait Required**: Many methods on `Expr` (e.g., `add`, `eq`, `like`) now require `ExprTrait` to be in scope
  - Solution: Add `use sea_orm::ExprTrait;` to files using these methods
- **sqlx 0.9 Upgrade**: Dependency upgraded to sqlx 0.9
- **Type System Changes**: More strict type checking for expressions

### Migration Best Practices (2025-2026)

1. **Migration File Naming**: `mYYYYMMDD_HHMMSS_migration_name.rs`
2. **Migration Structure**:
   ```rust
   use sea_orm_migration::prelude::*;
   
   #[derive(DeriveMigrationName)]
   pub struct Migration;
   
   #[async_trait]
   impl MigrationTrait for Migration {
       async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
           // Create/alter schema
       }
       
       async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
           // Revert changes
       }
   }
   ```

3. **Migration Registration**: Migrations must be registered in `migration/src/lib.rs` in chronological order

4. **Migration Table**: Default name is `seaql_migrations`, can be customized via `MigratorTrait::migration_table_name()`

### Entity Relationship Patterns (2025-2026)

1. **One-to-One Relations**:
   ```rust
   #[sea_orm::model]
   #[derive(DeriveEntityModel, ..)]
   pub struct Model {
       #[sea_orm(has_one)]
       pub related: HasOne<super::related::Entity>,
   }
   ```

2. **One-to-Many Relations**:
   ```rust
   #[sea_orm::model]
   #[derive(DeriveEntityModel, ..)]
   pub struct Model {
       #[sea_orm(has_many)]
       pub children: HasMany<super::child::Entity>,
   }
   ```

3. **Many-to-Many Relations**:
   ```rust
   #[sea_orm::model]
   #[derive(DeriveEntityModel, ..)]
   pub struct Model {
       #[sea_orm(has_many, via = "junction_table")]
       pub related: HasMany<super::related::Entity>,
   }
   ```

4. **Belongs-To Relations** (Inverse):
   ```rust
   #[sea_orm::model]
   #[derive(DeriveEntityModel, ..)]
   pub struct Model {
       pub parent_id: Option<i32>,
       #[sea_orm(belongs_to, from = "parent_id", to = "id")]
       pub parent: HasOne<super::parent::Entity>,
   }
   ```

5. **Complex/Chained Relations**: Use `Linked` trait for multi-hop relationships

### Database Migration Strategies (2025)

1. **Incremental Migrations**: Break large changes into smaller, reversible steps
2. **Data Migration**: Can use SeaORM API within migrations for data transformation
3. **Conditional DDL**: Execute different DDL based on conditions
4. **Connection Pooling**: Use `r2d2` or `deadpool` for connection management
5. **Migration Validation**: Always test migrations on copy of production data first

### Key Takeaways for Our Implementation

1. **Use SeaORM 2.0 patterns**: Ensure `ExprTrait` is imported where needed
2. **Single migration file**: Combine all schema changes in one migration for atomic execution
3. **Foreign key constraints**: Define all FK relationships in migration
4. **Indexes**: Add indexes for performance-critical queries
5. **Data migration**: Include data transformation in same migration file
6. **Rollback support**: Implement `down()` method for all migrations
7. **Testing**: Validate migrations on test database before production

## Implementation Notes

- Backend binary: `zeltra` (not zeltra-api)
- Use SeaORM 2.0 syntax with `ExprTrait`
- Combine all schema changes in single migration file: `m20260125_000001_add_entities.rs`
- Include data migration in same file for atomic execution
- Add comprehensive indexes for entity_id foreign keys
- Implement proper rollback in `down()` method
