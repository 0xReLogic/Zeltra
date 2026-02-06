'use client'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { DollarSign, TrendingDown, Clock, Activity } from 'lucide-react'
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts'

import { useDashboardMetrics, useCashFlowData } from '@/lib/queries/dashboard'
import { formatCurrency } from '@/lib/utils/format'
import { RecentActivity } from '@/components/dashboard/RecentActivity'
import { BudgetVsActual } from '@/components/dashboard/BudgetVsActual'
import { EntitySelector } from '@/components/entities/EntitySelector'
import { useAuthStore } from '@/lib/stores/authStore'

export default function DashboardPage() {
  const currentEntityId = useAuthStore((state) => state.currentEntityId)
  const { data: metrics } = useDashboardMetrics(currentEntityId || undefined)
  const { data: cashFlow } = useCashFlowData(currentEntityId || undefined)
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">Overview</h1>
        <EntitySelector />
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Cash Position
            </CardTitle>
            <DollarSign className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
                {formatCurrency(parseFloat(metrics?.cash_position.balance || '0'))}
            </div>
            <p className="text-xs text-muted-foreground">
              +{metrics?.cash_position.change_percent}% from last month
            </p>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Monthly Burn Rate
            </CardTitle>
            <TrendingDown className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
                {formatCurrency(parseFloat(metrics?.burn_rate.monthly || '0'))}
            </div>
            <p className="text-xs text-muted-foreground">
              {formatCurrency(parseFloat(metrics?.burn_rate.daily || '0'))} / day
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Runway</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{metrics?.runway_days || 0} Days</div>
            <p className="text-xs text-muted-foreground">
              Based on current burn rate
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Pending Approvals
            </CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{metrics?.pending_approvals.count || 0}</div>
            <p className="text-xs text-muted-foreground">
              Total value: {formatCurrency(parseFloat(metrics?.pending_approvals.total_amount || '0'))}
            </p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        <Card className="col-span-4">
          <CardHeader>
            <CardTitle>Cash Flow</CardTitle>
          </CardHeader>
          <CardContent className="h-[300px]">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={cashFlow || []} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
                <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
                <XAxis dataKey="month" className="text-xs" />
                <YAxis className="text-xs" tickFormatter={(value) => `$${value/1000}k`} />
                <Tooltip formatter={(value: number | string | undefined) => `$${Number(value || 0).toLocaleString()}`} />
                <Area type="monotone" dataKey="inflow" stackId="1" stroke="#34d399" fill="#34d399" fillOpacity={0.6} name="Inflow" />
                <Area type="monotone" dataKey="outflow" stackId="2" stroke="#f87171" fill="#f87171" fillOpacity={0.6} name="Outflow" />
              </AreaChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
        
        <RecentActivity />
      </div>

      {/* Budget vs Actual */}
      <div className="grid gap-4 md:grid-cols-1">
        <BudgetVsActual />
      </div>
    </div>
  )
}

