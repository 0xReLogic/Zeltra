'use client'

import React, { useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import { ArrowLeft, Calendar as CalendarIcon, Download, Loader2 } from 'lucide-react'
import { format } from 'date-fns'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { useAccount, useAccountLedger } from '@/lib/queries/accounts'
import { formatCurrency } from '@/lib/utils/format'
import { cn } from '@/lib/utils'

export default function AccountDetailPage() {
  const params = useParams()
  const router = useRouter()
  const id = params.id as string

  // Query State
  const [page, setPage] = useState(1)
  
  const { data: account, isLoading: isLoadingAccount } = useAccount(id)
  const { data: ledger, isLoading: isLoadingLedger } = useAccountLedger(id, { page, limit: 50 })

  if (isLoadingAccount || isLoadingLedger) {
    return (
      <div className="flex h-96 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!account) {
    return (
       <div className="flex flex-col items-center justify-center h-96 space-y-4">
          <h2 className="text-xl font-semibold">Account not found</h2>
          <Button variant="outline" onClick={() => router.back()}>
             <ArrowLeft className="mr-2 h-4 w-4" /> Go Back
          </Button>
       </div>
    )
  }

  // Calculate totals for simple visual check (mock data doesn't sum up perfectly usually)
  const totalDebit = ledger?.data.reduce((sum, entry) => sum + parseFloat(entry.debit), 0) || 0
  const totalCredit = ledger?.data.reduce((sum, entry) => sum + parseFloat(entry.credit), 0) || 0

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div className="flex items-center gap-4">
           <Button variant="outline" size="icon" onClick={() => router.back()}>
              <ArrowLeft className="h-4 w-4" />
           </Button>
           <div>
              <div className="flex items-center gap-2">
                <h2 className="text-2xl font-bold tracking-tight">{account.code} - {account.name}</h2>
                <Badge variant="outline">{account.account_type.toUpperCase()}</Badge>
              </div>
              <p className="text-muted-foreground">General Ledger</p>
           </div>
        </div>
        <div className="flex items-center gap-2">
           <Button variant="outline">
              <CalendarIcon className="mr-2 h-4 w-4" /> Jan 2026
           </Button>
           <Button variant="outline">
              <Download className="mr-2 h-4 w-4" /> Export CSV
           </Button>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-3">
         <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
               <CardTitle className="text-sm font-medium">Current Balance</CardTitle>
            </CardHeader>
            <CardContent>
               <div className="text-2xl font-bold">{formatCurrency(parseFloat(account.balance), 'USD')}</div>
            </CardContent>
         </Card>
      </div>

      {/* Ledger Table */}
      <Card>
         <CardHeader>
            <CardTitle>Transactions</CardTitle>
            <CardDescription>
               Detailed movements in this account for the selected period.
            </CardDescription>
         </CardHeader>
         <CardContent>
            <div className="rounded-md border">
               <Table>
                  <TableHeader>
                     <TableRow>
                        <TableHead>Date</TableHead>
                        <TableHead>Reference</TableHead>
                        <TableHead className="w-[40%]">Description</TableHead>
                        <TableHead className="text-right text-red-600">Debit</TableHead>
                        <TableHead className="text-right text-green-600">Credit</TableHead>
                        <TableHead className="text-right">Balance</TableHead>
                     </TableRow>
                  </TableHeader>
                  <TableBody>
                     {ledger?.data.map((entry) => (
                        <TableRow key={entry.id}>
                           <TableCell>{entry.transaction_date}</TableCell>
                           <TableCell className="font-mono text-xs">{entry.reference_number}</TableCell>
                           <TableCell>
                              <div className="font-medium text-sm">{entry.description}</div>
                           </TableCell>
                           <TableCell className="text-right text-red-600 font-mono">
                              {parseFloat(entry.debit) > 0 ? formatCurrency(parseFloat(entry.debit), 'USD') : '-'}
                           </TableCell>
                           <TableCell className="text-right text-green-600 font-mono">
                              {parseFloat(entry.credit) > 0 ? formatCurrency(parseFloat(entry.credit), 'USD') : '-'}
                           </TableCell>
                           <TableCell className="text-right font-mono font-medium">
                              {formatCurrency(parseFloat(entry.running_balance), 'USD')}
                           </TableCell>
                        </TableRow>
                     ))}
                     {ledger?.data.length === 0 && (
                        <TableRow>
                           <TableCell colSpan={6} className="h-24 text-center">
                              No transactions found for this period.
                           </TableCell>
                        </TableRow>
                     )}
                  </TableBody>
               </Table>
            </div>
         </CardContent>
      </Card>
    </div>
  )
}
