import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import Link from 'next/link'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Activity, CreditCard, FileText, UserPlus, CheckCircle, XCircle, Clock } from 'lucide-react'
import { useRecentActivity } from '@/lib/queries/dashboard'
import { formatDistanceToNow } from 'date-fns'

const ActivityIcon = ({ type, action }: { type: string, action: string }) => {
  if (action === 'approved') return <CheckCircle className="h-4 w-4 text-green-500" />
  if (action === 'rejected' || action === 'voided') return <XCircle className="h-4 w-4 text-red-500" />
  
  switch (type) {
    case 'transaction_created':
      return <CreditCard className="h-4 w-4 text-blue-500" />
    case 'budget_created':
    case 'budget_updated':
      return <FileText className="h-4 w-4 text-purple-500" />
    case 'user_invited':
      return <UserPlus className="h-4 w-4 text-orange-500" />
    default:
      return <Activity className="h-4 w-4 text-gray-500" />
  }
}

export function RecentActivity() {
  const { data, isLoading } = useRecentActivity()

  return (
    <Card className="col-span-3">
      <CardHeader>
        <CardTitle>Recent Activity</CardTitle>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[300px] pr-4">
          {isLoading ? (
            <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
                Loading activity...
            </div>
          ) : (
            <div className="space-y-4">
              {data?.activities.map((item) => {
                const getLink = () => {
                   switch (item.entity_type) {
                     case 'transaction': return `/dashboard/transactions/${item.entity_id}`
                     case 'budget': return `/dashboard/budgets/${item.entity_id}`
                     default: return '#'
                   }
                }
                const link = getLink()
                const isClickable = link !== '#'

                const Content = (
                  <div className={`flex items-start gap-4 text-sm ${isClickable ? 'hover:bg-muted/50 p-2 rounded-md transition-colors cursor-pointer' : 'p-2'}`}>
                    <div className="mt-1">
                      <ActivityIcon type={item.type} action={item.action} />
                    </div>
                    <div className="grid gap-1">
                      <p className="font-medium leading-none">
                        {item.user.full_name} <span className="text-muted-foreground font-normal">{item.action.replace('_', ' ')}</span> {item.entity_type}
                      </p>
                      <p className="text-muted-foreground line-clamp-1">
                        {item.description}
                      </p>
                      {item.amount && (
                          <p className="font-mono text-xs text-muted-foreground">
                              {item.currency} {parseFloat(item.amount).toLocaleString('en-US', { minimumFractionDigits: 2 })}
                          </p>
                      )}
                    </div>
                    <div className="ml-auto text-xs text-muted-foreground whitespace-nowrap">
                      {formatDistanceToNow(new Date(item.timestamp), { addSuffix: true })}
                    </div>
                  </div>
                )

                return (
                  <div key={item.id}>
                    {isClickable ? (
                      <Link href={link} className="block">
                        {Content}
                      </Link>
                    ) : Content}
                  </div>
                )
              })}
              
              {!isLoading && (!data?.activities || data.activities.length === 0) && (
                  <div className="text-center text-muted-foreground text-sm py-4">
                      No recent activity
                  </div>
              )}
            </div>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  )
}
