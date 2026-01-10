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
import { Plus, MoreHorizontal, Pencil, Ban, CheckCircle } from 'lucide-react'
import { DimensionType, DimensionValue, useCreateDimensionValue, useEditDimensionValue, useToggleDimensionValueActive } from '@/lib/queries/dimensions'
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
    const createDimension = useCreateDimensionValue()
    const editDimension = useEditDimensionValue()
    const toggleDimension = useToggleDimensionValueActive()
    const [open, setOpen] = React.useState(false)
    const [editingValue, setEditingValue] = React.useState<DimensionValue | null>(null)

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
                    description: editingValue.description || '',
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

    const onSubmit = (values: z.infer<typeof valueSchema>) => {
        if (editingValue) {
            editDimension.mutate({
                typeId: dimension.id,
                id: editingValue.id,
                ...values
            }, {
                onSuccess: () => {
                    toast.success(`Updated ${values.name}`)
                    setOpen(false)
                    setEditingValue(null)
                },
                onError: () => toast.error('Failed to update')
            })
        } else {
            createDimension.mutate({
                typeId: dimension.id,
                ...values
            }, {
                onSuccess: () => {
                    toast.success(`Created ${values.name}`)
                    setOpen(false)
                },
                onError: () => toast.error('Failed to create')
            })
        }
    }

    const handleToggle = (val: DimensionValue) => {
        toggleDimension.mutate({
            typeId: dimension.id,
            id: val.id,
            isActive: !(val.is_active ?? true)
        }, {
            onSuccess: () => toast.success(`Value ${val.is_active === false ? 'activated' : 'deactivated'}`)
        })
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
                                    <Button type="submit" disabled={createDimension.isPending || editDimension.isPending}>
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
                            <TableHead>Description</TableHead>
                            <TableHead className="w-[100px] text-right">Status</TableHead>
                            <TableHead className="w-[50px]"></TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {dimension.values.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={5} className="text-center h-24 text-muted-foreground">
                                    No values found.
                                </TableCell>
                            </TableRow>
                        ) : (
                            dimension.values.map((val) => (
                                <TableRow key={val.id} className={val.is_active === false ? 'opacity-50' : ''}>
                                    <TableCell className="font-medium font-mono">{val.code}</TableCell>
                                    <TableCell>{val.name}</TableCell>
                                    <TableCell className="text-muted-foreground">{val.description || '-'}</TableCell>
                                    <TableCell className="text-right">
                                        <span className={`text-xs px-2 py-1 rounded-full ${val.is_active !== false ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-700'}`}>
                                            {val.is_active !== false ? 'Active' : 'Inactive'}
                                        </span>
                                    </TableCell>
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
                                                <DropdownMenuItem onClick={() => handleToggle(val)}>
                                                    {val.is_active === false ? (
                                                        <><CheckCircle className="mr-2 h-4 w-4" /> Activate</>
                                                    ) : (
                                                        <><Ban className="mr-2 h-4 w-4" /> Deactivate</>
                                                    )}
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
