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
import { useTrialBalance } from '@/lib/queries/reports'
import { ReportsNav } from '@/components/reports/ReportsNav'

export default function TrialBalancePage() {
  const { data, isLoading } = useTrialBalance()

  const handleExportCSV = () => {
    if (!data?.data) return
    const exportData = data.data.map(item => ({
      Account_Code: item.code,
      Account_Name: item.name,
      Debit: item.debit !== '0' ? item.debit : '',
      Credit: item.credit !== '0' ? item.credit : ''
    }))
    exportData.push({
      Account_Code: 'TOTAL',
      Account_Name: '',
      Debit: data.total_debit,
      Credit: data.total_credit
    })
    downloadCSV(exportData, `Trial_Balance_${new Date().toISOString().split('T')[0]}.csv`)
    toast.success('CSV exported successfully')
  }

  const handleExportPDF = () => {
    if (!data?.data) return
    const headers = ['Code', 'Account Name', 'Debit', 'Credit']
    const tableData = data.data.map(item => [
      item.code,
      item.name,
      item.debit !== '0' ? parseFloat(item.debit).toLocaleString('en-US', { minimumFractionDigits: 2 }) : '-',
      item.credit !== '0' ? parseFloat(item.credit).toLocaleString('en-US', { minimumFractionDigits: 2 }) : '-'
    ])
    // Add Total Row
    tableData.push(['', 'TOTAL', 
      parseFloat(data.total_debit).toLocaleString('en-US', { minimumFractionDigits: 2 }),
      parseFloat(data.total_credit).toLocaleString('en-US', { minimumFractionDigits: 2 })
    ])
    
    exportPDF('Trial Balance Report', headers, tableData, `Trial_Balance_${new Date().toISOString().split('T')[0]}.pdf`)
    toast.success('PDF exported successfully')
  }

  if (isLoading) {
    return <div className="p-8 text-center text-muted-foreground">Loading report...</div>
  }

  const report = data?.data || []
  const totalDebit = parseFloat(data?.total_debit || '0')
  const totalCredit = parseFloat(data?.total_credit || '0')
  const isBalanced = Math.abs(totalDebit - totalCredit) < 0.01

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Trial Balance</h1>
          <p className="text-muted-foreground mt-2">
            Summary of all ledger account balances for the current period.
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
                <TableHead className="text-right">Debit</TableHead>
                <TableHead className="text-right">Credit</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {report.map((item) => (
                <TableRow key={item.code}>
                  <TableCell className="font-medium">{item.code}</TableCell>
                  <TableCell>{item.name}</TableCell>
                  <TableCell className="text-right font-mono">
                    {parseFloat(item.debit) > 0 ? parseFloat(item.debit).toLocaleString('en-US', { minimumFractionDigits: 2 }) : '-'}
                  </TableCell>
                  <TableCell className="text-right font-mono">
                    {parseFloat(item.credit) > 0 ? parseFloat(item.credit).toLocaleString('en-US', { minimumFractionDigits: 2 }) : '-'}
                  </TableCell>
                </TableRow>
              ))}
              <TableRow className="border-t-2 font-bold bg-muted/50">
                <TableCell colSpan={2}>Total</TableCell>
                <TableCell className="text-right font-mono">
                  {totalDebit.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
                <TableCell className="text-right font-mono">
                  {totalCredit.toLocaleString('en-US', { minimumFractionDigits: 2 })}
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
