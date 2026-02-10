# syntax=docker/dockerfile:1

# Stage 1: Dependencies
FROM oven/bun:1.1.38-slim AS deps

WORKDIR /app

# Copy workspace root files
COPY package.json bun.lock ./
COPY turbo.json ./

# Copy frontend package files
COPY apps/frontend/package.json ./apps/frontend/

# Install dependencies with caching
RUN --mount=type=cache,target=/root/.bun/install/cache \
    bun install --frozen-lockfile

# Stage 2: Builder
FROM oven/bun:1.1.38-slim AS builder

WORKDIR /app

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules
COPY --from=deps /app/apps/frontend/node_modules ./apps/frontend/node_modules

# Copy workspace and frontend source
COPY package.json bun.lock turbo.json ./
COPY apps/frontend ./apps/frontend

# Build the application
WORKDIR /app/apps/frontend
RUN bun run build

# Stage 3: Production
FROM oven/bun:1.1.38-slim AS runner

WORKDIR /app

ENV NODE_ENV=production

# Create non-root user
RUN groupadd -r nodejs && useradd -r -g nodejs nodejs

# Copy built application
COPY --from=builder --chown=nodejs:nodejs /app/apps/frontend/.output ./.output
COPY --from=builder --chown=nodejs:nodejs /app/apps/frontend/package.json ./package.json

USER nodejs

EXPOSE 3000

ENV PORT=3000
ENV HOST=0.0.0.0

CMD ["bun", "run", "start"]
