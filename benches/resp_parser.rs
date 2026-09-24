// Optional parser benchmark plan.
//
// Add Criterion imports and a benchmark registration only after the parser is
// correct and its unit tests pass.

// Function reference: BENCH-RESP-001
// Function name: benchmark_parser
// Input: Criterion benchmark context.
// Output: parser timing measurements.
// Description: Measure parsing cost separately from networking and store work.
// Why it exists: It helps identify protocol overhead without confusing it with
// lock contention or socket performance.