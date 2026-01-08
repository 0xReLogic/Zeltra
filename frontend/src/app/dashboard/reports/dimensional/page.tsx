'use client'

import React, { useState } from 'react'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Label } from '@/components/ui/label'
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
import { useDimensionalReport } from '@/lib/queries/reports'
import { formatCurrency } from '@/lib/utils/format'
import { Calendar } from 'lucide-react'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'

export default function DimensionalReportPage() {
  const [dimension, setDimension] = useState('DEPT')
  const [startDate, setStartDate] = useState('2026-01-01')
  const [endDate, setEndDate] = useState('2026-12-31')

  const { data: report } = useDimensionalReport({
      startDate,
      endDate,
      dimension
  })

  const chartData = report?.data.map(item => ({
      name: item.name,
      Revenue: parseFloat(item.revenue),
      Expense: parseFloat(item.expense),
      Profit: parseFloat(item.net_profit)
  })) || []

  return (
    <div className="space-y-6">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Dimensional Reports</h1>
          <p className="text-muted-foreground mt-2">
            Analyze financial performance by dimension.
          </p>
        </div>
        <div className="flex items-center gap-2">
           <div className="grid gap-1">
                <Label htmlFor="dimension" className="sr-only">Dimension</Label>
                <Select value={dimension} onValueChange={setDimension}>
                    <SelectTrigger className="w-[180px]">
                        <SelectValue placeholder="Select Dimension" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="DEPT">Department</SelectItem>
                        <SelectItem value="PROJ">Project</SelectItem>
                        <SelectItem value="COST">Cost Center</SelectItem>
                    </SelectContent>
                </Select>
           </div>
           <div className="flex items-center gap-2 border rounded-md px-3 py-2 bg-background">
               <Calendar className="h-4 w-4 text-muted-foreground" />
               <input 
                  type="date" 
                  value={startDate} 
                  onChange={(e) => setStartDate(e.target.value)} 
                  className="bg-transparent text-sm outline-none w-[110px]"
               />
               <span className="text-muted-foreground">-</span>
               <input 
                  type="date" 
                  value={endDate} 
                  onChange={(e) => setEndDate(e.target.value)} 
                  className="bg-transparent text-sm outline-none w-[110px]"
               />
           </div>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        <Card className="col-span-4">
            <CardHeader>
                <CardTitle>Financial Performance by {dimension === 'DEPT' ? 'Department' : dimension === 'PROJ' ? 'Project' : 'Cost Center'}</CardTitle>
                <CardDescription>Revenue vs Expense vs Net Profit</CardDescription>
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
                        <Bar dataKey="Revenue" fill="#0ea5e9" radius={[4, 4, 0, 0]} />
                        <Bar dataKey="Expense" fill="#ef4444" radius={[4, 4, 0, 0]} />
                        <Bar dataKey="Profit" fill="#10b981" radius={[4, 4, 0, 0]} />
                    </BarChart>
                </ResponsiveContainer>
            </CardContent>
        </Card>

        <Card className="col-span-3">
             <CardHeader>
                <CardTitle>Summary</CardTitle>
                <CardDescription>Global totals for selected period</CardDescription>
             </CardHeader>
             <CardContent>
                 <div className="space-y-4">
                     <div className="flex items-center justify-between border-b pb-4">
                         <div className="text-sm font-medium">Total Revenue</div>
                         <div className="text-2xl font-bold">{formatCurrency(parseFloat(report?.summary.global_revenue || '0'))}</div>
                     </div>
                     <div className="flex items-center justify-between border-b pb-4">
                         <div className="text-sm font-medium">Total Expense</div>
                         <div className="text-2xl font-bold text-red-500">{formatCurrency(parseFloat(report?.summary.global_expense || '0'))}</div>
                     </div>
                     <div className="flex items-center justify-between pt-2">
                         <div className="text-sm font-medium">Net Profit</div>
                         <div className="text-2xl font-bold text-emerald-500">{formatCurrency(parseFloat(report?.summary.global_net || '0'))}</div>
                     </div>
                 </div>
             </CardContent>
        </Card>
      </div>

       <Card>
            <CardHeader>
                <CardTitle>Detailed Breakdown</CardTitle>
            </CardHeader>
            <CardContent>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>Dimension Value</TableHead>
                            <TableHead className="text-right">Revenue</TableHead>
                            <TableHead className="text-right">Expense</TableHead>
                            <TableHead className="text-right">Net Profit</TableHead>
                            <TableHead>Top Contributors</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {report?.data.map((item) => (
                            <TableRow key={item.id}>
                                <TableCell className="font-medium">{item.name}</TableCell>
                                <TableCell className="text-right">{formatCurrency(parseFloat(item.revenue))}</TableCell>
                                <TableCell className="text-right text-red-500">{formatCurrency(parseFloat(item.expense))}</TableCell>
                                <TableCell className={`text-right font-bold ${parseFloat(item.net_profit) < 0 ? 'text-red-500' : 'text-emerald-500'}`}>
                                    {formatCurrency(parseFloat(item.net_profit))}
                                </TableCell>
                                <TableCell className="text-xs text-muted-foreground">
                                    {item.breakdown.map((b, i) => (
                                        <div key={i}>{b.account}: {formatCurrency(parseFloat(b.amount))}</div>
                                    ))}
                                </TableCell>
                            </TableRow>
                        ))}
                    </TableBody>
                </Table>
            </CardContent>
       </Card>
    </div>
  )
}
