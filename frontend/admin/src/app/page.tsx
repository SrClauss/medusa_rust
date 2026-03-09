import { redirect } from "next/navigation";

/**
 * Root page: redirect unauthenticated users to /login,
 * authenticated users to /dashboard.
 */
export default function RootPage() {
  // The middleware (src/middleware.ts) handles auth gating.
  // This redirect ensures the root path is never blank.
  redirect("/dashboard");
}
