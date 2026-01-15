'use client'

import React, { useState, useEffect } from 'react'
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
import type { DimensionalReportRowResponse, DimensionValueResponse } from '@/types/dimensional-report'
import { apiClient } from '@/lib/api/client'

interface DimensionType {
  id: string
  code: string
  name: string
}

export default function DimensionalReportPage() {
  const [selectedDimensionType, setSelectedDimensionType] = useState<DimensionType | undefined>()
  const [dimensionTypes, setDimensionTypes] = useState<DimensionType[]>([])
  const [loadingDimensions, setLoadingDimensions] = useState(true)
  const [startDate, setStartDate] = useState('2026-01-01')
  const [endDate, setEndDate] = useState('2026-12-31')

  // Fetch available dimension types
  useEffect(() => {
    const fetchDimensionTypes = async () => {
      try {
        const response = await apiClient<{ dimension_types: DimensionType[] }>(
          '/dimension-types'
        )
        setDimensionTypes(response.dimension_types || [])
      } catch (error) {
        console.error('Failed to fetch dimension types:', error)
      } finally {
        setLoadingDimensions(false)
      }
    }

    fetchDimensionTypes()
  }, [])

  const { data: report, isLoading } = useDimensionalReport({
      startDate,
      endDate,
      dimensionTypeId: selectedDimensionType?.code
  })

  const chartData = report?.rows.map((row: DimensionalReportRowResponse) => {
    const dimensionLabel = row.dimensions.map((d: DimensionValueResponse) => d.name).join(' - ')
    return {
      name: dimensionLabel,
      Debit: parseFloat(row.total_debit),
      Credit: parseFloat(row.total_credit),
      Balance: parseFloat(row.balance)
    }
  }) || []

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
                <Select value={selectedDimensionType?.id || ''} onValueChange={(id) => {
                  const selected = dimensionTypes.find(dt => dt.id === id)
                  setSelectedDimensionType(selected)
                }}>
                    <SelectTrigger className="w-[180px]">
                        <SelectValue placeholder="Select Dimension" />
                    </SelectTrigger>
                    <SelectContent>
                        {loadingDimensions ? (
                          <SelectItem value="loading" disabled>Loading...</SelectItem>
                        ) : dimensionTypes.length === 0 ? (
                          <SelectItem value="none" disabled>No dimensions available</SelectItem>
                        ) : (
                          dimensionTypes.map((dt) => (
                            <SelectItem key={dt.id} value={dt.id}>
                              {dt.name}
                            </SelectItem>
                          ))
                        )}
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

      {isLoading && (
        <div className="text-center py-8 text-muted-foreground">Loading report...</div>
      )}

      {!isLoading && !report && (
        <div className="text-center py-8 text-muted-foreground">No data available</div>
      )}

      {!isLoading && report && (
        <>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
            <Card className="col-span-4">
                <CardHeader>
                    <CardTitle>Financial Performance by {report.group_by.join(', ')}</CardTitle>
                    <CardDescription>Debit vs Credit vs Balance</CardDescription>
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
                                tickFormatter={(value) => `${value}`}
                            />
                            <Tooltip 
                                cursor={{ fill: 'transparent' }}
                                formatter={(value) => [`${Number(value).toLocaleString()}`, '']}
                            />
                            <Legend />
                            <Bar dataKey="Debit" fill="#0ea5e9" radius={[4, 4, 0, 0]} />
                            <Bar dataKey="Credit" fill="#ef4444" radius={[4, 4, 0, 0]} />
                            <Bar dataKey="Balance" fill="#10b981" radius={[4, 4, 0, 0]} />
                        </BarChart>
                    </ResponsiveContainer>
                </CardContent>
            </Card>

            <Card className="col-span-3">
                <CardHeader>
                    <CardTitle>Summary</CardTitle>
                    <CardDescription>Report period: {report.period_start} to {report.period_end}</CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="space-y-4">
                        <div className="flex items-center justify-between border-b pb-4">
                            <div className="text-sm font-medium">Currency</div>
                            <div className="text-lg font-semibold">{report.currency}</div>
                        </div>
                        <div className="flex items-center justify-between border-b pb-4">
                            <div className="text-sm font-medium">Grand Total</div>
                            <div className="text-2xl font-bold">{formatCurrency(parseFloat(report.grand_total))}</div>
                        </div>
                        <div className="flex items-center justify-between pt-2">
                            <div className="text-sm font-medium">Grouped By</div>
                            <div className="text-sm text-muted-foreground">{report.group_by.join(', ')}</div>
                        </div>
                    </div>
                </CardContent>
            </Card>
          </div>

          {report.rows.length > 0 && (
            <Card>
                <CardHeader>
                    <CardTitle>Detailed Breakdown</CardTitle>
                </CardHeader>
                <CardContent>
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>Dimension Values</TableHead>
                                <TableHead className="text-right">Total Debit</TableHead>
                                <TableHead className="text-right">Total Credit</TableHead>
                                <TableHead className="text-right">Balance</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {report.rows.map((row: DimensionalReportRowResponse, idx: number) => {
                              const dimensionLabel = row.dimensions.map((d: DimensionValueResponse) => `${d.dimension_type}: ${d.name}`).join(', ')
                              return (
                                <TableRow key={idx}>
                                    <TableCell className="font-medium">{dimensionLabel || 'No dimensions'}</TableCell>
                                    <TableCell className="text-right">{formatCurrency(parseFloat(row.total_debit))}</TableCell>
                                    <TableCell className="text-right">{formatCurrency(parseFloat(row.total_credit))}</TableCell>
                                    <TableCell className={`text-right font-bold ${parseFloat(row.balance) < 0 ? 'text-red-500' : 'text-emerald-500'}`}>
                                        {formatCurrency(parseFloat(row.balance))}
                                    </TableCell>
                                </TableRow>
                              )
                            })}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>
          )}

          {report.rows.length === 0 && (
            <Card>
              <CardContent className="py-8 text-center text-muted-foreground">
                No dimensional data found for the selected period.
              </CardContent>
            </Card>
          )}
        </>
      )}
    </div>
  )
}
