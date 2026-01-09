'use client'

import { useState } from 'react'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import * as z from 'zod'
import { format } from 'date-fns'
import { CalendarIcon, Loader2 } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'
import { Calendar } from '@/components/ui/calendar'
import { cn } from '@/lib/utils'
import { useCreateTransaction } from '@/lib/queries/transactions'
import { useAccounts } from '@/lib/queries/accounts'
import { useDimensions } from '@/lib/queries/dimensions'
import { toast } from 'sonner'

const formSchema = z.object({
  transaction_type: z.enum(['expense', 'revenue', 'transfer', 'journal']),
  transaction_date: z.date(),
  reference_number: z.string().min(1, 'Reference number is required'),
  description: z.string().min(1, 'Description is required'),
  amount: z.string().refine((val) => !isNaN(parseFloat(val)) && parseFloat(val) > 0, {
    message: 'Amount must be a positive number',
  }),
  main_account: z.string().min(1, 'Account is required'), // e.g. Bank
  contra_account: z.string().min(1, 'Category/Contra account is required'), // e.g. Expense
  department: z.string().optional(),
  project: z.string().optional(),
})

export function CreateTransactionDialog() {
  const [open, setOpen] = useState(false)
  const createMutation = useCreateTransaction()
  const { data: accountsData } = useAccounts()
  const { data: dimensionsData } = useDimensions()

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      transaction_type: 'expense',
      reference_number: 'REF-NEW',
      description: '',
      amount: '',
      department: '',
      project: 'none',
    },
  })




  // Filter accounts based on logic if needed, for now show all or split by type
  // Usually:
  // Expense Txn: Credit Asset (Bank), Debit Expense
  // Revenue Txn: Debit Asset (Bank), Credit Revenue
  
  function onSubmit(values: z.infer<typeof formSchema>) {
    // Construct entries based on type
    const amount = values.amount
    let entries = []

    // Construct dimensions array
    const dims: string[] = []
    if (values.department) dims.push(values.department)
    if (values.project && values.project !== 'none') dims.push(values.project)

    if (values.transaction_type === 'expense') {
        // Dr Expense (with Dims), Cr Asset
        entries = [
            { account_code: values.contra_account, debit: amount, credit: '0', dimensions: dims },
            { account_code: values.main_account, debit: '0', credit: amount }
        ]
    } else if (values.transaction_type === 'revenue') {
        // Dr Asset, Cr Revenue (with Dims)
        entries = [
            { account_code: values.main_account, debit: amount, credit: '0' },
            { account_code: values.contra_account, debit: '0', credit: amount, dimensions: dims }
        ]
    } else {
        // Fallback for transfer/journal (simplified)
        entries = [
             { account_code: values.main_account, debit: amount, credit: '0' },
             { account_code: values.contra_account, debit: '0', credit: amount, dimensions: dims }
        ]
    }

    createMutation.mutate({
        ...values,
        transaction_date: format(values.transaction_date, 'yyyy-MM-dd'),
        entries
    }, {
        onSuccess: () => {
            toast.success('Transaction created successfully')
            setOpen(false)
            form.reset()
        },
        onError: (error) => {
            toast.error(error.message || 'Failed to create transaction')
        }
    })
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>Create Transaction</Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Create Transaction</DialogTitle>
          <DialogDescription>
            Record a new transaction. Entries will be automatically generated.
          </DialogDescription>
        </DialogHeader>
        
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            
            <div className="grid grid-cols-2 gap-4">
                <FormField
                control={form.control}
                name="transaction_type"
                render={({ field }) => (
                    <FormItem>
                    <FormLabel>Type</FormLabel>
                    <Select onValueChange={field.onChange} defaultValue={field.value}>
                        <FormControl>
                        <SelectTrigger>
                            <SelectValue placeholder="Select type" />
                        </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                        <SelectItem value="expense">Expense</SelectItem>
                        <SelectItem value="revenue">Revenue</SelectItem>
                        <SelectItem value="transfer">Transfer</SelectItem>
                        </SelectContent>
                    </Select>
                    <FormMessage />
                    </FormItem>
                )}
                />

                <FormField
                    control={form.control}
                    name="transaction_date"
                    render={({ field }) => (
                        <FormItem className="flex flex-col mt-2.5">
                        <FormLabel>Date</FormLabel>
                        <Popover>
                            <PopoverTrigger asChild>
                            <FormControl>
                                <Button
                                variant={"outline"}
                                className={cn(
                                    "w-full pl-3 text-left font-normal",
                                    !field.value && "text-muted-foreground"
                                )}
                                >
                                {field.value ? (
                                    format(field.value, "PPP")
                                ) : (
                                    <span>Pick a date</span>
                                )}
                                <CalendarIcon className="ml-auto h-4 w-4 opacity-50" />
                                </Button>
                            </FormControl>
                            </PopoverTrigger>
                            <PopoverContent className="w-auto p-0" align="start">
                            <Calendar
                                mode="single"
                                selected={field.value}
                                onSelect={field.onChange}
                                disabled={(date) =>
                                date > new Date() || date < new Date("1900-01-01")
                                }
                                initialFocus
                            />
                            </PopoverContent>
                        </Popover>
                        <FormMessage />
                        </FormItem>
                    )}
                    />
            </div>

            <FormField
              control={form.control}
              name="reference_number"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Reference Number</FormLabel>
                  <FormControl>
                    <Input placeholder="REF-001" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Description</FormLabel>
                  <FormControl>
                    <Input placeholder="e.g. Server costs" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            {/* Dimensions Section */}
            <div className="grid grid-cols-2 gap-4">
                 <FormField
                    control={form.control}
                    name="department"
                    render={({ field }) => (
                        <FormItem>
                        <FormLabel>Department</FormLabel>
                        <Select onValueChange={field.onChange} defaultValue={field.value}>
                            <FormControl>
                            <SelectTrigger>
                                <SelectValue placeholder="Select Dept" />
                            </SelectTrigger>
                            </FormControl>
                            <SelectContent>
                                {dimensionsData?.find(d => d.code === 'DEPT')?.values.map((v) => (
                                    <SelectItem key={v.id} value={v.id}>{v.name}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        <FormMessage />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name="project"
                    render={({ field }) => (
                        <FormItem>
                        <FormLabel>Project (Optional)</FormLabel>
                         <Select onValueChange={field.onChange} defaultValue={field.value}>
                            <FormControl>
                            <SelectTrigger>
                                <SelectValue placeholder="Select Project" />
                            </SelectTrigger>
                            </FormControl>
                            <SelectContent>
                                 <SelectItem value="none">None</SelectItem>
                                {dimensionsData?.find(d => d.code === 'PROJ')?.values.map((v) => (
                                    <SelectItem key={v.id} value={v.id}>{v.name}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        <FormMessage />
                        </FormItem>
                    )}
                />
            </div>

            <div className="grid grid-cols-2 gap-4">
                <FormField
                control={form.control}
                name="main_account"
                render={({ field }) => (
                    <FormItem>
                    <FormLabel>Paid From / To</FormLabel>
                    <Select onValueChange={field.onChange} defaultValue={field.value}>
                        <FormControl>
                        <SelectTrigger>
                            <SelectValue placeholder="Select account" />
                        </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                            {accountsData?.data
                                .filter(a => a.account_type === 'asset' || a.account_type === 'liability')
                                .map((acc) => (
                                <SelectItem key={acc.id} value={acc.code}>
                                    {acc.name}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                    <FormMessage />
                    </FormItem>
                )}
                />

                <FormField
                control={form.control}
                name="contra_account"
                render={({ field }) => (
                    <FormItem>
                    <FormLabel>Category (Account)</FormLabel>
                    <Select onValueChange={field.onChange} defaultValue={field.value}>
                        <FormControl>
                        <SelectTrigger>
                            <SelectValue placeholder="Select category" />
                        </SelectTrigger>
                        </FormControl>
                         <SelectContent>
                            {accountsData?.data
                                .filter(a => a.account_type === 'expense' || a.account_type === 'revenue')
                                .map((acc) => (
                                <SelectItem key={acc.id} value={acc.code}>
                                    {acc.name}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                    <FormMessage />
                    </FormItem>
                )}
                />
            </div>

            <FormField
              control={form.control}
              name="amount"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Amount</FormLabel>
                  <FormControl>
                    <Input type="number" placeholder="0.00" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                Create transaction
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
