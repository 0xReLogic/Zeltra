'use client'

import React from 'react'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Plus } from 'lucide-react'
import { Input } from '@/components/ui/input'
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
import { toast } from 'sonner'
import { useCreateDimensionType, useDimensions } from '@/lib/queries/dimensions'
import { useOrganization } from '@/lib/queries/organizations'
import { useUpgradeStore } from '@/lib/stores/upgradeStore'

const typeSchema = z.object({
    code: z.string().min(2, 'Code must be at least 2 chars').max(10).toUpperCase(),
    name: z.string().min(2, 'Name is required'),
})

export function DimensionTypeDialog() {
    const [open, setOpen] = React.useState(false)
    const createType = useCreateDimensionType()
    const { data: dimensions } = useDimensions()
    const { data: org } = useOrganization()
    const { openModal } = useUpgradeStore()

    const handleOpenChange = (newOpen: boolean) => {
        if (newOpen) {
            const currentDimensions = dimensions?.dimension_types || []
            // If checking fails or types are off, default to allowing usage (fail open) unless we are sure
            const maxDimensions = org?.limits?.max_dimensions
            
            if (maxDimensions !== null && maxDimensions !== undefined && currentDimensions.length >= maxDimensions) {
                openModal(`Your plan is limited to ${maxDimensions} dimensions. Upgrade to add more.`)
                return
            }
        }
        setOpen(newOpen)
    }

    const form = useForm<z.infer<typeof typeSchema>>({
        resolver: zodResolver(typeSchema),
        defaultValues: { code: '', name: '' }
    })

    const onSubmit = (values: z.infer<typeof typeSchema>) => {
        createType.mutate(values, {
            onSuccess: () => {
                toast.success(`Created dimension type: ${values.name}`)
                setOpen(false)
                form.reset()
            },
            onError: () => toast.error('Failed to create dimension type')
        })
    }

    return (
        <Dialog open={open} onOpenChange={handleOpenChange}>
            <DialogTrigger asChild>
                <Button variant="outline">
                    <Plus className="mr-2 h-4 w-4" />
                    New Dimension Type
                </Button>
            </DialogTrigger>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>Create Dimension Type</DialogTitle>
                    <DialogDescription>
                        Add a new dimension for tracking and reporting (e.g. Cost Center, Region).
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
                                        <Input placeholder="e.g. COST_CENTER" {...field} onChange={e => field.onChange(e.target.value.toUpperCase())} />
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
                                        <Input placeholder="e.g. Cost Center" {...field} />
                                    </FormControl>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />
                        <div className="flex justify-end pt-4">
                            <Button type="submit" disabled={createType.isPending}>
                                Save
                            </Button>
                        </div>
                    </form>
                </Form>
            </DialogContent>
        </Dialog>
    )
}
