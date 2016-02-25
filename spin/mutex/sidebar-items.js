initSidebarItems({"struct":[["Mutex","This type provides MUTual EXclusion based on spinning.DescriptionThis structure behaves a lot like a normal Mutex. There are some differences:It may be used outside the runtime. A normal mutex will fail when used without the runtime, this will just lock When the runtime is present, it will call the deschedule function when appropriate No lock poisoning. When a fail occurs when the lock is held, no guarantees are made When calling rust functions from bare threads, such as C `pthread`s, this lock will be very helpful. In other cases however, you are encouraged to use the locks from the standard library.Simple exampleThread-safety example"],["MutexGuard","A guard to which the protected data can be accessedWhen the guard falls out of scope it will release the lock."]]});