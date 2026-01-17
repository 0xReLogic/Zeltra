/**
 * React Query hooks for simulation attachments
 */
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { apiClient } from '@/lib/api/client';
import type { 
  SimulationAttachmentResponse, 
  RequestUploadRequest, 
  RequestUploadResponse, 
  ConfirmUploadRequest 
} from '@/types/attachments';

// Query keys
export const simulationAttachmentKeys = {
  all: ['simulation-attachments'] as const,
  simulation: (simulationId: string) => [...simulationAttachmentKeys.all, 'simulation', simulationId] as const,
};

/**
 * Hook to fetch simulation attachments
 */
export function useSimulationAttachments(simulationId: string) {
  return useQuery({
    queryKey: simulationAttachmentKeys.simulation(simulationId),
    queryFn: async (): Promise<SimulationAttachmentResponse[]> => {
      const response = await apiClient<{ attachments: SimulationAttachmentResponse[] }>(
        `/simulations/${simulationId}/attachments`
      );
      return response.attachments;
    },
    enabled: !!simulationId,
  });
}

/**
 * Hook to request simulation attachment upload URL
 */
export function useRequestSimulationUpload(simulationId: string) {
  return useMutation({
    mutationFn: async (request: RequestUploadRequest): Promise<RequestUploadResponse> => {
      return apiClient<RequestUploadResponse>(
        `/simulations/${simulationId}/attachments/upload`,
        {
          method: 'POST',
          body: JSON.stringify(request),
        }
      );
    },
    onError: (error) => {
      console.error('Failed to request upload URL:', error);
      toast.error('Failed to request upload URL');
    },
  });
}

/**
 * Hook to confirm simulation attachment upload
 */
export function useConfirmSimulationUpload(simulationId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: ConfirmUploadRequest): Promise<SimulationAttachmentResponse> => {
      return apiClient<SimulationAttachmentResponse>(
        `/simulations/${simulationId}/attachments`,
        {
          method: 'POST',
          body: JSON.stringify(request),
        }
      );
    },
    onSuccess: () => {
      // Invalidate and refetch simulation attachments
      queryClient.invalidateQueries({
        queryKey: simulationAttachmentKeys.simulation(simulationId),
      });
      toast.success('File uploaded successfully');
    },
    onError: (error) => {
      console.error('Failed to confirm upload:', error);
      toast.error('Failed to upload file');
    },
  });
}