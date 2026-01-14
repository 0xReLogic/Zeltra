'use client';

import { useState } from 'react';
import { FileIcon, Download, Trash2, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { useTransactionAttachments, useDeleteAttachment } from '@/lib/queries/attachments';
import type { AttachmentResponse } from '@/types/attachments';

interface AttachmentListProps {
  transactionId: string;
  allowDelete?: boolean;
}

export function AttachmentList({ transactionId, allowDelete = false }: AttachmentListProps) {
  const { data: attachments, isLoading, error } = useTransactionAttachments(transactionId);
  const deleteAttachment = useDeleteAttachment();
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const handleDelete = async (attachmentId: string) => {
    setDeletingId(attachmentId);
    try {
      await deleteAttachment.mutateAsync(attachmentId);
    } finally {
      setDeletingId(null);
    }
  };

  const handleDownload = (attachment: AttachmentResponse) => {
    if (attachment.download_url) {
      window.open(attachment.download_url, '_blank');
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-4">
        <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertDescription>Failed to load attachments</AlertDescription>
      </Alert>
    );
  }

  if (!attachments || attachments.length === 0) {
    return (
      <p className="text-sm text-muted-foreground text-center py-4">
        No attachments
      </p>
    );
  }

  return (
    <div className="space-y-2">
      {attachments.map((attachment) => (
        <div
          key={attachment.id}
          className="flex items-center gap-3 p-3 border rounded-lg"
        >
          <FileIcon className="h-8 w-8 text-muted-foreground flex-shrink-0" />
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium truncate">{attachment.filename}</p>
            <p className="text-xs text-muted-foreground">
              {attachment.mime_type} • {(attachment.file_size / 1024).toFixed(1)} KB
            </p>
          </div>
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => handleDownload(attachment)}
              disabled={!attachment.download_url}
            >
              <Download className="h-4 w-4" />
            </Button>
            {allowDelete && (
              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    disabled={deletingId === attachment.id}
                  >
                    {deletingId === attachment.id ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <Trash2 className="h-4 w-4 text-destructive" />
                    )}
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>Delete Attachment</AlertDialogTitle>
                    <AlertDialogDescription>
                      Are you sure you want to delete &quot;{attachment.filename}&quot;? This action cannot be undone.
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>Cancel</AlertDialogCancel>
                    <AlertDialogAction onClick={() => handleDelete(attachment.id)}>
                      Delete
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
