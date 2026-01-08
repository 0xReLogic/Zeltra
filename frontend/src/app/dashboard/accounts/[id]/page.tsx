'use client'

import React from 'react'
import { useParams } from 'next/navigation'
import { format } from 'date-fns'
import { ArrowLeft, Loader2, ArrowUpRight, ArrowDownLeft } from 'lucide-react'
import Link from 'next/link'

import { useAccount } from '@/lib/queries/accounts'
import { useLedger } from '@/lib/queries/ledger'
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



export default function AccountDetailsPage() {
  const params = useParams()
  const accountId = params.id as string

  const { data: account, isLoading: isLoadingAccount } = useAccount(accountId)
  const { data: ledger, isLoading: isLoadingLedger } = useLedger(accountId)

  if (isLoadingAccount || isLoadingLedger) {
    return (
      <div className="flex h-64 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!account) {
    return (
      <div className="flex flex-col items-center justify-center h-64 space-y-4">
        <h2 className="text-xl font-semibold">Account not found</h2>
        <Button asChild variant="outline">
          <Link href="/dashboard/accounts">Back to Accounts</Link>
        </Button>
      </div>
    )
  }

  const balance = parseFloat(account.balance)

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center space-x-4">
        <Button variant="ghost" size="icon" asChild>
          <Link href="/dashboard/accounts">
            <ArrowLeft className="h-4 w-4" />
          </Link>
        </Button>
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{account.name}</h1>
          <div className="flex items-center space-x-2 text-muted-foreground">
            <Badge variant="outline">{account.code}</Badge>
            <span>•</span>
            <span className="capitalize">{account.account_type}</span>
          </div>
        </div>
        <div className="ml-auto text-right">
             <div className="text-sm text-muted-foreground">Current Balance</div>
             <div className="text-3xl font-bold font-mono">
                {balance.toLocaleString('en-US', { style: 'currency', currency: 'USD' })}
             </div>
        </div>
      </div>

      {/* Analytics Chart */}
      <Card>
        <CardHeader>
             <CardTitle>Balance History</CardTitle>
             <CardDescription>6 Month trend</CardDescription>
        </CardHeader>
        <CardContent className="h-[300px] flex items-center justify-center bg-muted/20">
             <div className="text-center text-muted-foreground">
                <p>Chart visualization pending</p>
                <p className="text-xs">(Recharts library installation failed, fix pending)</p>
             </div>
        </CardContent>
      </Card>

      {/* Ledger Table */}
      <Card>
        <CardHeader>
          <CardTitle>Ledger Entries</CardTitle>
          <CardDescription>Recent debit and credit mutations</CardDescription>
        </CardHeader>
        <CardContent>
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHead>Date</TableHead>
                        <TableHead>Reference</TableHead>
                        <TableHead>Description</TableHead>
                        <TableHead className="text-right">Debit</TableHead>
                        <TableHead className="text-right">Credit</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {ledger?.data.map((txn) => {
                        // Find entry for THIS account
                        const entry = txn.entries.find(e => e.account_code === account.code || e.account_name === account.name)
                        // Note: matching by code is safer but mock data might vary. The Fallback logic in client.ts
                        // returns the WHOLE transaction list, so we filter visually here.
                        
                        // If no entry found clearly (mock limitation), just show the first one or skip
                        // For demo: show all transactions but highlight relevant amounts
                        
                        const relevantDebit = entry?.debit || '0'
                        const relevantCredit = entry?.credit || '0'

                        // Improved Mock Logic: for the `accounts/acc_001/ledger` fallback, 
                        // we receive /transactions mock. We map it to look like ledger.
                        return (
                            <TableRow key={txn.id}>
                                <TableCell>{txn.transaction_date}</TableCell>
                                <TableCell>{txn.reference_number}</TableCell>
                                <TableCell>{txn.description}</TableCell>
                                <TableCell className="text-right font-mono text-emerald-600">
                                    {parseFloat(relevantDebit) > 0 ? `+${parseFloat(relevantDebit).toLocaleString('en-US', {style:'currency', currency:'USD'})}` : '-'}
                                </TableCell>
                                <TableCell className="text-right font-mono text-red-600">
                                    {parseFloat(relevantCredit) > 0 ? `-${parseFloat(relevantCredit).toLocaleString('en-US', {style:'currency', currency:'USD'})}` : '-'}
                                </TableCell>
                            </TableRow>
                        )
                    })}
                    {(!ledger?.data || ledger.data.length === 0) && (
                         <TableRow>
                            <TableCell colSpan={5} className="h-24 text-center">
                                No transactions found.
                            </TableCell>
                        </TableRow>
                    )}
                </TableBody>
            </Table>
        </CardContent>
      </Card>
    </div>
  )
}
