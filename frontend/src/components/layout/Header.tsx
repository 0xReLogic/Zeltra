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
import { ChevronDown, Loader2, Menu } from 'lucide-react'

export function Header() {
  const user = useAuthStore((state) => state.user)
  const logout = useLogout()
  
  const initials = user?.full_name
    ? user.full_name.split(' ').map((n) => n[0]).join('').substring(0, 2).toUpperCase()
    : 'U'

  return (
    <header className="fixed top-0 z-30 flex h-16 w-full items-center justify-between border-b bg-background px-6 md:pl-72 transition-all">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" className="md:hidden">
          <Menu className="h-5 w-5" />
        </Button>
        <div className="text-sm font-medium text-muted-foreground hidden md:block">
          {user?.organizations[0]?.name || 'Organization'}
        </div>
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
