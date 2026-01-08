'use client'

import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { 
  CalendarRange, 
  Layers, 
  RefreshCw, 
  BookOpen,
  ArrowRight
} from 'lucide-react'
import Link from 'next/link'

const masterDataItems = [
  {
    title: 'Chart of Accounts',
    description: 'Manage your general ledger accounts structure.',
    icon: BookOpen,
    href: '/dashboard/accounts',
    color: 'text-blue-500',
  },
  {
    title: 'Fiscal Periods',
    description: 'Manage open/close periods and fiscal years.',
    icon: CalendarRange,
    href: '/dashboard/master-data/fiscal-periods',
    color: 'text-orange-500',
  },
  {
    title: 'Dimensions',
    description: 'Configure analytics dimensions (Department, Project, etc).',
    icon: Layers,
    href: '/dashboard/master-data/dimensions',
    color: 'text-purple-500',
  },
  {
    title: 'Exchange Rates',
    description: 'Set daily or monthly currency exchange rates.',
    icon: RefreshCw,
    href: '/dashboard/master-data/exchange-rates',
    color: 'text-green-500',
  },
]

export default function MasterDataPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Master Data</h1>
        <p className="text-muted-foreground mt-2">
          Configuration and base data settings.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {masterDataItems.map((item) => (
          <Link href={item.href} key={item.title}>
            <Card className="hover:bg-muted/50 transition-colors cursor-pointer h-full group">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  {item.title}
                </CardTitle>
                <item.icon className={`h-4 w-4 ${item.color}`} />
              </CardHeader>
              <CardContent>
                <CardDescription className="mb-4">
                  {item.description}
                </CardDescription>
                <div className="flex items-center text-sm text-primary group-hover:underline">
                  Open setting <ArrowRight className="ml-1 h-3 w-3" />
                </div>
              </CardContent>
            </Card>
          </Link>
        ))}
      </div>
    </div>
  )
}
