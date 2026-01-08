'use client'

import React, { useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Loader2, MoreHorizontal, Shield, Trash, UserPlus } from 'lucide-react'
import { toast } from 'sonner' // Correct import for Sonner

import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
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
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
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
import { Badge } from '@/components/ui/badge'
import { useOrganizationUsers, useInviteUser, useUpdateUserRole, useRemoveUser } from '@/lib/queries/organizations'
import { OrganizationUser } from '@/types/organizations'

const inviteUserSchema = z.object({
  email: z.string().email('Invalid email address'),
  role: z.enum(['owner', 'admin', 'accountant', 'approver', 'viewer']),
})

type InviteUserValues = z.infer<typeof inviteUserSchema>

export default function TeamManagementPage() {
  const [isInviteOpen, setIsInviteOpen] = useState(false)
  
  const { data: usersData, isLoading } = useOrganizationUsers()
  const inviteUser = useInviteUser()
  const updateUserRole = useUpdateUserRole()
  const removeUser = useRemoveUser()

  const form = useForm<InviteUserValues>({
    resolver: zodResolver(inviteUserSchema),
    defaultValues: {
      email: '',
      role: 'viewer',
    },
  })

  const users = usersData?.data || []

  function onInvite(data: InviteUserValues) {
    inviteUser.mutate(data, {
      onSuccess: () => {
        setIsInviteOpen(false)
        form.reset()
        toast.success(`Invitation sent to ${data.email}`)
      },
      onError: (error) => {
        toast.error(error.message || 'Failed to send invitation')
      }
    })
  }

  function onRoleChange(userId: string, newRole: string) {
    updateUserRole.mutate({ 
      userId, 
      data: { role: newRole as OrganizationUser['role'] } 
    }, {
      onSuccess: () => toast.success('User role updated'),
      onError: () => toast.error('Failed to update role')
    })
  }

  function onRemoveUser(userId: string) {
    if (confirm('Are you sure you want to remove this user?')) {
      removeUser.mutate(userId, {
        onSuccess: () => toast.success('User removed from organization'),
        onError: () => toast.error('Failed to remove user')
      })
    }
  }

  const getRoleBadgeColor = (role: string) => {
    switch(role) {
      case 'owner': return 'default'
      case 'admin': return 'secondary'
      case 'accountant': return 'outline'
      default: return 'outline'
    }
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
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">Team Management</h3>
          <p className="text-sm text-muted-foreground">
            Invite users and manage their access roles.
          </p>
        </div>
        <Dialog open={isInviteOpen} onOpenChange={setIsInviteOpen}>
          <DialogTrigger asChild>
            <Button>
              <UserPlus className="mr-2 h-4 w-4" />
              Invite User
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Invite Team Member</DialogTitle>
              <DialogDescription>
                Send an invitation email to add a new user to your organization.
              </DialogDescription>
            </DialogHeader>
            <Form {...form}>
              <form onSubmit={form.handleSubmit(onInvite)} className="space-y-4">
                <FormField
                  control={form.control}
                  name="email"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Email Address</FormLabel>
                      <FormControl>
                        <Input placeholder="colleague@company.com" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="role"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Role</FormLabel>
                      <Select onValueChange={field.onChange} defaultValue={field.value}>
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue placeholder="Select a role" />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                          <SelectItem value="admin">Admin</SelectItem>
                          <SelectItem value="accountant">Accountant</SelectItem>
                          <SelectItem value="approver">Approver</SelectItem>
                          <SelectItem value="viewer">Viewer</SelectItem>
                        </SelectContent>
                      </Select>
                      <FormDescription>
                        Admins have full access. Accountants can manage books. Viewers are read-only.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <DialogFooter className="mt-4">
                  <Button type="submit" disabled={inviteUser.isPending}>
                    {inviteUser.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                    Send Invitation
                  </Button>
                </DialogFooter>
              </form>
            </Form>
          </DialogContent>
        </Dialog>
      </div>

      <div className="border rounded-md">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>User</TableHead>
              <TableHead>Role</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Joined At</TableHead>
              <TableHead className="w-[80px]"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {users.map((user) => (
              <TableRow key={user.id}>
                <TableCell>
                  <div className="flex flex-col">
                    <span className="font-medium">{user.full_name}</span>
                    <span className="text-xs text-muted-foreground">{user.email}</span>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge variant={getRoleBadgeColor(user.role) as 'default' | 'secondary' | 'outline' | 'destructive'}>
                    {user.role}
                  </Badge>
                </TableCell>
                <TableCell>
                  <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                    user.status === 'active' ? 'bg-green-100 text-green-700' : 'bg-yellow-100 text-yellow-700'
                  }`}>
                    {user.status}
                  </span>
                </TableCell>
                <TableCell className="text-sm text-muted-foreground">
                    {user.joined_at ? new Date(user.joined_at).toLocaleDateString() : '-'}
                </TableCell>
                <TableCell>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" className="h-8 w-8 p-0">
                        <span className="sr-only">Open menu</span>
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuLabel>Actions</DropdownMenuLabel>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem onClick={() => onRoleChange(user.id, 'admin')}>
                        <Shield className="mr-2 h-4 w-4" /> Make Admin
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => onRoleChange(user.id, 'accountant')}>
                        <Shield className="mr-2 h-4 w-4" /> Make Accountant
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => onRoleChange(user.id, 'viewer')}>
                         <Shield className="mr-2 h-4 w-4" /> Make Viewer
                      </DropdownMenuItem>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem className="text-red-600" onClick={() => onRemoveUser(user.id)}>
                        <Trash className="mr-2 h-4 w-4" /> Remove User
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </TableCell>
              </TableRow>
            ))}
            {users.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} className="h-24 text-center">
                  No users found.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}
