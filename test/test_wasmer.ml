open Wasmer

let value = Alcotest.of_pp Wasmer.Value.pp

let run_wat code function_name arguments =
  let store = Store.default () in
  let exports =
    code
    |> Module.make store
    |> Instance.make
    |> Instance.exports
  in
  let fn = Exports.get_function exports function_name in
  Function.call fn arguments

let test_add_i32 () =
  let code = {|
    (module
      (func $add (param i32) (param i32) (result i32)
        local.get 0
        local.get 1
        i32.add)
      (export "add" (func $add)))
  |} in
  let result = run_wat code "add" [Value.I32 42l ; Value.I32 10l] in
  Alcotest.(check @@ list value) "Same value" result [Value.I32 52l]

let test_add_i64 () =
  let code = {|
    (module
      (func $add (param i64) (param i64) (result i64)
        local.get 0
        local.get 1
        i64.add)
      (export "add" (func $add)))
  |} in
  let result = run_wat code "add" [Value.I64 42L ; Value.I64 10L] in
  Alcotest.(check @@ list value) "Same value" result [Value.I64 52L]
let test_add_f32 () =
  let code = {|
    (module
      (func $add (param f32) (param f32) (result f32)
        local.get 0
        local.get 1
        f32.add)
      (export "add" (func $add)))
  |} in
  let result = run_wat code "add" [Value.F32 42.0 ; Value.F32 10.0] in
  Alcotest.(check @@ list value) "Same value" result [Value.F32 52.0]
  
let () =
  let open Alcotest in
  run "WASMer" [ "Simple add", [ test_case "i32" `Quick test_add_i32
                               ; test_case "i64" `Quick test_add_i64
                               ; test_case "f32" `Quick test_add_f32 ] ]  