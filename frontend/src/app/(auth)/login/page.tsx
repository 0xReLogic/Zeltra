'use client'

import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { useLogin } from '@/lib/queries/auth'
import Link from 'next/link'

const loginSchema = z.object({
  email: z.string().email('Invalid email address'),
  password: z.string().min(8, 'Password must be at least 8 characters'),
})

type LoginFormValues = z.infer<typeof loginSchema>


import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { useState } from 'react'
import { useResendVerification } from '@/lib/queries/auth'

export default function LoginPage() {
  const { mutate: login, isPending } = useLogin()
  const resendVerification = useResendVerification()
  const [resendEmail, setResendEmail] = useState('')
  const [open, setOpen] = useState(false)

  const form = useForm<LoginFormValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      email: '',
      password: '',
    },
  })

  function onSubmit(data: LoginFormValues) {
    login(data)
  }

  function handleResend() {
    if (!resendEmail) return
    resendVerification.mutate(
      { email: resendEmail },
      {
        onSuccess: () => {
          setOpen(false)
          setResendEmail('')
        },
      }
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Welcome back</CardTitle>
        <CardDescription>
          Enter your email and password to sign in to your account
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Email</FormLabel>
                  <FormControl>
                    <Input placeholder="name@example.com" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="password"
              render={({ field }) => (
                <FormItem>
                  <div className="flex items-center justify-between">
                    <FormLabel>Password</FormLabel>
                    <Link href="/forgot-password" className="text-sm text-muted-foreground hover:text-primary">
                      Forgot password?
                    </Link>
                  </div>
                  <FormControl>
                    <Input type="password" placeholder="••••••••" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button type="submit" className="w-full" disabled={isPending}>
              {isPending ? 'Signing in...' : 'Sign in'}
            </Button>
          </form>
        </Form>
      </CardContent>
      <CardFooter className="flex flex-col gap-2 justify-center">
        <p className="text-sm text-muted-foreground">
          Don&apos;t have an account?{' '}
          <Link href="/register" className="font-medium text-primary hover:underline">
            Sign up
          </Link>
        </p>
        
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button variant="link" className="text-xs text-muted-foreground font-normal h-auto p-0">
              Didn&apos;t receive verification email?
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Resend Verification Email</DialogTitle>
              <DialogDescription>
                Enter your email address and we&apos;ll send you a new link.
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-2">
              <div className="space-y-2">
                <label htmlFor="resend-email" className="text-sm font-medium">Email</label>
                <Input 
                  id="resend-email"
                  placeholder="name@example.com"
                  value={resendEmail}
                  onChange={(e) => setResendEmail(e.target.value)}
                />
              </div>
              <Button 
                onClick={handleResend} 
                className="w-full"
                disabled={resendVerification.isPending || !resendEmail}
              >
                {resendVerification.isPending ? 'Sending...' : 'Send Verification Email'}
              </Button>
            </div>
          </DialogContent>
        </Dialog>
      </CardFooter>
    </Card>
  )
}
