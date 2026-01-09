# Zeltra Frontend - Multi-stage Dockerfile
# Optimized Next.js production build

# ============================================================
# Stage 1: Dependencies
# ============================================================
FROM node:22-alpine AS deps

RUN corepack enable && corepack prepare pnpm@9 --activate

WORKDIR /app

# Copy package files
COPY package.json pnpm-lock.yaml ./

# Install dependencies
RUN pnpm install --frozen-lockfile

# ============================================================
# Stage 2: Build
# ============================================================
FROM node:22-alpine AS builder

RUN corepack enable && corepack prepare pnpm@9 --activate

WORKDIR /app

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules
COPY . .

# Set environment variables for build
ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_ENV=production

# Build the application
RUN pnpm build

# ============================================================
# Stage 3: Runtime
# ============================================================
FROM node:22-alpine AS runner

WORKDIR /app

# Set environment variables
ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

# Create non-root user
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 nextjs

# Copy built assets
COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000 || exit 1

# Start the application
CMD ["node", "server.js"]
