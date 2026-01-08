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
import { 
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu'
import { ChevronDown, ChevronRight, MoreHorizontal, Lock, Unlock, Archive, Plus, Loader2 } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useFiscalYears, useUpdatePeriodStatus, useCreateFiscalYear } from '@/lib/queries/fiscal'
import { toast } from 'sonner'
import { useState } from 'react'

export default function FiscalPeriodsPage() {
  const { data, isLoading } = useFiscalYears()
  const updateStatus = useUpdatePeriodStatus()
  const createYear = useCreateFiscalYear()
  const [expandedYear, setExpandedYear] = React.useState<string | null>('2026')
  const [isCreateOpen, setIsCreateOpen] = useState(false)

  const toggleExpand = (yearId: string) => {
    setExpandedYear(expandedYear === yearId ? null : yearId)
  }

  const handleStatusChange = (id: string, status: 'open' | 'closed' | 'locked') => {
    updateStatus.mutate({ id, status })
  }

  const handleCreate = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const formData = new FormData(e.currentTarget)
    const name = formData.get('name') as string
    const start_date = formData.get('start_date') as string
    const include_adjustment = formData.get('include_adjustment') === 'on'
    
    // Auto-calculate end date (Dec 31 of same year)
    const year = new Date(start_date).getFullYear()
    const end_date = `${year}-12-31`

    createYear.mutate({ name, start_date, end_date, include_adjustment }, {
      onSuccess: () => {
        toast.success(`Fiscal Year ${name} created`)
        setIsCreateOpen(false)
      },
      onError: () => toast.error('Failed to create fiscal year')
    })
  }

  if (isLoading) return <div>Loading...</div>

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
           <h1 className="text-3xl font-bold tracking-tight">Fiscal Periods</h1>
           <p className="text-muted-foreground mt-2">Manage open/close periods for accounting.</p>
        </div>
        <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
            <DialogTrigger asChild>
                <Button>
                    <Plus className="mr-2 h-4 w-4" /> New Fiscal Year
                </Button>
            </DialogTrigger>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>Create Fiscal Year</DialogTitle>
                    <DialogDescription>
                        Create a new fiscal year. Monthly periods will be generated automatically.
                    </DialogDescription>
                </DialogHeader>
                <form onSubmit={handleCreate} className="space-y-4">
                    <div className="space-y-2">
                        <Label htmlFor="name">Name</Label>
                        <Input id="name" name="name" placeholder="e.g. FY 2027" required />
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="start_date">Start Date</Label>
                        <Input 
                            id="start_date" 
                            name="start_date" 
                            type="date" 
                            required 
                            defaultValue="2027-01-01"
                        />
                        <p className="text-[0.8rem] text-muted-foreground">End date will be automatically set to Dec 31st.</p>
                    </div>
                    <div className="flex items-center space-x-2">
                        <input 
                            type="checkbox" 
                            id="include_adjustment" 
                            name="include_adjustment"
                            className="h-4 w-4 rounded border-gray-300"
                        />
                        <Label htmlFor="include_adjustment" className="text-sm font-normal">
                            Include Adjustment Period (Period 13)
                        </Label>
                    </div>
                    <DialogFooter>
                        <Button type="submit" disabled={createYear.isPending}>
                            {createYear.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                            Create Year
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Fiscal Years</CardTitle>
          <CardDescription>Click on a year to view monthly periods.</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-[50px]"></TableHead>
                <TableHead>Year</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Start Date</TableHead>
                <TableHead>End Date</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data?.data.map((year) => (
                <React.Fragment key={year.id}>
                  <TableRow 
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => toggleExpand(year.id)}
                  >
                    <TableCell>
                      {expandedYear === year.id ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
                    </TableCell>
                    <TableCell className="font-medium">{year.name}</TableCell>
                    <TableCell>
                      <Badge variant={year.status === 'open' ? 'default' : 'secondary'}>
                        {year.status}
                      </Badge>
                    </TableCell>
                    <TableCell>{year.start_date}</TableCell>
                    <TableCell>{year.end_date}</TableCell>
                  </TableRow>
                  {expandedYear === year.id && (
                    <TableRow>
                      <TableCell colSpan={5} className="p-0 bg-muted/30">
                        <div className="p-4 pl-12">
                          <Table>
                            <TableHeader>
                              <TableRow>
                                <TableHead>Period</TableHead>
                                <TableHead>Status</TableHead>
                                <TableHead className="text-right">Action</TableHead>
                              </TableRow>
                            </TableHeader>
                            <TableBody>
                              {year.periods.map((period) => (
                                <TableRow key={period.id}>
                                  <TableCell>{period.name}</TableCell>
                                  <TableCell>
                                     <Badge 
                                      variant="outline"
                                      className={
                                        period.status === 'open' ? 'border-green-500 text-green-500' :
                                        period.status === 'closed' ? 'border-yellow-500 text-yellow-500' :
                                        'border-gray-500 text-gray-500'
                                      }
                                     >
                                      {period.status}
                                     </Badge>
                                  </TableCell>
                                  <TableCell className="text-right">
                                    <DropdownMenu>
                                      <DropdownMenuTrigger asChild>
                                        <Button variant="ghost" className="h-8 w-8 p-0">
                                          <MoreHorizontal className="h-4 w-4" />
                                        </Button>
                                      </DropdownMenuTrigger>
                                      <DropdownMenuContent align="end">
                                        <DropdownMenuItem onClick={() => handleStatusChange(period.id, 'open')}>
                                          <Unlock className="mr-2 h-4 w-4" /> Open
                                        </DropdownMenuItem>
                                        <DropdownMenuItem onClick={() => handleStatusChange(period.id, 'closed')}>
                                          <Archive className="mr-2 h-4 w-4" /> Close
                                        </DropdownMenuItem>
                                        <DropdownMenuItem onClick={() => handleStatusChange(period.id, 'locked')}>
                                          <Lock className="mr-2 h-4 w-4" /> Lock
                                        </DropdownMenuItem>
                                      </DropdownMenuContent>
                                    </DropdownMenu>
                                  </TableCell>
                                </TableRow>
                              ))}
                            </TableBody>
                          </Table>
                        </div>
                      </TableCell>
                    </TableRow>
                  )}
                </React.Fragment>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
