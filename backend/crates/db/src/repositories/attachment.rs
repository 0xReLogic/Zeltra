//! Attachment repository for database operations.
//!
//! Implements attachment CRUD operations using SeaORM.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use uuid::Uuid;

use crate::entities::{
    attachments, sea_orm_active_enums::AttachmentType as DbAttachmentType,
    sea_orm_active_enums::StorageProvider as DbStorageProvider, transactions,
};
use zeltra_core::attachment::{
    Attachment, AttachmentError, AttachmentRepository as AttachmentRepoTrait, AttachmentType,
    CreateAttachmentInput,
};

/// Attachment repository implementation.
#[derive(Debug, Clone)]
pub struct AttachmentRepository {
    db: DatabaseConnection,
}

impl AttachmentRepository {
    /// Create a new attachment repository.
    #[must_use]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl AttachmentRepoTrait for AttachmentRepository {
    async fn create(&self, input: CreateAttachmentInput) -> Result<Attachment, AttachmentError> {
        let active_model = attachments::ActiveModel {
            id: Set(input.id),
            organization_id: Set(input.organization_id),
            transaction_id: Set(input.transaction_id),
            attachment_type: Set(to_db_attachment_type(input.attachment_type)),
            file_name: Set(input.filename.clone()),
            file_size: Set(input.file_size),
            mime_type: Set(input.mime_type.clone()),
            checksum_sha256: Set(input.checksum_sha256.clone()),
            storage_provider: Set(to_db_storage_provider(&input.storage_provider)),
            storage_bucket: Set(input.storage_bucket.clone()),
            storage_key: Set(input.storage_key.clone()),
            storage_region: Set(input.storage_region.clone()),
            extracted_data: Set(None),
            ocr_processed_at: Set(None),
            uploaded_by: Set(input.uploaded_by),
            created_at: Set(Utc::now().into()),
        };

        let model = active_model
            .insert(&self.db)
            .await
            .map_err(|e| AttachmentError::repository(e.to_string()))?;

        Ok(to_domain(model))
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        organization_id: Uuid,
    ) -> Result<Option<Attachment>, AttachmentError> {
        let model = attachments::Entity::find_by_id(id)
            .filter(attachments::Column::OrganizationId.eq(organization_id))
            .one(&self.db)
            .await
            .map_err(|e| AttachmentError::repository(e.to_string()))?;

        Ok(model.map(to_domain))
    }

    async fn list_by_transaction(
        &self,
        transaction_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<Attachment>, AttachmentError> {
        let models = attachments::Entity::find()
            .filter(attachments::Column::TransactionId.eq(transaction_id))
            .filter(attachments::Column::OrganizationId.eq(organization_id))
            .order_by_desc(attachments::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| AttachmentError::repository(e.to_string()))?;

        Ok(models.into_iter().map(to_domain).collect())
    }

    async fn delete(&self, id: Uuid, organization_id: Uuid) -> Result<bool, AttachmentError> {
        let result = attachments::Entity::delete_many()
            .filter(attachments::Column::Id.eq(id))
            .filter(attachments::Column::OrganizationId.eq(organization_id))
            .exec(&self.db)
            .await
            .map_err(|e| AttachmentError::repository(e.to_string()))?;

        Ok(result.rows_affected > 0)
    }

    async fn transaction_exists(
        &self,
        transaction_id: Uuid,
        organization_id: Uuid,
    ) -> Result<bool, AttachmentError> {
        let count: u64 = transactions::Entity::find_by_id(transaction_id)
            .filter(transactions::Column::OrganizationId.eq(organization_id))
            .count(&self.db)
            .await
            .map_err(|e| AttachmentError::repository(e.to_string()))?;

        Ok(count > 0)
    }
}

/// Convert domain attachment type to database enum.
fn to_db_attachment_type(t: AttachmentType) -> DbAttachmentType {
    match t {
        AttachmentType::Receipt => DbAttachmentType::Receipt,
        AttachmentType::Invoice => DbAttachmentType::Invoice,
        AttachmentType::Contract => DbAttachmentType::Contract,
        AttachmentType::SupportingDocument => DbAttachmentType::SupportingDocument,
        AttachmentType::Other => DbAttachmentType::Other,
    }
}

/// Convert database attachment type to domain enum.
fn from_db_attachment_type(t: &DbAttachmentType) -> AttachmentType {
    match t {
        DbAttachmentType::Receipt => AttachmentType::Receipt,
        DbAttachmentType::Invoice => AttachmentType::Invoice,
        DbAttachmentType::Contract => AttachmentType::Contract,
        DbAttachmentType::SupportingDocument => AttachmentType::SupportingDocument,
        DbAttachmentType::Other => AttachmentType::Other,
    }
}

/// Convert storage provider string to database enum.
fn to_db_storage_provider(provider: &str) -> DbStorageProvider {
    match provider {
        "s3" | "cloudflare_r2" => DbStorageProvider::CloudflareR2,
        "aws_s3" => DbStorageProvider::AwsS3,
        "azure_blob" => DbStorageProvider::AzureBlob,
        "digitalocean_spaces" => DbStorageProvider::DigitaloceanSpaces,
        "supabase_storage" => DbStorageProvider::SupabaseStorage,
        _ => DbStorageProvider::Local,
    }
}

/// Convert database storage provider to string.
fn from_db_storage_provider(provider: &DbStorageProvider) -> String {
    match provider {
        DbStorageProvider::CloudflareR2 => "cloudflare_r2".to_string(),
        DbStorageProvider::AwsS3 => "aws_s3".to_string(),
        DbStorageProvider::AzureBlob => "azure_blob".to_string(),
        DbStorageProvider::DigitaloceanSpaces => "digitalocean_spaces".to_string(),
        DbStorageProvider::SupabaseStorage => "supabase_storage".to_string(),
        DbStorageProvider::Local => "local".to_string(),
    }
}

/// Convert database model to domain model.
fn to_domain(model: attachments::Model) -> Attachment {
    Attachment {
        id: model.id,
        organization_id: model.organization_id,
        transaction_id: model.transaction_id,
        attachment_type: from_db_attachment_type(&model.attachment_type),
        filename: model.file_name,
        file_size: model.file_size,
        mime_type: model.mime_type,
        checksum_sha256: model.checksum_sha256,
        storage_provider: from_db_storage_provider(&model.storage_provider),
        storage_bucket: model.storage_bucket,
        storage_key: model.storage_key,
        storage_region: model.storage_region,
        uploaded_by: model.uploaded_by,
        created_at: model.created_at.with_timezone(&chrono::Utc),
    }
}
