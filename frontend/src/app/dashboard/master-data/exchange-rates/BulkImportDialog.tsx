'use client'

import React from 'react'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Upload, AlertCircle } from 'lucide-react'
import { Textarea } from '@/components/ui/textarea'
import { toast } from 'sonner'
import { useBulkImportExchangeRates } from '@/lib/queries/exchange-rates'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'

export function BulkImportDialog() {
    const [open, setOpen] = React.useState(false)
    const [csvData, setCsvData] = React.useState('')
    const bulkImport = useBulkImportExchangeRates()

    const handleImport = () => {
        try {
            const lines = csvData.trim().split('\n')
            const rates = lines.map(line => {
                const [from, to, rate, date] = line.split(',').map(s => s.trim())
                if (!from || !to || !rate || !date) throw new Error('Invalid format')
                return {
                    from_currency: from.toUpperCase(),
                    to_currency: to.toUpperCase(),
                    rate,
                    date
                }
            })

            bulkImport.mutate({ rates }, {
                onSuccess: () => {
                    toast.success(`Successfully imported ${rates.length} rates`)
                    setOpen(false)
                    setCsvData('')
                },
                onError: () => toast.error('Failed to import rates')
            })
        } catch {
            toast.error('Invalid CSV format. Please check your input.')
        }
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button variant="outline">
                    <Upload className="mr-2 h-4 w-4" />
                    Bulk Import
                </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[500px]">
                <DialogHeader>
                    <DialogTitle>Bulk Import Exchange Rates</DialogTitle>
                    <DialogDescription>
                        Paste CSV data below. Format: <code className="bg-muted px-1 rounded">FROM,TO,RATE,YYYY-MM-DD</code>
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-4">
                    <Alert variant="default" className="bg-muted/50">
                        <AlertCircle className="h-4 w-4" />
                        <AlertTitle>Example</AlertTitle>
                        <AlertDescription className="font-mono text-xs mt-1">
                            USD,IDR,15500,2026-01-01<br />
                            SGD,IDR,11500,2026-01-01
                        </AlertDescription>
                    </Alert>

                    <Textarea
                        placeholder="Paste your CSV data here..."
                        className="h-[200px] font-mono text-sm"
                        value={csvData}
                        onChange={(e) => setCsvData(e.target.value)}
                    />

                    <div className="flex justify-end pt-2">
                        <Button onClick={handleImport} disabled={!csvData || bulkImport.isPending}>
                            {bulkImport.isPending ? 'Importing...' : 'Import Rates'}
                        </Button>
                    </div>
                </div>
            </DialogContent>
        </Dialog>
    )
}
