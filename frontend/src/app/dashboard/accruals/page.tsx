'use client'

import React, { useState } from 'react'
import { Plus, CalendarClock, DollarSign, Clock, CheckCircle2, Loader2, Lock } from 'lucide-react'
import { useAccrualSchedules, useCreateAccrualSchedule } from '@/lib/queries/sentinel'
import { useAccounts } from '@/lib/queries/accounts'
import { useOrganization } from '@/lib/queries/organizations'
import { useUpgradeStore } from '@/lib/stores/upgradeStore'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
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
import { Textarea } from '@/components/ui/textarea'
import { Skeleton } from '@/components/ui/skeleton'
import { toast } from 'sonner'
import type { CreateAccrualScheduleRequest } from '@/types/api-helpers'

export default function AccrualsPage() {
  const { data: org } = useOrganization()
  const { openModal } = useUpgradeStore()
  const { data: schedules, isLoading, isError } = useAccrualSchedules()
  const { data: accountsData } = useAccounts()
  const createAccrual = useCreateAccrualSchedule()
  
  const [isOpen, setIsOpen] = useState(false)
  const [formData, setFormData] = useState<Partial<CreateAccrualScheduleRequest>>({
    frequency: 'monthly',
    currency_id: 'USD',
  })

  // Check tier access
  const hasAccruals = org?.limits?.has_auto_accruals ?? false

  // Show loading state first
  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <Skeleton className="h-9 w-48" />
          <Skeleton className="h-10 w-32" />
        </div>
        <div className="grid gap-4 md:grid-cols-3">
          {[1, 2, 3].map(i => (
            <Card key={i}>
              <CardHeader className="pb-2">
                <Skeleton className="h-4 w-24" />
              </CardHeader>
              <CardContent>
                <Skeleton className="h-8 w-16" />
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
  if (org && !hasAccruals) {
    return (
      <div className="space-y-6">
        <h1 className="text-3xl font-bold tracking-tight">Accruals Management</h1>
        <Card className="border-amber-500/50">
          <CardContent className="flex flex-col items-center justify-center py-16 text-center">
            <div className="rounded-full bg-amber-500/10 p-4 mb-4">
              <Lock className="h-8 w-8 text-amber-500" />
            </div>
            <h2 className="text-xl font-semibold mb-2">Enterprise Feature</h2>
            <p className="text-muted-foreground mb-6 max-w-md">
              Automated Accruals is an Enterprise feature that helps you automate 
              recurring journal entries for prepaid expenses and deferred revenue.
            </p>
            <Button onClick={() => openModal('Unlock Automated Accruals and other Enterprise features.')}>
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
        <h1 className="text-3xl font-bold tracking-tight">Accruals Management</h1>
        <Card className="border-destructive/50">
          <CardContent className="flex flex-col items-center justify-center py-16 text-center">
            <p className="text-destructive mb-4">Failed to load accrual schedules</p>
            <Button variant="outline" onClick={() => window.location.reload()}>
              Try Again
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  const accounts = accountsData?.accounts ?? []
  const scheduleList = Array.isArray(schedules) ? schedules : []

  // Calculate summary stats
  const activeSchedules = scheduleList.filter(s => s.status === 'active').length
  const completedSchedules = scheduleList.filter(s => s.status === 'completed').length
  const totalAmount = scheduleList.reduce((acc, s) => acc + parseFloat(s.total_amount || '0'), 0)

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    
    if (!formData.name || !formData.total_amount || !formData.debit_account_id || 
        !formData.credit_account_id || !formData.start_date || !formData.end_date || 
        !formData.total_periods) {
      toast.error('Please fill all required fields')
      return
    }

    createAccrual.mutate(formData as CreateAccrualScheduleRequest, {
      onSuccess: () => {
        toast.success('Accrual schedule created successfully')
        setIsOpen(false)
        setFormData({ frequency: 'monthly', currency_id: 'USD' })
      },
      onError: (error) => {
        toast.error(error.message || 'Failed to create accrual schedule')
      }
    })
  }

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'active':
        return <Badge className="bg-green-500/10 text-green-600 hover:bg-green-500/20">Active</Badge>
      case 'completed':
        return <Badge variant="secondary">Completed</Badge>
      case 'paused':
        return <Badge variant="outline">Paused</Badge>
      default:
        return <Badge variant="outline">{status}</Badge>
    }
  }

  const formatCurrency = (amount: string) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(parseFloat(amount || '0'))
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Accruals Management</h1>
          <p className="text-muted-foreground mt-1">
            Automate recurring journal entries for prepaid expenses and deferred revenue.
          </p>
        </div>
        <Dialog open={isOpen} onOpenChange={setIsOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" /> Create Schedule
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[500px]">
            <DialogHeader>
              <DialogTitle>Create Accrual Schedule</DialogTitle>
              <DialogDescription>
                Set up an automated accrual schedule for recurring journal entries.
              </DialogDescription>
            </DialogHeader>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="name">Schedule Name *</Label>
                <Input
                  id="name"
                  placeholder="e.g. Prepaid Insurance 2026"
                  value={formData.name || ''}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                  required
                />
              </div>
              
              <div className="space-y-2">
                <Label htmlFor="description">Description</Label>
                <Textarea
                  id="description"
                  placeholder="Optional description..."
                  value={formData.description || ''}
                  onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="total_amount">Total Amount *</Label>
                  <Input
                    id="total_amount"
                    type="number"
                    step="0.01"
                    placeholder="12000.00"
                    value={formData.total_amount || ''}
                    onChange={(e) => setFormData(prev => ({ ...prev, total_amount: e.target.value }))}
                    required
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="currency_id">Currency</Label>
                  <Select
                    value={formData.currency_id || 'USD'}
                    onValueChange={(value) => setFormData(prev => ({ ...prev, currency_id: value }))}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="USD">USD</SelectItem>
                      <SelectItem value="EUR">EUR</SelectItem>
                      <SelectItem value="IDR">IDR</SelectItem>
                      <SelectItem value="SGD">SGD</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="debit_account_id">Debit Account *</Label>
                  <Select
                    value={formData.debit_account_id || ''}
                    onValueChange={(value) => setFormData(prev => ({ ...prev, debit_account_id: value }))}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select account" />
                    </SelectTrigger>
                    <SelectContent>
                      {accounts.map((acc) => (
                        <SelectItem key={acc.id} value={acc.id}>
                          {acc.code} - {acc.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="credit_account_id">Credit Account *</Label>
                  <Select
                    value={formData.credit_account_id || ''}
                    onValueChange={(value) => setFormData(prev => ({ ...prev, credit_account_id: value }))}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select account" />
                    </SelectTrigger>
                    <SelectContent>
                      {accounts.map((acc) => (
                        <SelectItem key={acc.id} value={acc.id}>
                          {acc.code} - {acc.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="start_date">Start Date *</Label>
                  <Input
                    id="start_date"
                    type="date"
                    value={formData.start_date || ''}
                    onChange={(e) => setFormData(prev => ({ ...prev, start_date: e.target.value }))}
                    required
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="end_date">End Date *</Label>
                  <Input
                    id="end_date"
                    type="date"
                    value={formData.end_date || ''}
                    onChange={(e) => setFormData(prev => ({ ...prev, end_date: e.target.value }))}
                    required
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="frequency">Frequency *</Label>
                  <Select
                    value={formData.frequency || 'monthly'}
                    onValueChange={(value) => setFormData(prev => ({ ...prev, frequency: value }))}
                  >
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="daily">Daily</SelectItem>
                      <SelectItem value="weekly">Weekly</SelectItem>
                      <SelectItem value="monthly">Monthly</SelectItem>
                      <SelectItem value="quarterly">Quarterly</SelectItem>
                      <SelectItem value="yearly">Yearly</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="total_periods">Total Periods *</Label>
                  <Input
                    id="total_periods"
                    type="number"
                    min="1"
                    placeholder="12"
                    value={formData.total_periods || ''}
                    onChange={(e) => setFormData(prev => ({ ...prev, total_periods: parseInt(e.target.value) || undefined }))}
                    required
                  />
                </div>
              </div>

              <DialogFooter>
                <Button type="button" variant="outline" onClick={() => setIsOpen(false)}>
                  Cancel
                </Button>
                <Button type="submit" disabled={createAccrual.isPending}>
                  {createAccrual.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  Create Schedule
                </Button>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Schedules</CardTitle>
            <CalendarClock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{activeSchedules}</div>
            <p className="text-xs text-muted-foreground">Currently running</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Amount</CardTitle>
            <DollarSign className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatCurrency(totalAmount.toString())}</div>
            <p className="text-xs text-muted-foreground">Across all schedules</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Completed</CardTitle>
            <CheckCircle2 className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{completedSchedules}</div>
            <p className="text-xs text-muted-foreground">Finished schedules</p>
          </CardContent>
        </Card>
      </div>

      {/* Schedules Table */}
      <Card>
        <CardHeader>
          <CardTitle>Accrual Schedules</CardTitle>
          <CardDescription>Manage your automated accrual schedules</CardDescription>
        </CardHeader>
        <CardContent>
          {scheduleList.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <CalendarClock className="h-12 w-12 text-muted-foreground mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Accrual Schedules</h3>
              <p className="text-muted-foreground mb-4 max-w-sm">
                Create your first accrual schedule to automate recurring journal entries.
              </p>
              <Button onClick={() => setIsOpen(true)}>
                <Plus className="mr-2 h-4 w-4" /> Create Schedule
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Total Amount</TableHead>
                  <TableHead>Progress</TableHead>
                  <TableHead>Frequency</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Next Run</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {scheduleList.map((schedule) => {
                  const progress = (schedule.periods_processed / schedule.total_periods) * 100
                  return (
                    <TableRow key={schedule.id} className="cursor-pointer hover:bg-muted/50">
                      <TableCell>
                        <div>
                          <div className="font-medium">{schedule.name}</div>
                          {schedule.description && (
                            <div className="text-sm text-muted-foreground truncate max-w-[200px]">
                              {schedule.description}
                            </div>
                          )}
                        </div>
                      </TableCell>
                      <TableCell className="font-medium">
                        {formatCurrency(schedule.total_amount)}
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <Progress value={progress} className="w-20 h-2" />
                          <span className="text-sm text-muted-foreground">
                            {schedule.periods_processed}/{schedule.total_periods}
                          </span>
                        </div>
                      </TableCell>
                      <TableCell className="capitalize">{schedule.frequency}</TableCell>
                      <TableCell>{getStatusBadge(schedule.status)}</TableCell>
                      <TableCell>
                        {schedule.next_run_date ? (
                          <div className="flex items-center gap-1 text-sm">
                            <Clock className="h-3 w-3" />
                            {schedule.next_run_date}
                          </div>
                        ) : (
                          <span className="text-muted-foreground">-</span>
                        )}
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
