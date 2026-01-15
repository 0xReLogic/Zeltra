'use client'

import React from 'react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { cn } from '@/lib/utils'
import { 
  LayoutDashboard, 
  Wallet, 
  ArrowRightLeft, 
  Settings, 
  LifeBuoy,
  Briefcase,
  CheckSquare,
  PieChart,
  FileText,
  TrendingUp,
  SearchCode,
  Lock
} from 'lucide-react'
import { useOrganization } from '@/lib/queries/organizations'

const navItems = [
  { label: 'Overview', href: '/dashboard', icon: LayoutDashboard },
  { label: 'Accounts', href: '/dashboard/accounts', icon: Wallet },
  { label: 'Transactions', href: '/dashboard/transactions', icon: ArrowRightLeft },
  { label: 'Approvals', href: '/dashboard/approvals', icon: CheckSquare },
  { label: 'Budgeting', href: '/dashboard/budgets', icon: PieChart },
  { label: 'Simulation', href: '/dashboard/simulation', icon: TrendingUp, tier: 'enterprise' },
  { label: 'Forensic', href: '/dashboard/forensic', icon: SearchCode, tier: 'enterprise' },
  { label: 'Reports', href: '/dashboard/reports/trial-balance', icon: FileText },
  { label: 'Master Data', href: '/dashboard/master-data', icon: Briefcase },
  { label: 'Settings', href: '/dashboard/settings', icon: Settings },
]

export function Sidebar() {
  const pathname = usePathname()
  const { data: org } = useOrganization()

  return (
    <aside className="fixed left-0 top-0 z-40 h-screen w-64 border-r bg-background hidden md:block">
      <div className="flex h-16 items-center border-b px-6">
        <Link href="/dashboard" className="flex items-center gap-2 font-bold text-xl">
          <span>Zeltra</span>
        </Link>
      </div>
      
      <div className="flex flex-col justify-between h-[calc(100vh-64px)] p-4">
        <nav className="space-y-1">
          {navItems.map((item) => {
            const isActive = pathname === item.href
            const isLocked = item.tier === 'enterprise' && org?.subscription_tier !== 'enterprise'

            return (
              <Link
                key={item.href}
                href={item.href}
                className={cn(
                  "flex items-center justify-between rounded-md px-3 py-2 text-sm font-medium transition-colors",
                  isActive 
                    ? "bg-primary text-primary-foreground" 
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"
                )}
              >
                <div className="flex items-center gap-3">
                  <item.icon className="h-4 w-4" />
                  {item.label}
                </div>
                {isLocked && (
                  <Lock className="h-3 w-3 text-amber-500" />
                )}
              </Link>
            )
          })}
        </nav>

        <div className="border-t pt-4">
           <Link
              href="/dashboard/help"
              className="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
            >
              <LifeBuoy className="h-4 w-4" />
              Help & Support
            </Link>
        </div>
      </div>
    </aside>
  )
}
