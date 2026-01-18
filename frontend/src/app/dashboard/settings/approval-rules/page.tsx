'use client'

import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Shield, Plus, AlertCircle } from 'lucide-react'
import { CreateApprovalRuleDialog } from '@/components/approval-rules/CreateApprovalRuleDialog'
import { ApprovalRulesTable } from '@/components/approval-rules/ApprovalRulesTable'
import { useApprovalRules } from '@/lib/queries/approval-rules'
import { TRANSACTION_TYPES, ROLES } from '@/lib/validations/approval-rule'
import { Skeleton } from '@/components/ui/skeleton'

export default function ApprovalRulesPage() {
  const [filters, setFilters] = useState({
    page: 1,
    per_page: 20,
    is_active: undefined as boolean | undefined,
    transaction_type: undefined as string | undefined,
    required_role: undefined as string | undefined,
    sort_by: 'priority',
    sort_order: 'asc' as 'asc' | 'desc',
  })

  const { data, isLoading, error } = useApprovalRules(filters)

  const handleFilterChange = (key: string, value: string | boolean | undefined) => {
    setFilters(prev => ({
      ...prev,
      [key]: value === 'all' ? undefined : value,
      page: 1, // Reset to first page when filtering
    }))
  }

  const clearFilters = () => {
    setFilters(prev => ({
      ...prev,
      is_active: undefined,
      transaction_type: undefined,
      required_role: undefined,
      page: 1,
    }))
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight">Approval Rules</h1>
            <p className="text-muted-foreground">
              Manage approval workflows for transactions
            </p>
          </div>
        </div>
        
        <Card>
          <CardContent className="flex items-center justify-center py-12">
            <div className="text-center space-y-4">
              <AlertCircle className="h-12 w-12 text-destructive mx-auto" />
              <div>
                <h3 className="text-lg font-semibold">Failed to load approval rules</h3>
                <p className="text-muted-foreground">
                  There was an error loading the approval rules. Please try again.
                </p>
              </div>
              <Button onClick={() => window.location.reload()}>
                Try Again
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Approval Rules</h1>
          <p className="text-muted-foreground">
            Manage approval workflows for transactions
          </p>
        </div>
        <CreateApprovalRuleDialog />
      </div>

      {/* Filters Bar */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Filters</CardTitle>
          <CardDescription>
            Filter approval rules by status, transaction type, or required role
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-4">
            <div className="flex-1 min-w-[200px]">
              <label className="text-sm font-medium mb-2 block">Status</label>
              <Select
                value={filters.is_active === undefined ? 'all' : filters.is_active.toString()}
                onValueChange={(value) => 
                  handleFilterChange('is_active', value === 'all' ? undefined : value === 'true')
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All</SelectItem>
                  <SelectItem value="true">Active</SelectItem>
                  <SelectItem value="false">Inactive</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex-1 min-w-[200px]">
              <label className="text-sm font-medium mb-2 block">Transaction Type</label>
              <Select
                value={filters.transaction_type || 'all'}
                onValueChange={(value) => handleFilterChange('transaction_type', value)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Types</SelectItem>
                  {TRANSACTION_TYPES.map((type) => (
                    <SelectItem key={type} value={type}>
                      <span className="capitalize">{type.replace('_', ' ')}</span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="flex-1 min-w-[200px]">
              <label className="text-sm font-medium mb-2 block">Required Role</label>
              <Select
                value={filters.required_role || 'all'}
                onValueChange={(value) => handleFilterChange('required_role', value)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Roles</SelectItem>
                  {ROLES.map((role) => (
                    <SelectItem key={role} value={role}>
                      <span className="capitalize">{role}</span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-end">
              <Button variant="outline" onClick={clearFilters}>
                Clear Filters
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Data Table */}
      {isLoading ? (
        <Card>
          <CardContent className="p-6">
            <div className="space-y-4">
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-8 w-full" />
            </div>
          </CardContent>
        </Card>
      ) : data?.data.length === 0 ? (
        <Card>
          <CardContent className="flex items-center justify-center py-12">
            <div className="text-center space-y-4">
              <Shield className="h-12 w-12 text-muted-foreground mx-auto" />
              <div>
                <h3 className="text-lg font-semibold">No approval rules found</h3>
                <p className="text-muted-foreground">
                  {Object.values(filters).some(v => v !== undefined && v !== 'priority' && v !== 'asc' && v !== 1 && v !== 20)
                    ? 'No rules match your current filters. Try adjusting the filters above.'
                    : 'Get started by creating your first approval rule.'
                  }
                </p>
              </div>
              <CreateApprovalRuleDialog>
                <Button>
                  <Plus className="h-4 w-4 mr-2" />
                  Create First Rule
                </Button>
              </CreateApprovalRuleDialog>
            </div>
          </CardContent>
        </Card>
      ) : data ? (
        <ApprovalRulesTable 
          data={data} 
          filters={filters}
          onFiltersChange={setFilters}
        />
      ) : null}
    </div>
  )
}