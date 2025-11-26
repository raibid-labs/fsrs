(* String concatenation with ++ operator *)

(* Simple concatenation *)
let greeting = "Hello" ++ " " ++ "World" in

(* URL building example from issue #139 *)
let host = "localhost" in
let port = "8080" in
let url = "http://" ++ host ++ ":" ++ port in

(* Building paths *)
let base_path = "/home" in
let user = "alice" in
let file = "document.txt" in
let full_path = base_path ++ "/" ++ user ++ "/" ++ file in

(* Returning the results *)
(greeting, url, full_path)
