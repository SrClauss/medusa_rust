import axios from "axios";

/**
 * Pre-configured Axios instance for the MedusaRust Admin API.
 *
 * During development Next.js rewrites /api/* → NEXT_PUBLIC_API_URL/*
 * (see next.config.js), so the browser never hits a CORS preflight.
 *
 * In production the same rewrites work when the frontend is served by
 * the Next.js server. Alternatively you can configure your reverse
 * proxy (nginx / Caddy) to route /api/* → the Rust backend.
 */
const BASE_URL =
  typeof window === "undefined"
    ? // Server-side: use the internal API URL when available (e.g., within Docker),
      // otherwise fall back to the public API URL.
      process.env.INTERNAL_API_URL ??
      process.env.NEXT_PUBLIC_API_URL ??
      "http://localhost:9000"
    : // Client-side: use the Next.js rewrite proxy (/api/*) to avoid CORS.
      "";

export const adminApi = axios.create({
  baseURL: BASE_URL,
  headers: {
    "Content-Type": "application/json",
  },
});

/**
 * Attach the stored JWT to every outgoing request.
 */
adminApi.interceptors.request.use((config) => {
  if (typeof window !== "undefined") {
    const token = localStorage.getItem("medusa_admin_token");
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
  }
  return config;
});

/**
 * Redirect to /login on 401 Unauthorized.
 */
adminApi.interceptors.response.use(
  (res) => res,
  (err) => {
    if (err.response?.status === 401 && typeof window !== "undefined") {
      localStorage.removeItem("medusa_admin_token");
      document.cookie = "medusa_admin_token=; path=/; max-age=0; SameSite=Lax";
      window.location.href = "/login";
    }
    return Promise.reject(err);
  }
);
