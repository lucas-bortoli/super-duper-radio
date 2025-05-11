# Stage 1: Build stage
FROM rust:latest AS build

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Install dependencies with a dummy source file (src/lib.rs)
RUN mkdir src && touch src/lib.rs
RUN cargo build --release
RUN rm -r src/

# Copy the rest of the project
COPY . .

# Build the application
RUN cargo build --release

#############################


# Stage 2: Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install ffmpeg and ffprobe
RUN apt-get update && \
    apt-get install -y ffmpeg && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/super-duper-radio .
COPY --from=build /app/src/ui src/ui

# Expose the port Rocket is using (default 8000)
EXPOSE 8000
ENV ROCKET_ADDRESS="0.0.0.0"
VOLUME ["/app/stations"]
CMD ["/app/super-duper-radio"]