
'use client'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

import { ResponsiveContainer, ComposedChart, Line, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend } from 'recharts'
import { AccountProjection } from '@/types/simulation'

interface SimulationChartProps {
  data: AccountProjection[] | null
}

export function SimulationChart({ data }: SimulationChartProps) {
  if (!data) return null

  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle>Financial Projections</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="h-[400px] w-full">
          <ResponsiveContainer width="100%" height="100%">
            <ComposedChart data={data} margin={{ top: 20, right: 20, bottom: 20, left: 20 }}>
                <CartesianGrid strokeDasharray="3 3" vertical={false} />
                <XAxis dataKey="month" axisLine={false} tickLine={false} />
                <YAxis axisLine={false} tickLine={false} tickFormatter={(val) => `$${val / 1000}k`} />
                <Tooltip 
                    formatter={(val: number | string | Array<number | string> | undefined) => {
                        if (val === undefined) return ''
                        const numVal = typeof val === 'number' ? val : Number(val)
                        return isNaN(numVal) ? val : `$${numVal.toLocaleString()}`
                    }}
                    contentStyle={{ borderRadius: '8px' }}
                />
                <Legend />
                <Bar dataKey="revenue" name="Revenue" fill="#10b981" radius={[4, 4, 0, 0]} />
                <Bar dataKey="expenses" name="Expenses" fill="#f43f5e" radius={[4, 4, 0, 0]} />
                <Line type="monotone" dataKey="net_income" name="Net Income" stroke="#3b82f6" strokeWidth={3} dot={{ r: 4 }} />
            </ComposedChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  )
}
