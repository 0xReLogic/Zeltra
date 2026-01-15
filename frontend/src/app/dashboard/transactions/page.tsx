'use client'

import { useTransactions } from '@/lib/queries/transactions'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { CreateTransactionDialog } from '@/components/transactions/CreateTransactionDialog'
import { PayInvoiceDialog } from '@/components/transactions/PayInvoiceDialog'
import { TransactionListItem } from '@/types/transactions'
import { Button } from '@/components/ui/button'
import { Loader2, Filter, CreditCard } from 'lucide-react'
import Link from 'next/link'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
import { useDimensions, useDimensionValues } from '@/lib/queries/dimensions'
import { useState } from 'react'

export default function TransactionsPage() {
  const [filterDim, setFilterDim] = useState<string>('all')
  const [page, setPage] = useState(0)
  
  // Pay Invoice State
  const [selectedInvoice, setSelectedInvoice] = useState<TransactionListItem | null>(null)
  const [payOpen, setPayOpen] = useState(false)

  const { data, isLoading, isError } = useTransactions({
    page,
    limit: 50,
    dimension_value_id: filterDim !== 'all' ? filterDim : undefined,
  })
  const { data: dimensionsData } = useDimensions()
  
  // Ensure dimensionsData is an array
  const dimensions = Array.isArray(dimensionsData) ? dimensionsData : []
  
  // Get dimension type IDs
  const deptTypeId = dimensions.find(d => d.code === 'DEPT')?.id
  const projTypeId = dimensions.find(d => d.code === 'PROJ')?.id
  
  // Fetch values
  const { data: deptValues } = useDimensionValues(deptTypeId)
  const { data: projValues } = useDimensionValues(projTypeId)
  
  const departmentOptions = Array.isArray(deptValues) ? deptValues : []
  const projectOptions = Array.isArray(projValues) ? projValues : []

  // Backend returns structured object with transactions array
  const txnList = data?.transactions || []

  if (isLoading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (isError) {
    return (
      <div className="rounded-md bg-destructive/15 p-4 text-destructive">
        Failed to load transactions. Please try again.
      </div>
    )
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'posted': return 'default' // primary
      case 'approved': return 'secondary' // green-ish in some themes or secondary
      case 'pending': return 'secondary' // yellow-ish usually needs custom class
      case 'draft': return 'outline'
      case 'voided': return 'destructive'
      default: return 'outline'
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">Transactions</h1>
        <div className="flex gap-2">
            <Select value={filterDim} onValueChange={setFilterDim}>
                <SelectTrigger className="w-[180px]">
                    <Filter className="w-4 h-4 mr-2" />
                    <SelectValue placeholder="Filter by Dept" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="all">All</SelectItem>
                    {departmentOptions.map((v) => (
                        <SelectItem key={v.code} value={v.code}>{v.name}</SelectItem>
                    ))}
                    {projectOptions.map((v) => (
                        <SelectItem key={v.code} value={v.code}>Proj: {v.name}</SelectItem>
                    ))}
                </SelectContent>
            </Select>
           <CreateTransactionDialog />
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Recent Transactions</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Date</TableHead>
                <TableHead>Reference</TableHead>
                <TableHead>Description</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {txnList.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={6} className="text-center text-muted-foreground py-8">
                    No transactions found. Create your first transaction to get started.
                  </TableCell>
                </TableRow>
              ) : (
                txnList.map((txn) => (
                  <Link href={`/dashboard/transactions/${txn.id}`} key={txn.id} className="contents">
                  <TableRow className="cursor-pointer hover:bg-muted/50">
                    <TableCell className="font-medium">
                      {txn.transaction_date}
                    </TableCell>
                    <TableCell>{txn.reference_number || '-'}</TableCell>
                    <TableCell className="max-w-[300px] truncate">
                      {txn.description}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className="capitalize">
                        {txn.transaction_type}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Badge variant={getStatusColor(txn.status) as 'default' | 'secondary' | 'outline' | 'destructive'}>
                        {txn.status}
                      </Badge>
                    </TableCell>
                     <TableCell>
                      {/* Only show Pay button for posted expenses/bills */}
                      {txn.status === 'posted' && ['expense', 'journal'].includes(txn.transaction_type) && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={(e) => {
                            e.preventDefault()
                            e.stopPropagation()
                            setSelectedInvoice(txn)
                            setPayOpen(true)
                          }}
                          className="h-8 w-8 p-0"
                          title="Pay Invoice"
                        >
                          <CreditCard className="h-4 w-4 text-primary" />
                          <span className="sr-only">Pay</span>
                        </Button>
                      )}
                    </TableCell>
                  </TableRow>
                  </Link>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
      
      <PayInvoiceDialog 
        invoice={selectedInvoice} 
        open={payOpen} 
        onOpenChange={setPayOpen} 
      />

      <div className="flex items-center justify-end space-x-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => setPage((p) => Math.max(0, p - 1))}
          disabled={page === 0}
        >
          Previous
        </Button>
        <div className="text-sm text-muted-foreground">
          Page {page + 1} of {data?.pagination ? Math.ceil(data.pagination.total / data.pagination.limit) : 1}
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setPage((p) => p + 1)}
          disabled={!data?.pagination || page >= Math.ceil(data.pagination.total / data.pagination.limit) - 1}
        >
          Next
        </Button>
      </div>
    </div>
  )
}
