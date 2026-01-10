
'use client'

import { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Loader2, Play } from 'lucide-react'

import { SimulationRequest } from '@/types/simulation'

interface SimulationControlsProps {
  onRun: (params: SimulationRequest) => void
  isLoading: boolean
}

export function SimulationControls({ onRun, isLoading }: SimulationControlsProps) {
  const [months, setMonths] = useState('12')
  const [revenueGrowth, setRevenueGrowth] = useState('0.10')
  const [expenseGrowth, setExpenseGrowth] = useState('0.05')
  const [baseStart, setBaseStart] = useState('2025-01-01')

  const handleRun = () => {
    onRun({
        base_period_start: baseStart,
        projection_months: parseInt(months),
        revenue_growth_rate: revenueGrowth,
        expense_growth_rate: expenseGrowth
    })
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Simulation Parameters</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
            <Label>Base Period Start</Label>
            <Input 
                type="date" 
                value={baseStart} 
                onChange={(e) => setBaseStart(e.target.value)} 
            />
        </div>

        <div className="space-y-2">
            <Label>Projection Duration (Months)</Label>
            <Select value={months} onValueChange={setMonths}>
                <SelectTrigger>
                    <SelectValue placeholder="Select period" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="6">6 Months</SelectItem>
                    <SelectItem value="12">12 Months</SelectItem>
                    <SelectItem value="24">24 Months</SelectItem>
                    <SelectItem value="60">5 Years</SelectItem>
                </SelectContent>
            </Select>
        </div>

        <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
                <Label>Rev. Growth (Decimal)</Label>
                <Input 
                    type="number" 
                    step="0.01" 
                    value={revenueGrowth} 
                    onChange={(e) => setRevenueGrowth(e.target.value)} 
                    placeholder="0.10"
                />
                <p className="text-xs text-muted-foreground">0.10 = 10% Growth</p>
            </div>
            <div className="space-y-2">
                <Label>Exp. Growth (Decimal)</Label>
                <Input 
                    type="number" 
                    step="0.01" 
                    value={expenseGrowth} 
                    onChange={(e) => setExpenseGrowth(e.target.value)} 
                    placeholder="0.05"
                />
                <p className="text-xs text-muted-foreground">0.05 = 5% Growth</p>
            </div>
        </div>

        <Button className="w-full" onClick={handleRun} disabled={isLoading}>
            {isLoading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Play className="mr-2 h-4 w-4" />}
            Run Simulation
        </Button>
      </CardContent>
    </Card>
  )
}
