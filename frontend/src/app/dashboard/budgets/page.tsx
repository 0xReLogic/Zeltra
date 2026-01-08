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
import { AlertCircle, TrendingDown, TrendingUp, DollarSign } from 'lucide-react'

import { useBudgets } from '@/lib/queries/budgets'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'

export default function BudgetsPage() {
  const { data, isLoading } = useBudgets()

  const budgets = data?.data || []

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
                    formatter={(value: number) => [`$${value.toLocaleString()}`, '']}
                />
                <Legend />
                <Bar dataKey="Budget" fill="#0f172a" radius={[4, 4, 0, 0]} />
                <Bar dataKey="Actual" fill="#adfa1d" radius={[4, 4, 0, 0]} />
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
                   <div key={item.id} className="space-y-2">
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
                   </div>
                 )
              })}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
