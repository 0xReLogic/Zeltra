'use client'

/**
 * MSWProvider - Previously used for Mock Service Worker
 * 
 * MSW has been disabled as the frontend now uses real backend API.
 * This component is kept as a passthrough for backward compatibility.
 */
export function MSWProvider({ children }: { children: React.ReactNode }) {
  // MSW disabled - always render children directly
  return <>{children}</>
}
