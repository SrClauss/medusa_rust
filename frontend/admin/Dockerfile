# ── MedusaRust Admin UI — multi-stage Dockerfile ─────────────────────────────
#
# Stage 1 (deps):   Install npm dependencies with a clean cache.
# Stage 2 (builder): Build the Next.js production bundle.
# Stage 3 (runner):  Minimal runtime image serving with `next start`.

# ── Deps ─────────────────────────────────────────────────────────────────────
FROM node:18-alpine AS deps

WORKDIR /app

COPY package.json package-lock.json* ./
RUN npm ci --prefer-offline

# ── Builder ───────────────────────────────────────────────────────────────────
FROM node:18-alpine AS builder

WORKDIR /app

COPY --from=deps /app/node_modules ./node_modules
COPY . .

# NEXT_PUBLIC_API_URL is embedded at build time.
# Override with --build-arg or by setting the env var before running this build.
ARG NEXT_PUBLIC_API_URL=http://localhost:9000
ARG NEXT_PUBLIC_BASE_PATH=""

ENV NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL}
ENV NEXT_PUBLIC_BASE_PATH=${NEXT_PUBLIC_BASE_PATH}

RUN npm run build

# ── Runner ────────────────────────────────────────────────────────────────────
FROM node:18-alpine AS runner

WORKDIR /app

ENV NODE_ENV=production
ENV PORT=7001

# Only copy what `next start` needs.
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static

EXPOSE 7001

CMD ["node", "server.js"]
