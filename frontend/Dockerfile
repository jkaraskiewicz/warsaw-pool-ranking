FROM node:18-alpine AS builder

WORKDIR /app

# Install protobuf compiler
RUN apk add --no-cache protobuf

# Copy package files
COPY frontend/package*.json ./

# Install dependencies
RUN npm install --legacy-peer-deps

# Copy source code
COPY frontend/ .

# Copy proto definitions
COPY proto /proto

# Generate API models from Protobuf
RUN npm run proto:generate

# Build the Angular app for production
RUN npm run build

# Production stage - use nginx to serve
FROM nginx:alpine

# Copy built app from builder stage
COPY --from=builder /app/dist/warsaw-pool-rankings-frontend /usr/share/nginx/html

# Copy nginx configuration
COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
