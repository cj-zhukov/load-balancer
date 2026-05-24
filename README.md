# Load Balancer

A custom proxy-based load balancer written in Rust that distributes incoming traffic across multiple backend worker servers using adaptive load balancing strategies and real-time performance metrics.

The project supports multiple balancing algorithms, worker health monitoring, and dynamic decision making based on runtime statistics.

## Architecture

```text
                +-------------------+
                |      Client       |
                +---------+---------+
                          |
                          v
                +-------------------+
                |   Load Balancer   |
                |-------------------|
                |  Decision Engine  |
                |  Health Checker   |
                |  Metrics Tracker  |
                |    DataFusion     |
                +---------+---------+
                          |
        -----------------------------------------
        |                   |                   |
        v                   v                   v
+---------------+  +---------------+  +---------------+
| Worker Server |  | Worker Server |  | Worker Server |
|      #1       |  |      #2       |  |      #n       |
+---------------+  +---------------+  +---------------+
```

## Project Description

This custom proxy-based load balancer intelligently distributes incoming network traffic across multiple backend worker servers using real-time performance data.

The project includes:
- Multiple load balancing algorithms
- Health monitoring
- Adaptive decision engine
- Runtime metrics collection
- Performance optimization

## Features

### Load Balancer Structure
- Receive incoming requests using Axum or Hyper
- Maintain a list of backend worker servers
- Reroute requests using different balancing algorithms

### Health Checks
- Worker health check endpoints
- Periodic worker status validation
- Automatic exclusion of unhealthy workers

### Load Balancing Algorithms
- Random
- Round-robin
- Least connections

### Adaptive Decision Engine
- Collect runtime metrics:
  - Response times
  - Active connections
  - Worker availability
- Dynamically switch balancing algorithms depending on current system state

## Data Layer

The project uses:
- DataFusion as a query engine for selecting active workers
- PostgreSQL or SQLite for persistent worker storage and metadata

## Worker Server

This project is designed to work together with a separate worker server implementation:

- https://github.com/cj-zhukov/worker-server
