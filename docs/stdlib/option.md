# Option Module

The `Option` module provides functions for working with optional values. The Option type is a discriminated union that represents values that may or may not exist, providing a type-safe alternative to null values.

## Overview

The Option type is defined as:
```fsharp
type Option<'a> =
    | Some of 'a
    | None
```

Options are created using the constructors `Some(value)` and `None`. The Option module provides functions for safely working with these values without explicit pattern matching in simple cases.

## Constructors

### Some

Creates an Option containing a value.

**Signature:**
```fsharp
Some : 'a -> 'a option
```

**Parameters:**
- `value` - The value to wrap in Some

**Returns:**
- `Some(value)`

**Example:**
```fsharp
let someValue = Some(42)  // Some(42)
let someString = Some("hello")  // Some("hello")
```

---

### None

Creates an Option representing no value.

**Signature:**
```fsharp
None : 'a option
```

**Returns:**
- `None`

**Example:**
```fsharp
let noValue = None  // None
```

---

## Functions

### Option.isSome

Returns true if the option is Some.

**Signature:**
```fsharp
Option.isSome : 'a option -> bool
```

**Parameters:**
- `opt` - The option to check

**Returns:**
- `true` if the option is Some, `false` if None

**Example:**
```fsharp
let someValue = Some(42) in
let hasSome = Option.isSome someValue  // true

let noValue = None in
let hasNone = Option.isSome noValue  // false
```

