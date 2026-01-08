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
import { Plus, Loader2 } from 'lucide-react'
import { useExchangeRates, useCreateExchangeRate } from '@/lib/queries/exchange-rates'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { toast } from 'sonner'
import { useForm } from 'react-hook-form'
import { z } from 'zod'
import { zodResolver } from '@hookform/resolvers/zod'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

const formSchema = z.object({
  from_currency: z.string().min(1, 'Currency is required'),
  to_currency: z.string().min(1, 'Target currency is required'),
  rate: z.string().min(1, 'Rate is required'),
  date: z.string().min(1, 'Date is required'),
})

export default function ExchangeRatesPage() {
  const { data, isLoading } = useExchangeRates()
  const createRate = useCreateExchangeRate()
  const [open, setOpen] = React.useState(false)

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      from_currency: 'USD',
      to_currency: 'IDR',
      rate: '',
      date: new Date().toISOString().split('T')[0],
    },
  })

  const onSubmit = (values: z.infer<typeof formSchema>) => {
    createRate.mutate(values, {
      onSuccess: () => {
        toast.success('Exchange rate added')
        setOpen(false)
        form.reset({
          ...values,
          rate: ''
        })
      },
      onError: () => {
        toast.error('Failed to add exchange rate')
      }
    })
  }

  if (isLoading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Exchange Rates</h1>
          <p className="text-muted-foreground mt-2">
            Manage daily exchange rates for multi-currency transactions.
          </p>
        </div>
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              Add Rate
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add New Exchange Rate</DialogTitle>
              <DialogDescription>
                Set a conversion rate for a specific date.
              </DialogDescription>
            </DialogHeader>
            <Form {...form}>
              <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <FormField
                    control={form.control}
                    name="from_currency"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>From</FormLabel>
                        <Select onValueChange={field.onChange} defaultValue={field.value}>
                          <FormControl>
                            <SelectTrigger>
                              <SelectValue placeholder="Select currency" />
                            </SelectTrigger>
                          </FormControl>
                          <SelectContent>
                            <SelectItem value="USD">USD</SelectItem>
                            <SelectItem value="SGD">SGD</SelectItem>
                            <SelectItem value="EUR">EUR</SelectItem>
                          </SelectContent>
                        </Select>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <FormField
                    control={form.control}
                    name="to_currency"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>To</FormLabel>
                         <FormControl>
                            <Input {...field} readOnly className="bg-muted" />
                          </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>

                <FormField
                  control={form.control}
                  name="rate"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Exchange Rate</FormLabel>
                      <FormControl>
                        <Input type="number" step="0.01" placeholder="e.g. 15500" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="date"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Date</FormLabel>
                      <FormControl>
                        <Input type="date" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <div className="flex justify-end pt-4">
                  <Button type="submit" disabled={createRate.isPending}>
                    {createRate.isPending ? 'Saving...' : 'Save Rate'}
                  </Button>
                </div>
              </form>
            </Form>
          </DialogContent>
        </Dialog>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Rate History</CardTitle>
          <CardDescription>
             List of recorded exchange rates ordered by date.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Date</TableHead>
                <TableHead>From</TableHead>
                <TableHead>To</TableHead>
                <TableHead>Rate</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data?.data.sort((a,b) => b.date.localeCompare(a.date)).map((rate) => (
                <TableRow key={rate.id}>
                  <TableCell>{rate.date}</TableCell>
                  <TableCell className="font-medium">{rate.from_currency}</TableCell>
                  <TableCell>{rate.to_currency}</TableCell>
                  <TableCell>
                    {parseFloat(rate.rate).toLocaleString('id-ID')}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
