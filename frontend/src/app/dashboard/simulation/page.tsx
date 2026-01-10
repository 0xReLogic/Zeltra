
'use client'

import { useState } from 'react'

import { SimulationControls } from '@/components/simulation/SimulationControls'
import { SimulationChart } from '@/components/simulation/SimulationChart'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { ArrowUpRight, ArrowDownRight, TrendingUp } from 'lucide-react'

import { SimulationRequest, SimulationResult } from '@/types/simulation'

// Simple specific fetcher for this page to keep it self-contained or use existing lib
const runSimulation = async (params: SimulationRequest): Promise<SimulationResult> => {
  const res = await fetch('/api/v1/simulation/run', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(params)
  })
  if (!res.ok) throw new Error('Simulation failed')
  return res.json()
}

export default function SimulationPage() {
  const [result, setResult] = useState<SimulationResult | null>(null)
  const [loading, setLoading] = useState(false)

  const handleRun = async (params: SimulationRequest) => {
    setLoading(true)
    try {
        const data = await runSimulation(params)
        setResult(data)
    } catch (error) {
        console.error(error)
    } finally {
        setLoading(false)
    }
  }

  return (
    <div className="p-8 space-y-8">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">Budget Simulator</h1>
        <p className="text-muted-foreground">Draft & Forecast Scenarios</p>
      </div>

      <div className="grid grid-cols-12 gap-6">
        {/* Controls */}
        <div className="col-span-12 md:col-span-3">
          <SimulationControls onRun={handleRun} isLoading={loading} />
        </div>

        {/* Main Chart */}
        <div className="col-span-12 md:col-span-9">
          {result ? (
             <div className="space-y-6">
                {/* Summary Cards */}
                <div className="grid grid-cols-3 gap-4">
                    <Card>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                           <CardTitle className="text-sm font-medium">Proj. Revenue</CardTitle>
                           <ArrowUpRight className="h-4 w-4 text-emerald-500" />
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">${parseFloat(result.annual_summary.total_projected_revenue).toLocaleString()}</div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                           <CardTitle className="text-sm font-medium">Proj. Expense</CardTitle>
                           <ArrowDownRight className="h-4 w-4 text-rose-500" />
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">${parseFloat(result.annual_summary.total_projected_expenses).toLocaleString()}</div>
                        </CardContent>
                    </Card>
                     <Card>
                        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                           <CardTitle className="text-sm font-medium">Net Margin</CardTitle>
                           <TrendingUp className="h-4 w-4 text-blue-500" />
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">{result.annual_summary.net_profit_margin}%</div>
                        </CardContent>
                    </Card>
                </div>

                <SimulationChart data={result.projections} />
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
