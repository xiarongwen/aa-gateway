# AI Gateway Hub - Multi-stage Dockerfile
# Stage 1: Build frontend
FROM node:20-alpine AS frontend-builder

WORKDIR /app/frontend

# Copy package files
COPY frontend/package*.json ./
RUN npm install

# Copy source and build
COPY frontend/ ./
RUN npm run build

# Stage 2: Build backend
FROM rust:1.75-alpine AS backend-builder

# Install required dependencies for building
RUN apk add --no-cache musl-dev openssl-dev sqlite-dev

WORKDIR /app/backend

# Copy Cargo files for caching
COPY backend/Cargo.toml ./
COPY backend/Cargo.lock* ./

# Copy source code
COPY backend/src ./src

# Build release binary
RUN cargo build --release

# Stage 3: Final runtime image
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    sqlite-libs \
    libgcc

WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /app/backend/target/release/ai-gateway /app/ai-gateway

# Copy frontend static files
COPY --from=frontend-builder /app/frontend/dist /app/static

# Create data directory
RUN mkdir -p /root/.local/share/ai-gateway

# Expose ports
EXPOSE 8080 9090

# Set environment variables
ENV PORT=8080
ENV ENABLE_PROXY=true
ENV PROXY_PORT=9090
ENV RUST_LOG=info
ENV STATIC_DIR=/app/static

# Run the application
CMD ["/app/ai-gateway"]
