## Project Description
This custom proxy-based load balancer will intelligently distribute incoming network traffic across multiple backend worker servers, based on various real-time performance data. This project will involve implementing load balancing algorithms, health monitoring, and performance optimization.

## Project Requirements
1. Load balancer structure
• Receive incoming requests: Use a popular framework like Axum or Hyper.
• Define a worker servers list: Maintain a list of backend worker servers and corresponding addresses.
• Reroute incoming requests: Distribute incoming requests to worker servers using different load balancing algorithms (e.g., round-robin, least connections).

2. Health checks
• Implement health check endpoints: Ensure worker servers have a health check
endpoint to report their status.
• Check health status using the load balancer: Periodically check the health of worker servers.

3. Load balancing algorithms
• Round-robin: Distribute requests evenly across all worker servers.
• Least connections: Route requests to the worker server with the least active connections.

4. Adaptive load balancing with decision engine
• Real-time data: Continuously collect performance data such as response times and concurrent connections.
• Decision engine: Analyze the accumulated data to make decisions about which load balancing algorithm to use.
• Adaptive algorithm switching: Dynamically switch between different load balancing algorithms based on the decisions made by the decision engine.
