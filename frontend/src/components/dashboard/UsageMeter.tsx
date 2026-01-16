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

  const percentage = maxUsers ? Math.min((currentUsers / maxUsers) * 100, 100) : 0
  const isNearLimit = maxUsers ? percentage >= 80 : false
  const isAtLimit = maxUsers ? percentage >= 100 : false

  return (
    <div className={cn("space-y-2", className)}>
      <div className="flex justify-between items-center text-xs">
        <span className="font-medium text-muted-foreground">Seats Used</span>
        <span className={cn(
          "font-medium",
          isAtLimit ? "text-destructive" : isNearLimit ? "text-amber-500" : "text-muted-foreground"
        )}>
          {currentUsers} / {maxUsers || '∞'}
        </span>
      </div>
      {maxUsers && (
        <Progress 
          value={percentage} 
          className="h-2"
          // TODO: Pass custom indicator color to Progress if supported, 
          // currently Progress uses bg-primary which is fine.
          // If we want red, we need to modify Progress or use inline style.
        />
      )}
    </div>
  )
}
