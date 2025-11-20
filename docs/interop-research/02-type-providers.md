# F# Type Providers: Compile-Time Metaprogramming for FSRS

**Research Date**: 2025-11-19
**Status**: Exploration - Type Provider Feasibility for FSRS Scripting
**Target**: Understanding how F# Type Providers could enhance FSRS if scripts remain valid F#

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [What Are Type Providers](#what-are-type-providers)
3. [Architecture and Mechanics](#architecture-and-mechanics)
4. [Popular Type Providers](#popular-type-providers)
5. [Creating Custom Type Providers](#creating-custom-type-providers)
6. [Design-Time Integration](#design-time-integration)
7. [Real-World Use Cases](#real-world-use-cases)
8. [Type Providers vs Code Generation](#type-providers-vs-code-generation)
9. [Limitations and Considerations](#limitations-and-considerations)
10. [FSRS Integration Scenarios](#fsrs-integration-scenarios)
11. [Implementation Roadmap for FSRS](#implementation-roadmap-for-fsrs)
12. [References](#references)

---

## Executive Summary

**Type Providers** are F#'s compile-time metaprogramming mechanism for creating strongly-typed access to external data sources without manual type definitions. They represent a powerful capability that could significantly enhance FSRS if the runtime maintains F# compatibility.

### Key Insights

- **Compile-Time Magic**: Type providers generate types during compilation based on external schemas (JSON, CSV, SQL, APIs, etc.)
- **Zero Boilerplate**: Eliminate manual type definitions for external data sources
- **IntelliSense Support**: Full IDE integration with autocomplete and type checking
- **Erased vs Generative**: Two implementation strategies with different tradeoffs
- **FSRS Opportunity**: If FSRS scripts remain valid F#, type providers could provide seamless host interop

### Critical Questions for FSRS

1. **F# Compatibility**: Will FSRS maintain full F# language compatibility?
2. **Compilation Pipeline**: Does FSRS compile through the F# compiler or custom bytecode compiler?
3. **Design-Time Tooling**: Will FSRS support IDE tooling for script development?
4. **Host Interop Model**: How will FSRS scripts access host application data/APIs?

**Decision Point**: Type providers only make sense if FSRS maintains F# compiler compatibility. If FSRS uses a custom compiler (more likely given bytecode VM goals), type providers won't be available, but the *patterns* they enable are still valuable for FSRS host interop API design.

---

## What Are Type Providers

### Definition

From Microsoft's documentation:

> "An F# type provider is a component that provides types, properties, and methods for use in your program."

Type providers are **compile-time code generators** that create .NET types dynamically based on external data schemas. Unlike traditional code generation tools (like T4 templates), type providers integrate directly into the F# compilation process.

### Core Concept

```fsharp
// Traditional approach: Manual type definitions
type Person = { Name: string; Age: int }
let json = """{"name":"Alice","age":30}"""
let person = JsonConvert.DeserializeObject<Person>(json)

// Type Provider approach: Types inferred from sample
type People = JsonProvider<"""{"name":"Alice","age":30}""">
let person = People.Parse(json)
// person.Name and person.Age are strongly typed!
```

The type provider **analyzes the sample** at compile time and generates strongly-typed properties automatically.

### Key Characteristics

1. **Compile-Time Execution**: Type providers run during compilation, not runtime
2. **Schema Inference**: Analyze external data sources to determine type structures
3. **On-Demand Expansion**: Types are only fully generated when referenced
4. **IDE Integration**: Provide IntelliSense, tooltips, and compile-time errors
5. **Parameterized**: Accept static parameters (file paths, connection strings, URLs)

---

## Architecture and Mechanics

### Two Provider Categories

#### 1. Generative Type Providers

**Characteristics**:
- Generate **real .NET types** that persist in compiled assemblies
- Types can be consumed by other assemblies and languages (C#, VB.NET)
- Suitable for stable, well-defined schemas
- Example: SwaggerProvider generates actual classes for API models

**Tradeoffs**:
- Limited by .NET type system constraints
- Can increase assembly size
- Better for cross-language interop

#### 2. Erasing Type Providers

**Characteristics**:
- Generate **virtual types** that exist only at compile time
- Types are "erased" to base types (typically `obj`) at runtime
- Enable working with massive type spaces (thousands of types)
- Most common type provider pattern

**Tradeoffs**:
- Types only available in F# projects that reference the provider
- Cannot use reflection to inspect provided types at runtime
- More flexible for large/dynamic schemas

**Example**: FSharp.Data providers (JSON, CSV, XML) use type erasure

### Architecture Components

Type providers consist of two separate components:

```
┌─────────────────────────────────────────────────────┐
│  Type Provider Package                              │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌────────────────────────────────────────┐        │
│  │  Runtime Component (TPRTC)              │        │
│  │  - Referenced by application            │        │
│  │  - Contains helper functions            │        │
│  │  - Targets netstandard2.0+              │        │
│  │  - Example: FSharp.Data.dll             │        │
│  └────────────────────────────────────────┘        │
│                                                      │
│  ┌────────────────────────────────────────┐        │
│  │  Design-Time Component (TPDTC)          │        │
│  │  - Loaded by IDE/compiler               │        │
│  │  - Generates types at compile time      │        │
│  │  - Example: FSharp.Data.DesignTime.dll  │        │
│  └────────────────────────────────────────┘        │
│                                                      │
└─────────────────────────────────────────────────────┘
```

### How Type Providers Execute

```fsharp
// 1. Developer writes code with type provider
[<Literal>]
let sampleJson = """{"name":"Alice","age":30}"""
type Person = JsonProvider<sampleJson>

// 2. At compile time:
//    - F# compiler loads TPDTC
//    - TPDTC parses sample JSON
//    - TPDTC generates type definitions
//    - Compiler integrates generated types

// 3. IDE tooling:
//    - IntelliSense queries TPDTC for member lists
//    - Type checking validates against generated schema
//    - Go-to-definition navigates to virtual types

// 4. At runtime:
//    - Only TPRTC is loaded
//    - Generated types are erased to base types
//    - Helper functions parse actual data
```

### Type Provider SDK

Custom type providers are built using the **FSharp.TypeProviders.SDK**:

```fsharp
open ProviderImplementation.ProvidedTypes

[<TypeProvider>]
type MyProvider(config: TypeProviderConfig) as this =
    inherit TypeProviderForNamespaces(config)

    let ns = "MyNamespace"
    let asm = Assembly.GetExecutingAssembly()

    // Create provided type
    let myType = ProvidedTypeDefinition(asm, ns, "MyType", Some(typeof<obj>))

    // Add properties
    let prop = ProvidedProperty("Name", typeof<string>,
        getterCode = fun args -> <@@ "Hello" @@>)
    myType.AddMember(prop)

    // Register type
    this.AddNamespace(ns, [myType])
```

The SDK provides:
- `ProvidedTypeDefinition`: Create custom types
- `ProvidedProperty`: Add properties with getter/setter code
- `ProvidedMethod`: Add methods with implementation
- `ProvidedConstructor`: Define constructors
- F# Quotations: Express generated code

---

## Popular Type Providers

### FSharp.Data - The Foundation

The most widely-used type provider library, providing access to common data formats.

#### JSON Type Provider

**Basic Usage**:
```fsharp
open FSharp.Data

type Person = JsonProvider<"""{"name":"Alice","age":30}""">
let data = Person.Parse("""{"name":"Bob","age":25}""")
printfn "%s is %d years old" data.Name data.Age
```

**Type Inference**:
```fsharp
// Handles optional fields
type People = JsonProvider<"""
  [{"name":"Alice","age":30},
   {"name":"Bob"}]
""">

for person in People.GetSamples() do
    printf "%s" person.Name
    person.Age |> Option.iter (printf " (%d)")
    printfn ""
```

**Real-World API Integration**:
```fsharp
// GitHub Issues
type GitHub = JsonProvider<"https://api.github.com/repos/fsharp/fsharp/issues">

let issues = GitHub.GetSamples()
for issue in issues |> Seq.filter (fun i -> i.State = "open") do
    printfn "#%d: %s" issue.Number issue.Title
```

**Creating JSON**:
```fsharp
type NewIssue = JsonProvider<"""
{
  "title": "Bug report",
  "body": "Description",
  "labels": ["bug", "help wanted"]
}
""">

let issue = NewIssue.Root("Fix parser", "Parser fails on edge case", [|"bug"|])
issue.JsonValue.Request("https://api.github.com/repos/owner/repo/issues")
```

#### CSV Type Provider

**Schema Inference**:
```fsharp
type Stocks = CsvProvider<"data/MSFT.csv">
let msft = Stocks.Load("data/MSFT.csv")

for row in msft.Rows do
    printfn "%s: Open=%M Close=%M"
        (row.Date.ToShortDateString()) row.Open row.Close
```

**Units of Measure**:
```fsharp
open FSharp.Data.UnitSystems.SI.UnitNames

type Measurements = CsvProvider<"data/measurements.csv">
// CSV header: Name,Distance (metre),Time (second)

let data = Measurements.GetSample()
for row in data.Rows do
    let speed = row.Distance / row.Time  // Type: float<metre/second>
    printfn "%s: %.2f m/s" row.Name speed
```

**Custom Schema**:
```fsharp
// Override inferred types
type Titanic = CsvProvider<
    "data/Titanic.csv",
    Schema="Fare=float,PClass->PassengerClass">

let titanic = Titanic.GetSample()
for passenger in titanic.Rows do
    printfn "%s - Class %d - Fare $%.2f"
        passenger.Name passenger.PassengerClass passenger.Fare
```

#### XML Type Provider

```fsharp
type Rss = XmlProvider<"https://blog.fsprojects.net/rss.xml">

let feed = Rss.GetSample()
printfn "Feed: %s" feed.Channel.Title

for item in feed.Channel.Items do
    printfn "- %s (%s)" item.Title item.PubDate
```

#### HTML Type Provider

```fsharp
type WorldBank = HtmlProvider<"https://data.worldbank.org/country">

let data = WorldBank.GetSample()
for row in data.Tables.``Country List``.Rows do
    printfn "%s (%s)" row.Country row.Region
```

### SQL Type Providers

#### SQLProvider (Multi-Database)

Supports: SQL Server, SQLite, PostgreSQL, Oracle, MySQL, MS Access, Firebird, DuckDB

```fsharp
open FSharp.Data.Sql

[<Literal>]
let connStr = "Data Source=northwind.db;Version=3"

type Sql = SqlDataProvider<
    ConnectionString = connStr,
    DatabaseVendor = Common.DatabaseProviderTypes.SQLITE,
    UseOptionTypes = Common.NullableColumnType.OPTION>

let ctx = Sql.GetDataContext()

// LINQ queries with compile-time checking
query {
    for customer in ctx.Main.Customers do
    where (customer.ContactName =% "Matti%")
    sortBy customer.CompanyName
    select (customer.ContactName, customer.City)
} |> Seq.iter (printfn "%A")

// Relationship navigation (inferred from foreign keys)
let customer = ctx.Main.Customers.Individuals.``As ContactName``.``BERGS, Christina Berglund``
for order in customer.``main.Orders by CustomerID`` do
    printfn "Order %d: %s" order.OrderId order.ShipAddress.Value

// CRUD operations
let newCustomer = ctx.Main.Customers.Create()
newCustomer.CustomerId <- "NEWCO"
newCustomer.CompanyName <- "New Company"
ctx.SubmitUpdates()
```

#### FSharp.Data.SqlClient (SQL Server Specialized)

Three type providers for SQL Server:

1. **SqlCommandProvider**: Execute T-SQL with compile-time validation
2. **SqlProgrammabilityProvider**: Access stored procedures and functions
3. **SqlEnumProvider**: Generate enums from reference data

```fsharp
open FSharp.Data

[<Literal>]
let connStr = "Server=localhost;Database=AdventureWorks;Trusted_Connection=True"

// Compile-time T-SQL checking
type GetCustomers = SqlCommandProvider<"
    SELECT CustomerID, FirstName, LastName, EmailAddress
    FROM Sales.Customer c
    INNER JOIN Person.Person p ON c.PersonID = p.BusinessEntityID
    WHERE LastName LIKE @pattern
", connStr>

let cmd = new GetCustomers()
let customers = cmd.Execute(pattern = "Sm%")
for c in customers do
    printfn "%s %s <%s>" c.FirstName c.LastName c.EmailAddress

// Stored procedure access
type Procs = SqlProgrammabilityProvider<connStr>
let result = Procs.dbo.uspGetEmployeeManagers(employeeId = 42)
```

### GraphQL Type Providers

#### FSharp.Data.GraphQL

```fsharp
open FSharp.Data.GraphQL

type GitHub = GraphQLProvider<"https://api.github.com/graphql">

let query = GitHub.Operation<"""
query($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    name
    stargazerCount
    issues(first: 10, states: OPEN) {
      nodes {
        title
        createdAt
      }
    }
  }
}
""">()

let result = query.Run(owner = "fsharp", name = "fsharp", token = apiToken)
printfn "%s has %d stars" result.Repository.Name result.Repository.StargazerCount

for issue in result.Repository.Issues.Nodes do
    printfn "- %s (%s)" issue.Title (issue.CreatedAt.ToShortDateString())
```

Note: For Fable/client-side applications, use **Snowflaqe** instead, which generates F# projects compatible with Fable 2.0+.

### Configuration Type Providers

#### FSharp.Configuration (YAML, INI, AppSettings)

```fsharp
open FSharp.Configuration

[<Literal>]
let configFile = __SOURCE_DIRECTORY__ + "/config.yaml"

type Config = YamlConfig<configFile>

let config = Config()
config.Load(configFile)

printfn "Database: %s" config.Database.Host
printfn "Port: %d" config.Database.Port
printfn "API Key: %s" config.Api.Key

// Can modify and save (if ReadOnly = false)
config.Api.Key <- "new-api-key"
config.Save(configFile)
```

Example YAML structure:
```yaml
database:
  host: localhost
  port: 5432
  name: mydb
api:
  key: secret-key-123
  endpoint: https://api.example.com
```

### Specialized Type Providers

- **WorldBankProvider**: Access World Bank economic data
- **Azure Storage Provider**: Blob, Table, Queue resources
- **SwaggerProvider**: Generate clients from OpenAPI/Swagger specs
- **R Provider**: Interop with R language statistical packages
- **Regex Provider**: Compile-time regex validation with named groups

---

## Creating Custom Type Providers

### When to Build Custom Providers

From Microsoft's guidance:

> "The type provider mechanism is primarily designed for injecting stable data and service information spaces into the F# programming experience. You should use this mechanism only where necessary and where the development of a type provider yields very high value."

**Good Use Cases**:
- Domain-specific data sources with stable schemas
- Proprietary APIs/databases used across many projects
- Complex configuration formats
- Industry-standard protocols (MQTT, Protobuf, etc.)

**Poor Use Cases**:
- Frequently changing schemas
- One-off integrations (use existing providers)
- Data sources without compile-time schema availability

### Implementation Patterns

#### 1. Simple Erased Type Provider

```fsharp
open ProviderImplementation.ProvidedTypes
open Microsoft.FSharp.Core.CompilerServices

[<TypeProvider>]
type HelloWorldTypeProvider(config: TypeProviderConfig) as this =
    inherit TypeProviderForNamespaces(config)

    let ns = "Sample.HelloWorld"
    let asm = Assembly.GetExecutingAssembly()

    // Create 100 types
    let types = [
        for i in 1 .. 100 do
            let ty = ProvidedTypeDefinition(
                asm, ns, sprintf "Type%d" i, Some(typeof<obj>))

            // Add a property
            let prop = ProvidedProperty(
                "StaticProperty", typeof<string>,
                isStatic = true,
                getterCode = fun args -> <@@ sprintf "Value from Type%d" i @@>)

            ty.AddMember(prop)
            yield ty
    ]

    this.AddNamespace(ns, types)

[<assembly:TypeProviderAssembly>]
do ()
```

Usage:
```fsharp
open Sample.HelloWorld

let value = Type42.StaticProperty  // "Value from Type42"
```

#### 2. Parameterized Type Provider (Regex Example)

```fsharp
[<TypeProvider>]
type RegexTypedProvider(config: TypeProviderConfig) as this =
    inherit TypeProviderForNamespaces(config)

    let ns = "Samples.FSharp.RegexTypeProvider"
    let asm = Assembly.GetExecutingAssembly()

    let createRegexType typeName pattern =
        let ty = ProvidedTypeDefinition(asm, ns, typeName, Some(typeof<obj>))

        // Validate pattern at compile time
        let regex =
            try Regex(pattern)
            with ex -> failwithf "Invalid regex pattern: %s" ex.Message

        // Add IsMatch method
        let isMatch = ProvidedMethod(
            "IsMatch",
            [ProvidedParameter("input", typeof<string>)],
            typeof<bool>,
            isStatic = true,
            invokeCode = fun args ->
                <@@ Regex.IsMatch(%%args.[0], pattern) @@>)

        ty.AddMember(isMatch)

        // Add named groups as properties
        for group in regex.GetGroupNames() do
            if group <> "0" then  // Skip whole match group
                let prop = ProvidedMethod(
                    sprintf "Get%s" group,
                    [ProvidedParameter("input", typeof<string>)],
                    typeof<string option>,
                    isStatic = true,
                    invokeCode = fun args ->
                        <@@
                            let m = Regex.Match(%%args.[0], pattern)
                            if m.Success then Some m.Groups.[group].Value
                            else None
                        @@>)
                ty.AddMember(prop)

        ty

    // Static parameter: the regex pattern
    let staticParams = [ProvidedStaticParameter("pattern", typeof<string>)]

    let regexTy = ProvidedTypeDefinition(asm, ns, "RegexTyped", Some(typeof<obj>))

    regexTy.DefineStaticParameters(
        staticParams,
        fun typeName args ->
            createRegexType typeName (args.[0] :?> string))

    this.AddNamespace(ns, [regexTy])
```

Usage:
```fsharp
type Email = RegexTyped<"(?<user>[^@]+)@(?<domain>.+)">

match Email.IsMatch("alice@example.com") with
| true ->
    let user = Email.GetUser("alice@example.com")    // Some "alice"
    let domain = Email.GetDomain("alice@example.com") // Some "example.com"
    printfn "%A @ %A" user domain
| false -> printfn "Invalid email"
```

#### 3. Data-Backed Type Provider (Mini CSV)

```fsharp
[<TypeProvider>]
type MiniCsvProvider(config: TypeProviderConfig) as this =
    inherit TypeProviderForNamespaces(config)

    let createCsvType typeName (filename: string) =
        // Resolve file path
        let resolvedFilename = Path.Combine(config.ResolutionFolder, filename)

        // Read header
        let firstLine = File.ReadLines(resolvedFilename) |> Seq.head
        let headers = firstLine.Split(',')

        let ty = ProvidedTypeDefinition(asm, ns, typeName, Some(typeof<obj>))

        // Add constructor
        let ctor = ProvidedConstructor(
            [ProvidedParameter("filename", typeof<string>)],
            invokeCode = fun args -> <@@ %%args.[0] : string @@>)
        ty.AddMember(ctor)

        // Add Rows property
        let rowType = ProvidedTypeDefinition("Row", Some(typeof<obj>))

        // Add property for each column
        for header in headers do
            let prop = ProvidedProperty(
                header.Trim(),
                typeof<string>,
                getterCode = fun args ->
                    <@@ (%%args.[0] : obj) :?> string[] |> Array.item i @@>)
            rowType.AddMember(prop)

        ty.AddMember(rowType)

        let rowsProp = ProvidedProperty(
            "Rows",
            typedefof<seq<_>>.MakeGenericType(rowType),
            getterCode = fun args ->
                <@@
                    let filename = %%args.[0] : string
                    File.ReadLines(filename)
                    |> Seq.skip 1
                    |> Seq.map (fun line -> box (line.Split(',')))
                @@>)

        ty.AddMember(rowsProp)
        ty

    // Static parameter: filename
    let staticParams = [ProvidedStaticParameter("filename", typeof<string>)]

    let csvTy = ProvidedTypeDefinition(asm, ns, "MiniCsv", Some(typeof<obj>))
    csvTy.DefineStaticParameters(
        staticParams,
        fun typeName args ->
            createCsvType typeName (args.[0] :?> string))

    this.AddNamespace(ns, [csvTy])
```

Usage:
```fsharp
type Sales = MiniCsv<"sales.csv">  // CSV: Date,Product,Amount

let data = Sales("sales.csv")
for row in data.Rows do
    printfn "%s: %s sold for $%s" row.Date row.Product row.Amount
```

### Best Practices

1. **Validate Early**: Throw exceptions during type construction for invalid parameters
2. **Defer Computation**: Use `AddMembersDelayed` for large type spaces
3. **Add Documentation**: Use `AddXmlDocDelayed` for on-demand docs
4. **Hide Object Methods**: Set `hideObjectMethods = true` for cleaner IntelliSense
5. **Cache Schema**: Store schema information to enable offline development
6. **Version Appropriately**: Use `AddObsoleteAttribute` for deprecated members

---

## Design-Time Integration

### How IDEs Interact with Type Providers

Type providers integrate seamlessly with F# development tools:

#### IntelliSense Support

```fsharp
type GitHub = JsonProvider<"https://api.github.com/repos/fsharp/fsharp/issues">

let issues = GitHub.GetSamples()
for issue in issues do
    // IntelliSense shows:
    //   - issue.Number : int
    //   - issue.Title : string
    //   - issue.State : string
    //   - issue.CreatedAt : DateTime
    //   - issue.User : User
    //   - ...
    printfn "%d: %s" issue.Number issue.Title
```

The IDE queries the TPDTC at design time to provide:
- Property and method lists
- Type information
- Parameter hints
- Documentation tooltips

#### Compile-Time Error Checking

```fsharp
type Person = JsonProvider<"""{"name":"Alice","age":30}""">

let data = Person.Parse("""{"name":"Bob"}""")  // OK - age is optional
printfn "%s" data.Name  // OK
printfn "%d" data.Age.Value  // COMPILE ERROR if age is missing

// Type mismatch caught at compile time:
let badData = Person.Parse("""{"name":123,"age":"thirty"}""")  // Compiles
printfn "%s" badData.Name  // Runtime: "123" (coerced to string)
```

#### Go-to-Definition

Type providers can use `AddDefinitionLocation` to enable navigation:

```fsharp
// Navigate to original JSON schema file
type Config = JsonProvider<"config.json">
//                          ^--- Right-click -> Go to Definition opens config.json
```

### Design-Time vs Runtime Split

```
┌─────────────────────────────────────────────────────────┐
│  Development Time (IDE)                                 │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Developer writes code                                  │
│           ↓                                             │
│  IDE loads TPDTC (FSharp.Data.DesignTime.dll)          │
│           ↓                                             │
│  TPDTC fetches schema (HTTP, file, database)           │
│           ↓                                             │
│  TPDTC generates virtual types                         │
│           ↓                                             │
│  IDE provides IntelliSense from generated types         │
│                                                          │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  Compile Time                                            │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  F# compiler invokes TPDTC                              │
│           ↓                                             │
│  TPDTC provides type definitions                        │
│           ↓                                             │
│  Compiler type-checks code against provided types       │
│           ↓                                             │
│  Compiler generates IL using erased/generated types     │
│                                                          │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  Runtime                                                 │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Application loads TPRTC (FSharp.Data.dll)              │
│           ↓                                             │
│  TPRTC parsing/helper functions execute                 │
│           ↓                                             │
│  Actual data is parsed at runtime                       │
│           ↓                                             │
│  Provided types are erased to base types (obj, etc.)    │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Caching and Invalidation

Type providers can implement caching for performance:

```fsharp
// Cache schema information
let schemaCache = Dictionary<string, Schema>()

let getSchema (url: string) =
    match schemaCache.TryGetValue(url) with
    | true, schema -> schema
    | false, _ ->
        let schema = Http.RequestString(url) |> parseSchema
        schemaCache.[url] <- schema
        schema

// Invalidation when source changes
let watchFileChanges (path: string) =
    let watcher = new FileSystemWatcher(Path.GetDirectoryName(path))
    watcher.Changed.Add(fun _ ->
        schemaCache.Remove(path) |> ignore
        // Trigger recompilation
        this.Invalidate())
    watcher.EnableRaisingEvents <- true
```

---

## Real-World Use Cases

### 1. Data Science and Analytics

Type providers excel at exploratory data analysis:

```fsharp
// World Bank economic data
type WorldBank = WorldBankDataProvider<"v2">

let countries = WorldBank.GetDataContext()

let gdp =
    [ for c in countries.Countries do
        let data = c.Indicators.``GDP (current US$)``
        yield c.Name, data.[2020] ]
    |> List.sortByDescending snd
    |> List.truncate 10

for (country, value) in gdp do
    printfn "%s: $%.2fT" country (value / 1e12)
```

### 2. Database Access

Compile-time SQL validation prevents runtime errors:

```fsharp
type GetOrders = SqlCommandProvider<"
    SELECT o.OrderID, o.OrderDate, c.CompanyName, od.UnitPrice * od.Quantity AS Total
    FROM Orders o
    INNER JOIN Customers c ON o.CustomerID = c.CustomerID
    INNER JOIN OrderDetails od ON o.OrderID = od.OrderID
    WHERE o.OrderDate >= @startDate AND o.OrderDate <= @endDate
    ORDER BY o.OrderDate DESC
", ConnectionString>

let cmd = new GetOrders()
let orders = cmd.Execute(
    startDate = DateTime(2020, 1, 1),
    endDate = DateTime(2020, 12, 31))

let totalRevenue = orders |> Seq.sumBy (fun o -> o.Total)
printfn "2020 Revenue: $%.2f" totalRevenue
```

Compile-time checks:
- Table and column names validated
- Join conditions type-checked
- Parameter types enforced
- SQL syntax errors caught

### 3. API Integration

REST APIs become strongly typed:

```fsharp
// GitHub API
type GitHub = JsonProvider<"https://api.github.com/users/octocat">

let getUser username =
    let url = sprintf "https://api.github.com/users/%s" username
    GitHub.Load(url)

let user = getUser "octocat"
printfn "Name: %s" user.Name
printfn "Followers: %d" user.Followers
printfn "Public Repos: %d" user.PublicRepos
```

### 4. Configuration Management

YAML configs become type-safe:

```fsharp
type Config = YamlConfig<"appsettings.yaml">

let config = Config()
config.Load("appsettings.yaml")

// Build connection string from config
let connStr =
    sprintf "Server=%s;Port=%d;Database=%s;User=%s;Password=%s"
        config.Database.Host
        config.Database.Port
        config.Database.Name
        config.Database.User
        config.Database.Password

// Type-safe feature flags
if config.Features.EnableCaching then
    initializeCache config.Cache.SizeLimit config.Cache.ExpirationMinutes
```

### 5. ETL Pipelines

Data transformation with type safety:

```fsharp
// Source: CSV file
type Sales = CsvProvider<"sales.csv">

// Transform and validate
let validSales =
    Sales.Load("sales.csv").Rows
    |> Seq.filter (fun row ->
        row.Amount > 0.0M &&
        not (String.IsNullOrWhiteSpace(row.Product)))
    |> Seq.map (fun row ->
        {| Date = row.Date
           Product = row.Product.Trim()
           Amount = row.Amount
           Region = row.Region.ToUpper() |})

// Target: Database
type InsertSale = SqlCommandProvider<"
    INSERT INTO Sales (SaleDate, Product, Amount, Region)
    VALUES (@date, @product, @amount, @region)
", ConnectionString>

let cmd = new InsertSale()
for sale in validSales do
    cmd.Execute(
        date = sale.Date,
        product = sale.Product,
        amount = sale.Amount,
        region = sale.Region) |> ignore
```

### 6. Continuous Integration Challenges

Type providers can complicate CI/CD:

**Problem**: Type providers fetch schemas at compile time, which may not be available on build servers.

**Solutions**:

1. **Local Schema Caching**:
```fsharp
type GitHub = JsonProvider<
    "schema.json",
    SampleIsList=true,
    RootName="Issue">
// Check schema.json into source control
```

2. **Build-Time Schema Generation**:
```bash
# Before build, fetch schema
curl https://api.github.com/repos/owner/repo/issues > schema.json
dotnet build
```

3. **Conditional Compilation**:
```fsharp
#if DEBUG
type Config = YamlConfig<"config.dev.yaml">
#else
type Config = YamlConfig<"config.prod.yaml">
#endif
```

### 7. Testing with Type Providers

Unit testing type provider-based code:

```fsharp
[<Fact>]
let ``Parse valid JSON`` () =
    type Person = JsonProvider<"""{"name":"Alice","age":30}""">

    let json = """{"name":"Bob","age":25}"""
    let person = Person.Parse(json)

    Assert.Equal("Bob", person.Name)
    Assert.Equal(25, person.Age)

[<Fact>]
let ``Handle missing optional fields`` () =
    type Person = JsonProvider<"""
        [{"name":"Alice","age":30},
         {"name":"Bob"}]
    """>

    let person = Person.Parse("""{"name":"Charlie"}""")

    Assert.Equal("Charlie", person.Name)
    Assert.True(person.Age.IsNone)
```

---

## Type Providers vs Code Generation

### Comparison Matrix

| Aspect | Type Providers | Code Generation (T4, etc.) |
|--------|----------------|---------------------------|
| **Integration** | Built into F# compiler | Separate build step |
| **Workflow** | Seamless, no context switch | Invoke generator, reference output |
| **Invalidation** | Automatic when schema changes | Manual regeneration required |
| **F# Interactive** | Works perfectly | Requires pre-generation |
| **Scalability** | Handles millions of types (erasing) | Can generate excessive code |
| **Cross-Language** | F#-only (erasing), limited (generative) | Works with all .NET languages |
| **Reflection** | Limited (erased types) | Full reflection support |
| **Team Workflows** | Can complicate schema migrations | Source-controlled generated files |
| **Build Reproducibility** | Requires external data at build time | Self-contained in source control |
| **IDE Performance** | Can impact IntelliSense speed | No IDE overhead |

### When Type Providers Win

1. **Rapid Prototyping**: Explore APIs/data without writing types
2. **Stable Schemas**: Schema doesn't change frequently
3. **F#-Only Projects**: No need for C# interop
4. **Large Type Spaces**: Thousands of types (Freebase, etc.)
5. **Interactive Development**: Heavy use of F# Interactive (FSI)

### When Code Generation Wins

1. **Cross-Language**: Types consumed by C#, VB.NET, etc.
2. **CI/CD Pipelines**: Repeatable builds without external dependencies
3. **Team Collaboration**: Schema changes across multiple branches
4. **Reflection-Heavy**: Need full runtime type information
5. **Offline Development**: No network access to schemas

### Hybrid Approach

Best of both worlds:

```fsharp
// Development: Use type provider for exploration
#if DEBUG
type GitHub = JsonProvider<"https://api.github.com/repos/fsharp/fsharp">
#else
// Production: Use generated types (checked into source control)
type GitHub = GeneratedGitHubTypes
#endif
```

---

## Limitations and Considerations

### Technical Limitations

#### 1. No Generic Type Providers

Type providers cannot generate generic types:

```fsharp
// NOT POSSIBLE:
type GenericProvider<'T> = ...  // Error!

// Instead, use static parameters:
type Provider<"SomeParameter">  // OK
```

#### 2. Erased Type Reflection

Erased types lose information at runtime:

```fsharp
type Person = JsonProvider<"""{"name":"Alice"}""">
let person = Person.Parse("""{"name":"Bob"}""")

// At runtime:
printfn "%s" (person.GetType().Name)  // "Object" (not "Person")

// Can't use reflection to get properties
let props = person.GetType().GetProperties()  // Empty!
```

#### 3. Static Parameter Restrictions

Static parameters must be compile-time constants:

```fsharp
[<Literal>]
let schemaFile = "schema.json"  // Must be literal
type Data = JsonProvider<schemaFile>  // OK

let dynamicFile = getSchemaFile()  // Runtime value
type Data2 = JsonProvider<dynamicFile>  // ERROR!
```

#### 4. Schema Availability

Schemas must be accessible at compile time:

```fsharp
// Requires network access during compilation
type GitHub = JsonProvider<"https://api.github.com/...">

// Build server may not have network access!
// Solution: Cache schema locally
type GitHub = JsonProvider<"cached-schema.json">
```

### Performance Considerations

#### 1. Compile-Time Overhead

Type providers execute during compilation:

```fsharp
// Fetches data at compile time (can be slow)
type BigData = JsonProvider<"https://huge-api.com/data">

// Solution: Cache schema
type BigData = JsonProvider<"local-schema.json", SampleIsList=true>
```

#### 2. IDE Responsiveness

Complex type providers can slow IntelliSense:

```fsharp
// Every keystroke may query type provider
type ComplexDb = SqlProvider<connectionString>

// Solution: Use individuals sparingly
type ComplexDb = SqlProvider<
    connectionString,
    IndividualsAmount = 100>  // Limit generated individuals
```

#### 3. Runtime Performance

Type providers don't impact runtime performance (erasure):

```fsharp
type Person = JsonProvider<"""{"name":"Alice"}""">

// At runtime, this is just:
let person = JsonValue.Parse(json)  // Normal JSON parsing
```

### Development Challenges

#### 1. Debugging Type Providers

Hard to debug generated types:

```fsharp
// Can't step into provided type constructors
let person = Person.Parse(json)  // What code actually runs?

// Solution: Check generated quotations
// Use reflection to inspect provided members
```

#### 2. Version Conflicts

Design-time vs runtime version mismatches:

```fsharp
// Project references FSharp.Data 4.0.0
// But IDE loads FSharp.Data.DesignTime 3.3.3

// Solution: Ensure consistent versions in all tools
```

#### 3. Team Collaboration

Schema changes can break builds:

```
Developer A: Changes database schema, commits code
Developer B: Pulls changes, build fails (old schema cached)
Developer B: Spends hours troubleshooting type provider cache

Solution: Document schema migration steps
```

### When NOT to Use Type Providers

#### 1. Unstable Schemas

```fsharp
// API changes daily
type UnstableApi = JsonProvider<"https://daily-changes.com/api">
// Every schema change breaks builds!

// Better: Use dynamic parsing
let data = JsonValue.Load("https://daily-changes.com/api")
let name = data?name.AsString()
```

#### 2. One-Off Scripts

```fsharp
// Quick script to parse a single CSV
type OneTimeData = CsvProvider<"data.csv">  // Overkill

// Better: Use CSV parser directly
let rows = File.ReadLines("data.csv") |> Seq.map (fun l -> l.Split(','))
```

#### 3. C# Projects

```fsharp
// C# project wants to consume your types
type MyData = JsonProvider<"schema.json">  // Erased types don't work in C#

// Better: Use code generation or create wrapper types
```

---

## FSRS Integration Scenarios

### Critical Architecture Question

**Does FSRS maintain F# compiler compatibility?**

Two possible architectures:

#### Option A: F# Compiler Pipeline

```
.fsrs script → F# Compiler → .NET IL → FSRS VM executes IL
```

**Implications**:
- Type providers work out-of-the-box
- Full F# language support
- Leverage existing F# tooling
- Slower compilation (F# compiler overhead)
- Larger runtime dependency (FSharp.Core, F# compiler service)

#### Option B: Custom Compiler Pipeline

```
.fsrs script → Custom Parser → Custom Bytecode → FSRS VM
```

**Implications**:
- Type providers **DO NOT WORK** (custom compiler doesn't support them)
- Full control over compilation
- Faster compilation (optimized for FSRS subset)
- Smaller runtime footprint
- Must implement all tooling from scratch

**FSRS Reality Check**: Based on the ROADMAP and existing architecture, FSRS uses **Option B** (custom parser + bytecode VM). This means:

1. **Type providers won't be available in FSRS**
2. **But their patterns are still valuable for design**

### FSRS Without Type Providers: Alternative Patterns

Even without type providers, FSRS can adopt similar **ergonomic patterns** for host interop:

#### Pattern 1: Macro-Based Schema Generation

Instead of compile-time type providers, use **load-time macros**:

```fsharp
// FSRS macro system (hypothetical)
let! apiTypes = loadSchema "https://api.example.com/schema"

// Macro expands to record definitions at load time:
type User = { name: string; email: string }
type Post = { title: string; content: string; author: User }

// Usage is identical to hand-written types
let user = { name = "Alice"; email = "alice@example.com" }
```

#### Pattern 2: Reflection-Based Dynamic Types

```fsharp
// Host registers type schema
host.registerType "User" {
    fields = [
        { name = "name"; type = "string" }
        { name = "age"; type = "int" }
    ]
}

// FSRS accesses with dynamic member lookup
let user = host.User.create { name = "Alice"; age = 30 }
printfn "%s is %d" user.name user.age  // No static types, but IntelliSense via LSP
```

#### Pattern 3: Code Generation at Build Time

```bash
# Build script generates FSRS types from JSON schema
fsrs-gen schema.json --output generated.fsrs

# Script imports generated types
open Generated

let user: User = parseJson """{"name":"Alice","age":30}"""
```

### FSRS Host Interop: Type Provider-Inspired API

Even though FSRS won't have type providers, we can design host interop APIs that feel similar:

#### Schema-Driven Host Bindings

```rust
// Rust host application
use fsrs_host::HostInterop;

let mut host = HostInterop::new();

// Register database table schema (like SQLProvider)
host.register_table("Users", vec![
    ("id", Type::Int),
    ("name", Type::String),
    ("email", Type::String),
]);

// FSRS script can access with generated accessors
// (Generated at script load time, not compile time)
```

```fsharp
// FSRS script
let users = host.db.Users.query (fun u -> u.name.startsWith "A")
for user in users do
    printfn "User: %s <%s>" user.name user.email
```

#### JSON Schema Integration

```rust
// Host registers JSON schema
host.register_json_schema("Config", include_str!("config-schema.json"));
```

```fsharp
// FSRS script gets typed config access
let config = host.Config.load "app.json"
printfn "Database: %s:%d" config.database.host config.database.port
```

### FSRS LSP Server: Simulating Type Provider IntelliSense

Even without true type providers, FSRS can provide IDE integration:

```
┌─────────────────────────────────────────────────────┐
│  FSRS Language Server Protocol (LSP) Server         │
├─────────────────────────────────────────────────────┤
│                                                      │
│  1. Parse .fsrs script                              │
│  2. Detect host interop calls                       │
│  3. Query host schema registry                      │
│  4. Provide completion suggestions                  │
│  5. Type-check against host schemas                 │
│                                                      │
└─────────────────────────────────────────────────────┘
```

Example LSP features:
- **Autocomplete**: Suggest host API members based on schema
- **Hover**: Show type information for host bindings
- **Go-to-Definition**: Jump to schema definition files
- **Diagnostics**: Validate host API usage at edit time

### FSRS Type Provider Lessons for Design

#### 1. Minimize Boilerplate

Type providers eliminate manual type definitions. FSRS should too:

```fsharp
// BAD: Manual type definition
type User = { name: string; email: string }
let parseUser json = (* manual parsing *) ...

// GOOD: Schema-driven parsing
let user = Json.parse<"User"> json
// Schema registered by host or imported from file
```

#### 2. Design-Time Validation

Type providers catch errors at compile time. FSRS should validate at load time:

```fsharp
// FSRS loads script
// Before execution, validates:
//   - Host API calls against registered schemas
//   - JSON parsing against schemas
//   - Database queries against table definitions

// If validation fails, show helpful errors:
// Error: Unknown field 'emal' (did you mean 'email'?)
```

#### 3. IDE Integration Priority

Type providers shine because of IntelliSense. FSRS LSP should be a first-class concern:

```fsharp
// As developer types:
host.database.Users.
//                  ^--- LSP shows: query, insert, update, delete, count

// LSP queries host schema and provides completions
```

#### 4. Sample-Based Schema Inference

Type providers infer types from samples. FSRS could too:

```fsharp
// Infer type from sample JSON
let sample = """{"name":"Alice","age":30}"""
let! Person = inferType sample

// Generate parsing function
let parse = Json.parseAs<Person>

let alice = parse """{"name":"Alice","age":30}"""  // OK
let bob = parse """{"name":"Bob"}"""                // Error: missing 'age'
```

---

## Implementation Roadmap for FSRS

### Phase 1: Schema Registry System

**Goal**: Enable host applications to register type schemas for FSRS scripts

**Implementation**:

```rust
// rust/crates/fsrs-host/src/schema.rs

pub enum SchemaType {
    Int,
    Float,
    String,
    Bool,
    List(Box<SchemaType>),
    Record(HashMap<String, SchemaType>),
    Optional(Box<SchemaType>),
}

pub struct TypeSchema {
    name: String,
    fields: HashMap<String, SchemaType>,
}

pub struct SchemaRegistry {
    types: HashMap<String, TypeSchema>,
}

impl SchemaRegistry {
    pub fn register(&mut self, schema: TypeSchema) {
        self.types.insert(schema.name.clone(), schema);
    }

    pub fn lookup(&self, name: &str) -> Option<&TypeSchema> {
        self.types.get(name)
    }

    pub fn validate_value(&self, type_name: &str, value: &Value) -> Result<(), String> {
        // Validate value matches schema
    }
}
```

**FSRS Script Usage**:

```fsharp
// Script has access to registered schemas
let user = host.createUser { name = "Alice"; email = "alice@example.com" }
// VM validates field names/types against registered User schema
```

### Phase 2: JSON Schema Integration

**Goal**: Import JSON schemas for validation and code completion

**Implementation**:

```rust
// Parse JSON Schema -> TypeSchema conversion
pub fn parse_json_schema(schema: &str) -> Result<TypeSchema, Error> {
    let parsed: serde_json::Value = serde_json::from_str(schema)?;
    // Convert JSON Schema to TypeSchema
}
```

**FSRS Script Usage**:

```fsharp
// Import external schema
schema "User" from "schemas/user.schema.json"

// Parser validates usage
let user: User = parseJson userJsonString
```

### Phase 3: LSP Server for IntelliSense

**Goal**: Provide type provider-like IDE experience

**Implementation**:

```rust
// rust/crates/fsrs-lsp/src/lib.rs

pub struct FsrsLanguageServer {
    schema_registry: SchemaRegistry,
    // ...
}

impl LanguageServer for FsrsLanguageServer {
    fn completion(&self, params: CompletionParams) -> Vec<CompletionItem> {
        // 1. Parse FSRS script up to cursor position
        // 2. Determine context (e.g., "host.database.Users.")
        // 3. Query schema registry for completion candidates
        // 4. Return completion items
    }

    fn hover(&self, params: HoverParams) -> Option<Hover> {
        // Provide type information on hover
    }
}
```

**VS Code Extension**:

```typescript
// editors/vscode/src/extension.ts

const client = new LanguageClient(
    'fsrs',
    'FSRS Language Server',
    serverOptions,
    clientOptions
);

client.start();
```

### Phase 4: Compile-Time Schema Validation

**Goal**: Validate host API usage when script loads

**Implementation**:

```rust
// During script loading
pub fn validate_script(script: &Ast, registry: &SchemaRegistry) -> Result<(), Vec<Error>> {
    let mut errors = Vec::new();

    // Walk AST, validate:
    for expr in &script.expressions {
        match expr {
            Expr::HostCall(obj, method, args) => {
                // Validate method exists on host object schema
                // Validate argument types match schema
            }
            // ...
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

### Phase 5: Dynamic Schema Discovery

**Goal**: Scripts can query available schemas at runtime

**FSRS Script Usage**:

```fsharp
// Discover available types
let schemas = host.listSchemas ()
for schema in schemas do
    printfn "Type: %s" schema.name
    for field in schema.fields do
        printfn "  - %s: %s" field.name field.type

// Dynamic invocation based on schema
let createEntity typeName fields =
    host.invoke typeName "create" fields
```

---

## Comparison: Type Providers vs FSRS Schema System

| Feature | F# Type Providers | FSRS Schema System |
|---------|------------------|-------------------|
| **Timing** | Compile-time | Script load-time |
| **Generation** | F# Compiler | FSRS VM |
| **IntelliSense** | Full (via TPDTC) | Partial (via LSP) |
| **Validation** | Compile errors | Load-time errors |
| **Schemas** | External (DB, JSON, etc.) | Host-registered + JSON |
| **Reflection** | Limited (erasing) | Full (runtime schemas) |
| **Cross-Language** | F#-only | Host language agnostic |
| **Dynamic Types** | No | Yes (runtime discovery) |
| **Invalidation** | Automatic (design-time) | Manual (reload script) |

---

## References

### Official Documentation

- [Microsoft Docs: F# Type Providers](https://learn.microsoft.com/en-us/dotnet/fsharp/tutorials/type-providers/)
- [Creating a Type Provider Tutorial](https://learn.microsoft.com/en-us/dotnet/fsharp/tutorials/type-providers/creating-a-type-provider)
- [F# Type Provider SDK](https://fsprojects.github.io/FSharp.TypeProviders.SDK/)
- [Technical Notes](https://fsprojects.github.io/FSharp.TypeProviders.SDK/technical-notes.html)

### Popular Type Provider Libraries

- [FSharp.Data](https://fsprojects.github.io/FSharp.Data/) - JSON, CSV, XML, HTML
  - [JSON Provider Docs](https://fsprojects.github.io/FSharp.Data/library/JsonProvider.html)
  - [CSV Provider Docs](https://fsprojects.github.io/FSharp.Data/library/CsvProvider.html)
- [SQLProvider](https://fsprojects.github.io/SQLProvider/) - Multi-database support
- [FSharp.Data.SqlClient](https://fsprojects.github.io/FSharp.Data.SqlClient/) - SQL Server specialized
- [FSharp.Configuration](https://fsprojects.github.io/FSharp.Configuration/) - YAML, INI, AppSettings
- [FSharp.Data.GraphQL](https://fsprojects.github.io/FSharp.Data.GraphQL/) - GraphQL type provider

### Articles and Tutorials

- [Introduction to F# Type Providers - TheSharperDev](https://thesharperdev.com/introduction-to-fsharp-type-providers/)
- [Magic of F# Type Providers - Max Fedotov](https://maximcus.medium.com/magic-of-f-type-providers-225b1169c7a0)
- [Writing a F# Type Provider - Aaron Powell](https://www.aaron-powell.com/posts/2015-02-06-writing-a-fsharp-type-provider/)
- [Accessing Your Data with F# - CODE Magazine](https://www.codemag.com/article/1703051/Accessing-Your-Data-with-F)
- [Using F# Type Providers for Data Science](https://www.codemag.com/article/1701051/Using-F)

### Research Papers

- [FSharp.Data Research Paper](https://www.microsoft.com/en-us/research/publication/information-rich-programming-with-fsharp/) - Distinguished Paper Award, PLDI 2016

### Community Resources

- [F# Software Foundation](https://fsharp.org/)
- [F# for Fun and Profit](https://fsharpforfunandprofit.com/)
- [F# Community Projects](https://fsharp.org/community/projects/)

---

## Conclusion

Type providers represent F#'s unique approach to compile-time metaprogramming, offering zero-boilerplate access to external data sources with full type safety and IDE integration. While FSRS likely won't support true type providers (due to its custom compiler architecture), the **patterns and principles** they embody should heavily influence FSRS's host interop design.

**Key Takeaways for FSRS**:

1. **Schema-Driven Development**: Enable host applications to register type schemas
2. **Load-Time Validation**: Catch errors before script execution
3. **LSP-Based Tooling**: Provide IntelliSense via Language Server Protocol
4. **Minimize Boilerplate**: Auto-generate parsing/serialization from schemas
5. **Dynamic Discovery**: Allow scripts to query available types at runtime

By adopting type provider-inspired patterns at the **load-time** and **LSP** layers, FSRS can deliver a similar developer experience without requiring F# compiler integration.

**Next Steps**:
1. Implement SchemaRegistry in fsrs-host crate
2. Design host interop API for schema registration
3. Build FSRS LSP server for IDE integration
4. Create JSON Schema import tooling
5. Document schema-driven host binding patterns

---

**Status**: Research complete. Type providers are F#-specific compile-time feature, but their design patterns are highly applicable to FSRS host interop architecture.
