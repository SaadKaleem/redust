# Title: Concurrency Strategy | ID: 03

# Status

**Pending** - Date of Decision: ...


# Context  

The TCP server needs to handle incoming client connections and process requests concurrently, otherwise we would not be able to scale our server to many clients.

Therefore we need to efficiently manage concurrency to ensure responsive communication with clients while effectively utilizing system resources.

---

# Decision 

To use the Tokio asynchronous runtime for managing concurrency in the TCP server because of the following reasons:

**Efficient Resource Utilization**: Tokio's event-driven architecture ensures that our server can efficiently manage a large number of concurrent connections without creating a one-to-one thread-to-connection ratio. This minimizes resource overhead and maximizes scalability. 

Moreover, multithreading has greater thread overhead due to each thread having its own stack, which can cause excessive memory utilization. Asynchronous approaches, on the other hand, typically involve fewer threads managed by a runtime, Tokio in our case, which can lead to more efficient memory usage.

**Fewer Context Switches**: Context switching between threads incurs a cost in terms of memory usage. When the OS switches between threads, it needs to save and restore the state of each thread, including its registers and stack. This context switching overhead can be more pronounced in multithreaded applications compared to asynchronous approaches.

---

# Consequences


**Limited CPU-Bound Parallelism**: For tasks, that may require high CPU time, an asynchronous approach would not be the choice. As tokio's strength lies in handling I/O-bound operations. 

For CPU-bound tasks that require true parallelism across multiple cores, we may need to explore hybrid approaches that combine Tokio with traditional threading or other techniques. 