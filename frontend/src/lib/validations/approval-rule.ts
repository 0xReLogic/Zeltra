import { z } from 'zod'

// Transaction types enum from OpenAPI spec
export const TRANSACTION_TYPES = [
  'journal',
  'invoice', 
  'bill',
  'payment',
  'expense',
  'transfer',
  'adjustment',
  'opening_balance',
  'reversal',
  'accrual',
  'revaluation',
  'intercompany',
] as const

// Roles enum from OpenAPI spec
export const ROLES = [
  'viewer',
  'submitter', 
  'approver',
  'accountant',
  'admin',
  'owner',
] as const

// Amount format validation regex (up to 2 decimal places)
const AMOUNT_REGEX = /^[0-9]+(\.[0-9]{1,2})?$/

export const approvalRuleSchema = z.object({
  name: z
    .string()
    .min(1, 'Name is required')
    .max(255, 'Name must be 255 characters or less'),
  
  description: z
    .string()
    .max(1000, 'Description must be 1000 characters or less')
    .optional()
    .nullable(),
  
  transaction_types: z
    .array(z.enum(TRANSACTION_TYPES))
    .min(1, 'At least one transaction type is required')
    .max(10, 'Maximum 10 transaction types allowed'),
  
  required_role: z.enum(ROLES),
  
  priority: z
    .number()
    .int('Priority must be a whole number')
    .min(1, 'Priority must be at least 1')
    .max(100, 'Priority must be at most 100'),
  
  min_amount: z
    .string()
    .regex(AMOUNT_REGEX, 'Amount must be a valid decimal with up to 2 decimal places')
    .optional()
    .nullable(),
  
  max_amount: z
    .string()
    .regex(AMOUNT_REGEX, 'Amount must be a valid decimal with up to 2 decimal places')
    .optional()
    .nullable(),
  
  is_active: z.boolean(),
}).refine(
  (data) => {
    // Cross-field validation: min_amount <= max_amount
    if (data.min_amount && data.max_amount) {
      const minAmount = parseFloat(data.min_amount)
      const maxAmount = parseFloat(data.max_amount)
      return minAmount <= maxAmount
    }
    return true
  },
  {
    message: 'Minimum amount must be less than or equal to maximum amount',
    path: ['min_amount'],
  }
)

export type ApprovalRuleFormValues = z.infer<typeof approvalRuleSchema>