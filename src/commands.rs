// Command validation and dispatch module.
//
// Add imports for duration handling, RESP frame types, and the shared store.

// Function reference: COMMANDS-001
// Function name: parse_command
// Input: a parsed RESP array of bulk-string arguments.
// Output: a typed command or command-specific error.
// Description: Validate command name, arity, options, and numeric arguments.
// Why it exists: Keep wire parsing separate from command grammar and semantics.

// Function reference: COMMANDS-002
// Function name: dispatch
// Input: typed command and shared store handle.
// Output: typed RESP response.
// Description: Execute PING, GET, SET, DEL, EXISTS, EXPIRE, or TTL and map the
// result to the proper response type.
// Why it exists: This is the narrow boundary between protocol and storage code.

// Function reference: COMMANDS-003
// Function name: parse_expiration
// Input: SET option tokens such as EX and its seconds argument.
// Output: optional duration or validation error.
// Description: Validate and convert expiration syntax before it reaches storage.
// Why it exists: Invalid, negative, incomplete, or overflowing TTL input should
// fail as a command error rather than corrupting store state.