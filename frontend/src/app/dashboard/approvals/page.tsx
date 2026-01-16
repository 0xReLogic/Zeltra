'use client'

import React from 'react'
import Link from 'next/link'
import { CheckCircle, XCircle } from 'lucide-react'

import { usePendingTransactions, useApproveTransaction, useRejectTransaction, useBulkApprove } from '@/lib/queries/transactions'
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
  const { data: transactionsData, isLoading } = usePendingTransactions()
  const approveMutation = useApproveTransaction()
  const rejectMutation = useRejectTransaction()

  const [selectedIds, setSelectedIds] = React.useState<string[]>([])

  const transactions = transactionsData?.data || []

  const handleApprove = (id: string) => {
    approveMutation.mutate(id, {
      onSuccess: () => {
        toast.success("Transaction Approved", {
          description: "Transaction has been approved successfully.",
        })
      }
    })
  }

  const { mutate: bulkApprove, isPending: isBulkApproving } = useBulkApprove()

  const handleBulkApprove = () => {
      bulkApprove(selectedIds, {
          onSuccess: (data) => {
              toast.success(`Processed ${selectedIds.length} transactions`)
              setSelectedIds([])
          },
          onError: () => {
              toast.error("Failed to approve transactions")
          }
      })
  }

  const handleReject = (id: string) => {
    rejectMutation.mutate({ id, reason: 'Rejected by approver' }, {
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
            {transactions.length} transactions waiting for your review.
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
                    disabled={isBulkApproving}
                 >
                    <CheckCircle className="h-4 w-4 mr-2" />
                    {isBulkApproving ? 'Approving...' : `Approve Selected (${selectedIds.length})`}
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
                            if (e.target.checked && transactions.length > 0) {
                                setSelectedIds(transactions.map(t => t.id))
                            } else {
                                setSelectedIds([])
                            }
                        }}
                        checked={transactions.length > 0 ? selectedIds.length === transactions.length : false}
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
                ) : transactions.length > 0 ? (
                  transactions.map((txn: any) => {
                     // Total amount is now available in transaction list item
                     const totalAmount = parseFloat(txn.total_amount) || 0
                     
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
                                disabled={approveMutation.isPending || !txn.can_approve}
                                title={!txn.can_approve ? `You do not have permission to approve this transaction` : ''}
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
