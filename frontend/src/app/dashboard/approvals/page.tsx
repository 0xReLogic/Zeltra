'use client'

import React from 'react'
import Link from 'next/link'
import { CheckCircle, XCircle } from 'lucide-react'

import { usePendingTransactions, useApproveTransaction, useRejectTransaction } from '@/lib/queries/transactions'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'


import { toast } from "sonner"

export default function ApprovalsPage() {
  const { data, isLoading } = usePendingTransactions()
  const approveMutation = useApproveTransaction()
  const rejectMutation = useRejectTransaction()

  const [selectedIds, setSelectedIds] = React.useState<string[]>([])

  const handleApprove = (id: string) => {
    approveMutation.mutate(id, {
      onSuccess: () => {
        toast.success("Transaction Approved", {
          description: "Transaction has been approved successfully.",
        })
      }
    })
  }

  const handleBulkApprove = async () => {
      // In a real app we'd use a mutation for this
      try {
          await fetch(`/api/v1/organizations/org_1/transactions/bulk-approve`, {
              method: 'POST',
              body: JSON.stringify({ transaction_ids: selectedIds })
          })
          toast.success(`Approved ${selectedIds.length} transactions`)
          setSelectedIds([])
          // Ideally invalidate queries here to refresh list
          window.location.reload() // Simple refresh for now
      } catch {
          toast.error("Failed to approve transactions")
      }
  }

  const handleReject = (id: string) => {
    rejectMutation.mutate(id, {
      onSuccess: () => {
        toast.error("Transaction Rejected", {
          description: "Transaction has been rejected.",
        })
      }
    })
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Approval Queue</h1>
          <p className="text-muted-foreground mt-2">
            Review and approve pending transactions.
          </p>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Pending Transactions</CardTitle>
          <CardDescription>
            {data?.data.length || 0} transactions waiting for your review.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {/* Bulk Actions Toolbar */}
          <div className="mb-4 flex items-center justify-between">
             <div className="text-sm text-muted-foreground">
                {selectedIds.length} selected
             </div>
             {selectedIds.length > 0 && (
                 <Button 
                    size="sm" 
                    className="bg-emerald-600 hover:bg-emerald-700 text-white"
                    onClick={handleBulkApprove}
                 >
                    <CheckCircle className="h-4 w-4 mr-2" />
                    Approve Selected ({selectedIds.length})
                 </Button>
             )}
          </div>

          <Table>
             <TableHeader>
               <TableRow>
                 <TableHead className="w-[50px]">
                    <input 
                        type="checkbox" 
                        className="translate-y-[2px]"
                        onChange={(e) => {
                            if (e.target.checked && data?.data) {
                                setSelectedIds(data.data.map(t => t.id))
                            } else {
                                setSelectedIds([])
                            }
                        }}
                        checked={data?.data?.length ? selectedIds.length === data.data.length && data.data.length > 0 : false}
                    />
                 </TableHead>
                 <TableHead>Date</TableHead>
                 <TableHead>Reference</TableHead>
                 <TableHead>Description</TableHead>
                 <TableHead>Amount</TableHead>
                 <TableHead className="text-right">Actions</TableHead>
               </TableRow>
             </TableHeader>
             <TableBody>
                {isLoading ? (
                  <TableRow>
                    <TableCell colSpan={6} className="h-24 text-center">
                      Loading...
                    </TableCell>
                  </TableRow>
                ) : data?.data && data.data.length > 0 ? (
                  data.data.map((txn) => {
                     const totalAmount = Math.max(
                        ...txn.entries.map(e => parseFloat(e.debit) || parseFloat(e.credit))
                     )
                     const isSelected = selectedIds.includes(txn.id)

                     return (
                       <TableRow key={txn.id} data-state={isSelected ? "selected" : undefined}>
                         <TableCell>
                            <input 
                                type="checkbox" 
                                className="translate-y-[2px]"
                                checked={isSelected}
                                onChange={(e) => {
                                    if (e.target.checked) {
                                        setSelectedIds(curr => [...curr, txn.id])
                                    } else {
                                        setSelectedIds(curr => curr.filter(id => id !== txn.id))
                                    }
                                }}
                            />
                         </TableCell>
                         <TableCell className="font-medium">{txn.transaction_date}</TableCell>
                         <TableCell>
                            <Link href={`/dashboard/transactions/${txn.id}`} className="hover:underline text-primary">
                                {txn.reference_number}
                            </Link>
                         </TableCell>
                         <TableCell>{txn.description}</TableCell>
                         <TableCell>
                            {totalAmount.toLocaleString('en-US', { style: 'currency', currency: 'USD' })}
                         </TableCell>
                         <TableCell className="text-right space-x-2">
                            <Button 
                                size="sm" 
                                variant="outline" 
                                className="text-emerald-600 hover:text-emerald-700 hover:bg-emerald-50"
                                onClick={() => handleApprove(txn.id)}
                                disabled={approveMutation.isPending}
                            >
                                <CheckCircle className="h-4 w-4 mr-1" />
                                Approve
                            </Button>
                            <Button 
                                size="sm" 
                                variant="outline" 
                                className="text-red-600 hover:text-red-700 hover:bg-red-50"
                                onClick={() => handleReject(txn.id)}
                                disabled={rejectMutation.isPending}
                            >
                                <XCircle className="h-4 w-4 mr-1" />
                                Reject
                            </Button>
                         </TableCell>
                       </TableRow>
                     )
                  })
                ) : (
                  <TableRow>
                    <TableCell colSpan={6} className="h-24 text-center text-muted-foreground">
                        <div className="flex flex-col items-center justify-center space-y-2">
                            <CheckCircle className="h-6 w-6 text-emerald-500" />
                            <span>All caught up! No pending transactions.</span>
                        </div>
                    </TableCell>
                  </TableRow>
                )}
             </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
