'use client'

import React from 'react'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Download } from 'lucide-react'
import { downloadCSV } from '@/lib/utils/export'
import { exportPDF } from '@/lib/utils/export-pdf'
import { toast } from 'sonner'
import { useIncomeStatement } from '@/lib/queries/reports'
import { ReportsNav } from '@/components/reports/ReportsNav'

export default function IncomeStatementPage() {
  const { data, isLoading } = useIncomeStatement()

  const handleExportCSV = () => {
    if (!data?.data) return
    const report = data.data
    const exportData = [
        ...report.revenues.map(item => ({ Type: 'Revenue', ...item })),
        ...report.expenses.map(item => ({ Type: 'Expense', ...item })),
        { Type: 'Net Income', code: '', name: 'Total', amount: report.net_income }
    ]
    downloadCSV(exportData, `Income_Statement_${new Date().toISOString().split('T')[0]}.csv`)
    toast.success('CSV exported successfully')
  }

  const handleExportPDF = () => {
    if (!data?.data) return
    const report = data.data
    const headers = ['Type', 'Code', 'Account Name', 'Amount']
    const tableData: (string | number)[][] = []

    // Revenues
    report.revenues.forEach(item => 
        tableData.push(['Revenue', item.code, item.name, parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })])
    )
    tableData.push(['', '', 'Total Revenues', parseFloat(report.total_revenue).toLocaleString('en-US', { minimumFractionDigits: 2 })])

    // Expenses
    report.expenses.forEach(item => 
        tableData.push(['Expense', item.code, item.name, parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })])
    )
    tableData.push(['', '', 'Total Expenses', parseFloat(report.total_expenses).toLocaleString('en-US', { minimumFractionDigits: 2 })])
    
    // Net Income
    tableData.push(['', '', 'NET INCOME', parseFloat(report.net_income).toLocaleString('en-US', { minimumFractionDigits: 2 })])

    exportPDF('Income Statement', headers, tableData, `Income_Statement_${new Date().toISOString().split('T')[0]}.pdf`)
    toast.success('PDF exported successfully')
  }

  if (isLoading) {
    return <div className="p-8 text-center text-muted-foreground">Loading report...</div>
  }

  const report = data?.data
  const revenues = report?.revenues || []
  const expenses = report?.expenses || []
  const netIncome = parseFloat(report?.net_income || '0')

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Income Statement</h1>
          <p className="text-muted-foreground mt-2">
            Profit and Loss statement for the current period.
          </p>
        </div>
        <div className="flex gap-2">
            <Button variant="outline" onClick={handleExportCSV}>
            <Download className="mr-2 h-4 w-4" />
            CSV
            </Button>
            <Button variant="outline" onClick={handleExportPDF}>
            <Download className="mr-2 h-4 w-4" />
            PDF
            </Button>
        </div>
      </div>

      <ReportsNav />

      <Card>
        <CardHeader>
          <CardTitle>Jan 2026</CardTitle>
          <CardDescription>
             Report generated on {new Date().toLocaleDateString()}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-[100px]">Code</TableHead>
                <TableHead>Account Name</TableHead>
                <TableHead className="text-right">Amount</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {/* Revenues */}
              <TableRow className="bg-muted/30">
                <TableCell colSpan={3} className="font-semibold text-muted-foreground pt-4">Revenues</TableCell>
              </TableRow>
              {revenues.map((item) => (
                <TableRow key={item.code} className="border-0">
                  <TableCell className="font-medium">{item.code}</TableCell>
                  <TableCell>{item.name}</TableCell>
                  <TableCell className="text-right font-mono">
                    {parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })}
                  </TableCell>
                </TableRow>
              ))}
              <TableRow className="border-t font-semibold">
                <TableCell colSpan={2}>Total Revenues</TableCell>
                <TableCell className="text-right font-mono">
                  {parseFloat(report?.total_revenue || '0').toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>

              {/* Expenses */}
              <TableRow className="bg-muted/30">
                <TableCell colSpan={3} className="font-semibold text-muted-foreground pt-6">Expenses</TableCell>
              </TableRow>
              {expenses.map((item) => (
                <TableRow key={item.code} className="border-0">
                  <TableCell className="font-medium">{item.code}</TableCell>
                  <TableCell>{item.name}</TableCell>
                   <TableCell className="text-right font-mono">
                    {parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })}
                  </TableCell>
                </TableRow>
              ))}
              <TableRow className="border-t font-semibold">
                <TableCell colSpan={2}>Total Expenses</TableCell>
                <TableCell className="text-right font-mono">
                  {parseFloat(report?.total_expenses || '0').toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>

              {/* Net Income */}
              <TableRow className="border-t-2 bg-muted/50 font-bold text-lg">
                <TableCell colSpan={2}>Net Income</TableCell>
                <TableCell className={`text-right font-mono ${netIncome >= 0 ? 'text-emerald-700 dark:text-emerald-400' : 'text-red-700 dark:text-red-400'}`}>
                  {netIncome.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
