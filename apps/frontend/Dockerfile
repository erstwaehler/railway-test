# syntax=docker/dockerfile:1

# Stage 1: Dependencies
FROM oven/bun:1.1.38-slim AS deps

WORKDIR /app/frontend

# Copy frontend package files only
COPY apps/frontend/package.json ./

# Install dependencies with caching
RUN --mount=type=cache,target=/root/.bun/install/cache \
    bun install --frozen-lockfile

# Stage 2: Builder
FROM oven/bun:1.1.38-slim AS builder

WORKDIR /app/frontend

# Copy dependencies from deps stage
COPY --from=deps /app/frontend/node_modules ./node_modules

# Copy frontend source
COPY apps/frontend ./

# Build the application
RUN bun run build

# Stage 3: Production
FROM oven/bun:1.1.38-slim AS runner

WORKDIR /app

ENV NODE_ENV=production

# Create non-root user
RUN groupadd -r nodejs && useradd -r -g nodejs nodejs

# Copy built application
COPY --from=builder --chown=nodejs:nodejs /app/frontend/.output ./.output
COPY --from=builder --chown=nodejs:nodejs /app/frontend/package.json ./package.json

USER nodejs

EXPOSE 3000

ENV PORT=3000
ENV HOST=0.0.0.0

CMD ["bun", "run", "start"]
