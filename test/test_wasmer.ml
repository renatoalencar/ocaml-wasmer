open Wasmer

let value = Alcotest.of_pp Wasmer.Value.pp

let run_wat ?store ?imports code function_name arguments =
  let store = Option.value ~default:(Store.default ()) store in
  let imports = Option.value ~default:(Imports.make ()) imports in
  let exports =
    code
    |> Module.make store
    |> Instance.make imports
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
    |}
  in
  let result = run_wat code "add" [Value.F32 42.0 ; Value.F32 10.0] in
  Alcotest.(check @@ list value) "Same value" result [Value.F32 52.0]

let test_external_functions () =
  let code = {|
    (module
      (func $add (import "env" "add") (param i32) (param i32) (result i32))
      (func $main (param i32) (param i32) (result i32)
        local.get 0
        local.get 1
        call $add)
      (export "main" (func $main)))
    |}
  in
  let store = Store.default () in
  let add args =
    match args with
    | Value.[ I32 a ; I32 b ] -> Value.[I32 (Int32.add a b)]
    | _ -> assert false
  in
  let imports =
    let imports = Imports.make () in
    let signature = Type.([I32 ; I32], [I32]) in
    let exports = Exports.of_list [ "add", Function.make store signature add ] in
    Imports.register imports "env" exports;
    imports
  in
  let result = run_wat ~store ~imports code "main" Value.[I32 42l; I32 57l] in
  Alcotest.(check @@ list value) "Same value" result [Value.I32 99l]

let () =
  let open Alcotest in
  run "WASMer" [ "Simple add", [ test_case "i32" `Quick test_add_i32
                               ; test_case "i64" `Quick test_add_i64
                               ; test_case "f32" `Quick test_add_f32 ]
               ; "Imports",    [ test_case "Functions" `Quick test_external_functions ] ]  