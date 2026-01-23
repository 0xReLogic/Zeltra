import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import { approvalRuleSchema, TRANSACTION_TYPES, ROLES } from './approval-rule'

describe('Approval Rule Zod Schema Properties', () => {
  // Common generators
  const validTransactionTypesGen = fc.subarray(
    [...TRANSACTION_TYPES],
    { minLength: 1, maxLength: 10 }
  )
  const validRoleGen = fc.constantFrom(...ROLES)
  const validPriorityGen = fc.integer({ min: 1, max: 100 })
  const validNameGen = fc.string({ minLength: 1, maxLength: 255 })

  it('Property 2: Amount Range Validation', () => {
    // Valid: min <= max
    fc.assert(
      fc.property(
        validNameGen,
        validTransactionTypesGen,
        validRoleGen,
        validPriorityGen,
        fc.double({ min: 0, max: 1000, noNaN: true }).map(n => n.toFixed(2)), // min
        fc.double({ min: 0, max: 1000, noNaN: true }).map(n => n.toFixed(2)), // diff
        (name, types, role, priority, minVal, diffVal) => {
           const min = minVal
           const max = (parseFloat(minVal) + parseFloat(diffVal)).toFixed(2)
           
           const input = {
             name,
             transaction_types: types,
             required_role: role,
             priority,
             is_active: true,
             min_amount: min,
             max_amount: max,
           }
           const result = approvalRuleSchema.safeParse(input)
           expect(result.success).toBe(true)
        }
      )
    )

    // Invalid: min > max
    fc.assert(
      fc.property(
        validNameGen,
        validTransactionTypesGen,
        validRoleGen,
        validPriorityGen,
        fc.double({ min: 0, max: 1000, noNaN: true }).map(n => n.toFixed(2)), // min
        fc.double({ min: 0.01, max: 1000, noNaN: true }).map(n => n.toFixed(2)), // diff > 0
        (name, types, role, priority, baseVal, diffVal) => {
           const min = (parseFloat(baseVal) + parseFloat(diffVal)).toFixed(2)
           const max = baseVal
           
           const input = {
             name,
             transaction_types: types,
             required_role: role,
             priority,
             is_active: true,
             min_amount: min,
             max_amount: max,
           }
           const result = approvalRuleSchema.safeParse(input)
           expect(result.success).toBe(false)
           if (!result.success) {
               expect(result.error.issues[0].path).toContain('min_amount')
           }
        }
      )
    )
  })

  it('Property 3: Priority Range Enforcement', () => {
    // Valid: 1-100
    fc.assert(
       fc.property(validPriorityGen, (priority) => {
           // We construct a minimal valid object with varying priority
           const input = {
             name: 'Valid Name',
             transaction_types: ['bill'],
             required_role: 'approver',
             priority,
             is_active: true
           }
           const result = approvalRuleSchema.safeParse(input)
           expect(result.success).toBe(true)
       })
    )

    // Invalid: < 1 or > 100
    fc.assert(
       fc.property(
           fc.oneof(
               fc.integer({ max: 0 }),
               fc.integer({ min: 101 })
           ), 
           (priority) => {
               const input = {
                 name: 'Valid Name',
                 transaction_types: ['bill'],
                 required_role: 'approver',
                 priority,
                 is_active: true
               }
               const result = approvalRuleSchema.safeParse(input)
               expect(result.success).toBe(false)
           }
       )
    )
  })

  it('Property 5: String Length Constraints', () => {
    // Name validation
    fc.assert(
        fc.property(fc.string(), (name) => {
             const input = {
                 name,
                 transaction_types: ['bill'],
                 required_role: 'approver',
                 priority: 1,
                 is_active: true
               }
             const result = approvalRuleSchema.safeParse(input)
             if (name.length >= 1 && name.length <= 255) {
                 expect(result.success).toBe(true)
             } else {
                 expect(result.success).toBe(false)
             }
        })
    )

    // Description validation
    fc.assert(
        fc.property(fc.string(), (description) => {
             const input = {
                 name: 'Valid Name',
                 description,
                 transaction_types: ['bill'],
                 required_role: 'approver',
                 priority: 1,
                 is_active: true
               }
             const result = approvalRuleSchema.safeParse(input)
             if (description.length <= 1000) {
                 expect(result.success).toBe(true) // null/optional also valid, but string prop checks string
             } else {
                 expect(result.success).toBe(false)
             }
        })
    )
  })

  it('Property 4: Transaction Type Completeness', () => {
    // All 12 valid transaction types should be accepted
    const allTypes = [...TRANSACTION_TYPES]
    
    fc.assert(
      fc.property(
        fc.subarray(allTypes, { minLength: 1, maxLength: 10 }),
        (types) => {
          const input = {
            name: 'Valid Name',
            transaction_types: types,
            required_role: 'approver',
            priority: 1,
            is_active: true
          }
          const result = approvalRuleSchema.safeParse(input)
          expect(result.success).toBe(true)
        }
      )
    )

    // Invalid transaction types should be rejected
    fc.assert(
      fc.property(
        fc.string().filter(s => !(TRANSACTION_TYPES as readonly string[]).includes(s)),
        (invalidType) => {
          const input = {
            name: 'Valid Name',
            transaction_types: [invalidType],
            required_role: 'approver',
            priority: 1,
            is_active: true
          }
          const result = approvalRuleSchema.safeParse(input)
          expect(result.success).toBe(false)
        }
      )
    )
  })

  it('Property 6: Enum Validation', () => {
      // Invalid Roles
      fc.assert(
          fc.property(fc.string().filter(s => !(ROLES as readonly string[]).includes(s)), (invalidRole) => {
               const input = {
                 name: 'Valid Name',
                 transaction_types: ['bill'],
                 required_role: invalidRole,
                 priority: 1,
                 is_active: true
               }
               const result = approvalRuleSchema.safeParse(input)
               expect(result.success).toBe(false)
          })
      )
  })
})
