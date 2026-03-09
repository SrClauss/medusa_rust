import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

/**
 * Middleware: protect /dashboard (and any other non-public routes).
 *
 * If the request is for a protected path and the auth cookie is absent,
 * redirect to /login.
 *
 * NOTE: localStorage is not available in middleware (Edge runtime), so
 * the token must also be stored as an HttpOnly cookie by the login flow.
 * The login page sets `medusa_admin_token` as a cookie after a successful
 * API call. This middleware reads that cookie.
 */
export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  const publicPaths = ["/login"];
  if (publicPaths.some((p) => pathname.startsWith(p))) {
    return NextResponse.next();
  }

  const token = request.cookies.get("medusa_admin_token")?.value;
  if (!token) {
    const loginUrl = new URL("/login", request.url);
    loginUrl.searchParams.set("from", pathname);
    return NextResponse.redirect(loginUrl);
  }

  return NextResponse.next();
}

export const config = {
  matcher: ["/dashboard/:path*"],
};
