import { NextResponse } from 'next/server'

/**
 * Next.js Middleware for route protection.
 * 
 * IMPORTANT LIMITATION: This middleware cannot access localStorage where our auth tokens are stored.
 * Zustand persist uses localStorage by default, which is only available in the browser (client-side).
 * Middleware runs on the server/edge, so it cannot read localStorage.
 * 
 * CURRENT IMPLEMENTATION:
 * - We rely on client-side protection in layout components (dashboard/layout.tsx)
 * - The dashboard layout checks auth state after hydration and redirects if needed
 * - This provides adequate protection for our use case
 * 
 * ALTERNATIVE APPROACHES (if needed in future):
 * 1. Sync auth tokens to httpOnly cookies (most secure)
 *    - Middleware can read cookies
 *    - Protects against XSS attacks
 *    - Requires backend to set cookies on login
 * 
 * 2. Use Zustand persist with cookie storage instead of localStorage
 *    - Middleware can read cookies
 *    - Less secure than httpOnly cookies (accessible via JS)
 *    - Simpler to implement than option 1
 * 
 * 3. Keep current approach (client-side only protection)
 *    - Simplest implementation
 *    - Adequate for most use cases
 *    - User might briefly see protected content before redirect
 * 
 * For now, we use approach #3 with client-side protection.
 */
export function middleware() {
  // For now, we pass through all requests and rely on client-side protection
  // If we implement cookie-based auth in future, we can add checks here
  
  // Example of how to use these in future:
  // const protectedRoutes = ['/dashboard']
  // const authRoutes = ['/login', '/register']
  // const { pathname } = request.nextUrl
  // const authCookie = request.cookies.get('auth-storage')?.value
  // if (authCookie && authRoutes.some(route => pathname.startsWith(route))) {
  //   return NextResponse.redirect(new URL('/dashboard', request.url))
  // }
  // if (!authCookie && protectedRoutes.some(route => pathname.startsWith(route))) {
  //   return NextResponse.redirect(new URL('/login', request.url))
  // }
  
  return NextResponse.next()
}

export const config = {
  matcher: ['/((?!api|_next/static|_next/image|favicon.ico).*)'],
}
