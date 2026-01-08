'use client'

import React, { useState } from 'react'
import { useParams } from 'next/navigation'
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
import { Progress } from '@/components/ui/progress'
import { Plus, ArrowLeft } from 'lucide-react'
import Link from 'next/link'
import { useBudget, useAddBudgetLine, useUpdateBudgetStatus } from '@/lib/queries/budgets'
import { formatCurrency } from '@/lib/utils/format'
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
} from "@/components/ui/select"
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from 'sonner'
import { Loader2, Lock, Unlock } from 'lucide-react'

export default function BudgetDetailPage() {
  const params = useParams()
  const id = params.id as string
  const { data: budget, isLoading } = useBudget(id)
  const addLine = useAddBudgetLine()
  const updateStatus = useUpdateBudgetStatus()
  const [isAddOpen, setIsAddOpen] = useState(false)

  const handleAddLine = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const formData = new FormData(e.currentTarget)
    const account_name = formData.get('account_name') as string
    const limit = formData.get('limit') as string
    const dimension_value_id = formData.get('dimension_value_id') as string

    addLine.mutate({ budgetId: id, data: { account_name, limit, dimension_value_id } }, {
      onSuccess: () => {
        toast.success(`Budget line for ${account_name} added`)
        setIsAddOpen(false)
      },
      onError: () => toast.error('Failed to add budget line')
    })
  }
  
  const toggleLock = () => {
      const newStatus = budget?.status === 'locked' ? 'open' : 'locked'
      updateStatus.mutate({ budgetId: id, status: newStatus }, {
          onSuccess: () => toast.success(`Budget ${newStatus === 'locked' ? 'locked' : 'unlocked'}`)
      })
  }

  if (isLoading) return <div>Loading...</div>
  if (!budget) return <div>Budget not found</div>

  const totalLimit = parseFloat(budget.budget_limit)
  const totalSpent = parseFloat(budget.actual_spent)
  const percent = totalLimit > 0 ? (totalSpent / totalLimit) * 100 : 0
  const isLocked = budget.status === 'locked'

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
            <Link href="/dashboard/budgets">
            <Button variant="ghost" size="icon">
                <ArrowLeft className="h-4 w-4" />
            </Button>
            </Link>
            <div>
            <h1 className="text-3xl font-bold tracking-tight flex items-center gap-2">
                {budget.department} Budget
                {isLocked && <Lock className="h-5 w-5 text-muted-foreground" />}
            </h1>
            <p className="text-muted-foreground">{budget.period}</p>
            </div>
        </div>
        <div className="flex gap-2">
            <Button variant="outline" onClick={toggleLock} disabled={updateStatus.isPending}>
                {updateStatus.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : 
                 isLocked ? <Unlock className="h-4 w-4 mr-2" /> : <Lock className="h-4 w-4 mr-2" />}
                {isLocked ? 'Unlock Budget' : 'Lock Budget'}
            </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
         <Card>
            <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Total Limit</CardTitle>
            </CardHeader>
            <CardContent>
                <div className="text-2xl font-bold">{formatCurrency(totalLimit)}</div>
            </CardContent>
         </Card>
         <Card>
            <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Total Spent</CardTitle>
            </CardHeader>
            <CardContent>
                <div className={`text-2xl font-bold ${percent > 100 ? 'text-red-500' : ''}`}>
                    {formatCurrency(totalSpent)}
                </div>
                <Progress value={Math.min(percent, 100)} className="h-2 mt-2" />
                <p className="text-xs text-muted-foreground mt-1">{percent.toFixed(1)}% used</p>
            </CardContent>
         </Card>
         <Card>
            <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Variance</CardTitle>
            </CardHeader>
            <CardContent>
                <div className={`text-2xl font-bold ${totalLimit - totalSpent < 0 ? 'text-red-500' : 'text-emerald-500'}`}>
                    {formatCurrency(totalLimit - totalSpent)}
                </div>
            </CardContent>
         </Card>
      </div>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>Budget Lines</CardTitle>
            <CardDescription>Allocation per account</CardDescription>
          </div>
          <Dialog open={isAddOpen} onOpenChange={setIsAddOpen}>
              <DialogTrigger asChild>
                  <Button disabled={isLocked}>
                      <Plus className="mr-2 h-4 w-4" /> Add Line Item
                  </Button>
              </DialogTrigger>
              <DialogContent>
                  <DialogHeader>
                      <DialogTitle>Add Budget Line</DialogTitle>
                      <DialogDescription>Allocated budget for a specific account</DialogDescription>
                  </DialogHeader>
                   <form onSubmit={handleAddLine} className="space-y-4">
                      <div className="space-y-2">
                          <Label htmlFor="account_name">Account Name</Label>
                          <Input id="account_name" name="account_name" placeholder="e.g. Advertising Expense" required />
                      </div>
                      <div className="space-y-2">
                          <Label htmlFor="limit">Limit ($)</Label>
                          <Input id="limit" name="limit" type="number" step="0.01" placeholder="0.00" required />
                      </div>
                      <div className="space-y-2">
                          <Label htmlFor="dimension_value_id">Project (Optional)</Label>
                          <Select name="dimension_value_id">
                            <SelectTrigger>
                                <SelectValue placeholder="Select Project" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="val_p1">Website Redesign (P001)</SelectItem>
                                <SelectItem value="val_p2">Q1 Campaign (P002)</SelectItem>
                            </SelectContent>
                          </Select>
                      </div>
                      <DialogFooter>
                          <Button type="submit" disabled={addLine.isPending}>
                              {addLine.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                              Add Line
                          </Button>
                      </DialogFooter>
                   </form>
              </DialogContent>
          </Dialog>
        </CardHeader>
        <CardContent>
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHead>Account</TableHead>
                        <TableHead>Project</TableHead>
                        <TableHead className="text-right">Budget Limit</TableHead>
                        <TableHead className="text-right">Actual Spent</TableHead>
                        <TableHead className="text-right">Variance</TableHead>
                        <TableHead className="text-right">Utilization</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {budget.lines?.map((line) => {
                        const lineLimit = parseFloat(line.limit)
                        const lineActual = parseFloat(line.actual)
                        const linePercent = lineLimit > 0 ? (lineActual / lineLimit) * 100 : 0
                        const variance = lineLimit - lineActual
                        const isFavorable = variance >= 0
                        
                        return (
                            <TableRow key={line.id}>
                                <TableCell className="font-medium">{line.account_name}</TableCell>
                                <TableCell className="text-muted-foreground">
                                    {line.dimension_value_id === 'val_p1' ? 'Website Redesign' : 
                                     line.dimension_value_id === 'val_p2' ? 'Q1 Campaign' : '-'}
                                </TableCell>
                                <TableCell className="text-right">{formatCurrency(lineLimit)}</TableCell>
                                <TableCell className="text-right">{formatCurrency(lineActual)}</TableCell>
                                <TableCell className={`text-right font-medium ${isFavorable ? 'text-emerald-600' : 'text-red-600'}`}>
                                    {isFavorable ? '+' : ''}{formatCurrency(variance)}
                                </TableCell>
                                <TableCell className="text-right w-[200px]">
                                    <div className="flex items-center justify-end gap-2">
                                        <span className={`text-xs w-[40px] text-right ${linePercent > 100 ? 'text-red-600 font-medium' : 'text-muted-foreground'}`}>{linePercent.toFixed(0)}%</span>
                                        <Progress value={Math.min(linePercent, 100)} className={`h-2 w-[100px] ${linePercent > 100 ? '[&>div]:bg-red-500' : ''}`} />
                                    </div>
                                </TableCell>
                            </TableRow>
                        )
                    })}
                    {(!budget.lines || budget.lines.length === 0) && (
                        <TableRow>
                            <TableCell colSpan={6} className="text-center text-muted-foreground h-24">
                                No budget lines added yet.
                            </TableCell>
                        </TableRow>
                    )}
                </TableBody>
            </Table>
        </CardContent>
      </Card>
    </div>
  )
}
