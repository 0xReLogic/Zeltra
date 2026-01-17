/**
 * Attachment types - re-exported from OpenAPI generated types
 */
import { components } from './api.generated';

// Main attachment types
export type AttachmentResponse = components['schemas']['AttachmentResponse'];
export type RequestUploadRequest = components['schemas']['RequestUploadRequest'];
export type RequestUploadResponse = components['schemas']['RequestUploadResponse'];
export type ConfirmUploadRequest = components['schemas']['ConfirmUploadRequest'];

// Simulation attachment types
export interface SimulationAttachmentResponse extends AttachmentResponse {
  simulation_id: string;
}
