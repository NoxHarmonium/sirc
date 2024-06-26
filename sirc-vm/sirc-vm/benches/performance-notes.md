# Performance Testing Notes

## Speed Improvement Log

### 29th May 2024

1. Tried inlining common shared instruction decoding functions - no change
2. Enable LTO - 30% improvement (makes sense due to how modular everything is) - thin LTO doesn't work as well for some reason
3. Tried rayon to parallelise the bus device processing (poll_all).- It had way too much overhead in a tight loop. The benchmark reported a +91897% time increase
4. Tried changing phase in CpuPeripheral::poll to a simple integer and pass integers around instead of enum references. It didn't affect performance but made it less readable. The optimiser is good!

Roughly 1.6220 ms -> 1.0487 ms = 35% speedup!

### 30th May 2024

1. Tried to rework the main loop so that the CPU could be polled at the same time as the other devices. I realised this can't work because the CPU is has to output an address for the devices for the devices to output data. In a real system everything would happen at once and then settle and then be latched at the next clock, but in a simulation on a turing machine I don't think this can be done efficiently.

### 31st May 2024

1. Did a refactor to get better reporting on simulation time vs expected time and got 4% performance improvement. I think because the bus polling loop was moved to the CPU module and so the hot path doesn't go over the crate boundary. It might mean that there will be less of a performance hit going to thin LTO now too

## Observations

### Byte Sieve

#### Instruments

```
cargo instruments --bench byte_sieve -t time --release -- --bench
```

- Only has two devices on the bus, the program ROM and the memory mapped scratch file
- 53% of CPU time in the CPU simulation and 47% in the bus devices
  - This would probably be more skewed in other examples with more devices
- Finding it hard to track down where all the CPU time in the bus is going, once I expand the `poll_all` node, about half the time goes missing.
- There is some missing time in the CPU code too. It kind of seems that it just does a lot of work. I think I might need a try a different profiler with a higher resolution.
- After enabling LTO the profile changed a lot. It seems like a lot of functions that were hot paths like the instruction decode functions got inlined
- 5% improvement in passing bus assertions in to the poll_all function instead of using a field on the struct to store it

#### Flame graph

- Doesn't seem to profile across crates, so it basically just tells me the CPU time is going to `poll` and `poll_all`

#### samply

```
samply record --rate 1997 target/release/deps/byte_sieve-976bfd0ddf1734b5 --bench
```

- Responsive UI, actually maps to the code properly so you can see hot spots and has both call tree and flame graph. I think this is probably the winner although you have to run the actual benchmark binary, rather than go via cargo
