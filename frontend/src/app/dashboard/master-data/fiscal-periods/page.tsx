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
import { ChevronDown, ChevronRight, MoreHorizontal, Lock, Unlock, Archive } from 'lucide-react'
import { useFiscalYears, useUpdatePeriodStatus } from '@/lib/queries/fiscal'

export default function FiscalPeriodsPage() {
  const { data, isLoading } = useFiscalYears()
  const updateStatus = useUpdatePeriodStatus()
  const [expandedYear, setExpandedYear] = React.useState<string | null>('2026')

  const toggleExpand = (yearId: string) => {
    setExpandedYear(expandedYear === yearId ? null : yearId)
  }

  const handleStatusChange = (id: string, status: 'open' | 'closed' | 'locked') => {
    updateStatus.mutate({ id, status })
  }

  if (isLoading) return <div>Loading...</div>

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
           <h1 className="text-3xl font-bold tracking-tight">Fiscal Periods</h1>
           <p className="text-muted-foreground mt-2">Manage open/close periods for accounting.</p>
        </div>
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
              {data?.map((year) => (
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
