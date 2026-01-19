'use client'

import { useState, useEffect, useRef } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Shield, Plus, AlertCircle } from 'lucide-react'
import { CreateApprovalRuleDialog } from '@/components/approval-rules/CreateApprovalRuleDialog'
import { ApprovalRulesTable } from '@/components/approval-rules/ApprovalRulesTable'
import { FiltersBar } from '@/components/approval-rules/FiltersBar'
import { useApprovalRules } from '@/lib/queries/approval-rules'
import { Skeleton } from '@/components/ui/skeleton'

export default function ApprovalRulesPage() {
  const [filters, setFilters] = useState({
    page: 1,
    per_page: 20,
    search: undefined as string | undefined,
    is_active: undefined as boolean | undefined,
    transaction_type: undefined as string | undefined,
    required_role: undefined as string | undefined,
    sort_by: 'priority',
    sort_order: 'asc' as 'asc' | 'desc',
  })

  const createButtonRef = useRef<HTMLButtonElement>(null)
  const { data, isLoading, error } = useApprovalRules(filters)

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+N or Cmd+N to open create dialog
      if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
        e.preventDefault()
        createButtonRef.current?.click()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  const handleFiltersChange = (newFilters: {
    page?: number
    per_page?: number
    search?: string
    is_active?: boolean
    transaction_type?: string
    required_role?: string
    sort_by?: string
    sort_order?: 'asc' | 'desc'
  }) => {
    setFilters((prev) => ({
      ...prev,
      ...newFilters,
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
        <CreateApprovalRuleDialog>
          <Button ref={createButtonRef}>
            <Plus className="h-4 w-4 mr-2" />
            Create Rule
          </Button>
        </CreateApprovalRuleDialog>
      </div>

      {/* Filters Bar */}
      <FiltersBar filters={filters} onFiltersChange={handleFiltersChange} />

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
          onFiltersChange={handleFiltersChange}
        />
      ) : null}
    </div>
  )
}