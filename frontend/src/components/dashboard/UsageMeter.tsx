import React from 'react'
import { Progress } from '@/components/ui/progress'
import { useOrganization, useOrganizationUsers } from '@/lib/queries/organizations'
import { cn } from '@/lib/utils'

interface UsageMeterProps {
  className?: string
}

export function UsageMeter({ className }: UsageMeterProps) {
  const { data: org, isLoading: isOrgLoading } = useOrganization()
  const { data: usersData, isLoading: isUsersLoading } = useOrganizationUsers()

  if (isOrgLoading || isUsersLoading) {
    return null // Or skeleton
  }

  if (!org || !org.limits) {
    return null
  }

  const maxUsers = org.limits.max_users
  const currentUsers = usersData?.data?.length || 0

  // If unlimited, don't show usage meter or show "Unlimited"
  if (maxUsers === null) {
    return null
    // Alternatively: 
    // return <div className="text-xs text-muted-foreground">Users: Unlimited</div>
  }

  const percentage = Math.min((currentUsers / maxUsers) * 100, 100)
  const isNearLimit = percentage >= 80
  const isAtLimit = percentage >= 100

  return (
    <div className={cn("space-y-2", className)}>
      <div className="flex justify-between items-center text-xs">
        <span className="font-medium text-muted-foreground">Seats Used</span>
        <span className={cn(
          "font-medium",
          isAtLimit ? "text-destructive" : isNearLimit ? "text-amber-500" : "text-muted-foreground"
        )}>
          {currentUsers} / {maxUsers}
        </span>
      </div>
      <Progress 
        value={percentage} 
        className="h-2"
        // TODO: Pass custom indicator color to Progress if supported, 
        // currently Progress uses bg-primary which is fine.
        // If we want red, we need to modify Progress or use inline style.
      />
    </div>
  )
}
