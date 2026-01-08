'use client'

import React from 'react'
import { useParams } from 'next/navigation'
import { ArrowLeft, Loader2, CheckCircle, XCircle, Clock, FileText, Send } from 'lucide-react'
import Link from 'next/link'

import { useTransaction } from '@/lib/queries/transactions'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
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
                <Button variant="default">
                  <CheckCircle className="h-4 w-4 mr-2" />
                  Approve
                </Button>
                <Button variant="destructive">
                  <XCircle className="h-4 w-4 mr-2" />
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
