export interface BuiltinEntry {
  module: string;
  name: string;
  arity: number;
  description: string;
}

function e(module: string, name: string, arity: number, description: string): BuiltinEntry {
  return { module, name, arity, description };
}

const ENTRIES: BuiltinEntry[] = [
  e("json", "parse", 1, "Parse a JSON string into a value"),
  e("json", "encode", 1, "Encode a value as a JSON string"),
  e("json", "encode_pretty", 1, "Encode a value as pretty-printed JSON"),

  e("ctx", "empty", 1, "Create an empty record"),
  e("ctx", "load", 1, "Load a record from a file"),
  e("ctx", "save", 2, "Save a record to a file"),
  e("ctx", "get", 2, "Get a field from a record"),
  e("ctx", "set", 3, "Set a field in a record"),
  e("ctx", "remove", 2, "Remove a field from a record"),
  e("ctx", "keys", 1, "Get all keys from a record"),
  e("ctx", "merge", 2, "Merge two records"),

  e("math", "abs", 1, "Absolute value"),
  e("math", "ceil", 1, "Round up to nearest integer"),
  e("math", "floor", 1, "Round down to nearest integer"),
  e("math", "round", 1, "Round to nearest integer"),
  e("math", "pow", 2, "Raise to a power"),
  e("math", "sqrt", 1, "Square root"),
  e("math", "min", 2, "Minimum of two values"),
  e("math", "max", 2, "Maximum of two values"),

  e("fs", "read", 1, "Read file contents as string"),
  e("fs", "write", 2, "Write string to file"),
  e("fs", "append", 2, "Append string to file"),
  e("fs", "exists", 1, "Check if file or directory exists"),
  e("fs", "remove", 1, "Delete a file or directory"),
  e("fs", "mkdir", 1, "Create a directory"),
  e("fs", "ls", 1, "List directory contents"),
  e("fs", "stat", 1, "Get file metadata"),

  e("env", "get", 1, "Get an environment variable"),
  e("env", "vars", 1, "Get all environment variables as a record"),
  e("env", "args", 1, "Get command-line arguments"),
  e("env", "cwd", 1, "Get current working directory"),
  e("env", "home", 1, "Get home directory path"),

  e("re", "match", 2, "Find first regex match"),
  e("re", "find_all", 2, "Find all regex matches"),
  e("re", "replace", 3, "Replace first regex match"),
  e("re", "replace_all", 3, "Replace all regex matches"),
  e("re", "split", 2, "Split string by regex pattern"),
  e("re", "is_match", 2, "Test if pattern matches"),

  e("time", "now", 1, "Current timestamp as record"),
  e("time", "sleep", 1, "Sleep for milliseconds"),
  e("time", "format", 2, "Format a timestamp"),
  e("time", "parse", 2, "Parse a time string"),

  e("http", "get", 1, "HTTP GET request"),
  e("http", "post", 2, "HTTP POST request"),
  e("http", "put", 2, "HTTP PUT request"),
  e("http", "delete", 1, "HTTP DELETE request"),
  e("http", "patch", 2, "HTTP PATCH request"),

  e("test", "assert", 1, "Assert a value is truthy"),
  e("test", "assert_eq", 2, "Assert two values are equal"),
  e("test", "assert_ne", 2, "Assert two values are not equal"),
  e("test", "assert_err", 1, "Assert a value is an error"),

  e("agent", "spawn", 1, "Spawn a new agent"),
  e("agent", "send", 2, "Send a message to an agent"),
  e("agent", "recv", 1, "Receive a message from an agent"),

  e("user", "ask", 1, "Prompt user for text input"),
  e("user", "confirm", 1, "Prompt user for yes/no confirmation"),
  e("user", "choose", 2, "Prompt user to choose from a list"),

  e("trace", "span", 2, "Create a trace span"),
  e("trace", "event", 1, "Emit a trace event"),

  e("diag", "extract", 1, "Extract diagram from lx source"),
  e("diag", "extract_file", 1, "Extract diagram from lx file"),
  e("diag", "to_mermaid", 1, "Render diagram as Mermaid syntax"),

  e("describe", "extract", 1, "Extract program description from source"),
  e("describe", "extract_file", 1, "Extract program description from file"),
  e("describe", "render", 1, "Render program description as text"),

  e("git", "status", 0, "Get git repository status"),
  e("git", "diff", 0, "Get git diff"),
  e("git", "log", 1, "Get git commit log"),

  e("md", "parse", 1, "Parse markdown into structured sections"),
  e("md", "render", 1, "Render structured sections as markdown"),

  e("ai", "chat", 1, "Send a chat message to an AI model"),
  e("ai", "complete", 1, "Generate a text completion"),
];

