# kdtree-collisions

A Rust library to handle collisions between rectangles in a 2d space efficiently using a kind of KD-tree. 

The tree structure is mutable. The access, insertion and removal should be O(log(n)) on average, with reasonable constant overhead and cache-friendlyness, but it can grow to O(n) if the space only grows in one direction.

It is not super-optimised, but should be reasonably fast for accesses. Insertions and removals are a little less efficient.


I made this to replace Rapier's broad-phase collision detection in the "imapirate" project. The time used to do broad-phase collision detection went from "most of the time in the frame" to "negligible time" so I did not bother to optimize it further.
