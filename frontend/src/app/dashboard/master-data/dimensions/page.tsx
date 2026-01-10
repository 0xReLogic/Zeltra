'use client'

import React from 'react'
import { useDimensions } from '@/lib/queries/dimensions'
import { Loader2 } from 'lucide-react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { DimensionValues } from './DimensionValues'
import { DimensionTypeDialog } from './DimensionTypeDialog'

export default function DimensionsPage() {
  const { data, isLoading } = useDimensions()
  const [activeTab, setActiveTab] = React.useState<string>('')

  // Set initial tab when data loads
  React.useEffect(() => {
    if (data && data.length > 0 && !activeTab) {
      setActiveTab(data[0].code)
    }
  }, [data, activeTab])

  if (isLoading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Dimensions</h1>
          <p className="text-muted-foreground mt-2">
            Manage cost centers, projects, and other analytic dimensions.
          </p>
        </div>
        <DimensionTypeDialog />
      </div>

      {(!data || data.length === 0) ? (
        <div className="text-center py-10 border rounded-lg bg-muted/20">
          <p className="text-muted-foreground">No dimensions defined yet.</p>
        </div>
      ) : (
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList>
            {data.map(dim => (
              <TabsTrigger key={dim.id} value={dim.code}>
                {dim.name}
              </TabsTrigger>
            ))}
          </TabsList>
          {data.map(dim => (
            <TabsContent key={dim.id} value={dim.code}>
              <DimensionValues dimension={dim} />
            </TabsContent>
          ))}
        </Tabs>
      )}
    </div>
  )
}
