'use client'

import React, { useEffect } from 'react'
import { useRouter, useSearchParams } from 'next/navigation'
import { useVerifyEmail, useResendVerification } from '@/lib/queries/auth'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Loader2, CheckCircle2, XCircle } from 'lucide-react'
import { toast } from 'sonner'

function VerifyEmailContent() {
  const searchParams = useSearchParams()
  const token = searchParams.get('token')
  const router = useRouter()
  const verifyEmail = useVerifyEmail()
  const resendVerification = useResendVerification()
  const [email, setEmail] = React.useState('')
  const [showResend, setShowResend] = React.useState(false)

  useEffect(() => {
    if (!token) {
      setShowResend(true)
      return
    }

    verifyEmail.mutate({ token }, {
      onSuccess: () => {
        setTimeout(() => {
          router.push('/login?verified=true')
        }, 3000)
      },
      onError: () => {
        setShowResend(true)
      }
    })
    // Execute once on mount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token])

  const handleResend = () => {
    if (!email) {
      toast.error('Please enter your email address')
      return
    }
    resendVerification.mutate({ email })
  }

  if (verifyEmail.isPending) {
    return (
      <div className="flex flex-col items-center justify-center space-y-4 py-8">
        <Loader2 className="h-12 w-12 animate-spin text-primary" />
        <p className="text-muted-foreground">Verifying your email...</p>
      </div>
    )
  }

  if (verifyEmail.isSuccess) {
    return (
      <div className="flex flex-col items-center justify-center space-y-4 py-8">
        <CheckCircle2 className="h-12 w-12 text-green-500" />
        <h3 className="text-xl font-semibold">Email Verified!</h3>
        <p className="text-muted-foreground text-center">
          Your email has been successfully verified. <br />
          Redirecting to login page...
        </p>
      </div>
    )
  }

  return (
    <div className="flex flex-col space-y-4">
      <div className="flex flex-col items-center justify-center space-y-2 py-4">
        <XCircle className="h-12 w-12 text-destructive" />
        <h3 className="text-xl font-semibold text-destructive">Verification Failed</h3>
        <p className="text-muted-foreground text-center">
          The verification link is invalid or has expired.
        </p>
      </div>

      {showResend && (
        <div className="space-y-4 pt-4 border-t">
          <div className="space-y-2">
            <label htmlFor="email" className="text-sm font-medium">
              Enter your email to resend verification link
            </label>
            <input
              id="email"
              type="email"
              placeholder="name@example.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
            />
          </div>
          <Button 
            onClick={handleResend} 
            className="w-full"
            disabled={resendVerification.isPending}
          >
            {resendVerification.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Resend Verification Email
          </Button>
        </div>
      )}
    </div>
  )
}

export default function VerifyEmailPage() {
  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>Email Verification</CardTitle>
        <CardDescription>Confirming your email address for Zeltra</CardDescription>
      </CardHeader>
      <CardContent>
        <React.Suspense fallback={<div className="flex justify-center py-8"><Loader2 className="h-8 w-8 animate-spin" /></div>}>
           <VerifyEmailContent />
        </React.Suspense>
      </CardContent>
    </Card>
  )
}
