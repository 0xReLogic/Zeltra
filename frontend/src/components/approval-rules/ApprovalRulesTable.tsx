'use client'

import { useState } from 'react'
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
  SortingState,
  getSortedRowModel,
} from '@tanstack/react-table'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { ArrowUpDown, ChevronLeft, ChevronRight } from 'lucide-react'
import { EditApprovalRuleDialog } from './EditApprovalRuleDialog'
import { DeleteApprovalRuleDialog } from './DeleteApprovalRuleDialog'
import { useUpdateApprovalRule } from '@/lib/queries/approval-rules'
import { toast } from 'sonner'
import type { components } from '@/types/api.generated'

type ApprovalRuleResponse = components['schemas']['ApprovalRuleResponse']
type PaginatedApprovalRulesResponse = components['schemas']['PaginatedApprovalRulesResponse']

interface ApprovalRulesTableProps {
  data: PaginatedApprovalRulesResponse
  filters: {
    page: number
    per_page: number
    sort_by?: string
    sort_order?: 'asc' | 'desc'
  }
  onFiltersChange: (filters: any) => void
}

export function ApprovalRulesTable({ data, filters, onFiltersChange }: ApprovalRulesTableProps) {
  const [sorting, setSorting] = useState<SortingState>([
    { id: filters.sort_by || 'priority', desc: filters.sort_order === 'desc' }
  ])
  
  const updateMutation = useUpdateApprovalRule()

  const handleToggleActive = async (rule: ApprovalRuleResponse) => {
    try {
      await updateMutation.mutateAsync({
        id: rule.id,
        data: {
          is_active: !rule.is_active,
        },
      })
      toast.success(`Rule ${!rule.is_active ? 'activated' : 'deactivated'} successfully`)
    } catch (error) {
      toast.error('Failed to update rule status')
      console.error('Toggle active error:', error)
    }
  }

  const getPriorityBadgeVariant = (priority: number) => {
    if (priority <= 10) return 'destructive' // High priority (red)
    if (priority <= 50) return 'default' // Medium priority (gray)
    return 'secondary' // Low priority (light gray)
  }

  const getRoleBadgeVariant = (role: string) => {
    switch (role) {
      case 'owner':
      case 'admin':
        return 'destructive'
      case 'accountant':
      case 'approver':
        return 'default'
      default:
        return 'secondary'
    }
  }

  const formatAmountRange = (minAmount: string | null | undefined, maxAmount: string | null | undefined) => {
    if (!minAmount && !maxAmount) return 'Any amount'
    if (minAmount && !maxAmount) return `≥ $${minAmount}`
    if (!minAmount && maxAmount) return `≤ $${maxAmount}`
    return `$${minAmount} - $${maxAmount}`
  }

  const formatTransactionTypes = (types: string[]) => {
    if (types.length <= 2) {
      return types.map(type => (
        <Badge key={type} variant="outline" className="mr-1">
          {type.replace('_', ' ')}
        </Badge>
      ))
    }
    
    return (
      <div className="flex items-center gap-1">
        <Badge variant="outline">{types[0].replace('_', ' ')}</Badge>
        <Badge variant="secondary">+{types.length - 1} more</Badge>
      </div>
    )
  }

  const columns: ColumnDef<ApprovalRuleResponse>[] = [
    {
      accessorKey: 'priority',
      header: ({ column }) => (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
          className="h-auto p-0 font-semibold"
        >
          Priority
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      ),
      cell: ({ row }) => (
        <Badge variant={getPriorityBadgeVariant(row.original.priority)}>
          {row.original.priority}
        </Badge>
      ),
    },
    {
      accessorKey: 'name',
      header: ({ column }) => (
        <Button
          variant="ghost"
          onClick={() => column.toggleSorting(column.getIsSorted() === 'asc')}
          className="h-auto p-0 font-semibold"
        >
          Name
          <ArrowUpDown className="ml-2 h-4 w-4" />
        </Button>
      ),
      cell: ({ row }) => (
        <div>
          <div className="font-medium">{row.original.name}</div>
          {row.original.description && (
            <div className="text-sm text-muted-foreground truncate max-w-xs">
              {row.original.description}
            </div>
          )}
        </div>
      ),
    },
    {
      accessorKey: 'transaction_types',
      header: 'Transaction Types',
      cell: ({ row }) => (
        <div className="flex flex-wrap gap-1">
          {formatTransactionTypes(row.original.transaction_types)}
        </div>
      ),
    },
    {
      accessorKey: 'required_role',
      header: 'Required Role',
      cell: ({ row }) => (
        <Badge variant={getRoleBadgeVariant(row.original.required_role)}>
          {row.original.required_role}
        </Badge>
      ),
    },
    {
      id: 'amount_range',
      header: 'Amount Range',
      cell: ({ row }) => (
        <span className="text-sm">
          {formatAmountRange(row.original.min_amount, row.original.max_amount)}
        </span>
      ),
    },
    {
      accessorKey: 'is_active',
      header: 'Status',
      cell: ({ row }) => (
        <Button
          variant="ghost"
          size="sm"
          onClick={() => handleToggleActive(row.original)}
          disabled={updateMutation.isPending}
          className={`px-3 py-1 text-xs font-medium rounded-full ${
            row.original.is_active
              ? 'bg-green-100 text-green-800 hover:bg-green-200'
              : 'bg-gray-100 text-gray-800 hover:bg-gray-200'
          }`}
        >
          {row.original.is_active ? 'Active' : 'Inactive'}
        </Button>
      ),
    },
    {
      id: 'actions',
      header: 'Actions',
      cell: ({ row }) => (
        <div className="flex items-center gap-2">
          <EditApprovalRuleDialog rule={row.original} />
          <DeleteApprovalRuleDialog rule={row.original} />
        </div>
      ),
    },
  ]

  const table = useReactTable({
    data: data.data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: (updater) => {
      const newSorting = typeof updater === 'function' ? updater(sorting) : updater
      setSorting(newSorting)
      
      if (newSorting.length > 0) {
        const sort = newSorting[0]
        onFiltersChange({
          ...filters,
          sort_by: sort.id,
          sort_order: sort.desc ? 'desc' : 'asc',
          page: 1,
        })
      }
    },
    state: {
      sorting,
    },
    manualSorting: true,
  })

  const handlePageChange = (newPage: number) => {
    onFiltersChange({
      ...filters,
      page: newPage,
    })
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardContent className="p-0">
          <Table>
            <TableHeader>
              {table.getHeaderGroups().map((headerGroup) => (
                <TableRow key={headerGroup.id}>
                  {headerGroup.headers.map((header) => (
                    <TableHead key={header.id}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                    </TableHead>
                  ))}
                </TableRow>
              ))}
            </TableHeader>
            <TableBody>
              {table.getRowModel().rows?.length ? (
                table.getRowModel().rows.map((row) => (
                  <TableRow
                    key={row.id}
                    data-state={row.getIsSelected() && 'selected'}
                  >
                    {row.getVisibleCells().map((cell) => (
                      <TableCell key={cell.id}>
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext()
                        )}
                      </TableCell>
                    ))}
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={columns.length}
                    className="h-24 text-center"
                  >
                    No results.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {/* Pagination Controls */}
      {data.meta.total_pages > 1 && (
        <div className="flex items-center justify-between">
          <div className="text-sm text-muted-foreground">
            Showing {((data.meta.page - 1) * data.meta.per_page) + 1} to{' '}
            {Math.min(data.meta.page * data.meta.per_page, data.meta.total)} of{' '}
            {data.meta.total} results
          </div>
          <div className="flex items-center space-x-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => handlePageChange(data.meta.page - 1)}
              disabled={data.meta.page <= 1}
            >
              <ChevronLeft className="h-4 w-4 mr-1" />
              Previous
            </Button>
            <div className="text-sm">
              Page {data.meta.page} of {data.meta.total_pages}
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => handlePageChange(data.meta.page + 1)}
              disabled={data.meta.page >= data.meta.total_pages}
            >
              Next
              <ChevronRight className="h-4 w-4 ml-1" />
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}