'use client'

import React from 'react'
import { useAccounts, useCreateAccount } from '@/lib/queries/accounts'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { MoreHorizontal, Pencil, Trash, Loader2, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import Link from 'next/link'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { AccountForm } from '@/components/accounts/AccountForm'
import { toast } from 'sonner'
import { Account, CreateAccountRequest } from '@/types/accounts'
import { useUpdateAccount, useDeleteAccount } from '@/lib/queries/accounts'

export default function AccountsPage() {
  const { data, isLoading, isError } = useAccounts()
  const createAccount = useCreateAccount()
  const updateAccount = useUpdateAccount()
  const deleteAccount = useDeleteAccount()
  const [open, setOpen] = React.useState(false)
  const [editingAccount, setEditingAccount] = React.useState<Account | null>(null)

  const handleSubmit = (values: CreateAccountRequest) => {
    if (editingAccount) {
      updateAccount.mutate({ ...values, id: editingAccount.id }, {
        onSuccess: () => {
          toast.success('Account updated', {
            description: 'The account has been successfully updated.',
          })
          setOpen(false)
          setEditingAccount(null)
        },
        onError: () => {
          toast.error('Error', { description: 'Failed to update account.' })
        }
      })
    } else {
      createAccount.mutate(values, {
        onSuccess: () => {
          toast.success('Account created', {
            description: 'The new account has been successfully added.',
          })
          setOpen(false)
        },
        onError: () => {
          toast.error('Error', { description: 'Failed to create account.' })
        }
      })
    }
  }

  const handleEdit = (account: Account) => {
    setEditingAccount(account)
    setOpen(true)
  }

  const handleDelete = (id: string) => {
    if (confirm('Are you sure you want to delete this account?')) {
      deleteAccount.mutate(id, {
        onSuccess: () => toast.success('Account deleted'),
        onError: () => toast.error('Failed to delete account')
      })
    }
  }

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
        Failed to load accounts. Please try again.
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold tracking-tight">Accounts</h1>
        <Dialog open={open} onOpenChange={(val) => {
          setOpen(val)
          if (!val) setEditingAccount(null)
        }}>
          <DialogTrigger asChild>
            <Button onClick={() => setEditingAccount(null)}>
              <Plus className="mr-2 h-4 w-4" />
              New Account
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[425px]">
            <DialogHeader>
              <DialogTitle>{editingAccount ? 'Edit Account' : 'Create Account'}</DialogTitle>
              <DialogDescription>
                {editingAccount ? 'Update account details.' : 'Add a new account to your chart of accounts.'}
              </DialogDescription>
            </DialogHeader>
            <AccountForm 
              onSubmit={handleSubmit} 
              defaultValues={editingAccount ? {
                code: editingAccount.code,
                name: editingAccount.name,
                account_type: editingAccount.account_type
              } : undefined}
              isSubmitting={createAccount.isPending || updateAccount.isPending} 
            />
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {data?.data.map((account) => (
          <Link href={`/dashboard/accounts/${account.id}`} key={account.id}>
            <Card className="hover:bg-muted/50 transition-colors cursor-pointer h-full group relative">
              <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button variant="ghost" className="h-8 w-8 p-0" onClick={(e) => e.preventDefault()}>
                      <MoreHorizontal className="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem onClick={(e) => {
                      e.preventDefault()
                      handleEdit(account)
                    }}>
                      <Pencil className="mr-2 h-4 w-4" />
                      Edit
                    </DropdownMenuItem>
                    <DropdownMenuItem 
                      className="text-destructive focus:text-destructive"
                      onClick={(e) => {
                        e.preventDefault()
                        handleDelete(account.id)
                      }}
                    >
                      <Trash className="mr-2 h-4 w-4" />
                      Delete
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  {account.name}
                </CardTitle>
                <Badge variant="outline" className="uppercase text-[10px]">
                  {account.account_type}
                </Badge>
              </CardHeader>
              <CardContent>
                <div className="flex items-baseline gap-2">
                  <span className="text-2xl font-bold">
                    {parseFloat(account.balance).toLocaleString('en-US', {
                      style: 'currency',
                      currency: 'USD',
                    })}
                  </span>
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  Code: {account.code}
                </p>
              </CardContent>
            </Card>
          </Link>
        ))}
      </div>
    </div>
  )
}
