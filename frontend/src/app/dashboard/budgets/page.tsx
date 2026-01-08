'use client'

import React from 'react'
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
// ... imports
import { Plus, AlertCircle, TrendingDown, TrendingUp, DollarSign } from 'lucide-react'
import { useBudgets, useCreateBudget } from '@/lib/queries/budgets'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useState } from 'react'
import { toast } from 'sonner'
import Link from 'next/link'

export default function BudgetsPage() {
  const { data, isLoading } = useBudgets()
  const createBudget = useCreateBudget()
  const [isOpen, setIsOpen] = useState(false)

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
     e.preventDefault()
     const formData = new FormData(e.currentTarget)
     const department = formData.get('department') as string
     const budget_limit = formData.get('budget_limit') as string
     const period = formData.get('period') as string
     
     createBudget.mutate({ department, budget_limit, period }, {
         onSuccess: () => {
             toast.success('Budget created successfully')
             setIsOpen(false)
         },
         onError: () => {
             toast.error('Failed to create budget')
         }
     })
  }

  const budgets = data?.data || []

  // ... existing calculation logic ...
  const totalBudget = budgets.reduce((acc, curr) => acc + parseFloat(curr.budget_limit), 0)
  const totalSpent = budgets.reduce((acc, curr) => acc + parseFloat(curr.actual_spent), 0)
  const totalVariance = totalBudget - totalSpent
  const spentPercentage = totalBudget > 0 ? (totalSpent / totalBudget) * 100 : 0
  
  const chartData = budgets.map(b => ({
    name: b.department,
    Budget: parseFloat(b.budget_limit),
    Actual: parseFloat(b.actual_spent)
  }))

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Budget vs Actual</h1>
          <p className="text-muted-foreground mt-2">
            Monitor department spending against allocated budgets.
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
                    <DialogDescription>allocate budget for a department</DialogDescription>
                </DialogHeader>
                <form onSubmit={handleSubmit} className="space-y-4">
                    <div className="space-y-2">
                        <Label htmlFor="department">Department</Label>
                        <Input id="department" name="department" placeholder="e.g. Engineering" required />
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="period">Fiscal Period</Label>
                        <Input id="period" name="period" type="month" defaultValue="2026-01" required />
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="budget_limit">Total Limit ($)</Label>
                        <Input id="budget_limit" name="budget_limit" type="number" placeholder="50000" required />
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
            <CardTitle className="text-sm font-medium">Total Budget</CardTitle>
            <DollarSign className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              ${totalBudget.toLocaleString('en-US')}
            </div>
            <p className="text-xs text-muted-foreground">
              For current period (Jan 2026)
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Spent</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${spentPercentage > 100 ? 'text-red-500' : ''}`}>
              ${totalSpent.toLocaleString('en-US')}
            </div>
            <div className="mt-2">
               <Progress value={Math.min(spentPercentage, 100)} className="h-2" />
               <p className="text-xs text-muted-foreground mt-1">
                 {spentPercentage.toFixed(1)}% used
               </p>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Variance</CardTitle>
            <TrendingDown className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${totalVariance < 0 ? 'text-red-500' : 'text-emerald-500'}`}>
              ${Math.abs(totalVariance).toLocaleString('en-US')}
            </div>
            <p className="text-xs text-muted-foreground">
              {totalVariance >= 0 ? 'Remaining' : 'Overrun'}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Alerts for Overspending */}
      {budgets.some(b => parseFloat(b.actual_spent) > parseFloat(b.budget_limit)) && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>Budget Overrun Alert</AlertTitle>
          <AlertDescription>
            One or more departments have exceeded their budget limit. Please review immediately.
          </AlertDescription>
        </Alert>
      )}

      {/* Main Content Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        
        {/* Chart */}
        <Card className="col-span-4">
          <CardHeader>
            <CardTitle>Overview by Department</CardTitle>
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
                  tickFormatter={(value) => `$${value}`}
                />
                <Tooltip 
                    cursor={{ fill: 'transparent' }}
                    formatter={(value) => [`$${Number(value).toLocaleString()}`, '']}
                />
                <Legend />
                <Bar dataKey="Budget" fill="#34d399" radius={[4, 4, 0, 0]} />
                <Bar dataKey="Actual" fill="#f87171" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* List Details */}
        <Card className="col-span-3">
          <CardHeader>
            <CardTitle>Department Breakdown</CardTitle>
            <CardDescription>
              Detailed view of spending vs limits.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-8">
              {budgets.map((item) => {
                 const limit = parseFloat(item.budget_limit)
                 const spent = parseFloat(item.actual_spent)
                 const percentage = (spent / limit) * 100
                 const isOver = spent > limit

                 return (
                   <Link href={`/dashboard/budgets/${item.id}`} key={item.id} className="block space-y-2 hover:bg-muted/50 -mx-4 px-4 py-2 rounded-lg transition-colors">
                      <div className="flex items-center justify-between">
                        <div className="font-semibold">{item.department}</div>
                        <div className={`text-sm ${isOver ? 'text-red-500 font-bold' : 'text-muted-foreground'}`}>
                           {percentage.toFixed(1)}%
                        </div>
                      </div>
                      <Progress 
                        value={Math.min(percentage, 100)} 
                        className={`h-2 ${isOver ? '[&>div]:bg-red-500' : ''}`}
                      />
                      <div className="flex items-center justify-between text-xs text-muted-foreground">
                        <span>Spent: ${spent.toLocaleString()}</span>
                        <span>Limit: ${limit.toLocaleString()}</span>
                      </div>
                   </Link>
                 )
              })}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
