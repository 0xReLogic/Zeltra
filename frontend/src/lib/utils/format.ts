export function formatCurrency(amount: number | string, currency: string = 'USD'): string {
  const value = typeof amount === 'string' ? parseFloat(amount) : amount
  
  if (isNaN(value)) {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: currency,
    }).format(0)
  }

  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value)
}
