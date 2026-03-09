# MedusaRust Admin Panel

Next.js + Tailwind CSS admin panel for the [MedusaRust](../../README.md) API.

## Prerequisites

- **Node.js ≥ 18**
- **npm ≥ 9**
- A running MedusaRust API (see root `README.md`)

## Quick Start (development)

```bash
# 1. Copy the example env file
cp .env.example .env.local

# 2. Edit .env.local — set NEXT_PUBLIC_API_URL to your running API
#    Default: http://localhost:9000

# 3. Install dependencies
npm install

# 4. Start the dev server (listens on port 7001 by default)
npm run dev
```

Open [http://localhost:7001](http://localhost:7001) in your browser.  
You will be redirected to `/login`. Use your MedusaRust admin credentials.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `NEXT_PUBLIC_API_URL` | `http://localhost:9000` | MedusaRust API base URL |
| `NEXT_PUBLIC_BASE_PATH` | *(empty)* | Sub-path when served under a prefix (e.g. `/admin`) |
| `PORT` | `7001` | Port for `npm run dev` / `npm start` |

> **Note:** Variables prefixed with `NEXT_PUBLIC_` are embedded in the browser
> bundle at build time. Rebuild the app after changing them.

## Scripts

| Command | Description |
|---|---|
| `npm run dev` | Development server with hot reload |
| `npm run build` | Production build (outputs to `.next/`) |
| `npm start` | Start production server (requires `npm run build` first) |
| `npm run lint` | Run ESLint |
| `npm run type-check` | TypeScript type check (no emit) |

## Production Build (Docker)

The `docker-compose.yml` in the repository root includes an `admin_ui` service
that builds and serves this panel. Run:

```bash
docker compose up -d admin_ui
```

The panel is then available at [http://localhost:7001](http://localhost:7001).

## Pointing to a Different API

Change `NEXT_PUBLIC_API_URL` in `.env.local` (development) or in the Docker
service environment (production):

```dotenv
NEXT_PUBLIC_API_URL=https://api.example.com
```

The Next.js dev server proxies all `/api/*` requests to the API URL to avoid
CORS issues (see `next.config.js`). In production you should configure your
reverse proxy (nginx, Caddy, etc.) to route accordingly.

## Syncing with Upstream MedusaJS Admin

If you want to incorporate updates from the upstream MedusaJS admin package:

```bash
# Update the vendored MedusaJS repository
cd vendor/medusa_js
git pull origin main

# Copy the admin package into this directory
cd ../..
cp -r vendor/medusa_js/packages/admin/. frontend/admin/

# Review the diff and apply patches as needed
git diff frontend/admin/
```

Keep custom changes (API URL configuration, environment wiring) in a separate
patchset so merges stay clean.
