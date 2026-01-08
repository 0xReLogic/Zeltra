'use client'

import { useTransactions } from '@/lib/queries/transactions'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { CreateTransactionDialog } from '@/components/transactions/CreateTransactionDialog'
import { Loader2 } from 'lucide-react'
import Link from 'next/link'

export default function TransactionsPage() {
  const { data, isLoading, isError } = useTransactions()

  if (isLoading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (isError) {
    return (
      <div className="rounded-md bg-destructive/15 p-4 text-destructive">
        Failed to load transactions. Please try again.
      </div>
    )
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'posted': return 'default' // primary
      case 'approved': return 'secondary' // green-ish in some themes or secondary
      case 'pending': return 'secondary' // yellow-ish usually needs custom class
      case 'draft': return 'outline'
      case 'voided': return 'destructive'
      default: return 'outline'
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">Transactions</h1>
        <div>
           <CreateTransactionDialog />
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Recent Transactions</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Date</TableHead>
                <TableHead>Reference</TableHead>
                <TableHead>Description</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Amount</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data?.data.map((txn) => {
                // Simple logic to find the main amount (max value in entries)
                // In real accounting, we'd sum debits or display differently.
                // For list view, we usually show the total transaction value.
                const totalAmount = Math.max(
                    ...txn.entries.map(e => parseFloat(e.debit) || parseFloat(e.credit))
                )

                return (
                  <Link href={`/dashboard/transactions/${txn.id}`} key={txn.id} className="contents">
                  <TableRow className="cursor-pointer hover:bg-muted/50">
                    <TableCell className="font-medium">
                      {txn.transaction_date}
                    </TableCell>
                    <TableCell>{txn.reference_number}</TableCell>
                    <TableCell className="max-w-[300px] truncate">
                      {txn.description}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className="capitalize">
                        {txn.transaction_type}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Badge variant={getStatusColor(txn.status) as 'default' | 'secondary' | 'outline' | 'destructive'}>
                        {txn.status}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right font-bold">
                       {totalAmount.toLocaleString('en-US', {
                          style: 'currency',
                          currency: 'USD',
                        })}
                    </TableCell>
                  </TableRow>
                  </Link>
                )
              })}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
