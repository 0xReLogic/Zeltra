'use client'

import React from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Loader2, Save } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Form,
  FormControl,
  FormDescription,
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
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useOrganization, useUpdateOrganization } from '@/lib/queries/organizations'
import { toast } from 'sonner'

const orgSettingsSchema = z.object({
  base_currency: z.string().min(1, 'Base currency is required'),
  timezone: z.string().min(1, 'Timezone is required'),
})

type OrgSettingsValues = z.infer<typeof orgSettingsSchema>

export default function OrganizationSettingsPage() {

  const { data: org, isLoading } = useOrganization()
  const updateOrg = useUpdateOrganization()

  const form = useForm<OrgSettingsValues>({
    resolver: zodResolver(orgSettingsSchema),
    defaultValues: {
      base_currency: '',
      timezone: '',
    },
  })

  // Reset form when data loads
  React.useEffect(() => {
    if (org) {
      form.reset({
        base_currency: org.base_currency,
        timezone: org.timezone,
      })
    }
  }, [org, form])

  async function onSubmit(data: OrgSettingsValues) {
    updateOrg.mutate(data, {
      onSuccess: () => {
        toast.success('Organization updated', {
          description: 'Your organization settings have been saved.',
        })
      },
      onError: (error) => {
        toast.error('Error', {
          description: error.message || 'Failed to update organization.',
        })
      }
    })
  }

  if (isLoading) {
    return (
      <div className="flex h-96 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium">Organization Settings</h3>
        <p className="text-sm text-muted-foreground">
          Manage your organization&apos;s general preferences.
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>General Information</CardTitle>
          <CardDescription>
            Update your organization&apos;s base currency and timezone.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
              <div className="grid gap-4 md:grid-cols-2">
                 <div className="space-y-2">
                    <FormLabel>Organization Name</FormLabel>
                    <Input value={org?.name} disabled readOnly />
                    <p className="text-[0.8rem] text-muted-foreground">
                      Contact support to change organization name.
                    </p>
                 </div>
                 <div className="space-y-2">
                    <FormLabel>Subscription Plan</FormLabel>
                    <div className="flex items-center space-x-2">
                       <Input value={org?.subscription_tier.toUpperCase()} disabled readOnly className="w-1/2"/>
                       <span className="text-xs text-emerald-600 font-medium px-2 py-1 bg-emerald-100 rounded-full">
                         Active
                       </span>
                    </div>
                 </div>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <FormField
                  control={form.control}
                  name="base_currency"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Base Currency</FormLabel>
                      <Select onValueChange={field.onChange} defaultValue={field.value} value={field.value}>
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue placeholder="Select currency" />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                          <SelectItem value="USD">USD - US Dollar</SelectItem>
                          <SelectItem value="IDR">IDR - Indonesian Rupiah</SelectItem>
                          <SelectItem value="SGD">SGD - Singapore Dollar</SelectItem>
                          <SelectItem value="EUR">EUR - Euro</SelectItem>
                        </SelectContent>
                      </Select>
                      <FormDescription>
                        The primary currency used for reporting and consolidation.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <FormField
                  control={form.control}
                  name="timezone"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Timezone</FormLabel>
                      <Select onValueChange={field.onChange} defaultValue={field.value} value={field.value}>
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue placeholder="Select timezone" />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                          <SelectItem value="UTC">UTC (Coordinated Universal Time)</SelectItem>
                          <SelectItem value="Asia/Jakarta">Asia/Jakarta (WIB)</SelectItem>
                          <SelectItem value="Asia/Singapore">Asia/Singapore (SGT)</SelectItem>
                          <SelectItem value="America/New_York">America/New_York (EST)</SelectItem>
                        </SelectContent>
                      </Select>
                      <FormDescription>
                        Used for timestamping transactions and reports.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>

              <div className="flex justify-end">
                <Button type="submit" disabled={updateOrg.isPending}>
                  {updateOrg.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  <Save className="mr-2 h-4 w-4" />
                  Save Changes
                </Button>
              </div>
            </form>
          </Form>
        </CardContent>
      </Card>
    </div>
  )
}
