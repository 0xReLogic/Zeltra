import React, { useState } from 'react'
import { 
  useReconciliation, 
  useBenford, 
  useHealthScore,
  type ReconciliationResponse,
  type BenfordResponse,
  type HealthScoreResponse
} from '@/lib/queries/forensic'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { 
  Loader2, 
  CheckCircle2, 
  AlertTriangle, 
  XCircle, 
  RefreshCw,
  BarChart3,
  Activity,
  ShieldCheck,
  Scale
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useQueryClient } from '@tanstack/react-query'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/components/ui/tabs'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts'

export default function ForensicPage() {
  const queryClient = useQueryClient()
  const [activeTab, setActiveTab] = useState('reconciliation')

  const { 
    data: recData, 
    isLoading: recLoading, 
    error: recError, 
    refetch: refetchRec,
    isFetching: recFetching 
  } = useReconciliation()

  const { 
    data: benfordData, 
    isLoading: benfordLoading, 
    error: benfordError,
    refetch: refetchBenford,
    isFetching: benfordFetching
  } = useBenford()

  const { 
    data: healthData, 
    isLoading: healthLoading, 
    error: healthError,
    refetch: refetchHealth,
    isFetching: healthFetching
  } = useHealthScore()

  const handleRefresh = () => {
    if (activeTab === 'reconciliation') {
      queryClient.invalidateQueries({ queryKey: ['forensic', 'reconciliation'] })
      refetchRec()
    } else if (activeTab === 'benford') {
      queryClient.invalidateQueries({ queryKey: ['forensic', 'benford'] })
      refetchBenford()
    } else if (activeTab === 'health') {
      queryClient.invalidateQueries({ queryKey: ['forensic', 'health-score'] })
      refetchHealth()
    }
  }

  const isGlobalFetching = recFetching || benfordFetching || healthFetching

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Forensic Analysis</h1>
          <p className="text-muted-foreground">
            Zeltra Audit-Grade Integrity & Fraud Detection Suite
          </p>
        </div>
        <Button onClick={handleRefresh} disabled={isGlobalFetching} variant="outline">
          <RefreshCw className={`mr-2 h-4 w-4 ${isGlobalFetching ? 'animate-spin' : ''}`} />
          Refresh Analysis
        </Button>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
        <TabsList>
          <TabsTrigger value="reconciliation">
            <CheckCircle2 className="mr-2 h-4 w-4" />
            Reconciliation
          </TabsTrigger>
          <TabsTrigger value="benford">
            <BarChart3 className="mr-2 h-4 w-4" />
            Benford&apos;s Law
          </TabsTrigger>
          <TabsTrigger value="health">
            <ShieldCheck className="mr-2 h-4 w-4" />
            Financial Health
          </TabsTrigger>
        </TabsList>

        <TabsContent value="reconciliation" className="space-y-4">
          {recError ? (
            <ErrorState message="Failed to load reconciliation data" error={recError} />
          ) : recLoading ? (
            <LoadingState />
          ) : (
            <ReconciliationLayout data={recData} />
          )}
        </TabsContent>

        <TabsContent value="benford" className="space-y-4">
          {benfordError ? (
            <ErrorState message="Failed to load Benford analysis" error={benfordError} />
          ) : benfordLoading ? (
            <LoadingState />
          ) : (
            <BenfordLayout data={benfordData} />
          )}
        </TabsContent>

        <TabsContent value="health" className="space-y-4">
          {healthError ? (
            <ErrorState message="Failed to load health score data" error={healthError} />
          ) : healthLoading ? (
            <LoadingState />
          ) : (
            <HealthLayout data={healthData} />
          )}
        </TabsContent>
      </Tabs>
    </div>
  )
}

function LoadingState() {
  return (
    <div className="flex items-center justify-center min-h-[400px]">
      <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
    </div>
  )
}

function ErrorState({ message, error }: { message: string, error: unknown }) {
  const errorMessage = error instanceof Error ? error.message : 'Enterprise tier required'
  return (
    <div className="flex flex-col items-center justify-center min-h-[400px] gap-4">
      <XCircle className="h-12 w-12 text-destructive" />
      <p className="text-muted-foreground">{message}</p>
      <p className="text-sm text-muted-foreground">
        {errorMessage}
      </p>
    </div>
  )
}

