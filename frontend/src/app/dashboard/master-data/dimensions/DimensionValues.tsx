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
import { Plus, MoreHorizontal, Pencil, Loader2 } from 'lucide-react'
import { useCreateDimensionValue, useUpdateDimensionValue, useDimensionValues } from '@/lib/queries/dimensions'
import type { DimensionType, DimensionValue } from '@/types/dimensions'
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
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

const valueSchema = z.object({
    code: z.string().min(1, 'Code is required'),
    name: z.string().min(2, 'Name must be at least 2 characters'),
    description: z.string().optional(),
})

interface DimensionValuesProps {
    dimension: DimensionType
}

export function DimensionValues({ dimension }: DimensionValuesProps) {
    const { data: valuesData, isLoading } = useDimensionValues(dimension.id)
    const createDimension = useCreateDimensionValue()
    const updateDimension = useUpdateDimensionValue()
    const [open, setOpen] = React.useState(false)
    const [editingValue, setEditingValue] = React.useState<DimensionValue | null>(null)

    const values = Array.isArray(valuesData) ? valuesData : []

    const form = useForm<z.infer<typeof valueSchema>>({
        resolver: zodResolver(valueSchema),
        defaultValues: {
            code: '',
            name: '',
            description: '',
        },
    })

    // Reset form when dialog opens/closes
    React.useEffect(() => {
        if (open) {
            if (editingValue) {
                form.reset({
                    code: editingValue.code,
                    name: editingValue.name,
                    description: '',
                })
            } else {
                form.reset({
                    code: '',
                    name: '',
                    description: '',
                })
            }
        }
    }, [open, editingValue, form])

    const onSubmit = (formValues: z.infer<typeof valueSchema>) => {
        if (editingValue) {
            updateDimension.mutate({
                id: editingValue.code, // Use code as ID since DimensionValueResponse doesn't have id
                data: {
                    code: formValues.code,
                    name: formValues.name,
                    description: formValues.description,
                }
            }, {
                onSuccess: () => {
                    toast.success(`Updated ${formValues.name}`)
                    setOpen(false)
                    setEditingValue(null)
                },
                onError: () => toast.error('Failed to update')
            })
        } else {
            createDimension.mutate({
                dimension_type_id: dimension.id,
                code: formValues.code,
                name: formValues.name,
                description: formValues.description,
            }, {
                onSuccess: () => {
                    toast.success(`Created ${formValues.name}`)
                    setOpen(false)
                },
                onError: () => toast.error('Failed to create')
            })
        }
    }

    if (isLoading) {
        return (
            <Card>
                <CardContent className="flex items-center justify-center py-12">
                    <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                </CardContent>
            </Card>
        )
    }

    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between">
                <div>
                    <CardTitle>{dimension.name} List</CardTitle>
                    <CardDescription>
                        Active {dimension.name.toLowerCase()} values used in transactions.
                    </CardDescription>
                </div>
                <Dialog open={open} onOpenChange={(val) => {
                    setOpen(val)
                    if (!val) setEditingValue(null)
                }}>
                    <DialogTrigger asChild>
                        <Button size="sm" onClick={() => setEditingValue(null)}>
                            <Plus className="mr-2 h-4 w-4" />
                            New {dimension.name}
                        </Button>
                    </DialogTrigger>
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>{editingValue ? 'Edit' : 'Create'} {dimension.name}</DialogTitle>
                            <DialogDescription>
                                {editingValue ? 'Update value details.' : `Create a new value for ${dimension.name}.`}
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
                                                <Input placeholder="e.g. ENG-001" {...field} disabled={!!editingValue} />
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
                                                <Input placeholder="e.g. Engineering" {...field} />
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
                                                <Input {...field} />
                                            </FormControl>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />
                                <div className="flex justify-end pt-4">
                                    <Button type="submit" disabled={createDimension.isPending || updateDimension.isPending}>
                                        Save
                                    </Button>
                                </div>
                            </form>
                        </Form>
                    </DialogContent>
                </Dialog>
            </CardHeader>
            <CardContent>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="w-[100px]">Code</TableHead>
                            <TableHead>Name</TableHead>
                            <TableHead>Dimension Type</TableHead>
                            <TableHead className="w-[50px]"></TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {values.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={4} className="text-center h-24 text-muted-foreground">
                                    No values found.
                                </TableCell>
                            </TableRow>
                        ) : (
                            values.map((val) => (
                                <TableRow key={val.code}>
                                    <TableCell className="font-medium font-mono">{val.code}</TableCell>
                                    <TableCell>{val.name}</TableCell>
                                    <TableCell className="text-muted-foreground">{val.dimension_type}</TableCell>
                                    <TableCell>
                                        <DropdownMenu>
                                            <DropdownMenuTrigger asChild>
                                                <Button variant="ghost" className="h-8 w-8 p-0">
                                                    <MoreHorizontal className="h-4 w-4" />
                                                </Button>
                                            </DropdownMenuTrigger>
                                            <DropdownMenuContent align="end">
                                                <DropdownMenuItem onClick={() => {
                                                    setEditingValue(val)
                                                    setOpen(true)
                                                }}>
                                                    <Pencil className="mr-2 h-4 w-4" />
                                                    Edit
                                                </DropdownMenuItem>
                                            </DropdownMenuContent>
                                        </DropdownMenu>
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </CardContent>
        </Card>
    )
}
