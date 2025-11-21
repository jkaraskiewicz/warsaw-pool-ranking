FROM node:18-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --legacy-peer-deps

# Copy source code
COPY . .

# Build the Angular app for production
RUN npm run build

# Production stage - use nginx to serve
FROM nginx:alpine

# Copy built app from builder stage
COPY --from=builder /app/dist/warsaw-pool-rankings-frontend /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