function ReconciliationLayout({ data }: { data: ReconciliationResponse | undefined }) {
  if (!data) return null
  return (
    <div className="space-y-4">
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Integrity Status</CardTitle>
            {data.is_clean ? <CheckCircle2 className="h-4 w-4 text-green-500" /> : <AlertTriangle className="h-4 w-4 text-yellow-500" />}
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data.is_clean ? 'Clean' : 'Review Needed'}</div>
            <p className="text-xs text-muted-foreground">Balance drift detection</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Matched</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{data.matched_count ?? 0}</div>
            <p className="text-xs text-muted-foreground">Accounts in sync</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Discrepancies</CardTitle>
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${data.discrepancy_count > 0 ? 'text-red-600' : 'text-green-600'}`}>{data.discrepancy_count ?? 0}</div>
            <p className="text-xs text-muted-foreground">Requires audit</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Drift Logic</CardTitle>
            <ShieldCheck className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-primary">Sentinel</div>
            <p className="text-xs text-muted-foreground">Calculated vs Stored</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Core Ledger Reconciliation</CardTitle>
          <CardDescription>Continuous audit of account balances against transaction history.</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Account</TableHead>
                <TableHead className="text-right">Stored</TableHead>
                <TableHead className="text-right">Calculated</TableHead>
                <TableHead className="text-right">Difference</TableHead>
                <TableHead>Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.accounts?.map((acc) => (
                <TableRow key={acc.account_id}>
                  <TableCell>
                    <div className="font-medium">{acc.account_name}</div>
                    <div className="text-xs text-muted-foreground">{acc.account_code}</div>
                  </TableCell>
                  <TableCell className="text-right font-mono">{parseFloat(acc.stored_balance).toLocaleString(undefined, { minimumFractionDigits: 2 })}</TableCell>
                  <TableCell className="text-right font-mono">{parseFloat(acc.calculated_balance).toLocaleString(undefined, { minimumFractionDigits: 2 })}</TableCell>
                  <TableCell className="text-right font-mono font-bold text-red-500">{parseFloat(acc.difference) !== 0 ? parseFloat(acc.difference).toLocaleString(undefined, { minimumFractionDigits: 2 }) : '-'}</TableCell>
                  <TableCell>
                    <Badge variant={acc.status === 'Matched' ? 'default' : 'destructive'}>{acc.status}</Badge>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}

function BenfordLayout({ data }: { data: BenfordResponse | undefined }) {
  const [digitType, setDigitType] = useState<'1st' | '2nd'>('1st')
  if (!data) return null
  const chartData = digitType === '1st' ? data.distribution_1st_digit : data.distribution_2nd_digit

  return (
    <div className="grid gap-4 md:grid-cols-6">
      <Card className="md:col-span-2">
        <CardHeader>
          <CardTitle>Statistical Verdict</CardTitle>
          <CardDescription>Benford&apos;s Law conformance</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="text-center p-6 border rounded-lg bg-muted/50">
            <div className={`text-xl font-bold ${data.mad_score < 0.006 ? 'text-green-600' : 'text-red-600'}`}>
              {data.mad_verdict}
            </div>
            <div className="text-sm text-muted-foreground mt-1">MAD Score: {data.mad_score?.toFixed(5)}</div>
          </div>
          <div className="space-y-4">
            <h4 className="text-sm font-semibold">Audit Guide (MAD)</h4>
            <div className="space-y-2 text-xs">
              <div className="flex justify-between"><span>0.000 - 0.006</span> <span className="text-green-600">Close Conformity</span></div>
              <div className="flex justify-between"><span>0.006 - 0.012</span> <span className="text-blue-600">Acceptable</span></div>
              <div className="flex justify-between"><span>0.012 - 0.015</span> <span className="text-yellow-600">Marginal</span></div>
              <div className="flex justify-between"><span>0.015+</span> <span className="text-red-600">Nonconformity</span></div>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card className="md:col-span-4">
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>Digit Distribution</CardTitle>
            <CardDescription>Observed vs Expected Frequency</CardDescription>
          </div>
          <div className="flex gap-2">
            <Button variant={digitType === '1st' ? 'default' : 'outline'} size="sm" onClick={() => setDigitType('1st')}>1st Digit</Button>
            <Button variant={digitType === '2nd' ? 'default' : 'outline'} size="sm" onClick={() => setDigitType('2nd')}>2nd Digit</Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="h-[300px]">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" vertical={false} />
                <XAxis dataKey="digit" />
                <YAxis />
                <Tooltip 
                  formatter={(value: number | string | undefined) => value ? (Number(value) * 100).toFixed(1) + '%' : '0%'}
                />
                <Legend />
                <Bar name="Actual" dataKey="actual_frequency" fill="#0ea5e9" radius={[4, 4, 0, 0]} />
                <Bar name="Expected" dataKey="expected_frequency" fill="#94a3b8" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function HealthLayout({ data }: { data: HealthScoreResponse | undefined }) {
  if (!data) return null
  return (
    <div className="grid gap-4 md:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5 text-blue-500" />
            Altman Z-Score
          </CardTitle>
          <CardDescription>Bankruptcy & Solvency Prediction</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center justify-between pb-4 border-b">
            <div>
              <div className="text-3xl font-bold">{data.z_score?.toFixed(2)}</div>
              <Badge 
                className="mt-1"
                variant={data.z_zone === 'Safe' ? 'default' : data.z_zone === 'Grey' ? 'secondary' : 'destructive'}
              >
                {data.z_zone} Zone
              </Badge>
            </div>
            <div className="text-right">
              <div className="text-sm font-medium text-muted-foreground italic">Reference:</div>
              <div className="text-xs space-y-1">
                <div className="text-green-600 font-bold">Safe: {'>'} 2.99</div>
                <div className="text-yellow-600 font-bold">Grey: 1.81 - 2.99</div>
                <div className="text-red-600 font-bold">Distress: {'<'} 1.81</div>
              </div>
            </div>
          </div>
          <div className="space-y-3">
            <h4 className="text-sm font-semibold">Z-Score Coefficients</h4>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <div className="flex justify-between border-b pb-1"><span>Working Capital/TA (X1)</span> <span>{data.z_details?.x1_liquidity?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>Retained Earnings/TA (X2)</span> <span>{data.z_details?.x2_profitability?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>EBIT/TA (X3)</span> <span>{data.z_details?.x3_leverage?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>Equity/TL (X4)</span> <span>{data.z_details?.x4_solvency?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>Sales/TA (X5)</span> <span>{data.z_details?.x5_activity?.toFixed(2)}</span></div>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Scale className="h-5 w-5 text-amber-500" />
            Beneish M-Score
          </CardTitle>
          <CardDescription>Financial Manipulation Detector</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center justify-between pb-4 border-b">
            <div>
              <div className="text-3xl font-bold">{data.m_score?.toFixed(2)}</div>
              <Badge 
                className="mt-1"
                variant={data.m_risk_level === 'Safe' ? 'default' : 'destructive'}
              >
                {data.m_risk_level}
              </Badge>
            </div>
            <div className="text-right">
              <div className="text-sm font-medium text-muted-foreground italic">Manipulation Probability:</div>
              <div className={`text-xl font-bold ${data.m_prob > 0.05 ? 'text-red-600' : 'text-green-600'}`}>
                {(data.m_prob * 100).toFixed(1)}%
              </div>
            </div>
          </div>
          <div className="space-y-3">
            <h4 className="text-sm font-semibold">M-Score Indices</h4>
            <div className="grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
              <div className="flex justify-between border-b pb-1"><span>Receivables (DSRI)</span> <span className={data.m_details?.dsri > 1.25 ? 'text-red-600 font-bold' : ''}>{data.m_details?.dsri?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>Gross Margin (GMI)</span> <span className={data.m_details?.gmi > 1.0 ? 'text-red-600 font-bold' : ''}>{data.m_details?.gmi?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>Asset Quality (AQI)</span> <span className={data.m_details?.aqi > 1.0 ? 'text-red-600 font-bold' : ''}>{data.m_details?.aqi?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>Sales Growth (SGI)</span> <span className={data.m_details?.sgi > 1.0 ? 'text-red-600 font-bold' : ''}>{data.m_details?.sgi?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>Depreciation (DEPI)</span> <span className={data.m_details?.depi > 1.0 ? 'text-red-600 font-bold' : ''}>{data.m_details?.depi?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>SGA Exp (SGAI)</span> <span className={data.m_details?.sgai > 1.0 ? 'text-red-600 font-bold' : ''}>{data.m_details?.sgai?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>Leverage (LVGI)</span> <span className={data.m_details?.lvgi > 1.0 ? 'text-red-600 font-bold' : ''}>{data.m_details?.lvgi?.toFixed(2)}</span></div>
              <div className="flex justify-between border-b pb-1"><span>Accruals (TATA)</span> <span className={data.m_details?.tata > 0.05 ? 'text-red-600 font-bold' : ''}>{data.m_details?.tata?.toFixed(2)}</span></div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

