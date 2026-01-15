'use client';

import { useState, useCallback } from 'react';
import { useDropzone } from 'react-dropzone';
import { Upload, X, FileIcon, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useRequestUpload, useConfirmUpload } from '@/lib/queries/attachments';

const MAX_SIZE = 10 * 1024 * 1024; // 10MB

interface AttachmentUploadProps {
  transactionId: string;
  onUploadComplete?: () => void;
}

export function AttachmentUpload({ transactionId, onUploadComplete }: AttachmentUploadProps) {
  const [file, setFile] = useState<File | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);

  const requestUpload = useRequestUpload(transactionId);
  const confirmUpload = useConfirmUpload(transactionId);

  const onDrop = useCallback((acceptedFiles: File[], rejectedFiles: { errors: readonly { message: string }[] }[]) => {
    setError(null);
    if (rejectedFiles.length > 0) {
      const err = rejectedFiles[0].errors[0];
      setError(err.message);
      return;
    }
    if (acceptedFiles.length > 0) {
      setFile(acceptedFiles[0]);
    }
  }, []);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'application/pdf': ['.pdf'],
      'image/png': ['.png'],
      'image/jpeg': ['.jpg', '.jpeg'],
      'application/msword': ['.doc'],
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document': ['.docx'],
    },
    maxSize: MAX_SIZE,
    multiple: false,
  });

  const handleUpload = async () => {
    if (!file) return;
    setUploading(true);
    setError(null);

    try {
      // Step 1: Request upload URL
      const uploadResponse = await requestUpload.mutateAsync({
        filename: file.name,
        content_type: file.type,
        file_size: file.size,
      });

      // Step 2: Upload file to presigned URL
      const uploadResult = await fetch(uploadResponse.upload_url, {
        method: 'PUT',
        body: file,
        headers: { 'Content-Type': file.type },
      });

      if (!uploadResult.ok) {
        throw new Error('Failed to upload file to storage');
      }

      // Step 3: Confirm upload
      await confirmUpload.mutateAsync({
        attachment_id: uploadResponse.attachment_id,
        attachment_type: 'document',
        content_type: file.type,
        file_size: file.size,
        filename: file.name,
        storage_key: uploadResponse.storage_key,
      });

      setFile(null);
      onUploadComplete?.();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Upload failed');
    } finally {
      setUploading(false);
    }
  };

  const clearFile = () => {
    setFile(null);
    setError(null);
  };

  return (
    <div className="space-y-4">
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {!file ? (
        <div
          {...getRootProps()}
          className={`border-2 border-dashed rounded-lg p-6 text-center cursor-pointer transition-colors ${
            isDragActive ? 'border-primary bg-primary/5' : 'border-muted-foreground/25 hover:border-primary/50'
          }`}
        >
          <input {...getInputProps()} />
          <Upload className="mx-auto h-8 w-8 text-muted-foreground mb-2" />
          <p className="text-sm text-muted-foreground">
            {isDragActive ? 'Drop file here' : 'Drag & drop or click to select'}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            PDF, PNG, JPG, DOC, DOCX (max 10MB)
          </p>
        </div>
      ) : (
        <div className="flex items-center gap-3 p-3 border rounded-lg">
          <FileIcon className="h-8 w-8 text-muted-foreground" />
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium truncate">{file.name}</p>
            <p className="text-xs text-muted-foreground">
              {(file.size / 1024).toFixed(1)} KB
            </p>
          </div>
          <Button variant="ghost" size="icon" onClick={clearFile} disabled={uploading}>
            <X className="h-4 w-4" />
          </Button>
        </div>
      )}

      {file && (
        <Button onClick={handleUpload} disabled={uploading} className="w-full">
          {uploading ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Uploading...
            </>
          ) : (
            'Upload Attachment'
          )}
        </Button>
      )}
    </div>
  );
}
