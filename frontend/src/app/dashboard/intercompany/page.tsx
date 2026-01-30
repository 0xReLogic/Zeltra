'use client'

import React, { useState } from 'react'
import { Building2, Link2, Plus, Lock, Loader2, Check, X } from 'lucide-react'
import { useIntercompanyMappings, useCreateIntercompanyMapping } from '@/lib/queries/sentinel'
import { useAccounts } from '@/lib/queries/accounts'
import { useUserSubscription } from '@/lib/queries/auth'
import { useAuthStore } from '@/lib/stores/authStore'
import { useUpgradeStore } from '@/lib/stores/upgradeStore'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Label } from '@/components/ui/label'
import { Skeleton } from '@/components/ui/skeleton'
import { toast } from 'sonner'
import type { CreateIntercompanyMappingRequest } from '@/types/api-helpers'

export default function IntercompanyPage() {
  const { data: subscription } = useUserSubscription()
  const { openModal } = useUpgradeStore()
  const { data: mappings, isLoading, isError, refetch } = useIntercompanyMappings()
  const { data: accountsData } = useAccounts()
  const userOrganizations = useAuthStore((state) => state.user?.organizations ?? [])
  const createMapping = useCreateIntercompanyMapping()
  
  const [isOpen, setIsOpen] = useState(false)
  const [formData, setFormData] = useState<Partial<CreateIntercompanyMappingRequest>>({})

  // Check tier access - intercompany is enterprise-only
  const hasIntercompany = subscription?.subscription_tier === 'enterprise'

  // Show loading state first
  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <Skeleton className="h-9 w-48" />
          <Skeleton className="h-10 w-40" />
        </div>
        <div className="grid gap-4 md:grid-cols-2">
          {[1, 2].map(i => (
            <Card key={i}>
              <CardHeader className="pb-2">
                <Skeleton className="h-4 w-24" />
              </CardHeader>
              <CardContent>
                <Skeleton className="h-8 w-16" />
              </CardContent>
            </Card>
          ))}
        </div>
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-32" />
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {[1, 2, 3].map(i => (
                <Skeleton key={i} className="h-12 w-full" />
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  // Show upgrade prompt if tier not available (check after loading)
  if (subscription && !hasIntercompany) {
    return (
      <div className="space-y-6">
        <h1 className="text-3xl font-bold tracking-tight">Intercompany Hub</h1>
        <Card className="border-amber-500/50">
          <CardContent className="flex flex-col items-center justify-center py-16 text-center">
            <div className="rounded-full bg-amber-500/10 p-4 mb-4">
              <Lock className="h-8 w-8 text-amber-500" />
            </div>
            <h2 className="text-xl font-semibold mb-2">Enterprise Feature</h2>
            <p className="text-muted-foreground mb-6 max-w-md">
              Intercompany Hub is an Enterprise feature that helps you manage 
              cross-entity transactions, elimination entries, and consolidation.
            </p>
            <Button onClick={() => openModal('Unlock Intercompany Hub and other Enterprise features.')}>
              Upgrade to Enterprise
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  // Show error state
  if (isError) {
    return (
      <div className="space-y-6">
        <h1 className="text-3xl font-bold tracking-tight">Intercompany Hub</h1>
        <Card className="border-destructive/50">
          <CardContent className="flex flex-col items-center justify-center py-16 text-center">
            <p className="text-destructive mb-4">Failed to load intercompany mappings</p>
            <Button variant="outline" onClick={() => refetch()}>
              Try Again
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  const accounts = accountsData?.accounts ?? []
  const organizations = userOrganizations
  const mappingList = Array.isArray(mappings) ? mappings : []

  // Helper to get account name by ID
  const getAccountName = (accountId: string) => {
    const account = accounts.find(a => a.id === accountId)
    return account ? `${account.code} - ${account.name}` : accountId.slice(0, 8) + '...'
  }

  // Helper to get org name by ID
  const getOrgName = (orgId: string) => {
    const organization = organizations.find(o => o.id === orgId)
    return organization?.name || orgId.slice(0, 8) + '...'
  }

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    
    if (!formData.source_account_id || !formData.target_org_id || !formData.target_account_id) {
      toast.error('Please fill all required fields')
      return
    }

    createMapping.mutate(formData as CreateIntercompanyMappingRequest, {
      onSuccess: () => {
        toast.success('Intercompany mapping created successfully')
        setIsOpen(false)
        setFormData({})
      },
      onError: (error) => {
        toast.error(error.message || 'Failed to create mapping')
      }
    })
  }

  const getMappingTypeBadge = (type: string) => {
    switch (type) {
      case 'elimination':
        return <Badge className="bg-purple-500/10 text-purple-600 hover:bg-purple-500/20">Elimination</Badge>
      case 'mirror':
        return <Badge className="bg-blue-500/10 text-blue-600 hover:bg-blue-500/20">Mirror</Badge>
      default:
        return <Badge variant="outline">{type}</Badge>
    }
  }

  // Calculate summary stats
  const eliminationCount = mappingList.filter(m => m.mapping_type === 'elimination').length
  const mirrorCount = mappingList.filter(m => m.mapping_type === 'mirror').length

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Intercompany Hub</h1>
          <p className="text-muted-foreground mt-1">
            Manage cross-entity transactions and elimination entries.
          </p>
        </div>
        <Dialog open={isOpen} onOpenChange={setIsOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" /> Connect Organizations
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[450px]">
            <DialogHeader>
              <DialogTitle>Create Intercompany Mapping</DialogTitle>
              <DialogDescription>
                Connect accounts between organizations for automatic transaction mirroring or elimination.
              </DialogDescription>
            </DialogHeader>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="source_account_id">Source Account *</Label>
                <Select
                  value={formData.source_account_id || ''}
                  onValueChange={(value) => setFormData(prev => ({ ...prev, source_account_id: value }))}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select source account" />
                  </SelectTrigger>
                  <SelectContent>
                    {accounts.map((acc) => (
                      <SelectItem key={acc.id} value={acc.id}>
                        {acc.code} - {acc.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  The account in your organization that will be linked.
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="target_org_id">Target Organization *</Label>
                <Select
                  value={formData.target_org_id || ''}
                  onValueChange={(value) => setFormData(prev => ({ ...prev, target_org_id: value }))}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select target organization" />
                  </SelectTrigger>
                  <SelectContent>
                    {organizations
                      .filter(o => o.id !== org?.id)
                      .map((organization) => (
                        <SelectItem key={organization.id} value={organization.id}>
                          {organization.name}
                        </SelectItem>
                      ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  The organization to connect with.
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="target_account_id">Target Account *</Label>
                <Select
                  value={formData.target_account_id || ''}
                  onValueChange={(value) => setFormData(prev => ({ ...prev, target_account_id: value }))}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select target account" />
                  </SelectTrigger>
                  <SelectContent>
                    {accounts.map((acc) => (
                      <SelectItem key={acc.id} value={acc.id}>
                        {acc.code} - {acc.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  The corresponding account in the target organization.
                </p>
              </div>

              <DialogFooter>
                <Button type="button" variant="outline" onClick={() => setIsOpen(false)}>
                  Cancel
                </Button>
                <Button type="submit" disabled={createMapping.isPending}>
                  {createMapping.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  Create Mapping
                </Button>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Elimination Mappings</CardTitle>
            <Link2 className="h-4 w-4 text-purple-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{eliminationCount}</div>
            <p className="text-xs text-muted-foreground">For consolidation</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Mirror Mappings</CardTitle>
            <Building2 className="h-4 w-4 text-blue-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mirrorCount}</div>
            <p className="text-xs text-muted-foreground">Auto-post enabled</p>
          </CardContent>
        </Card>
      </div>

      {/* Mappings Table */}
      <Card>
        <CardHeader>
          <CardTitle>Intercompany Mappings</CardTitle>
          <CardDescription>
            Account mappings between organizations for transaction processing
          </CardDescription>
        </CardHeader>
        <CardContent>
          {mappingList.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <Building2 className="h-12 w-12 text-muted-foreground mb-4" />
              <h3 className="text-lg font-semibold mb-2">No Intercompany Mappings</h3>
              <p className="text-muted-foreground mb-4 max-w-sm">
                Connect organizations to enable automatic transaction mirroring 
                and elimination entries for consolidation.
              </p>
              <Button onClick={() => setIsOpen(true)}>
                <Plus className="mr-2 h-4 w-4" /> Connect Organizations
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Source Organization</TableHead>
                  <TableHead>Source Account</TableHead>
                  <TableHead>Target Organization</TableHead>
                  <TableHead>Target Account</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Auto-Post</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {mappingList.map((mapping) => (
                  <TableRow key={mapping.id}>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <Building2 className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">{getOrgName(mapping.source_org_id)}</span>
                      </div>
                    </TableCell>
                    <TableCell>{getAccountName(mapping.source_account_id)}</TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <Building2 className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">{getOrgName(mapping.target_org_id)}</span>
                      </div>
                    </TableCell>
                    <TableCell>{getAccountName(mapping.target_account_id)}</TableCell>
                    <TableCell>{getMappingTypeBadge(mapping.mapping_type)}</TableCell>
                    <TableCell>
                      {mapping.auto_post ? (
                        <div className="flex items-center gap-1 text-green-600">
                          <Check className="h-4 w-4" />
                          <span className="text-sm">Enabled</span>
                        </div>
                      ) : (
                        <div className="flex items-center gap-1 text-muted-foreground">
                          <X className="h-4 w-4" />
                          <span className="text-sm">Disabled</span>
                        </div>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
