'use client'

import React from 'react'
import { useAuthStore } from '@/lib/stores/authStore'
import { useLogout } from '@/lib/queries/auth'
import { Button } from '@/components/ui/button'
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuLabel, 
  DropdownMenuSeparator, 
  DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { ChevronDown, Loader2, Menu, Building2, Check } from 'lucide-react'

export function Header() {
  const user = useAuthStore((state) => state.user)
  const currentOrgId = useAuthStore((state) => state.currentOrgId)
  const setOrg = useAuthStore((state) => state.setOrg)
  const logout = useLogout()
  
  const initials = user?.full_name
    ? user.full_name.split(' ').map((n) => n[0]).join('').substring(0, 2).toUpperCase()
    : 'U'

  const organizations = user?.organizations || []
  const currentOrg = organizations.find(org => org.id === currentOrgId) || organizations[0]

  const handleOrgSwitch = (orgId: string) => {
    setOrg(orgId)
    // In a real app, you might want to refresh data or redirect
    window.location.reload()
  }

  return (
    <header className="fixed top-0 z-30 flex h-16 w-full items-center justify-between border-b bg-background px-6 md:pl-72 transition-all">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" className="md:hidden">
          <Menu className="h-5 w-5" />
        </Button>
        
        {/* Organization Selector */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" className="hidden md:flex gap-2 h-9">
              <Building2 className="h-4 w-4 text-muted-foreground" />
              <span className="font-medium">{currentOrg?.name || 'Select Organization'}</span>
              <ChevronDown className="h-4 w-4 text-muted-foreground" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" className="w-64">
            <DropdownMenuLabel>Switch Organization</DropdownMenuLabel>
            <DropdownMenuSeparator />
            {organizations.map((org) => (
              <DropdownMenuItem 
                key={org.id}
                onClick={() => handleOrgSwitch(org.id)}
                className="cursor-pointer flex items-center justify-between"
              >
                <div className="flex items-center gap-2">
                  <div className="flex h-8 w-8 items-center justify-center rounded-md bg-primary/10 text-primary text-xs font-bold">
                    {org.name.substring(0, 2).toUpperCase()}
                  </div>
                  <div className="flex flex-col">
                    <span className="font-medium">{org.name}</span>
                    <span className="text-xs text-muted-foreground capitalize">{org.role}</span>
                  </div>
                </div>
                {org.id === currentOrgId && (
                  <Check className="h-4 w-4 text-primary" />
                )}
              </DropdownMenuItem>
            ))}
            <DropdownMenuSeparator />
            <DropdownMenuItem className="text-muted-foreground">
              + Create New Organization
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <div className="flex items-center gap-4">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" className="relative h-8 w-auto gap-2 rounded-full pl-2 pr-4">
              <Avatar className="h-8 w-8 border">
                <AvatarImage src="" alt={user?.full_name} />
                <AvatarFallback>{initials}</AvatarFallback>
              </Avatar>
              <div className="hidden flex-col items-start gap-0.5 text-xs sm:flex">
                <span className="font-semibold">{user?.full_name}</span>
                <span className="text-muted-foreground text-[10px]">{user?.email}</span>
              </div>
              <ChevronDown className="h-4 w-4 text-muted-foreground" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-56">
            <DropdownMenuLabel>My Account</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem>Profile</DropdownMenuItem>
            <DropdownMenuItem>Settings</DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem 
              className="text-red-600 focus:text-red-600 cursor-pointer"
              onClick={() => logout.mutate()}
              disabled={logout.isPending}
            >
              {logout.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
              Log out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  )
}