const GLOBAL_BUILTINS: BuiltinEntry[] = [
  e("global", "map", 2, "Transform each element of a collection"),
  e("global", "filter", 2, "Keep elements matching a predicate"),
  e("global", "fold", 3, "Reduce a collection to a single value"),
  e("global", "flat_map", 2, "Map and flatten results"),
  e("global", "each", 2, "Execute a side-effect for each element"),
  e("global", "take", 2, "Take first N elements"),
  e("global", "drop", 2, "Drop first N elements"),
  e("global", "zip", 2, "Pair elements from two collections"),
  e("global", "enumerate", 1, "Pair each element with its index"),
  e("global", "find", 2, "Find first element matching a predicate"),
  e("global", "sort_by", 2, "Sort by a key function"),
  e("global", "partition", 2, "Split into two lists by predicate"),
  e("global", "group_by", 2, "Group elements by a key function"),
  e("global", "chunks", 2, "Split into fixed-size chunks"),
  e("global", "windows", 2, "Sliding windows of given size"),
  e("global", "scan", 3, "Fold that yields intermediate values"),
  e("global", "tap", 2, "Side-effect passthrough"),
  e("global", "pmap", 2, "Parallel map across elements"),
  e("global", "pmap_n", 3, "Parallel map with concurrency limit"),
  e("global", "len", 1, "Length of a collection or string"),
  e("global", "head", 1, "First element of a list"),
  e("global", "tail", 1, "All elements except the first"),
  e("global", "last", 1, "Last element of a list"),
  e("global", "reverse", 1, "Reverse a list"),
  e("global", "contains?", 2, "Check if collection contains a value"),
  e("global", "empty?", 1, "Check if collection is empty"),
  e("global", "keys", 1, "Get keys from a record or map"),
  e("global", "values", 1, "Get values from a record or map"),
  e("global", "join", 2, "Join list elements with a separator"),
  e("global", "split", 2, "Split a string by separator"),
  e("global", "trim", 1, "Trim whitespace from a string"),
  e("global", "upper", 1, "Convert string to uppercase"),
  e("global", "lower", 1, "Convert string to lowercase"),
  e("global", "starts?", 2, "Check if string starts with prefix"),
  e("global", "ends?", 2, "Check if string ends with suffix"),
  e("global", "replace", 3, "Replace occurrences in a string"),
  e("global", "type_of", 1, "Get the type name of a value"),
  e("global", "to_str", 1, "Convert a value to string"),
  e("global", "to_int", 1, "Convert a value to integer"),
  e("global", "to_float", 1, "Convert a value to float"),
  e("global", "print", 1, "Print a value to stdout"),
  e("global", "println", 1, "Print a value to stdout with newline"),
  e("global", "debug", 1, "Print debug representation of a value"),
  e("global", "assert", 1, "Assert a value is truthy"),
];

export const BUILTINS: Map<string, BuiltinEntry[]> = new Map();
export const BUILTIN_FUNCTIONS: Map<string, BuiltinEntry> = new Map();
export const GLOBAL_FUNCTION_SET: Map<string, BuiltinEntry> = new Map();

for (const entry of ENTRIES) {
  const list = BUILTINS.get(entry.module) ?? [];
  list.push(entry);
  BUILTINS.set(entry.module, list);
  BUILTIN_FUNCTIONS.set(`${entry.module}.${entry.name}`, entry);
}

for (const entry of GLOBAL_BUILTINS) {
  const list = BUILTINS.get("global") ?? [];
  list.push(entry);
  BUILTINS.set("global", list);
  BUILTIN_FUNCTIONS.set(entry.name, entry);
  GLOBAL_FUNCTION_SET.set(entry.name, entry);
}
