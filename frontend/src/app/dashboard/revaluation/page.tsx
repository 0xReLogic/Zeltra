'use client'

import React, { useState } from 'react'
import { Scale, TrendingUp, TrendingDown, Calendar, Lock, RefreshCw } from 'lucide-react'
import { useRevaluationLogs } from '@/lib/queries/sentinel'
import { useAccounts } from '@/lib/queries/accounts'
import { useOrganization } from '@/lib/queries/organizations'
import { useUpgradeStore } from '@/lib/stores/upgradeStore'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Skeleton } from '@/components/ui/skeleton'
import { cn } from '@/lib/utils'

export default function RevaluationPage() {
  const { data: org } = useOrganization()
  const { openModal } = useUpgradeStore()
  const { data: accountsData } = useAccounts()
  
  // Date range filter state
  const [fromDate, setFromDate] = useState<string>('')
  const [toDate, setToDate] = useState<string>('')
  
  const { data: logs, isLoading, isError, refetch } = useRevaluationLogs({
    from: fromDate || undefined,
    to: toDate || undefined,
  })

  // Check tier access
  const hasMultiCurrency = org?.limits?.has_multi_currency ?? false

  // Show loading state first
  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <Skeleton className="h-9 w-48" />
        </div>
        <div className="grid gap-4 md:grid-cols-3">
          {[1, 2, 3].map(i => (
            <Card key={i}>
              <CardHeader className="pb-2">
                <Skeleton className="h-4 w-24" />
              </CardHeader>
              <CardContent>
                <Skeleton className="h-8 w-20" />
              </CardContent>
            </Card>
          ))}
        </div>
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-32" />
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {[1, 2, 3].map(i => (
                <Skeleton key={i} className="h-12 w-full" />
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  // Show upgrade prompt if tier not available (check after loading)
  if (org && !hasMultiCurrency) {
    return (
      <div className="space-y-6">
        <h1 className="text-3xl font-bold tracking-tight">Currency Revaluation</h1>
        <Card className="border-amber-500/50">
          <CardContent className="flex flex-col items-center justify-center py-16 text-center">
            <div className="rounded-full bg-amber-500/10 p-4 mb-4">
              <Lock className="h-8 w-8 text-amber-500" />
            </div>
            <h2 className="text-xl font-semibold mb-2">Enterprise Feature</h2>
            <p className="text-muted-foreground mb-6 max-w-md">
              Currency Revaluation is an Enterprise feature that helps you track 
              unrealized gains and losses from exchange rate fluctuations.
            </p>
            <Button onClick={() => openModal('Unlock Currency Revaluation and other Enterprise features.')}>
              Upgrade to Enterprise
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  // Show error state
  if (isError) {
    return (
      <div className="space-y-6">
        <h1 className="text-3xl font-bold tracking-tight">Currency Revaluation</h1>
        <Card className="border-destructive/50">
          <CardContent className="flex flex-col items-center justify-center py-16 text-center">
            <p className="text-destructive mb-4">Failed to load revaluation logs</p>
            <Button variant="outline" onClick={() => refetch()}>
              Try Again
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  const accounts = accountsData?.accounts ?? []
  const logList = Array.isArray(logs) ? logs : []

  // Calculate summary stats
  const totalGains = logList
    .filter(l => parseFloat(l.gain_loss_amount) > 0)
    .reduce((acc, l) => acc + parseFloat(l.gain_loss_amount), 0)
  
  const totalLosses = logList
    .filter(l => parseFloat(l.gain_loss_amount) < 0)
    .reduce((acc, l) => acc + Math.abs(parseFloat(l.gain_loss_amount)), 0)
  
  const netGainLoss = totalGains - totalLosses

  // Helper to get account name by ID
  const getAccountName = (accountId: string) => {
    const account = accounts.find(a => a.id === accountId)
    return account ? `${account.code} - ${account.name}` : accountId
  }

  const formatCurrency = (amount: string | number) => {
    const num = typeof amount === 'string' ? parseFloat(amount) : amount
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      signDisplay: 'always',
    }).format(num)
  }

  const formatRate = (rate: string) => {
    return parseFloat(rate).toFixed(6)
  }

  const handleClearFilters = () => {
    setFromDate('')
    setToDate('')
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Currency Revaluation</h1>
          <p className="text-muted-foreground mt-1">
            Track unrealized gains and losses from exchange rate fluctuations.
          </p>
        </div>
        <Button variant="outline" onClick={() => refetch()}>
          <RefreshCw className="mr-2 h-4 w-4" /> Refresh
        </Button>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Gains</CardTitle>
            <TrendingUp className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">
              {formatCurrency(totalGains)}
            </div>
            <p className="text-xs text-muted-foreground">Unrealized gains</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Losses</CardTitle>
            <TrendingDown className="h-4 w-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">
              {formatCurrency(-totalLosses)}
            </div>
            <p className="text-xs text-muted-foreground">Unrealized losses</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Net Position</CardTitle>
            <Scale className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className={cn(
              "text-2xl font-bold",
              netGainLoss >= 0 ? "text-green-600" : "text-red-600"
            )}>
              {formatCurrency(netGainLoss)}
            </div>
            <p className="text-xs text-muted-foreground">Net gain/loss</p>
          </CardContent>
        </Card>
      </div>

      {/* Date Range Filter */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Filter by Date Range</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap items-end gap-4">
            <div className="space-y-2">
              <Label htmlFor="from_date">From Date</Label>
              <Input
                id="from_date"
                type="date"
                value={fromDate}
                onChange={(e) => setFromDate(e.target.value)}
                className="w-40"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="to_date">To Date</Label>
              <Input
                id="to_date"
                type="date"
                value={toDate}
                onChange={(e) => setToDate(e.target.value)}
                className="w-40"
              />
            </div>
            {(fromDate || toDate) && (
              <Button variant="ghost" size="sm" onClick={handleClearFilters}>
                Clear Filters
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Revaluation Logs Table */}
      <Card>
        <CardHeader>
          <CardTitle>Revaluation History</CardTitle>
          <CardDescription>
            Historical record of currency revaluations and their impact
          </CardDescription>
        </CardHeader>
        <CardContent>
          {logList.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <Scale className="h-12 w-12 text-muted-foreground mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Revaluation Logs</h3>
              <p className="text-muted-foreground mb-4 max-w-sm">
                No currency revaluations have been recorded yet. Revaluations are 
                automatically generated for accounts with non-functional currencies.
              </p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Date</TableHead>
                  <TableHead>Account</TableHead>
                  <TableHead>Currency</TableHead>
                  <TableHead className="text-right">Exchange Rate</TableHead>
                  <TableHead className="text-right">Carrying Balance</TableHead>
                  <TableHead className="text-right">Gain/Loss</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {logList.map((log) => {
                  const gainLoss = parseFloat(log.gain_loss_amount)
                  const isGain = gainLoss >= 0
                  
                  return (
                    <TableRow key={log.id}>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <Calendar className="h-4 w-4 text-muted-foreground" />
                          {log.revaluation_date}
                        </div>
                      </TableCell>
                      <TableCell>
                        <div className="font-medium">
                          {getAccountName(log.account_id)}
                        </div>
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline">{log.source_currency}</Badge>
                      </TableCell>
                      <TableCell className="text-right font-mono">
                        {formatRate(log.exchange_rate)}
                      </TableCell>
                      <TableCell className="text-right font-mono">
                        {formatCurrency(log.carrying_balance)}
                      </TableCell>
                      <TableCell className="text-right">
                        <span className={cn(
                          "font-semibold",
                          isGain ? "text-green-600" : "text-red-600"
                        )}>
                          {formatCurrency(gainLoss)}
                        </span>
                      </TableCell>
                    </TableRow>
                  )
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
