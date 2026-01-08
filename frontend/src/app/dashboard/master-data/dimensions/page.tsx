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
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Plus, Loader2 } from 'lucide-react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useDimensions, useCreateDimensionValue } from '@/lib/queries/dimensions'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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

const formSchema = z.object({
  code: z.string().min(1, 'Code is required'),
  name: z.string().min(2, 'Name must be at least 2 characters'),
  description: z.string().optional(),
})

export default function DimensionsPage() {
  const { data, isLoading } = useDimensions()
  const createDimension = useCreateDimensionValue()
  const [open, setOpen] = React.useState(false)
  const [activeTab, setActiveTab] = React.useState('DEPT')

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      code: '',
      name: '',
      description: '',
    },
  })

  // Find active dimension type ID based on code (DEPT/PROJ)
  const activeType = data?.find(d => d.code === activeTab)

  const onSubmit = (values: z.infer<typeof formSchema>) => {
    if (!activeType) return

    createDimension.mutate({
      typeId: activeType.id,
      ...values
    }, {
      onSuccess: () => {
        toast.success(`New ${activeType.name} created`)
        setOpen(false)
        form.reset()
      },
      onError: () => {
        toast.error('Failed to create dimension')
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
          <h1 className="text-3xl font-bold tracking-tight">Dimensions</h1>
          <p className="text-muted-foreground mt-2">
            Manage cost centers, projects, and other analytic dimensions.
          </p>
        </div>
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              New {activeType?.name}
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add New {activeType?.name}</DialogTitle>
              <DialogDescription>
                Create a new value for the {activeType?.name} dimension.
              </DialogDescription>
            </DialogHeader>
            <Form {...form}>
              <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
                <FormField
                  control={form.control}
                  name="code"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Code</FormLabel>
                      <FormControl>
                        <Input placeholder="e.g. ENG-001" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="name"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Name</FormLabel>
                      <FormControl>
                        <Input placeholder="e.g. Engineering Team" {...field} />
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
                      <FormLabel>Description (Optional)</FormLabel>
                      <FormControl>
                        <Input {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <div className="flex justify-end pt-4">
                  <Button type="submit" disabled={createDimension.isPending}>
                    {createDimension.isPending ? 'Saving...' : 'Save'}
                  </Button>
                </div>
              </form>
            </Form>
          </DialogContent>
        </Dialog>
      </div>

      <Tabs defaultValue="DEPT" onValueChange={setActiveTab}>
        <TabsList>
          {data?.map(dim => (
            <TabsTrigger key={dim.id} value={dim.code}>
              {dim.name}
            </TabsTrigger>
          ))}
        </TabsList>
        {data?.map(dim => (
          <TabsContent key={dim.id} value={dim.code}>
            <Card>
              <CardHeader>
                <CardTitle>{dim.name} List</CardTitle>
                <CardDescription>
                  Active {dim.name.toLowerCase()} values used in transactions.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[100px]">Code</TableHead>
                      <TableHead>Name</TableHead>
                      <TableHead>Description</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {dim.values.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={3} className="text-center h-24 text-muted-foreground">
                          No values found. Create one to get started.
                        </TableCell>
                      </TableRow>
                    ) : (
                      dim.values.map((val) => (
                        <TableRow key={val.id}>
                          <TableCell className="font-medium font-mono">{val.code}</TableCell>
                          <TableCell>{val.name}</TableCell>
                          <TableCell className="text-muted-foreground">{val.description || '-'}</TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          </TabsContent>
        ))}
      </Tabs>
    </div>
  )
}
