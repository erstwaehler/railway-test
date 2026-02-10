# syntax=docker/dockerfile:1

# Stage 1: Dependencies
FROM oven/bun:1.1.38-slim AS deps

WORKDIR /app

# Copy only frontend package.json
COPY apps/frontend/package.json ./package.json

# Install dependencies (standalone mode)
RUN bun install

# Stage 2: Builder
FROM oven/bun:1.1.38-slim AS builder

WORKDIR /app

ENV PATH="/app/node_modules/.bin:$PATH"

# Copy dependencies from deps stage
COPY --from=deps /app/node_modules ./node_modules

# Copy all frontend source files (includes package.json)
COPY apps/frontend ./

# Build the application
RUN bun x vite build

# Stage 3: Production
FROM oven/bun:1.1.38-slim AS runner

WORKDIR /app

ENV NODE_ENV=production

# Create non-root user
RUN groupadd -r nodejs && useradd -r -g nodejs nodejs

# Copy built application and package.json
COPY --from=builder --chown=nodejs:nodejs /app/.output ./.output
COPY --from=builder --chown=nodejs:nodejs /app/package.json ./package.json

USER nodejs

EXPOSE 3000

ENV PORT=3000
ENV HOST=0.0.0.0

CMD ["bun", "run", "start"]
