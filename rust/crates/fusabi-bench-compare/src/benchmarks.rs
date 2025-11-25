// Benchmark implementations for Fusabi, Rhai, and Lua

use fusabi::Engine;
use mlua::Lua;
use rhai::Engine as RhaiEngine;

// ============================================================================
// Fibonacci Benchmarks
// ============================================================================

pub fn fusabi_fibonacci() {
    let source = r#"
let rec fib n =
    if n <= 1 then
        n
    else
        fib (n - 1) + fib (n - 2)
in fib 20
"#;

    let mut engine = Engine::new();
    engine.eval(source).unwrap();
}

pub fn rhai_fibonacci() {
    let script = r#"
fn fib(n) {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fib(20)
"#;

    let engine = RhaiEngine::new();
    engine.eval::<i64>(script).unwrap();
}

pub fn lua_fibonacci() {
    let script = r#"
function fib(n)
    if n <= 1 then
        return n
    else
        return fib(n - 1) + fib(n - 2)
    end
end

return fib(20)
"#;

    let lua = Lua::new();
    lua.load(script).eval::<i64>().unwrap();
}

// ============================================================================
// Sieve of Eratosthenes Benchmarks
// ============================================================================

pub fn fusabi_sieve() {
    let source = r#"
let rec filter predicate lst =
    match lst with
    | [] -> []
    | head :: tail ->
        if predicate head then
            head :: filter predicate tail
        else
            filter predicate tail
in
let rec range start end_val =
    if start > end_val then
        []
    else
        start :: range (start + 1) end_val
in
let rec sieve lst =
    match lst with
    | [] -> []
    | prime :: rest ->
        let is_not_multiple n = n % prime <> 0
        in prime :: sieve (filter is_not_multiple rest)
in
let numbers = range 2 100
in sieve numbers
"#;

    let mut engine = Engine::new();
    engine.eval(source).unwrap();
}

pub fn rhai_sieve() {
    let script = r#"
fn filter(lst, predicate) {
    let result = [];
    for item in lst {
        if predicate.call(item) {
            result.push(item);
        }
    }
    result
}

fn range(start, end_val) {
    let result = [];
    for i in start..=end_val {
        result.push(i);
    }
    result
}

fn sieve(lst) {
    if lst.is_empty() {
        return [];
    }

    let prime = lst[0];
    let rest = [];
    for i in 1..lst.len() {
        rest.push(lst[i]);
    }

    let filtered = filter(rest, |n| n % prime != 0);
    let result = [prime];
    for p in sieve(filtered) {
        result.push(p);
    }
    result
}

let numbers = range(2, 100);
sieve(numbers)
"#;

    let engine = RhaiEngine::new();
    engine.eval::<rhai::Array>(script).unwrap();
}

pub fn lua_sieve() {
    let script = r#"
function filter(lst, predicate)
    local result = {}
    for _, item in ipairs(lst) do
        if predicate(item) then
            table.insert(result, item)
        end
    end
    return result
end

function range(start, end_val)
    local result = {}
    for i = start, end_val do
        table.insert(result, i)
    end
    return result
end

function sieve(lst)
    if #lst == 0 then
        return {}
    end

    local prime = lst[1]
    local rest = {}
    for i = 2, #lst do
        table.insert(rest, lst[i])
    end

    local filtered = filter(rest, function(n) return n % prime ~= 0 end)
    local result = {prime}
    for _, p in ipairs(sieve(filtered)) do
        table.insert(result, p)
    end
    return result
end

local numbers = range(2, 100)
return sieve(numbers)
"#;

    let lua = Lua::new();
    lua.load(script).eval::<mlua::Table>().unwrap();
}

// ============================================================================
// Ackermann Function Benchmarks
// ============================================================================

pub fn fusabi_ackermann() {
    let source = r#"
let rec ack m n =
    if m = 0 then
        n + 1
    else if n = 0 then
        ack (m - 1) 1
    else
        ack (m - 1) (ack m (n - 1))
in ack 3 7
"#;

    let mut engine = Engine::new();
    engine.eval(source).unwrap();
}

pub fn rhai_ackermann() {
    let script = r#"
fn ack(m, n) {
    if m == 0 {
        n + 1
    } else if n == 0 {
        ack(m - 1, 1)
    } else {
        ack(m - 1, ack(m, n - 1))
    }
}

ack(3, 7)
"#;

    let engine = RhaiEngine::new();
    engine.eval::<i64>(script).unwrap();
}

pub fn lua_ackermann() {
    let script = r#"
function ack(m, n)
    if m == 0 then
        return n + 1
    elseif n == 0 then
        return ack(m - 1, 1)
    else
        return ack(m - 1, ack(m, n - 1))
    end
end

return ack(3, 7)
"#;

    let lua = Lua::new();
    lua.load(script).eval::<i64>().unwrap();
}

// ============================================================================
// Array Operations Benchmarks
// ============================================================================

pub fn fusabi_array_ops() {
    let source = r#"
let rec sum_array arr idx len acc =
    if idx >= len then
        acc
    else
        sum_array arr (idx + 1) len (acc + arr.[idx])
in
let arr = [| 1; 2; 3; 4; 5; 6; 7; 8; 9; 10 |]
in
let len = 10
in
let rec create_and_sum count acc =
    if count = 0 then
        acc
    else
        let result = sum_array arr 0 len 0
        in create_and_sum (count - 1) (acc + result)
in create_and_sum 100 0
"#;

    let mut engine = Engine::new();
    engine.eval(source).unwrap();
}

pub fn rhai_array_ops() {
    let script = r#"
fn sum_array(arr) {
    let sum = 0;
    for i in 0..arr.len() {
        sum += arr[i];
    }
    sum
}

let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

fn create_and_sum(count, acc) {
    if count == 0 {
        acc
    } else {
        let result = sum_array(arr);
        create_and_sum(count - 1, acc + result)
    }
}

create_and_sum(100, 0)
"#;

    let engine = RhaiEngine::new();
    engine.eval::<i64>(script).unwrap();
}

pub fn lua_array_ops() {
    let script = r#"
function sum_array(arr)
    local sum = 0
    for i = 1, #arr do
        sum = sum + arr[i]
    end
    return sum
end

local arr = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}

function create_and_sum(count, acc)
    if count == 0 then
        return acc
    else
        local result = sum_array(arr)
        return create_and_sum(count - 1, acc + result)
    end
end

return create_and_sum(100, 0)
"#;

    let lua = Lua::new();
    lua.load(script).eval::<i64>().unwrap();
}
