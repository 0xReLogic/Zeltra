'use client'

import { useState, useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useCreateOrganization } from '@/lib/queries/organizations'
import { useCurrencies } from '@/lib/queries/exchange-rates'
import { useAuthStore } from '@/lib/stores/authStore'
import { Building2, Loader2 }from 'lucide-react'

// Fallback currencies if API fails
const FALLBACK_CURRENCIES = [
  { value: 'USD', label: 'USD - US Dollar' },
  { value: 'EUR', label: 'EUR - Euro' },
  { value: 'GBP', label: 'GBP - British Pound' },
  { value: 'IDR', label: 'IDR - Indonesian Rupiah' },
  { value: 'SGD', label: 'SGD - Singapore Dollar' },
]

const TIMEZONES = [
  { value: 'UTC', label: 'UTC' },
  { value: 'Asia/Jakarta', label: 'Asia/Jakarta (WIB)' },
  { value: 'Asia/Singapore', label: 'Asia/Singapore' },
  { value: 'America/New_York', label: 'America/New_York (EST)' },
  { value: 'Europe/London', label: 'Europe/London (GMT)' },
]

export default function CreateOrganizationPage() {
  const router = useRouter()
  const user = useAuthStore((state) => state.user)
  const accessToken = useAuthStore((state) => state.accessToken)
  const [isChecking, setIsChecking] = useState(true)
  const [name, setName] = useState('')
  const [slug, setSlug] = useState('')
  const [baseCurrency, setBaseCurrency] = useState('USD')
  const [timezone, setTimezone] = useState('UTC')

  const createOrg = useCreateOrganization()
  const { data: currenciesData } = useCurrencies()
  
  // Get currencies from API or fallback
  const currencies = currenciesData?.currencies?.length 
    ? currenciesData.currencies.map(c => ({ value: c.code, label: `${c.code} - ${c.name}` }))
    : FALLBACK_CURRENCIES

  // Auth protection
  useEffect(() => {
    const checkAuth = () => {
      const state = useAuthStore.getState()
      if (!state.accessToken || !state.user) {
        router.replace('/login')
      } else {
        setIsChecking(false)
      }
    }
    const timer = setTimeout(checkAuth, 100)
    return () => clearTimeout(timer)
  }, [router])

  useEffect(() => {
    if (!isChecking && (!accessToken || !user)) {
      router.replace('/login')
    }
  }, [accessToken, user, isChecking, router])

  const generateSlug = (orgName: string) => {
    return orgName
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '')
  }

  const handleNameChange = (value: string) => {
    setName(value)
    setSlug(generateSlug(value))
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    createOrg.mutate(
      { name, slug, base_currency: baseCurrency, timezone },
      {
        onSuccess: () => {
          // Refresh the page to get new token with org
          window.location.href = '/dashboard'
        },
      }
    )
  }

  if (isChecking) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background to-muted">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!accessToken || !user) {
    return null
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-background to-muted p-4">
      <Card className="w-full max-w-lg">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
            <Building2 className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-2xl">Create Your Organization</CardTitle>
          <CardDescription>
            Welcome{user?.full_name ? `, ${user.full_name}` : ''}! Set up your organization to get started with Zeltra.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="name">Organization Name</Label>
              <Input
                id="name"
                placeholder="Acme Corporation"
                value={name}
                onChange={(e) => handleNameChange(e.target.value)}
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="slug">URL Slug</Label>
              <Input
                id="slug"
                placeholder="acme-corp"
                value={slug}
                onChange={(e) => setSlug(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, ''))}
                required
              />
              <p className="text-xs text-muted-foreground">
                Only lowercase letters, numbers, and hyphens.
              </p>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>Base Currency</Label>
                <Select value={baseCurrency} onValueChange={setBaseCurrency}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {currencies.map((c) => (
                      <SelectItem key={c.value} value={c.value}>
                        {c.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label>Timezone</Label>
                <Select value={timezone} onValueChange={setTimezone}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {TIMEZONES.map((tz) => (
                      <SelectItem key={tz.value} value={tz.value}>
                        {tz.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            <Button type="submit" className="w-full" disabled={createOrg.isPending || !name || !slug}>
              {createOrg.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Creating...
                </>
              ) : (
                'Create Organization'
              )}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  )
}
