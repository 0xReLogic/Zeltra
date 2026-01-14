import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '@/lib/api/client';
import type {
  AttachmentResponse,
  RequestUploadRequest,
  RequestUploadResponse,
  ConfirmUploadRequest,
} from '@/types/attachments';

// Query keys
export const attachmentKeys = {
  all: ['attachments'] as const,
  transaction: (transactionId: string) =>
    [...attachmentKeys.all, 'transaction', transactionId] as const,
  detail: (attachmentId: string) =>
    [...attachmentKeys.all, 'detail', attachmentId] as const,
};

// List attachments for a transaction
export function useTransactionAttachments(transactionId: string) {
  return useQuery({
    queryKey: attachmentKeys.transaction(transactionId),
    queryFn: async () => {
      const response = await apiClient<{ attachments: AttachmentResponse[] }>(
        `/transactions/${transactionId}/attachments`
      );
      return response.attachments;
    },
    enabled: !!transactionId,
  });
}

// Get single attachment with download URL
export function useAttachment(attachmentId: string) {
  return useQuery({
    queryKey: attachmentKeys.detail(attachmentId),
    queryFn: () =>
      apiClient<AttachmentResponse>(`/attachments/${attachmentId}`),
    enabled: !!attachmentId,
  });
}

// Request upload URL (step 1 of upload flow)
export function useRequestUpload(transactionId: string) {
  return useMutation({
    mutationFn: (data: RequestUploadRequest) =>
      apiClient<RequestUploadResponse>(`/transactions/${transactionId}/attachments/upload`, {
        method: 'POST',
        body: JSON.stringify(data),
      }),
  });
}

// Confirm upload (step 3 of upload flow, after uploading to presigned URL)
export function useConfirmUpload(transactionId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (data: ConfirmUploadRequest) =>
      apiClient<AttachmentResponse>(`/transactions/${transactionId}/attachments`, {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: attachmentKeys.transaction(transactionId) });
    },
  });
}

// Delete attachment
export function useDeleteAttachment() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (attachmentId: string) =>
      apiClient<void>(`/attachments/${attachmentId}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: attachmentKeys.all });
    },
  });
}
