# Stage 1: Build stage
FROM golang:1.22-alpine AS builder

WORKDIR /app

# Copy all source files
COPY . .

# Run go mod tidy to resolve and download dependencies
RUN go mod tidy

# Compile static binary directly
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" -o amud-bin ./cmd/server

# Stage 2: Runtime stage
FROM alpine:latest

# Install ca-certificates
RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/amud-bin .

# Expose port 8000
EXPOSE 8000

# Set entrypoint to run the binary
ENTRYPOINT ["./amud-bin"]
