FROM rust:latest

# Set working directory
WORKDIR /phoxi_interface

# Copy project files
COPY . .

# Build dependencies first (to leverage caching)
RUN cargo build --release

# Run the Rust application
CMD ["cargo", "run", "--release"]