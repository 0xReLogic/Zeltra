'use client'

import React, { useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import { ArrowLeft, Calendar as CalendarIcon, Download, Loader2, Pencil, Trash2, Power } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useAccount, useAccountLedger, useDeleteAccount, useToggleAccountActive, useUpdateAccount } from '@/lib/queries/accounts'
import { formatCurrency } from '@/lib/utils/format'
import { AccountForm } from '@/components/accounts/AccountForm'

export default function AccountDetailPage() {
  const params = useParams()
  const router = useRouter()
  const id = params.id as string

  // Query State
  const [page] = useState(0)
  const [isEditOpen, setIsEditOpen] = useState(false)
  
  const { data: account, isLoading: isLoadingAccount } = useAccount(id)
  const { data: ledger, isLoading: isLoadingLedger } = useAccountLedger(id, { page, limit: 50 })
  
  // Mutations
  const deleteAccount = useDeleteAccount()
  const toggleActive = useToggleAccountActive()
  const updateAccount = useUpdateAccount()

  const handleDelete = async () => {
    try {
      await deleteAccount.mutateAsync(id)
      toast.success('Account deleted successfully')
      router.push('/dashboard/accounts')
    } catch {
      toast.error('Failed to delete account. It may have transactions.')
    }
  }

  const handleToggleActive = async () => {
    if (!account) return
    try {
      await toggleActive.mutateAsync({ id, isActive: !account.is_active })
      toast.success(account.is_active ? 'Account deactivated' : 'Account activated')
    } catch {
      toast.error('Failed to update account status')
    }
  }

  if (isLoadingAccount || isLoadingLedger) {
    return (
      <div className="flex h-96 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!account) {
    return (
       <div className="flex flex-col items-center justify-center h-96 space-y-4">
          <h2 className="text-xl font-semibold">Account not found</h2>
          <Button variant="outline" onClick={() => router.back()}>
             <ArrowLeft className="mr-2 h-4 w-4" /> Go Back
          </Button>
       </div>
    )
  }


  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div className="flex items-center gap-4">
           <Button variant="outline" size="icon" onClick={() => router.back()}>
              <ArrowLeft className="h-4 w-4" />
           </Button>
           <div>
              <div className="flex items-center gap-2">
                <h2 className="text-2xl font-bold tracking-tight">{account.code} - {account.name}</h2>
                <Badge variant="outline">{(account.type ?? account.account_type ?? 'unknown').toUpperCase()}</Badge>
              </div>
              <p className="text-muted-foreground">General Ledger</p>
           </div>
        </div>
        <div className="flex items-center gap-2">
           <Button variant="outline" onClick={() => setIsEditOpen(true)}>
              <Pencil className="mr-2 h-4 w-4" /> Edit
           </Button>
           <Button 
             variant="outline" 
             onClick={handleToggleActive}
             disabled={toggleActive.isPending}
           >
              <Power className="mr-2 h-4 w-4" /> 
              {account.is_active ? 'Deactivate' : 'Activate'}
           </Button>
           <AlertDialog>
             <AlertDialogTrigger asChild>
               <Button variant="destructive" disabled={deleteAccount.isPending}>
                 <Trash2 className="mr-2 h-4 w-4" /> Delete
               </Button>
             </AlertDialogTrigger>
             <AlertDialogContent>
               <AlertDialogHeader>
                 <AlertDialogTitle>Delete Account</AlertDialogTitle>
                 <AlertDialogDescription>
                   Are you sure you want to delete this account? This action cannot be undone.
                   Accounts with transactions cannot be deleted.
                 </AlertDialogDescription>
               </AlertDialogHeader>
               <AlertDialogFooter>
                 <AlertDialogCancel>Cancel</AlertDialogCancel>
                 <AlertDialogAction onClick={handleDelete}>Delete</AlertDialogAction>
               </AlertDialogFooter>
             </AlertDialogContent>
           </AlertDialog>
           <Button variant="outline">
              <CalendarIcon className="mr-2 h-4 w-4" /> Jan 2026
           </Button>
           <Button variant="outline">
              <Download className="mr-2 h-4 w-4" /> Export CSV
           </Button>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-3">
         <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
               <CardTitle className="text-sm font-medium">Current Balance</CardTitle>
            </CardHeader>
            <CardContent>
               <div className="text-2xl font-bold">{formatCurrency(parseFloat(account.balance), 'USD')}</div>
            </CardContent>
         </Card>
      </div>

      {/* Ledger Table */}
      <Card>
         <CardHeader>
            <CardTitle>Transactions</CardTitle>
            <CardDescription>
               Detailed movements in this account for the selected period.
            </CardDescription>
         </CardHeader>
         <CardContent>
            <div className="rounded-md border">
               <Table>
                  <TableHeader>
                     <TableRow>
                        <TableHead>Date</TableHead>
                        <TableHead>Reference</TableHead>
                        <TableHead className="w-[40%]">Description</TableHead>
                        <TableHead className="text-right text-red-600">Debit</TableHead>
                        <TableHead className="text-right text-green-600">Credit</TableHead>
                        <TableHead className="text-right">Balance</TableHead>
                     </TableRow>
                  </TableHeader>
                  <TableBody>
                     {ledger?.entries?.map((entry) => (
                        <TableRow key={entry.id}>
                           <TableCell>{entry.transaction_date}</TableCell>
                           <TableCell className="font-mono text-xs">{entry.reference_number}</TableCell>
                           <TableCell>
                              <div className="font-medium text-sm">{entry.description}</div>
                           </TableCell>
                           <TableCell className="text-right text-red-600 font-mono">
                              {parseFloat(entry.debit) > 0 ? formatCurrency(parseFloat(entry.debit), 'USD') : '-'}
                           </TableCell>
                           <TableCell className="text-right text-green-600 font-mono">
                              {parseFloat(entry.credit) > 0 ? formatCurrency(parseFloat(entry.credit), 'USD') : '-'}
                           </TableCell>
                           <TableCell className="text-right font-mono font-medium">
                              {formatCurrency(parseFloat(entry.running_balance), 'USD')}
                           </TableCell>
                        </TableRow>
                     ))}
                     {(!ledger?.entries || ledger.entries.length === 0) && (
                        <TableRow>
                           <TableCell colSpan={6} className="h-24 text-center">
                              No transactions found for this period.
                           </TableCell>
                        </TableRow>
                     )}
                  </TableBody>
               </Table>
            </div>
         </CardContent>
      </Card>

      {/* Edit Account Dialog */}
      <Dialog open={isEditOpen} onOpenChange={setIsEditOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Edit Account</DialogTitle>
          </DialogHeader>
          <AccountForm
            mode="edit"
            defaultValues={{
              code: account.code,
              name: account.name,
              type: account.type ?? account.account_type,
              currency: account.currency,
              description: account.description ?? '',
            }}
            onSubmit={async (data) => {
              try {
                await updateAccount.mutateAsync({ id, ...data })
                toast.success('Account updated successfully')
                setIsEditOpen(false)
              } catch {
                toast.error('Failed to update account')
              }
            }}
            isLoading={updateAccount.isPending}
          />
        </DialogContent>
      </Dialog>
    </div>
  )
}
