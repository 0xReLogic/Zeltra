'use client'

import React, { useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Plus, ArrowLeft, Lock, Loader2, BarChart3 } from 'lucide-react'
import Link from 'next/link'
import { useBudget, useCreateBudgetLines, useLockBudget, useBudgetVsActual } from '@/lib/queries/budgets'
import { useAccounts } from '@/lib/queries/accounts'
import { useFiscalPeriods } from '@/lib/queries/fiscal'
import { formatCurrency } from '@/lib/utils/format'
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
} from "@/components/ui/select"
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from 'sonner'

export default function BudgetDetailPage() {
  const params = useParams()
  const router = useRouter()
  const id = params.id as string
  
  const { data: budget, isLoading } = useBudget(id)
  const { data: vsActual } = useBudgetVsActual(id)
  const { data: accountsData } = useAccounts()
  const { data: periodsData } = useFiscalPeriods(budget?.fiscal_year_id)
  
  const createLines = useCreateBudgetLines()
  const lockBudget = useLockBudget()
  
  const [isAddOpen, setIsAddOpen] = useState(false)
  const [showVsActual, setShowVsActual] = useState(false)
  const [lineForm, setLineForm] = useState({
    account_id: '',
    period_id: '',
    amount: '',
  })

  const accounts = accountsData?.accounts ?? []
  const periods = Array.isArray(periodsData) ? periodsData : []
  const lines = vsActual?.line_items ?? []

  const handleAddLine = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    if (!lineForm.account_id || !lineForm.period_id || !lineForm.amount) {
      toast.error('Please fill all required fields')
      return
    }

    createLines.mutate({
      budgetId: id,
      data: {
        lines: [{
          account_id: lineForm.account_id,
          fiscal_period_id: lineForm.period_id,
          amount: lineForm.amount,
        }]
      }
    }, {
      onSuccess: () => {
        toast.success('Budget line added')
        setIsAddOpen(false)
        setLineForm({ account_id: '', period_id: '', amount: '' })
      },
      onError: () => toast.error('Failed to add budget line')
    })
  }
  
  const handleLock = () => {
    lockBudget.mutate(id, {
      onSuccess: () => toast.success('Budget locked successfully'),
      onError: () => toast.error('Failed to lock budget')
    })
  }

  if (isLoading) {
    return (
      <div className="flex h-96 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }
  
  if (!budget) {
    return (
      <div className="flex flex-col items-center justify-center h-96 space-y-4">
        <h2 className="text-xl font-semibold">Budget not found</h2>
        <Button variant="outline" onClick={() => router.back()}>
          <ArrowLeft className="mr-2 h-4 w-4" /> Go Back
        </Button>
      </div>
    )
  }

  const totalBudgeted = parseFloat(budget.total_budgeted || '0')
  const isLocked = budget.is_locked

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Link href="/dashboard/budgets">
            <Button variant="ghost" size="icon">
              <ArrowLeft className="h-4 w-4" />
            </Button>
          </Link>
          <div>
            <h1 className="text-3xl font-bold tracking-tight flex items-center gap-2">
              {budget.name}
              {isLocked && <Lock className="h-5 w-5 text-muted-foreground" />}
            </h1>
            <p className="text-muted-foreground">{budget.fiscal_year_name} • {budget.budget_type}</p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button 
            variant="outline" 
            onClick={() => setShowVsActual(!showVsActual)}
          >
            <BarChart3 className="h-4 w-4 mr-2" />
            {showVsActual ? 'Hide' : 'Show'} vs Actual
          </Button>
          {!isLocked && (
            <Button variant="outline" onClick={handleLock} disabled={lockBudget.isPending}>
              {lockBudget.isPending ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <>
                  <Lock className="h-4 w-4 mr-2" />
                  Lock Budget
                </>
              )}
            </Button>
          )}
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Total Budgeted</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {formatCurrency(totalBudgeted, budget.currency)}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Budget Type</CardTitle>
          </CardHeader>
          <CardContent>
            <Badge variant="outline" className="text-lg capitalize">
              {budget.budget_type}
            </Badge>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Status</CardTitle>
          </CardHeader>
          <CardContent>
            <Badge variant={isLocked ? 'secondary' : 'default'}>
              {isLocked ? 'Locked' : 'Open'}
            </Badge>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Line Items</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{lines.length}</div>
          </CardContent>
        </Card>
      </div>

      {/* Budget vs Actual Section */}
      {showVsActual && vsActual && (
        <Card>
          <CardHeader>
            <CardTitle>Budget vs Actual</CardTitle>
            <CardDescription>Comparison of budgeted amounts vs actual spending</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 md:grid-cols-3">
              <div className="text-center p-4 bg-muted rounded-lg">
                <p className="text-sm text-muted-foreground">Total Budgeted</p>
                <p className="text-2xl font-bold">{formatCurrency(parseFloat(vsActual.summary?.total_budgeted || '0'), budget.currency)}</p>
              </div>
              <div className="text-center p-4 bg-muted rounded-lg">
                <p className="text-sm text-muted-foreground">Total Actual</p>
                <p className="text-2xl font-bold">{formatCurrency(parseFloat(vsActual.summary?.total_actual || '0'), budget.currency)}</p>
              </div>
              <div className="text-center p-4 bg-muted rounded-lg">
                <p className="text-sm text-muted-foreground">Variance</p>
                <p className={`text-2xl font-bold ${parseFloat(vsActual.summary?.variance || '0') >= 0 ? 'text-emerald-600' : 'text-red-600'}`}>
                  {formatCurrency(parseFloat(vsActual.summary?.variance || '0'), budget.currency)}
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>Budget Lines</CardTitle>
            <CardDescription>Allocation per account</CardDescription>
          </div>
          {!isLocked && (
            <Dialog open={isAddOpen} onOpenChange={setIsAddOpen}>
              <DialogTrigger asChild>
                <Button>
                  <Plus className="mr-2 h-4 w-4" /> Add Line Item
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Add Budget Line</DialogTitle>
                  <DialogDescription>Allocate budget for a specific account and period</DialogDescription>
                </DialogHeader>
                <form onSubmit={handleAddLine} className="space-y-4">
                  <div className="space-y-2">
                    <Label>Account</Label>
                    <Select 
                      value={lineForm.account_id} 
                      onValueChange={(v) => setLineForm(prev => ({ ...prev, account_id: v }))}
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
                    <Label>Period</Label>
                    <Select 
                      value={lineForm.period_id} 
                      onValueChange={(v) => setLineForm(prev => ({ ...prev, period_id: v }))}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select period" />
                      </SelectTrigger>
                      <SelectContent>
                        {periods.map((period) => (
                          <SelectItem key={period.id} value={period.id}>
                            {period.name}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label>Amount</Label>
                    <Input 
                      type="number" 
                      step="0.01" 
                      placeholder="0.00"
                      value={lineForm.amount}
                      onChange={(e) => setLineForm(prev => ({ ...prev, amount: e.target.value }))}
                      required 
                    />
                  </div>
                  <DialogFooter>
                    <Button type="submit" disabled={createLines.isPending}>
                      {createLines.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                      Add Line
                    </Button>
                  </DialogFooter>
                </form>
              </DialogContent>
            </Dialog>
          )}
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Account</TableHead>
                <TableHead className="text-right">Budgeted</TableHead>
                <TableHead className="text-right">Actual</TableHead>
                <TableHead className="text-right">Variance</TableHead>
                <TableHead className="text-right">Utilization</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {lines.map((line, idx) => {
                const budgeted = parseFloat(line.budgeted || '0')
                const actual = parseFloat(line.actual || '0')
                const variance = parseFloat(line.variance || '0')
                const percent = budgeted > 0 ? (actual / budgeted) * 100 : 0
                const isFavorable = variance >= 0
                
                return (
                  <TableRow key={`${line.account_id}-${idx}`}>
                    <TableCell className="font-medium">
                      {line.account_code} - {line.account_name}
                    </TableCell>
                    <TableCell className="text-right">
                      {formatCurrency(budgeted, budget.currency)}
                    </TableCell>
                    <TableCell className="text-right">
                      {formatCurrency(actual, budget.currency)}
                    </TableCell>
                    <TableCell className={`text-right font-medium ${isFavorable ? 'text-emerald-600' : 'text-red-600'}`}>
                      {isFavorable ? '+' : ''}{formatCurrency(variance, budget.currency)}
                    </TableCell>
                    <TableCell className="text-right w-[200px]">
                      <div className="flex items-center justify-end gap-2">
                        <span className={`text-xs w-[40px] text-right ${percent > 100 ? 'text-red-600 font-medium' : 'text-muted-foreground'}`}>
                          {percent.toFixed(0)}%
                        </span>
                        <Progress 
                          value={Math.min(percent, 100)} 
                          className={`h-2 w-[100px] ${percent > 100 ? '[&>div]:bg-red-500' : ''}`} 
                        />
                      </div>
                    </TableCell>
                  </TableRow>
                )
              })}
              {lines.length === 0 && (
                <TableRow>
                  <TableCell colSpan={5} className="text-center text-muted-foreground h-24">
                    No budget lines added yet.
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
