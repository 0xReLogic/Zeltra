/**
 * User Subscription Settings Page
 * 
 * Displays user subscription information including:
 * - Subscription tier (Starter, Growth, Enterprise)
 * - Subscription status (active, trialing, canceled)
 * - Trial end date if trialing
 * - Entity limits and current count
 * 
 * Requirements: 11.1, 11.2, 11.3, 11.4, 11.5
 */

'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Loader2, Building2, Calendar, CreditCard } from 'lucide-react'
import { useAuthStore } from '@/lib/stores/authStore'
import { useEntities } from '@/lib/queries/entities'
import { useUserSubscription } from '@/lib/queries/auth'

export default function SubscriptionPage() {
  const user = useAuthStore((state) => state.user)
  const { data: subscription, isLoading: subLoading } = useUserSubscription()
  const { data: entities, isLoading: entitiesLoading } = useEntities()

  if (!user || subLoading) {
    return (
      <div className="flex h-96 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  // Format date helper
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    })
  }

  // Get entity limit based on tier
  const getEntityLimit = (tier: string) => {
    switch (tier.toLowerCase()) {
      case 'starter':
        return 1
      case 'growth':
        return 5
      case 'enterprise':
        return 'Unlimited'
      default:
        return 1
    }
  }

  const subscriptionTier = subscription?.subscription_tier || 'starter'
  const subscriptionStatus = subscription?.subscription_status || 'active'
  const trialEndDate = subscription?.trial_ends_at

  const entityLimit = getEntityLimit(subscriptionTier)
  const entityCount = entities?.length || 0
  const isUnlimited = entityLimit === 'Unlimited'
  const isNearLimit = !isUnlimited && entityCount >= (entityLimit as number) * 0.8

  // Format status badge
  const getStatusBadge = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active':
        return <Badge variant="default" className="bg-green-500">Active</Badge>
      case 'trialing':
        return <Badge variant="secondary">Trialing</Badge>
      case 'canceled':
        return <Badge variant="destructive">Canceled</Badge>
      default:
        return <Badge variant="outline">{status}</Badge>
    }
  }

  // Format tier badge
  const getTierBadge = (tier: string) => {
    switch (tier.toLowerCase()) {
      case 'starter':
        return <Badge variant="outline">🛡️ Starter</Badge>
      case 'growth':
        return <Badge variant="default" className="bg-blue-500">🚀 Growth</Badge>
      case 'enterprise':
        return <Badge variant="default" className="bg-purple-500">👑 Enterprise</Badge>
      default:
        return <Badge variant="outline">{tier}</Badge>
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium">Subscription</h3>
        <p className="text-sm text-muted-foreground">
          Manage your subscription and view usage limits.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {/* Subscription Tier Card */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Subscription Tier</CardTitle>
            <CreditCard className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2">
              {getTierBadge(subscriptionTier)}
            </div>
            <p className="mt-2 text-xs text-muted-foreground">
              {subscriptionTier === 'starter' && 'Perfect for startups and small teams'}
              {subscriptionTier === 'growth' && 'Ideal for scaling companies'}
              {subscriptionTier === 'enterprise' && 'Full power for corporations'}
            </p>
          </CardContent>
        </Card>

        {/* Subscription Status Card */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Status</CardTitle>
            <Calendar className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2">
              {getStatusBadge(subscriptionStatus)}
            </div>
            {subscriptionStatus === 'trialing' && trialEndDate && (
              <p className="mt-2 text-xs text-muted-foreground">
                Trial ends: {formatDate(trialEndDate)}
              </p>
            )}
            {subscriptionStatus === 'active' && (
              <p className="mt-2 text-xs text-muted-foreground">
                Your subscription is active and in good standing
              </p>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Entity Usage Card */}
      <Card>
        <CardHeader>
          <CardTitle>Entity Usage</CardTitle>
          <CardDescription>
            Track your entity usage against your subscription limits
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Building2 className="h-5 w-5 text-muted-foreground" />
                <span className="font-medium">Entities</span>
              </div>
              <div className="flex items-center gap-2">
                {entitiesLoading ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <>
                    <span className="text-2xl font-bold">{entityCount}</span>
                    <span className="text-muted-foreground">
                      / {isUnlimited ? '∞' : entityLimit}
                    </span>
                  </>
                )}
              </div>
            </div>

            {/* Progress bar */}
            {!isUnlimited && (
              <div className="space-y-2">
                <div className="h-2 w-full overflow-hidden rounded-full bg-secondary">
                  <div
                    className={`h-full transition-all ${
                      isNearLimit ? 'bg-yellow-500' : 'bg-primary'
                    }`}
                    style={{
                      width: `${Math.min((entityCount / (entityLimit as number)) * 100, 100)}%`,
                    }}
                  />
                </div>
                {isNearLimit && (
                  <p className="text-xs text-yellow-600 dark:text-yellow-500">
                    ⚠️ You&apos;re approaching your entity limit. Consider upgrading to Growth or Enterprise tier.
                  </p>
                )}
              </div>
            )}

            {/* Tier comparison */}
            <div className="mt-6 rounded-lg border p-4">
              <h4 className="mb-3 font-medium">Entity Limits by Tier</h4>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">🛡️ Starter</span>
                  <span className="font-medium">1 entity</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">🚀 Growth</span>
                  <span className="font-medium">5 entities</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">👑 Enterprise</span>
                  <span className="font-medium">Unlimited entities</span>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
