// Auth Types
export interface LoginRequest {
  email: string
  password: string
}

export interface RegisterRequest {
  email: string
  password: string
  full_name: string
}

export interface AuthResponse {
  user: {
    id: string
    email: string
    full_name: string
    organizations: Array<{
      id: string
      name: string
      slug: string
      role: string
    }>
  }
  access_token: string
  refresh_token: string
  expires_in: number
}

export interface VerifyEmailRequest {
  token: string
}

export interface VerifyEmailResponse {
  message: string
  verified: boolean
}

export interface ResendVerificationRequest {
  email: string
}

export interface ResendVerificationResponse {
  message: string
}
