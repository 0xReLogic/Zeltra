'use client'

import { useReconciliation } from '@/lib/queries/forensic'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Loader2, CheckCircle2, AlertTriangle, XCircle, RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useQueryClient } from '@tanstack/react-query'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

export default function ForensicPage() {
  const { data, isLoading, error, refetch, isFetching } = useReconciliation()
  const queryClient = useQueryClient()

  const handleRefresh = () => {
    queryClient.invalidateQueries({ queryKey: ['forensic', 'reconciliation'] })
    refetch()
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
        <XCircle className="h-12 w-12 text-destructive" />
        <p className="text-muted-foreground">Failed to load reconciliation data</p>
        <p className="text-sm text-muted-foreground">
          {error instanceof Error ? error.message : 'Enterprise tier required'}
        </p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Forensic Analysis</h1>
          <p className="text-muted-foreground">
            Balance Reconciliation & Data Integrity Checks
          </p>
        </div>
        <Button onClick={handleRefresh} disabled={isFetching} variant="outline">
          <RefreshCw className={`mr-2 h-4 w-4 ${isFetching ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Status</CardTitle>
            {data?.is_clean ? (
              <CheckCircle2 className="h-4 w-4 text-green-500" />
            ) : (
              <AlertTriangle className="h-4 w-4 text-yellow-500" />
            )}
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {data?.is_clean ? (
                <span className="text-green-600">Clean</span>
              ) : (
                <span className="text-yellow-600">Review Needed</span>
              )}
            </div>
            <p className="text-xs text-muted-foreground">
              Last run: {data?.run_at ? new Date(data.run_at).toLocaleString() : 'N/A'}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Accounts</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data?.total_accounts ?? 0}</div>
            <p className="text-xs text-muted-foreground">Active accounts analyzed</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Matched</CardTitle>
            <CheckCircle2 className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">{data?.matched_count ?? 0}</div>
            <p className="text-xs text-muted-foreground">Perfect balance match</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Discrepancies</CardTitle>
            {(data?.discrepancy_count ?? 0) > 0 ? (
              <XCircle className="h-4 w-4 text-red-500" />
            ) : (
              <CheckCircle2 className="h-4 w-4 text-green-500" />
            )}
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${(data?.discrepancy_count ?? 0) > 0 ? 'text-red-600' : 'text-green-600'}`}>
              {data?.discrepancy_count ?? 0}
            </div>
            <p className="text-xs text-muted-foreground">Requires investigation</p>
          </CardContent>
        </Card>
      </div>

      {/* Account Details Table */}
      <Card>
        <CardHeader>
          <CardTitle>Account Details</CardTitle>
          <CardDescription>
            Comparison of stored vs calculated balances for all active accounts
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Code</TableHead>
                <TableHead>Account Name</TableHead>
                <TableHead className="text-right">Stored Balance</TableHead>
                <TableHead className="text-right">Calculated Balance</TableHead>
                <TableHead className="text-right">Difference</TableHead>
                <TableHead>Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data?.accounts?.map((account) => (
                <TableRow key={account.account_id}>
                  <TableCell className="font-mono">{account.account_code}</TableCell>
                  <TableCell>{account.account_name}</TableCell>
                  <TableCell className="text-right font-mono">
                    {parseFloat(account.stored_balance).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                  </TableCell>
                  <TableCell className="text-right font-mono">
                    {parseFloat(account.calculated_balance).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                  </TableCell>
                  <TableCell className="text-right font-mono">
                    {parseFloat(account.difference).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                  </TableCell>
                  <TableCell>
                    <Badge
                      variant={
                        account.status === 'Matched'
                          ? 'default'
                          : account.status === 'WithinTolerance'
                          ? 'secondary'
                          : 'destructive'
                      }
                    >
                      {account.status}
                    </Badge>
                  </TableCell>
                </TableRow>
              ))}
              {(!data?.accounts || data.accounts.length === 0) && (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                    No account data available
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
