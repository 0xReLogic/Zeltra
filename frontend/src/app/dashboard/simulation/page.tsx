
'use client'

import { SimulationControls } from '@/components/simulation/SimulationControls'
import { SimulationChart } from '@/components/simulation/SimulationChart'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { ArrowUpRight, ArrowDownRight, TrendingUp, AlertCircle } from 'lucide-react'
import { useRunSimulation } from '@/lib/queries/simulation'
import type { RunSimulationRequest } from '@/types/simulation'
import { toast } from 'sonner'

export default function SimulationPage() {
  const simulation = useRunSimulation()

  const handleRun = (params: RunSimulationRequest) => {
    simulation.mutate(params, {
      onError: (error) => {
        console.error('Simulation failed:', error)
        toast.error('Simulation failed. Please check your parameters.')
      }
    })
  }

  return (
    <div className="p-8 space-y-8">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">Budget Simulator</h1>
        <p className="text-muted-foreground">Draft & Forecast Scenarios</p>
      </div>

      {simulation.isError && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>
            {simulation.error instanceof Error ? simulation.error.message : 'Simulation failed. This feature requires Enterprise tier.'}
          </AlertDescription>
        </Alert>
      )}

      <div className="grid grid-cols-12 gap-6">
        {/* Controls */}
        <div className="col-span-12 md:col-span-3">
          <SimulationControls onRun={handleRun} isLoading={simulation.isPending} />
        </div>

        {/* Main Chart */}
        <div className="col-span-12 md:col-span-9">
          {simulation.data ? (
             <div className="space-y-6">
                {/* Summary Cards */}
                <div className="grid grid-cols-3 gap-4">
                    <Card>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                           <CardTitle className="text-sm font-medium">Proj. Revenue</CardTitle>
                           <ArrowUpRight className="h-4 w-4 text-emerald-500" />
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">${parseFloat(simulation.data.annual_summary.total_projected_revenue).toLocaleString()}</div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                           <CardTitle className="text-sm font-medium">Proj. Expense</CardTitle>
                           <ArrowDownRight className="h-4 w-4 text-rose-500" />
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">${parseFloat(simulation.data.annual_summary.total_projected_expenses).toLocaleString()}</div>
                        </CardContent>
                    </Card>
                     <Card>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                           <CardTitle className="text-sm font-medium">Net Margin</CardTitle>
                           <TrendingUp className="h-4 w-4 text-blue-500" />
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">{simulation.data.annual_summary.net_profit_margin}%</div>
                        </CardContent>
                    </Card>
                </div>

                <SimulationChart data={simulation.data.projections as unknown as { month: string; revenue: number; expenses: number; net_income: number }[]} />
             </div>
          ) : (
            <div className="h-[400px] flex items-center justify-center border rounded-lg bg-muted/10 border-dashed">
                <p className="text-muted-foreground">Run a simulation to see projections</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