**See also:**
- [Option.isNone](#optionisnone) - Check if option is None

---

### Option.isNone

Returns true if the option is None.

**Signature:**
```fsharp
Option.isNone : 'a option -> bool
```

**Parameters:**
- `opt` - The option to check

**Returns:**
- `true` if the option is None, `false` if Some

**Example:**
```fsharp
let someValue = Some(42) in
let isNone = Option.isNone someValue  // false

let noValue = None in
let isReallyNone = Option.isNone noValue  // true
```

**See also:**
- [Option.isSome](#optionissome) - Check if option is Some

---

### Option.defaultValue

Returns the value inside Some, or a default value if None.

**Signature:**
```fsharp
Option.defaultValue : 'a -> 'a option -> 'a
```

**Parameters:**
- `defaultValue` - The value to return if the option is None
- `opt` - The option to extract value from

**Returns:**
- The value inside Some, or the default value if None

**Example:**
```fsharp
let someValue = Some(42) in
let value = Option.defaultValue 0 someValue  // 42

let noValue = None in
let defaulted = Option.defaultValue 0 noValue  // 0

let maybeName = Some("Alice") in
let name = Option.defaultValue "Unknown" maybeName  // "Alice"
```

**Note:**
- The default value is always evaluated, even if not needed
- For lazy evaluation, use [Option.defaultWith](#optiondefaultwith)

**See also:**
- [Option.defaultWith](#optiondefaultwith) - Lazy default value

---

### Option.defaultWith

Returns the value inside Some, or calls a function to get a default value if None.

**Signature:**
```fsharp
Option.defaultWith : (unit -> 'a) -> 'a option -> 'a
```

**Parameters:**
- `defThunk` - A function that produces the default value (called only if needed)
- `opt` - The option to extract value from

**Returns:**
- The value inside Some, or the result of calling defThunk if None

**Example:**
```fsharp
let someValue = Some(42) in
let value = Option.defaultWith (fun () -> expensiveComputation()) someValue
// expensiveComputation() is NOT called, returns 42

let noValue = None in
let defaulted = Option.defaultWith (fun () -> 100) noValue
// Returns 100
```

**Note:**
- The function is only called if the option is None
- Use this for expensive default value computations

**See also:**
- [Option.defaultValue](#optiondefaultvalue) - Eager default value

---

### Option.map

Transforms the value inside Some with a function, returns None if the option is None.

**Signature:**
```fsharp
Option.map : ('a -> 'b) -> 'a option -> 'b option
```

**Parameters:**
- `mapper` - A function to apply to the value
- `opt` - The option to map over

**Returns:**
- `Some(mapper(value))` if the option is Some(value)
- `None` if the option is None

**Example:**
```fsharp
let someValue = Some(5) in
let doubled = Option.map (fun x -> x * 2) someValue  // Some(10)

let noValue = None in
let stillNone = Option.map (fun x -> x * 2) noValue  // None

let maybeName = Some("alice") in
let upperName = Option.map String.toUpper maybeName  // Some("ALICE")
```

**See also:**
- [Option.bind](#optionbind) - Monadic bind
- [List.map](list.md#listmap) - Map over lists

---

### Option.bind

Applies a function that returns an option to the value inside Some. Also known as flatMap or andThen.

**Signature:**
```fsharp
Option.bind : ('a -> 'b option) -> 'a option -> 'b option
```

**Parameters:**
- `binder` - A function that takes a value and returns an option
- `opt` - The option to bind over

**Returns:**
- The result of applying the binder function if the option is Some
- `None` if the option is None

**Example:**
```fsharp
let parseInt str =
    // Simplified - in reality would parse the string
    if str = "42" then Some(42) else None

let maybeStr = Some("42") in
let maybeInt = Option.bind parseInt maybeStr  // Some(42)

let badStr = Some("not a number") in
let noInt = Option.bind parseInt badStr  // None

let noStr = None in
let alsoNone = Option.bind parseInt noStr  // None
```

**Note:**
- Prevents nested options (Some(Some(value)))
- Essential for chaining optional operations

**See also:**
- [Option.map](#optionmap) - Simple transformation

---

### Option.iter

Calls a function with the value if Some, does nothing if None.

**Signature:**
```fsharp
Option.iter : ('a -> unit) -> 'a option -> unit
```

**Parameters:**
- `action` - A function to call with the value
- `opt` - The option to iterate over

**Returns:**
- `unit`

**Example:**
```fsharp
let someValue = Some(42) in
Option.iter (fun x -> printfn "Value: %d" x) someValue
// Prints: "Value: 42"

let noValue = None in
Option.iter (fun x -> printfn "Value: %d" x) noValue
// Prints nothing
```

**Note:**
- Used for side effects only
- Always returns unit

---

### Option.map2

Combines two options using a binary function.

**Signature:**
```fsharp
Option.map2 : ('a -> 'b -> 'c) -> 'a option -> 'b option -> 'c option
```

**Parameters:**
- `mapper` - A function that takes two values
- `opt1` - The first option
- `opt2` - The second option

**Returns:**
- `Some(mapper(value1, value2))` if both options are Some
- `None` if either option is None

**Example:**
```fsharp
let add x y = x + y in
let some1 = Some(3) in
let some2 = Some(4) in
let sum = Option.map2 add some1 some2  // Some(7)

let none1 = None in
let result = Option.map2 add none1 some2  // None

let concat x y = String.concat [x; " "; y] in
let firstName = Some("Alice") in
let lastName = Some("Smith") in
let fullName = Option.map2 concat firstName lastName  // Some("Alice Smith")
```

**Note:**
- Both options must be Some for the function to be called
- Useful for combining multiple optional values

---

### Option.orElse

Returns the first option if Some, otherwise returns the second option.

**Signature:**
```fsharp
Option.orElse : 'a option -> 'a option -> 'a option
```

**Parameters:**
- `opt1` - The primary option
- `opt2` - The fallback option

**Returns:**
- `opt1` if it is Some
- `opt2` otherwise

**Example:**
```fsharp
let primary = Some(42) in
let fallback = Some(99) in
let result = Option.orElse primary fallback  // Some(42)

let noPrimary = None in
let result2 = Option.orElse noPrimary fallback  // Some(99)

let bothNone = Option.orElse None None  // None
```

**Note:**
- Provides a way to chain fallback options
- Both arguments are always evaluated

---

## Usage Examples

### Safe Division

```fsharp
let safeDivide x y =
    if y = 0 then None
    else Some(x / y)

let result = safeDivide 10 2  // Some(5)
let error = safeDivide 10 0   // None

let withDefault = Option.defaultValue 0 (safeDivide 10 0)  // 0
```

### Chaining Optional Operations

```fsharp
let parseAndDouble str =
    str
    |> parseInt
    |> Option.map (fun x -> x * 2)

let result = parseAndDouble "21"  // Some(42)
let noResult = parseAndDouble "bad"  // None
```

### Working with Multiple Options

```fsharp
let createUser name age email =
    Option.map2
        (fun n a -> { name = n; age = a; email = email })
        name
        age

let maybeName = Some("Alice") in
let maybeAge = Some(30) in
let user = createUser maybeName maybeAge "alice@example.com"
// Some({ name = "Alice"; age = 30; email = "alice@example.com" })
```

### Fallback Chain

```fsharp
let getConfig key =
    let envVar = getEnvironmentVariable key in
    let configFile = readConfigFile key in
    let defaultVal = Some("default") in

    envVar
    |> Option.orElse configFile
    |> Option.orElse defaultVal
```

### Conditional Processing

```fsharp
let processIfValid input =
    if String.length input > 0
    then Some(String.toUpper input)
    else None

let result = processIfValid "hello"  // Some("HELLO")
let noResult = processIfValid ""     // None
```

### Extracting Optional Values

```fsharp
let getName user =
    user.name
    |> Option.defaultValue "Anonymous"

let getAge user =
    user.age
    |> Option.defaultWith (fun () -> calculateDefaultAge())
```

## Pattern Matching

While the Option module provides convenience functions, pattern matching is often the clearest way to work with options:

```fsharp
let describe opt =
    match opt with
    | Some(value) -> "Has value: " + toString value
    | None -> "No value"

let result = describe (Some(42))  // "Has value: 42"
```

## Type Safety

The Option type provides compile-time safety:
- No null reference errors
- Forces explicit handling of missing values
- Type checker ensures all cases are handled
- Generic type parameter maintains type safety

## Best Practices

1. **Use Option instead of null**: Options make absence explicit and type-safe
2. **Prefer map/bind over pattern matching**: For simple transformations
3. **Use defaultValue for simple cases**: When you have a constant default
4. **Use defaultWith for expensive defaults**: To avoid unnecessary computation
5. **Chain operations with bind**: To avoid nested pattern matching

## Performance Characteristics

| Function      | Time Complexity | Space Complexity |
|--------------|----------------|------------------|
| isSome       | O(1)           | O(1)             |
| isNone       | O(1)           | O(1)             |
| defaultValue | O(1)           | O(1)             |
| defaultWith  | O(1) + O(f)    | O(1)             |
| map          | O(f)           | O(1)             |
| bind         | O(f)           | O(1)             |
| iter         | O(f)           | O(1)             |
| map2         | O(f)           | O(1)             |
| orElse       | O(1)           | O(1)             |

Where:
- f = time complexity of the provided function

## Notes

- Option values are discriminated unions (variants)
- All Option operations are type-safe
- None can represent any option type
- Options can be nested (e.g., `Some(Some(42))`)
- Use bind to flatten nested options
- Options integrate seamlessly with pattern matching
