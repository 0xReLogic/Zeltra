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
import { EntitySelector } from '@/components/entities/EntitySelector'
import { useAuthStore } from '@/lib/stores/authStore'

export default function IncomeStatementPage() {
  const currentEntityId = useAuthStore((state) => state.currentEntityId)
  const { data, isLoading } = useIncomeStatement(currentEntityId || undefined)

  const handleExportCSV = () => {
    if (!data) return
    const exportData = [
        ...(data.revenue?.accounts || []).map(item => ({ Type: 'Revenue', ...item })),
        ...(data.cost_of_goods_sold?.accounts || []).map(item => ({ Type: 'Cost of Goods Sold', ...item })),
        ...(data.operating_expenses?.accounts || []).map(item => ({ Type: 'Operating Expense', ...item })),
        ...(data.other_income_expenses?.accounts || []).map(item => ({ Type: 'Other Income/Expense', ...item })),
        { Type: 'Net Income', code: '', name: 'Total', balance: data.net_income }
    ]
    downloadCSV(exportData, `Income_Statement_${new Date().toISOString().split('T')[0]}.csv`)
    toast.success('CSV exported successfully')
  }

  const handleExportPDF = () => {
    if (!data) return
    const headers = ['Type', 'Code', 'Account Name', 'Amount']
    const tableData: (string | number)[][] = []

    // Revenue
    data.revenue?.accounts?.forEach(item => 
        tableData.push(['Revenue', item.code, item.name, parseFloat(item.balance).toLocaleString('en-US', { minimumFractionDigits: 2 })])
    )
    tableData.push(['', '', 'Total Revenue', parseFloat(data.revenue?.total || '0').toLocaleString('en-US', { minimumFractionDigits: 2 })])

    // COGS
    if (data.cost_of_goods_sold?.accounts?.length) {
      data.cost_of_goods_sold.accounts.forEach(item => 
          tableData.push(['COGS', item.code, item.name, parseFloat(item.balance).toLocaleString('en-US', { minimumFractionDigits: 2 })])
      )
      tableData.push(['', '', 'Total COGS', parseFloat(data.cost_of_goods_sold?.total || '0').toLocaleString('en-US', { minimumFractionDigits: 2 })])
    }
    tableData.push(['', '', 'GROSS PROFIT', parseFloat(data.gross_profit || '0').toLocaleString('en-US', { minimumFractionDigits: 2 })])

    // Operating Expenses
    data.operating_expenses?.accounts?.forEach(item => 
        tableData.push(['Operating Expense', item.code, item.name, parseFloat(item.balance).toLocaleString('en-US', { minimumFractionDigits: 2 })])
    )
    tableData.push(['', '', 'Total Operating Expenses', parseFloat(data.operating_expenses?.total || '0').toLocaleString('en-US', { minimumFractionDigits: 2 })])
    tableData.push(['', '', 'OPERATING INCOME', parseFloat(data.operating_income || '0').toLocaleString('en-US', { minimumFractionDigits: 2 })])

    // Other Income/Expenses
    if (data.other_income_expenses?.accounts?.length) {
      data.other_income_expenses.accounts.forEach(item => 
          tableData.push(['Other', item.code, item.name, parseFloat(item.balance).toLocaleString('en-US', { minimumFractionDigits: 2 })])
      )
    }
    
    // Net Income
    tableData.push(['', '', 'NET INCOME', parseFloat(data.net_income || '0').toLocaleString('en-US', { minimumFractionDigits: 2 })])

    exportPDF('Income Statement', headers, tableData, `Income_Statement_${new Date().toISOString().split('T')[0]}.pdf`)
    toast.success('PDF exported successfully')
  }

  if (isLoading) {
    return <div className="p-8 text-center text-muted-foreground">Loading report...</div>
  }

  const revenues = data?.revenue?.accounts || []
  const cogs = data?.cost_of_goods_sold?.accounts || []
  const operatingExpenses = data?.operating_expenses?.accounts || []
  const otherIncomeExpenses = data?.other_income_expenses?.accounts || []
  
  const totalRevenue = parseFloat(data?.revenue?.total || '0')
  const totalCogs = parseFloat(data?.cost_of_goods_sold?.total || '0')
  const grossProfit = parseFloat(data?.gross_profit || '0')
  const totalOperatingExpenses = parseFloat(data?.operating_expenses?.total || '0')
  const operatingIncome = parseFloat(data?.operating_income || '0')
  const totalOther = parseFloat(data?.other_income_expenses?.total || '0')
  const netIncome = parseFloat(data?.net_income || '0')

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
            <EntitySelector />
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
              {/* Revenue */}
              <TableRow className="bg-muted/30">
                <TableCell colSpan={3} className="font-semibold text-muted-foreground pt-4">Revenue</TableCell>
              </TableRow>
              {revenues.map((item) => (
                <TableRow key={item.code} className="border-0">
                  <TableCell className="font-medium">{item.code}</TableCell>
                  <TableCell>{item.name}</TableCell>
                  <TableCell className="text-right font-mono">
                    {parseFloat(item.balance).toLocaleString('en-US', { minimumFractionDigits: 2 })}
                  </TableCell>
                </TableRow>
              ))}
              <TableRow className="border-t font-semibold">
                <TableCell colSpan={2}>Total Revenue</TableCell>
                <TableCell className="text-right font-mono">
                  {totalRevenue.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>

              {/* Cost of Goods Sold */}
              {cogs.length > 0 && (
                <>
                  <TableRow className="bg-muted/30">
                    <TableCell colSpan={3} className="font-semibold text-muted-foreground pt-6">Cost of Goods Sold</TableCell>
                  </TableRow>
                  {cogs.map((item) => (
                    <TableRow key={item.code} className="border-0">
                      <TableCell className="font-medium">{item.code}</TableCell>
                      <TableCell>{item.name}</TableCell>
                      <TableCell className="text-right font-mono">
                        {parseFloat(item.balance).toLocaleString('en-US', { minimumFractionDigits: 2 })}
                      </TableCell>
                    </TableRow>
                  ))}
                  <TableRow className="border-t font-semibold">
                    <TableCell colSpan={2}>Total COGS</TableCell>
                    <TableCell className="text-right font-mono">
                      {totalCogs.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                    </TableCell>
                  </TableRow>
                </>
              )}

              {/* Gross Profit */}
              <TableRow className="border-t bg-muted/50 font-semibold">
                <TableCell colSpan={2}>Gross Profit</TableCell>
                <TableCell className="text-right font-mono">
                  {grossProfit.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>

              {/* Operating Expenses */}
              <TableRow className="bg-muted/30">
                <TableCell colSpan={3} className="font-semibold text-muted-foreground pt-6">Operating Expenses</TableCell>
              </TableRow>
              {operatingExpenses.map((item) => (
                <TableRow key={item.code} className="border-0">
                  <TableCell className="font-medium">{item.code}</TableCell>
                  <TableCell>{item.name}</TableCell>
                   <TableCell className="text-right font-mono">
                    {parseFloat(item.balance).toLocaleString('en-US', { minimumFractionDigits: 2 })}
                  </TableCell>
                </TableRow>
              ))}
              <TableRow className="border-t font-semibold">
                <TableCell colSpan={2}>Total Operating Expenses</TableCell>
                <TableCell className="text-right font-mono">
                  {totalOperatingExpenses.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>

              {/* Operating Income */}
              <TableRow className="border-t bg-muted/50 font-semibold">
                <TableCell colSpan={2}>Operating Income</TableCell>
                <TableCell className="text-right font-mono">
                  {operatingIncome.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>

              {/* Other Income/Expenses */}
              {otherIncomeExpenses.length > 0 && (
                <>
                  <TableRow className="bg-muted/30">
                    <TableCell colSpan={3} className="font-semibold text-muted-foreground pt-6">Other Income/Expenses</TableCell>
                  </TableRow>
                  {otherIncomeExpenses.map((item) => (
                    <TableRow key={item.code} className="border-0">
                      <TableCell className="font-medium">{item.code}</TableCell>
                      <TableCell>{item.name}</TableCell>
                       <TableCell className="text-right font-mono">
                        {parseFloat(item.balance).toLocaleString('en-US', { minimumFractionDigits: 2 })}
                      </TableCell>
                    </TableRow>
                  ))}
                  <TableRow className="border-t font-semibold">
                    <TableCell colSpan={2}>Total Other</TableCell>
                    <TableCell className="text-right font-mono">
                      {totalOther.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                    </TableCell>
                  </TableRow>
                </>
              )}

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
