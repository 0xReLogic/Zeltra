'use client'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { AlertTriangle, CheckCircle, PieChart } from 'lucide-react'
import { useBudgetVsActual } from '@/lib/queries/dashboard'
import { useAuthStore } from '@/lib/stores/authStore'
import { formatCurrency } from '@/lib/utils/format'
import { Skeleton } from '@/components/ui/skeleton'

export function BudgetVsActual() {
  const currentEntityId = useAuthStore((state) => state.currentEntityId)
  const { data, isLoading, error } = useBudgetVsActual(currentEntityId || undefined)

  if (isLoading) {
    return (
      <Card className="col-span-4">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <PieChart className="h-4 w-4" />
            Budget vs Actual
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Skeleton className="h-4 w-48" />
          <Skeleton className="h-20 w-full" />
          <Skeleton className="h-16 w-full" />
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card className="col-span-4">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <PieChart className="h-4 w-4" />
            Budget vs Actual
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">Failed to load budget data</p>
        </CardContent>
      </Card>
    )
  }

  // Empty state - no active budget
  if (!data?.budget_id) {
    return (
      <Card className="col-span-4">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <PieChart className="h-4 w-4" />
            Budget vs Actual
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <PieChart className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-sm text-muted-foreground">No active budget found</p>
            <p className="text-xs text-muted-foreground mt-1">
              Create a budget to track spending against targets
            </p>
          </div>
        </CardContent>
      </Card>
    )
  }

  const { summary, line_items } = data
  const isOverBudget = summary.variance_percent < 0

  return (
    <Card className="col-span-4">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <PieChart className="h-4 w-4" />
          Budget vs Actual
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Budget Name & Summary */}
        <div className="space-y-2">
          <p className="text-sm font-medium">{data.budget_name}</p>
          <div className="grid grid-cols-3 gap-4 text-sm">
            <div>
              <p className="text-muted-foreground">Budgeted</p>
              <p className="font-medium">{formatCurrency(parseFloat(summary.total_budgeted))}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Actual</p>
              <p className="font-medium">{formatCurrency(parseFloat(summary.total_actual))}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Variance</p>
              <p className={`font-medium flex items-center gap-1 ${isOverBudget ? 'text-red-500' : 'text-green-500'}`}>
                {isOverBudget ? <AlertTriangle className="h-3 w-3" /> : <CheckCircle className="h-3 w-3" />}
                {formatCurrency(Math.abs(parseFloat(summary.variance)))}
                <span className="text-xs">({Math.abs(summary.variance_percent).toFixed(1)}%)</span>
              </p>
            </div>
          </div>
        </div>

        {/* Line Items */}
        {line_items.length > 0 && (
          <div className="space-y-3">
            <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
              Top Categories
            </p>
            {line_items.slice(0, 5).map((item) => {
              const budgeted = parseFloat(item.budgeted) || 1
              const actual = parseFloat(item.actual) || 0
              const progress = Math.min((actual / budgeted) * 100, 100)
              const isItemOverBudget = item.variance_percent < 0

              return (
                <div key={item.account_id} className="space-y-1">
                  <div className="flex items-center justify-between text-sm">
                    <span className="flex items-center gap-1">
                      {item.account_name}
                      {isItemOverBudget && <AlertTriangle className="h-3 w-3 text-red-500" />}
                    </span>
                    <span className="text-muted-foreground">
                      {formatCurrency(actual)} / {formatCurrency(budgeted)}
                    </span>
                  </div>
                  <Progress 
                    value={progress} 
                    className={`h-2 ${isItemOverBudget ? '[&>div]:bg-red-500' : ''}`}
                  />
                </div>
              )
            })}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
