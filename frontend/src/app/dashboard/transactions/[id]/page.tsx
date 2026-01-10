'use client'


import React, { useRef, useState } from 'react'
import { useParams } from 'next/navigation'
import { ArrowLeft, Loader2, CheckCircle, XCircle, Clock, FileText, Send } from 'lucide-react'
import Link from 'next/link'

import { useTransaction, useApproveTransaction, useRejectTransaction } from '@/lib/queries/transactions'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { toast } from 'sonner'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  TableFooter,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'

const STATUS_CONFIG = {
  draft: { label: 'Draft', variant: 'secondary' as const, icon: FileText },
  pending: { label: 'Pending', variant: 'outline' as const, icon: Clock },
  approved: { label: 'Approved', variant: 'default' as const, icon: CheckCircle },
  posted: { label: 'Posted', variant: 'default' as const, icon: CheckCircle },
  voided: { label: 'Voided', variant: 'destructive' as const, icon: XCircle },
}

const TYPE_LABELS = {
  expense: 'Expense',
  revenue: 'Revenue',
  transfer: 'Transfer',
  journal: 'Journal Entry',
}

export default function TransactionDetailPage() {
  const params = useParams()
  const transactionId = params.id as string

  const { data: transaction, isLoading, isError } = useTransaction(transactionId)
  const approve = useApproveTransaction()
  const reject = useRejectTransaction()
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [attachments, setAttachments] = useState<{name: string, id: string}[]>([])

  const handleUploadClick = () => {
    fileInputRef.current?.click()
  }

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      // Mock upload process
      const toastId = toast.loading("Uploading attachment...")
      
      // Simulate API call
      setTimeout(() => {
        setAttachments(prev => [...prev, { name: file.name, id: Math.random().toString() }])
        toast.dismiss(toastId)
        toast.success("File uploaded successfully")
      }, 1000)
    }
  }

  if (isLoading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (isError || !transaction) {
    return (
      <div className="flex flex-col items-center justify-center h-64 space-y-4">
        <h2 className="text-xl font-semibold">Transaction not found</h2>
        <Button asChild variant="outline">
          <Link href="/dashboard/transactions">Back to Transactions</Link>
        </Button>
      </div>
    )
  }

  const statusConfig = STATUS_CONFIG[transaction.status]
  const StatusIcon = statusConfig.icon

  // Calculate totals
  const totalDebit = transaction.entries.reduce((sum, e) => sum + parseFloat(e.debit || '0'), 0)
  const totalCredit = transaction.entries.reduce((sum, e) => sum + parseFloat(e.credit || '0'), 0)
  const isBalanced = Math.abs(totalDebit - totalCredit) < 0.01

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div className="flex items-center space-x-4">
          <Button variant="ghost" size="icon" asChild>
            <Link href="/dashboard/transactions">
              <ArrowLeft className="h-4 w-4" />
            </Link>
          </Button>
          <div>
            <h1 className="text-2xl font-bold tracking-tight">{transaction.reference_number}</h1>
            <div className="flex items-center space-x-2 text-muted-foreground mt-1">
              <Badge variant="outline">{TYPE_LABELS[transaction.transaction_type]}</Badge>
              <span>•</span>
              <span>{transaction.transaction_date}</span>
            </div>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant={statusConfig.variant} className="text-sm px-3 py-1">
            <StatusIcon className="h-3 w-3 mr-1" />
            {statusConfig.label}
          </Badge>
        </div>
      </div>

      {/* Description Card */}
      <Card>
        <CardHeader>
          <CardTitle>Description</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">{transaction.description || 'No description provided.'}</p>
        </CardContent>
      </Card>

      {/* Journal Entries */}
      <Card>
        <CardHeader>
          <CardTitle>Journal Entries</CardTitle>
          <CardDescription>
            Debit and credit breakdown for this transaction
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Account Code</TableHead>
                <TableHead>Account Name</TableHead>
                <TableHead className="text-right">Debit</TableHead>
                <TableHead className="text-right">Credit</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {transaction.entries.map((entry, index) => (
                <TableRow key={index}>
                  <TableCell className="font-mono">{entry.account_code}</TableCell>
                  <TableCell>{entry.account_name}</TableCell>
                  <TableCell className="text-right font-mono text-emerald-600">
                    {parseFloat(entry.debit) > 0
                      ? `$${parseFloat(entry.debit).toLocaleString('en-US', { minimumFractionDigits: 2 })}`
                      : '-'}
                  </TableCell>
                  <TableCell className="text-right font-mono text-red-600">
                    {parseFloat(entry.credit) > 0
                      ? `$${parseFloat(entry.credit).toLocaleString('en-US', { minimumFractionDigits: 2 })}`
                      : '-'}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
            <TableFooter>
              <TableRow>
                <TableCell colSpan={2} className="font-semibold">Total</TableCell>
                <TableCell className="text-right font-mono font-semibold text-emerald-600">
                  ${totalDebit.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
                <TableCell className="text-right font-mono font-semibold text-red-600">
                  ${totalCredit.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>
              <TableRow>
                <TableCell colSpan={4} className="text-center">
                  {isBalanced ? (
                    <Badge variant="default" className="bg-emerald-500">
                      <CheckCircle className="h-3 w-3 mr-1" />
                      Balanced
                    </Badge>
                  ) : (
                    <Badge variant="destructive">
                      <XCircle className="h-3 w-3 mr-1" />
                      Unbalanced (Diff: ${Math.abs(totalDebit - totalCredit).toFixed(2)})
                    </Badge>
                  )}
                </TableCell>
              </TableRow>
            </TableFooter>
          </Table>
        </CardContent>
      </Card>

      {/* Attachments & Audit */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <Card>
              <CardHeader>
                  <CardTitle>Attachments</CardTitle>
                  <CardDescription>Supporting documents</CardDescription>
              </CardHeader>
              <CardContent>
                  <input 
                    type="file" 
                    ref={fileInputRef} 
                    className="hidden" 
                    onChange={handleFileChange}
                  />
                  <div 
                    onClick={handleUploadClick}
                    className="border-2 border-dashed rounded-lg p-6 flex flex-col items-center justify-center text-center hover:bg-muted/50 transition-colors cursor-pointer"
                  >
                      <FileText className="h-8 w-8 text-muted-foreground mb-2" />
                      <p className="text-sm font-medium">Click to upload</p>
                      <p className="text-xs text-muted-foreground">or drag and drop files here</p>
                  </div>
                  <div className="mt-4 space-y-2">
                       {/* Mock list of attachments */}
                       <div className="flex items-center justify-between p-3 border rounded-md">
                           <div className="flex items-center space-x-3">
                               <FileText className="h-4 w-4 text-blue-500" />
                               <span className="text-sm font-medium">invoice_inv-2024-001.pdf</span>
                           </div>
                           <Button variant="ghost" size="sm">View</Button>
                       </div>
                       {attachments.map(file => (
                          <div key={file.id} className="flex items-center justify-between p-3 border rounded-md">
                              <div className="flex items-center space-x-3">
                                  <FileText className="h-4 w-4 text-blue-500" />
                                  <span className="text-sm font-medium">{file.name}</span>
                              </div>
                              <Button variant="ghost" size="sm">View</Button>
                          </div>
                       ))}
                  </div>
              </CardContent>
          </Card>

          <Card>
              <CardHeader>
                  <CardTitle>Audit Trail</CardTitle>
                  <CardDescription>History of changes</CardDescription>
              </CardHeader>
              <CardContent>
                  <div className="space-y-4">
                      <div className="flex items-start gap-4">
                          <div className="mt-1 bg-emerald-100 p-1 rounded-full dark:bg-emerald-900">
                             <CheckCircle className="h-3 w-3 text-emerald-600 dark:text-emerald-400" />
                          </div>
                          <div>
                              <p className="text-sm font-medium">Approved by Manager</p>
                              <p className="text-xs text-muted-foreground">Today at 10:30 AM</p>
                          </div>
                      </div>
                      <div className="flex items-start gap-4">
                          <div className="mt-1 bg-blue-100 p-1 rounded-full dark:bg-blue-900">
                             <Send className="h-3 w-3 text-blue-600 dark:text-blue-400" />
                          </div>
                          <div>
                              <p className="text-sm font-medium">Submitted for Approval</p>
                              <p className="text-xs text-muted-foreground">Yesterday at 4:15 PM</p>
                          </div>
                      </div>
                      <div className="flex items-start gap-4">
                          <div className="mt-1 bg-gray-100 p-1 rounded-full dark:bg-gray-800">
                             <FileText className="h-3 w-3 text-gray-600 dark:text-gray-400" />
                          </div>
                          <div>
                              <p className="text-sm font-medium">Created by User</p>
                              <p className="text-xs text-muted-foreground">Yesterday at 4:00 PM</p>
                          </div>
                      </div>
                  </div>
              </CardContent>
          </Card>
      </div>

      {/* Actions */}
      {(transaction.status === 'draft' || transaction.status === 'pending') && (
        <Card>
          <CardHeader>
            <CardTitle>Actions</CardTitle>
          </CardHeader>
          <CardContent className="flex space-x-2">
            {transaction.status === 'draft' && (
              <Button>
                <Send className="h-4 w-4 mr-2" />
                Submit for Approval
              </Button>
            )}
            {transaction.status === 'pending' && (
              <>
                <Button 
                  variant="outline"
                  className="text-emerald-600 border-emerald-600 hover:bg-emerald-50 hover:text-emerald-700 dark:hover:bg-emerald-950 dark:border-emerald-500 dark:text-emerald-500"
                  onClick={() => approve.mutate(transaction.id, {
                      onSuccess: () => toast.success("Transaction Approved!")
                  })}
                  disabled={approve.isPending}
                >
                  {approve.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin"/> : <CheckCircle className="h-4 w-4 mr-2" />}
                  Approve
                </Button>
                <Button 
                  variant="outline"
                  className="text-red-600 border-red-600 hover:bg-red-50 hover:text-red-700 dark:hover:bg-red-950 dark:border-red-500 dark:text-red-500"
                  onClick={() => reject.mutate(transaction.id, {
                      onSuccess: () => toast.success("Transaction Rejected")
                  })}
                  disabled={reject.isPending}
                >
                  {reject.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin"/> : <XCircle className="h-4 w-4 mr-2" />}
                  Reject
                </Button>
              </>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  )
}
