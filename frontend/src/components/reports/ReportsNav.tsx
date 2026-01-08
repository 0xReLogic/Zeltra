"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"

export function ReportsNav() {
  const pathname = usePathname()

  const tabs = [
    { name: "Trial Balance", href: "/dashboard/reports/trial-balance" },
    { name: "Income Statement", href: "/dashboard/reports/income-statement" },
    { name: "Balance Sheet", href: "/dashboard/reports/balance-sheet" },
  ]

  return (
    <div className="flex space-x-2 mb-6">
      {tabs.map((tab) => (
        <Button
          key={tab.href}
          variant={pathname === tab.href ? "default" : "outline"}
          asChild
        >
          <Link href={tab.href}>{tab.name}</Link>
        </Button>
      ))}
    </div>
  )
}
