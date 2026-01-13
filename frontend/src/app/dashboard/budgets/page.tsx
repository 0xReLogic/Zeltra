'use client'

import React, { useState } from 'react'
import { 
  BarChart, 
  Bar, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  Legend
} from 'recharts'
import { Plus, AlertCircle, TrendingDown, TrendingUp, DollarSign, Lock } from 'lucide-react'
import { useBudgets, useCreateBudget } from '@/lib/queries/budgets'
import { useFiscalYears } from '@/lib/queries/fiscal'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
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
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { toast } from 'sonner'
import Link from 'next/link'
import type { CreateBudgetRequest } from '@/types/budgets'

export default function BudgetsPage() {
  const { data } = useBudgets()
  const { data: fiscalYears } = useFiscalYears()
  const createBudget = useCreateBudget()
  const [isOpen, setIsOpen] = useState(false)
  const [formData, setFormData] = useState<Partial<CreateBudgetRequest>>({
    budget_type: 'annual',
  })

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    if (!formData.name || !formData.fiscal_year_id || !formData.budget_type) {
      toast.error('Please fill all required fields')
      return
    }
    
    createBudget.mutate(formData as CreateBudgetRequest, {
      onSuccess: () => {
        toast.success('Budget created successfully')
        setIsOpen(false)
        setFormData({ budget_type: 'annual' })
      },
      onError: () => {
        toast.error('Failed to create budget')
      }
    })
  }

  const budgets = Array.isArray(data) ? data : []
  const fiscalYearList = Array.isArray(fiscalYears) ? fiscalYears : []

  // Calculate totals - handle both old and new API format
  const totalBudget = budgets.reduce((acc, curr) => {
    const amount = parseFloat(curr.total_budgeted || '0')
    return acc + (isNaN(amount) ? 0 : amount)
  }, 0)
  
  const chartData = budgets.map(b => ({
    name: b.name,
    Budget: parseFloat(b.total_budgeted || '0'),
  }))

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Budgets</h1>
          <p className="text-muted-foreground mt-2">
            Manage and monitor your organization&apos;s budgets.
          </p>
        </div>
        <Dialog open={isOpen} onOpenChange={setIsOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" /> New Budget
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create New Budget</DialogTitle>
              <DialogDescription>Create a budget for a fiscal year</DialogDescription>
            </DialogHeader>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="name">Budget Name</Label>
                <Input 
                  id="name" 
                  placeholder="e.g. FY2026 Operations" 
                  value={formData.name || ''}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                  required 
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="fiscal_year_id">Fiscal Year</Label>
                <Select 
                  value={formData.fiscal_year_id || ''} 
                  onValueChange={(value) => setFormData(prev => ({ ...prev, fiscal_year_id: value }))}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select fiscal year" />
                  </SelectTrigger>
                  <SelectContent>
                    {fiscalYearList.map((fy) => (
                      <SelectItem key={fy.id} value={fy.id}>{fy.name}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="budget_type">Budget Type</Label>
                <Select 
                  value={formData.budget_type || 'annual'} 
                  onValueChange={(value) => setFormData(prev => ({ ...prev, budget_type: value }))}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select type" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="annual">Annual</SelectItem>
                    <SelectItem value="quarterly">Quarterly</SelectItem>
                    <SelectItem value="monthly">Monthly</SelectItem>
                    <SelectItem value="project">Project</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="description">Description (Optional)</Label>
                <Textarea 
                  id="description" 
                  placeholder="Budget description..."
                  value={formData.description || ''}
                  onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
                />
              </div>
              <DialogFooter>
                <Button type="submit" disabled={createBudget.isPending}>
                  {createBudget.isPending ? 'Creating...' : 'Create Budget'}
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
            <CardTitle className="text-sm font-medium">Total Budgets</CardTitle>
            <DollarSign className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{budgets.length}</div>
            <p className="text-xs text-muted-foreground">Active budgets</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Budgeted</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              ${totalBudget.toLocaleString('en-US', { minimumFractionDigits: 2 })}
            </div>
            <p className="text-xs text-muted-foreground">Across all budgets</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Locked</CardTitle>
            <Lock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {budgets.filter(b => b.is_locked).length}
            </div>
            <p className="text-xs text-muted-foreground">Locked budgets</p>
          </CardContent>
        </Card>
      </div>

      {/* Main Content Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        
        {/* Chart */}
        <Card className="col-span-4">
          <CardHeader>
            <CardTitle>Budget Overview</CardTitle>
          </CardHeader>
          <CardContent className="pl-2">
            <ResponsiveContainer width="100%" height={350}>
              <BarChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" vertical={false} />
                <XAxis 
                  dataKey="name" 
                  stroke="#888888" 
                  fontSize={12} 
                  tickLine={false} 
                  axisLine={false} 
                />
                <YAxis
                  stroke="#888888"
                  fontSize={12}
                  tickLine={false}
                  axisLine={false}
                  tickFormatter={(value) => `$${Number(value).toLocaleString()}`}
                />
                <Tooltip 
                  cursor={{ fill: 'transparent' }}
                  formatter={(value) => [`$${Number(value).toLocaleString()}`, 'Budget']}
                />
                <Legend />
                <Bar dataKey="Budget" fill="#34d399" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* List Details */}
        <Card className="col-span-3">
          <CardHeader>
            <CardTitle>Budget List</CardTitle>
            <CardDescription>All budgets in your organization</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {budgets.length === 0 ? (
                <p className="text-center text-muted-foreground py-8">
                  No budgets found. Create your first budget to get started.
                </p>
              ) : (
                budgets.map((item) => {
                  const amount = parseFloat(item.total_budgeted || '0')
                  
                  return (
                    <Link 
                      href={`/dashboard/budgets/${item.id}`} 
                      key={item.id} 
                      className="block space-y-2 hover:bg-muted/50 -mx-4 px-4 py-3 rounded-lg transition-colors border-b last:border-0"
                    >
                      <div className="flex items-center justify-between">
                        <div className="font-semibold">{item.name}</div>
                        <div className="flex items-center gap-2">
                          {item.is_locked && (
                            <Badge variant="secondary">
                              <Lock className="h-3 w-3 mr-1" /> Locked
                            </Badge>
                          )}
                          <Badge variant="outline">{item.budget_type}</Badge>
                        </div>
                      </div>
                      <div className="flex items-center justify-between text-sm text-muted-foreground">
                        <span>{item.fiscal_year_name}</span>
                        <span className="font-medium">
                          ${amount.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                        </span>
                      </div>
                    </Link>
                  )
                })
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
