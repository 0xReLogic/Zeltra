'use client'

import { useState, useEffect } from 'react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Search, X } from 'lucide-react'
import { useDebounce } from '@/lib/hooks/useDebounce'

interface FiltersBarProps {
  filters: {
    page?: number
    per_page?: number
    search?: string
    is_active?: boolean
    transaction_type?: string
    required_role?: string
  }
  onFiltersChange: (filters: {
    page?: number
    per_page?: number
    search?: string
    is_active?: boolean
    transaction_type?: string
    required_role?: string
    sort_by?: string
    sort_order?: 'asc' | 'desc'
  }) => void
}

const TRANSACTION_TYPES = [
  { value: 'bill', label: 'Bill' },
  { value: 'invoice', label: 'Invoice' },
  { value: 'journal', label: 'Journal' },
  { value: 'payment', label: 'Payment' },
  { value: 'expense', label: 'Expense' },
  { value: 'transfer', label: 'Transfer' },
  { value: 'accrual', label: 'Accrual' },
  { value: 'revaluation', label: 'Revaluation' },
  { value: 'intercompany', label: 'Intercompany' },
]

const ROLES = [
  { value: 'viewer', label: 'Viewer' },
  { value: 'submitter', label: 'Submitter' },
  { value: 'approver', label: 'Approver' },
  { value: 'accountant', label: 'Accountant' },
  { value: 'admin', label: 'Admin' },
  { value: 'owner', label: 'Owner' },
]

export function FiltersBar({ filters, onFiltersChange }: FiltersBarProps) {
  const [searchInput, setSearchInput] = useState(filters.search || '')
  const debouncedSearch = useDebounce(searchInput, 500)

  // Update filters when debounced search changes
  useEffect(() => {
    if (debouncedSearch !== filters.search) {
      onFiltersChange({
        ...filters,
        search: debouncedSearch || undefined,
        page: 1, // Reset to first page on search
      })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedSearch])

  const handleStatusChange = (value: string) => {
    onFiltersChange({
      ...filters,
      is_active: value === 'all' ? undefined : value === 'active',
      page: 1,
    })
  }

  const handleTransactionTypeChange = (value: string) => {
    onFiltersChange({
      ...filters,
      transaction_type: value === 'all' ? undefined : value,
      page: 1,
    })
  }

  const handleRoleChange = (value: string) => {
    onFiltersChange({
      ...filters,
      required_role: value === 'all' ? undefined : value,
      page: 1,
    })
  }

  const handleClearFilters = () => {
    setSearchInput('')
    onFiltersChange({
      page: 1,
      per_page: filters.per_page || 20,
    })
  }

  const hasActiveFilters =
    searchInput ||
    filters.is_active !== undefined ||
    filters.transaction_type ||
    filters.required_role

  return (
    <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
      {/* Search Input */}
      <div className="relative flex-1 max-w-sm">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          placeholder="Search rules by name..."
          value={searchInput}
          onChange={(e) => setSearchInput(e.target.value)}
          className="pl-9"
        />
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-2">
        {/* Status Filter */}
        <Select
          value={
            filters.is_active === undefined
              ? 'all'
              : filters.is_active
              ? 'active'
              : 'inactive'
          }
          onValueChange={handleStatusChange}
        >
          <SelectTrigger className="w-[140px]">
            <SelectValue placeholder="Status" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Status</SelectItem>
            <SelectItem value="active">Active</SelectItem>
            <SelectItem value="inactive">Inactive</SelectItem>
          </SelectContent>
        </Select>

        {/* Transaction Type Filter */}
        <Select
          value={filters.transaction_type || 'all'}
          onValueChange={handleTransactionTypeChange}
        >
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder="Transaction Type" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Types</SelectItem>
            {TRANSACTION_TYPES.map((type) => (
              <SelectItem key={type.value} value={type.value}>
                {type.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        {/* Role Filter */}
        <Select
          value={filters.required_role || 'all'}
          onValueChange={handleRoleChange}
        >
          <SelectTrigger className="w-[160px]">
            <SelectValue placeholder="Required Role" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All Roles</SelectItem>
            {ROLES.map((role) => (
              <SelectItem key={role.value} value={role.value}>
                {role.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        {/* Clear Filters Button */}
        {hasActiveFilters && (
          <Button
            variant="ghost"
            size="sm"
            onClick={handleClearFilters}
            className="h-9"
          >
            <X className="h-4 w-4 mr-1" />
            Clear
          </Button>
        )}
      </div>
    </div>
  )
}
