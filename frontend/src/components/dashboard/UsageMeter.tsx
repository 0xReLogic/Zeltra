import React from 'react'
import { Progress } from '@/components/ui/progress'
import { useOrganizationUsers } from '@/lib/queries/organizations'
import { useUserSubscription } from '@/lib/queries/auth'
import { useEntities } from '@/lib/queries/entities'
import { cn } from '@/lib/utils'

interface UsageMeterProps {
  className?: string
}

export function UsageMeter({ className }: UsageMeterProps) {
  const { data: subscription, isLoading: isSubLoading } = useUserSubscription()
  const { data: usersData, isLoading: isUsersLoading } = useOrganizationUsers()
  const { data: entities, isLoading: isEntitiesLoading } = useEntities()

  if (isSubLoading || isUsersLoading || isEntitiesLoading) {
    return null // Or skeleton
  }

  if (!subscription) {
    return null
  }

  // Get tier limits from subscription
  const tierLimits = getTierLimits(subscription.subscription_tier)
  const maxUsers = tierLimits.max_users
  const maxEntities = tierLimits.max_entities
  const currentUsers = usersData?.data?.length || 0
  const currentEntities = entities?.length || 0

  const userPercentage = maxUsers ? Math.min((currentUsers / maxUsers) * 100, 100) : 0
  const entityPercentage = maxEntities ? Math.min((currentEntities / maxEntities) * 100, 100) : 0
  
  const isUserNearLimit = maxUsers ? userPercentage >= 80 : false
  const isUserAtLimit = maxUsers ? userPercentage >= 100 : false
  const isEntityNearLimit = maxEntities ? entityPercentage >= 80 : false
  const isEntityAtLimit = maxEntities ? entityPercentage >= 100 : false

  return (
    <div className={cn("space-y-4", className)}>
      {/* Entity Usage */}
      <div className="space-y-2">
        <div className="flex justify-between items-center text-xs">
          <span className="font-medium text-muted-foreground">Entities</span>
          <span className={cn(
            "font-medium",
            isEntityAtLimit ? "text-destructive" : isEntityNearLimit ? "text-amber-500" : "text-muted-foreground"
          )}>
            {currentEntities} / {maxEntities || '∞'}
          </span>
        </div>
        {maxEntities && (
          <Progress 
            value={entityPercentage} 
            className="h-2"
          />
        )}
      </div>

      {/* User Seats Usage */}
      <div className="space-y-2">
        <div className="flex justify-between items-center text-xs">
          <span className="font-medium text-muted-foreground">Seats Used</span>
          <span className={cn(
            "font-medium",
            isUserAtLimit ? "text-destructive" : isUserNearLimit ? "text-amber-500" : "text-muted-foreground"
          )}>
            {currentUsers} / {maxUsers || '∞'}
          </span>
        </div>
        {maxUsers && (
          <Progress 
            value={userPercentage} 
            className="h-2"
          />
        )}
      </div>
    </div>
  )
}

// Helper function to get tier limits
function getTierLimits(tier: string) {
  switch (tier.toLowerCase()) {
    case 'starter':
      return { max_users: 50, max_entities: 1 }
    case 'growth':
      return { max_users: 200, max_entities: 5 }
    case 'enterprise':
      return { max_users: null, max_entities: null }
    default:
      return { max_users: 50, max_entities: 1 }
  }
}
