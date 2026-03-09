/** @type {import('next').NextConfig} */
const nextConfig = {
  // Use standalone output so the Docker image only contains what `next start` needs.
  output: "standalone",

  // The admin panel is served under /app when using a reverse proxy.
  // Set NEXT_PUBLIC_BASE_PATH to "/app" (or leave empty for root).
  basePath: process.env.NEXT_PUBLIC_BASE_PATH || "",

  // Allow images from the Rust API origin and any configured S3 bucket.
  images: {
    remotePatterns: [
      {
        protocol: "http",
        hostname: "localhost",
      },
      {
        protocol: "https",
        hostname: "**",
      },
    ],
  },

  // Proxy /api requests to the Rust backend during development so the
  // browser never needs to deal with CORS when running `npm run dev`.
  async rewrites() {
    // INTERNAL_API_URL is used by the Next.js server process (e.g., inside Docker)
    // to proxy /api/* requests to the Rust backend within the same network.
    // Falls back to NEXT_PUBLIC_API_URL for local development without Docker.
    const apiUrl =
      process.env.INTERNAL_API_URL ||
      process.env.NEXT_PUBLIC_API_URL ||
      "http://localhost:9000";
    return [
      {
        source: "/api/:path*",
        destination: `${apiUrl}/:path*`,
      },
    ];
  },
};

module.exports = nextConfig;
