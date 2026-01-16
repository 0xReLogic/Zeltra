// Auth Types - imported from OpenAPI generated types
import type { components } from './api.generated'

export type LoginRequest = components['schemas']['LoginRequest']
export type RegisterRequest = components['schemas']['RegisterRequest']
export type LoginResponse = components['schemas']['LoginResponse']
export type RegisterResponse = components['schemas']['RegisterResponse']
export type VerifyEmailRequest = components['schemas']['VerifyEmailRequest']
export type VerifyEmailResponse = components['schemas']['VerifyEmailResponse']
export type ResendVerificationRequest = components['schemas']['ResendVerificationRequest']
export type ResendVerificationResponse = components['schemas']['ResendVerificationResponse']
export type RefreshRequest = components['schemas']['RefreshRequest']
export type RefreshResponse = components['schemas']['RefreshResponse']
export type LogoutRequest = components['schemas']['LogoutRequest']
export type UserInfo = components['schemas']['UserInfo']
export type UserOrganization = components['schemas']['UserOrganization']

// Alias for backward compatibility
export type AuthResponse = LoginResponse
