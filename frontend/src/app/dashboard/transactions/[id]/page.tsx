'use client'

import React, { useState, useMemo } from 'react'
import { useParams } from 'next/navigation'
import { ArrowLeft, Loader2, CheckCircle, XCircle, Clock, FileText, Send, Ban, BookCheck } from 'lucide-react'
import Link from 'next/link'

import { useTransaction, useApproveTransaction, useRejectTransaction, useSubmitTransaction, usePostTransaction, useVoidTransaction } from '@/lib/queries/transactions'
import { useAccounts } from '@/lib/queries/accounts'
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
import { AttachmentUpload, AttachmentList } from '@/components/attachments'

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
  const { data: transaction, isLoading, isError, refetch } = useTransaction(transactionId)
  const { data: accountsData } = useAccounts()
  const approve = useApproveTransaction()
  const reject = useRejectTransaction()
  const submit = useSubmitTransaction()
  const post = usePostTransaction()
  const voidTx = useVoidTransaction()
  const [rejectReason, setRejectReason] = useState('')
  const [voidReason, setVoidReason] = useState('')
  const [showRejectDialog, setShowRejectDialog] = useState(false)
  const [showVoidDialog, setShowVoidDialog] = useState(false)

  // Create account lookup map for displaying account code/name from account_id
  const accountsMap = useMemo(() => {
    const map = new Map<string, { code: string; name: string }>()
    if (accountsData?.accounts) {
      for (const account of accountsData.accounts) {
        map.set(account.id, { code: account.code, name: account.name })
      }
    }
    return map
  }, [accountsData])

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

  const statusConfig = STATUS_CONFIG[transaction.status as keyof typeof STATUS_CONFIG] || STATUS_CONFIG.draft
  const StatusIcon = statusConfig.icon

  const totalDebit = transaction.entries.reduce((sum, e) => sum + parseFloat(e.debit || '0'), 0)
  const totalCredit = transaction.entries.reduce((sum, e) => sum + parseFloat(e.credit || '0'), 0)
  const isBalanced = Math.abs(totalDebit - totalCredit) < 0.01
  const isDraft = transaction.status === 'draft'

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
              <Badge variant="outline">{TYPE_LABELS[transaction.type as keyof typeof TYPE_LABELS] || transaction.type}</Badge>
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
          <CardDescription>Debit and credit breakdown for this transaction</CardDescription>
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
              {transaction.entries.map((entry, index) => {
                const account = accountsMap.get(entry.account_id)
                return (
                  <TableRow key={index}>
                    <TableCell className="font-mono">{account?.code || entry.account_id.slice(0, 8)}</TableCell>
                    <TableCell>{account?.name || 'Loading...'}</TableCell>
                    <TableCell className="text-right font-mono text-emerald-600">
                      {parseFloat(entry.debit) > 0
                        ? `${parseFloat(entry.debit).toLocaleString('en-US', { minimumFractionDigits: 2 })}`
                        : '-'}
                    </TableCell>
                    <TableCell className="text-right font-mono text-red-600">
                      {parseFloat(entry.credit) > 0
                        ? `${parseFloat(entry.credit).toLocaleString('en-US', { minimumFractionDigits: 2 })}`
                        : '-'}
                    </TableCell>
                  </TableRow>
                )
              })}
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
          <CardContent className="space-y-4">
            {isDraft && (
              <AttachmentUpload
                transactionId={transactionId}
                onUploadComplete={() => refetch()}
              />
            )}
            <AttachmentList
              transactionId={transactionId}
              allowDelete={isDraft}
            />
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
      <Card>
        <CardHeader>
          <CardTitle>Actions</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-2">
          {transaction.status === 'draft' && (
            <Button
              onClick={() => submit.mutate(transaction.id, {
                onSuccess: () => toast.success('Transaction submitted for approval')
              })}
              disabled={submit.isPending}
            >
              {submit.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Send className="h-4 w-4 mr-2" />}
              Submit for Approval
            </Button>
          )}

          {transaction.status === 'pending' && (
            <>
              <Button
                variant="outline"
                className="text-emerald-600 border-emerald-600 hover:bg-emerald-50 hover:text-emerald-700 dark:hover:bg-emerald-950 dark:border-emerald-500 dark:text-emerald-500"
                onClick={() => approve.mutate(transaction.id, {
                  onSuccess: () => toast.success('Transaction approved!')
                })}
                disabled={approve.isPending}
              >
                {approve.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <CheckCircle className="h-4 w-4 mr-2" />}
                Approve
              </Button>
              <Button
                variant="outline"
                className="text-red-600 border-red-600 hover:bg-red-50 hover:text-red-700 dark:hover:bg-red-950 dark:border-red-500 dark:text-red-500"
                onClick={() => setShowRejectDialog(true)}
                disabled={reject.isPending}
              >
                {reject.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <XCircle className="h-4 w-4 mr-2" />}
                Reject
              </Button>
            </>
          )}

          {transaction.status === 'approved' && (
            <Button
              onClick={() => post.mutate(transaction.id, {
                onSuccess: () => toast.success('Transaction posted!')
              })}
              disabled={post.isPending}
            >
              {post.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <BookCheck className="h-4 w-4 mr-2" />}
              Post Transaction
            </Button>
          )}

          {transaction.status === 'posted' && (
            <Button
              variant="destructive"
              onClick={() => setShowVoidDialog(true)}
              disabled={voidTx.isPending}
            >
              {voidTx.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Ban className="h-4 w-4 mr-2" />}
              Void Transaction
            </Button>
          )}
        </CardContent>
      </Card>

      {/* Reject Dialog */}
      {showRejectDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <Card className="w-full max-w-md">
            <CardHeader>
              <CardTitle>Reject Transaction</CardTitle>
              <CardDescription>Please provide a reason for rejection</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <textarea
                className="w-full p-2 border rounded-md min-h-[100px]"
                placeholder="Enter rejection reason..."
                value={rejectReason}
                onChange={(e) => setRejectReason(e.target.value)}
              />
              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={() => setShowRejectDialog(false)}>
                  Cancel
                </Button>
                <Button
                  variant="destructive"
                  onClick={() => {
                    reject.mutate({ id: transaction.id, reason: rejectReason }, {
                      onSuccess: () => {
                        toast.success('Transaction rejected')
                        setShowRejectDialog(false)
                        setRejectReason('')
                      }
                    })
                  }}
                  disabled={!rejectReason.trim() || reject.isPending}
                >
                  {reject.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : null}
                  Reject
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Void Dialog */}
      {showVoidDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <Card className="w-full max-w-md">
            <CardHeader>
              <CardTitle>Void Transaction</CardTitle>
              <CardDescription>This action cannot be undone. Please provide a reason.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <textarea
                className="w-full p-2 border rounded-md min-h-[100px]"
                placeholder="Enter void reason..."
                value={voidReason}
                onChange={(e) => setVoidReason(e.target.value)}
              />
              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={() => setShowVoidDialog(false)}>
                  Cancel
                </Button>
                <Button
                  variant="destructive"
                  onClick={() => {
                    voidTx.mutate({ id: transaction.id, reason: voidReason }, {
                      onSuccess: () => {
                        toast.success('Transaction voided')
                        setShowVoidDialog(false)
                        setVoidReason('')
                      }
                    })
                  }}
                  disabled={!voidReason.trim() || voidTx.isPending}
                >
                  {voidTx.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : null}
                  Void
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  )
}
