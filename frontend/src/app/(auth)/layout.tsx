export default function AuthLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/40 p-4">
      <div className="w-full max-w-sm space-y-8">
        <div className="text-center">
          <h1 className="text-2xl font-bold tracking-tight">Zeltra</h1>
          <p className="text-sm text-muted-foreground">
            B2B Expense & Budgeting Engine
          </p>
        </div>
        {children}
      </div>
    </div>
  )
}
