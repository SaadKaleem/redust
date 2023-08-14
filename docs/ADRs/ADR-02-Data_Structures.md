# Title: Data Structures to be utilized by Commands | ID: 02

# Status

**Pending** - Date of Decision: ...


# Context  

We will be going for an in-memory data structure store, due to the following factors:

* **Access Speed**: Disk I/O involves much greater overhead due to the need to manage: read/write requests, tracking file pointers, and eventual disk fragmententation etc.
* **Concurrency**: Disk-based systems require much greater complex synchronization mechanisms to manage concurrent access, compared to in-memory stores.

Moreover, the underlying data structures need to be optimized for performance, but with the least complexity possible. This involves finding a balance between maximizing performance and minimizing the time required for development.

---

# Decision 

We have decided to use the following data structures for building our basic commands:

**Strings**: To store text or binary data. This will be based on the traditional Rust [`std:string::String`](https://doc.rust-lang.org/std/string/struct.String.html) type, which is implemented as a collection of characters using the [`UTF-8`](https://en.wikipedia.org/wiki/UTF-8) encoding. Dynamic allocation on the heap, and managed by the Rust's core allocator. 

**Lists**: To store contiguous sequence of elements. This will be based on [` std::collections::LinkedList`](https://doc.rust-lang.org/std/collections/struct.LinkedList.html) which has an underlying Doubly-linked list implementation, where each element stores two pointers (previous and next). Insertion and deletion operations at the beginning and end, are `O(1)` which is suffice for our `push` and `pop` commands. Inserting or deleting from the middle would not require shifting of the elements. 


**Hashes**: Maps of key value pairs. This will be based on [`  std::collections::HashMap`](https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html) which is based on the Google's [`SwissTable`](https://abseil.io/blog/20180927-swisstables) implementation. It utilizes the [`SipHash`](https://en.wikipedia.org/wiki/SipHash) algorithm which produces at 64-bit hash value. To resolve collisions, [`Quadratic Probing`](https://en.wikipedia.org/wiki/Quadratic_probing) has also been utilized.

---

# Consequences

Using these data structures for building the basic commands provides a solid foundation for implementing common use cases with optimal performance characteristics. However, it's important to consider the limitations and trade-offs of each data structure when designing the commands and the overall system architecture. 

For example, we could have used an underlying `B-tree` implementation for the `List` data structure, to optimize for search from `O(n)` to `O(log n)` - However, this would introduce certain complexities in terms of balancing of the tree, and a clear trade-off would be that insertions and deletions could be much slower in the worse-case. Furthermore, a [`Vec<T>`](https://doc.rust-lang.org/std/vec/struct.Vec.html) could also be considered, however insertion in the middle would require shifting elements, which in the worse case could be `O(n)` to maintain the contiguous memory layout. 

It may necessary that as the system evolves and new features are added, we would need to revisit the choice of data structures and adjust as needed to accommodate new use cases and performance considerations.