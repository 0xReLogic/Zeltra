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
import { useBalanceSheet } from '@/lib/queries/reports'
import { ReportsNav } from '@/components/reports/ReportsNav'

export default function BalanceSheetPage() {
  const { data, isLoading } = useBalanceSheet()

  const handleExportCSV = () => {
    if (!data?.data) return
    const report = data.data
    const exportData = [
        ...report.assets.map(item => ({ Section: 'Assets', ...item })),
        ...report.liabilities.map(item => ({ Section: 'Liabilities', ...item })),
        ...report.equity.map(item => ({ Section: 'Equity', ...item })),
    ]
    downloadCSV(exportData, `Balance_Sheet_${new Date().toISOString().split('T')[0]}.csv`)
    toast.success('CSV exported successfully')
  }

  const handleExportPDF = () => {
    if (!data?.data) return
    const report = data.data
    const headers = ['Section', 'Code', 'Account Name', 'Amount']
    const tableData: (string | number)[][] = []

    // Assets
    report.assets.forEach(item => tableData.push(['Assets', item.code, item.name, parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })]))
    tableData.push(['', '', 'Total Assets', parseFloat(report.total_assets).toLocaleString('en-US', { minimumFractionDigits: 2 })])

    // Liabilities
    report.liabilities.forEach(item => tableData.push(['Liabilities', item.code, item.name, parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })]))
    tableData.push(['', '', 'Total Liabilities', parseFloat(report.total_liabilities).toLocaleString('en-US', { minimumFractionDigits: 2 })])

    // Equity
    report.equity.forEach(item => tableData.push(['Equity', item.code, item.name, parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })]))
    tableData.push(['', '', 'Total Equity', parseFloat(report.total_equity).toLocaleString('en-US', { minimumFractionDigits: 2 })])
    
    exportPDF('Balance Sheet', headers, tableData, `Balance_Sheet_${new Date().toISOString().split('T')[0]}.pdf`)
    toast.success('PDF exported successfully')
  }

  if (isLoading) {
    return <div className="p-8 text-center text-muted-foreground">Loading report...</div>
  }

  const report = data?.data
  const assets = report?.assets || []
  const liabilities = report?.liabilities || []
  const equity = report?.equity || []
  
  const totalAssets = parseFloat(report?.total_assets || '0')
  const totalLiabilities = parseFloat(report?.total_liabilities || '0')
  const totalEquity = parseFloat(report?.total_equity || '0')
  const totalLiabilitiesEquity = totalLiabilities + totalEquity

  const isBalanced = Math.abs(totalAssets - totalLiabilitiesEquity) < 0.01

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Balance Sheet</h1>
          <p className="text-muted-foreground mt-2">
            Statement of financial position: Assets = Liabilities + Equity.
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
           <div className="flex items-center justify-between">
            <div>
              <CardTitle>Jan 2026</CardTitle>
              <CardDescription>
                Report generated on {new Date().toLocaleDateString()}
              </CardDescription>
            </div>
            <div className={`px-4 py-1.5 rounded-full text-sm font-medium ${isBalanced ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400' : 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'}`}>
              {isBalanced ? 'Balanced' : 'Unbalanced'}
            </div>
          </div>
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
              {/* Assets Group */}
              <TableRow className="bg-muted/30">
                <TableCell colSpan={3} className="font-semibold text-muted-foreground pt-4">Assets</TableCell>
              </TableRow>
              {assets.map((item) => (
                <TableRow key={item.code} className="border-0">
                  <TableCell className="font-medium">{item.code}</TableCell>
                  <TableCell>{item.name}</TableCell>
                  <TableCell className="text-right font-mono">
                    {parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })}
                  </TableCell>
                </TableRow>
              ))}
              <TableRow className="border-t font-semibold">
                <TableCell colSpan={2}>Total Assets</TableCell>
                <TableCell className="text-right font-mono">
                  {totalAssets.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>

              {/* Liabilities Group */}
              <TableRow className="bg-muted/30">
                <TableCell colSpan={3} className="font-semibold text-muted-foreground pt-6">Liabilities</TableCell>
              </TableRow>
              {liabilities.map((item) => (
                <TableRow key={item.code} className="border-0">
                  <TableCell className="font-medium">{item.code}</TableCell>
                  <TableCell>{item.name}</TableCell>
                   <TableCell className="text-right font-mono">
                    {parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })}
                  </TableCell>
                </TableRow>
              ))}
              <TableRow className="border-t font-semibold">
                <TableCell colSpan={2}>Total Liabilities</TableCell>
                <TableCell className="text-right font-mono">
                  {totalLiabilities.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>

              {/* Equity Group */}
              <TableRow className="bg-muted/30">
                <TableCell colSpan={3} className="font-semibold text-muted-foreground pt-6">Equity</TableCell>
              </TableRow>
              {equity.map((item) => (
                <TableRow key={item.code} className="border-0">
                  <TableCell className="font-medium">{item.code}</TableCell>
                  <TableCell>{item.name}</TableCell>
                   <TableCell className="text-right font-mono">
                    {parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })}
                  </TableCell>
                </TableRow>
              ))}
              <TableRow className="border-t font-semibold">
                <TableCell colSpan={2}>Total Equity</TableCell>
                <TableCell className="text-right font-mono">
                  {totalEquity.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>

              {/* Liability + Equity Check */}
              <TableRow className="border-t-2 bg-muted/50 font-bold text-lg">
                <TableCell colSpan={2}>Total Liabilities & Equity</TableCell>
                <TableCell className="text-right font-mono">
                  {totalLiabilitiesEquity.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
