
open Core_bench.Bench

let add_int =
  Test.create ~name:"Add int"
    (fun () -> 1 + 1)

let add_int64 =
  Test.create ~name:"Add int64"
    (fun () -> Int64.add 1L 1L)

let add_float =
  Test.create ~name:"Add float"
    (fun () -> 1.0 +. 1.0)

let build_fn code function_name =
    let open Wasmer in
    let store = Store.default () in
    let imports = Imports.make () in
    let exports =
      code
      |> Module.make store
      |> Instance.make imports
      |> Instance.exports
    in
    Exports.get_function exports function_name

external build_fn_bench : string -> string -> Wasmer.Function.t = "__function_from_code"
external build_fn_bench_box_value : string -> string -> Wasmer.Function.t = "__function_from_code_box_value"

let code = {|
  (module
    (func $add (param i32) (param i32) (result i32)
      local.get 0
      local.get 1
      i32.add)
    (export "add" (func $add)))
|}

let bench_build_fn =
  Test.create ~name:"Wrapped build fn"
    (fun () -> build_fn code "add")

let bench_build_fn' =
  Test.create ~name:"Non-wrapped build fn"
    (fun () -> build_fn_bench code "add")

let bench_build_fn_box =
  Test.create ~name:"box_value build fn"
    (fun () -> build_fn_bench_box_value code "add")

let function_call =
  let fn = build_fn code "add" in
  let arguments = [Wasmer.Value.I32 42l ; Wasmer.Value.I32 10l] in
  Test.create ~name:"Function call wrapped"
    (fun () -> Wasmer.Function.call fn arguments)

external call_fn_bench : Wasmer.Function.t -> unit = "__call_for_bench"
let function_call_unwraped =
  let fn = build_fn code "add" in
  Test.create ~name:"Function call unwrapped"
    (fun () -> call_fn_bench fn)

external bench_alloc : unit -> Int64.t = "__bench_alloc"
external bench_no_alloc : unit -> int = "__bench_no_alloc"

external bench_alloc_abstract_ptr : unit -> 'a = "__bench_alloc_abstract_ptr"
let interop_with_alloc =
  Test.create ~name:"With alloc"
    (fun () -> bench_alloc ())

let interop_without_alloc =
  Test.create ~name:"Without alloc"
    (fun () -> bench_no_alloc ())

let interop_with_alloc_abstract_ptr =
  Test.create ~name:"With alloc abstract ptr"
    (fun () -> bench_alloc_abstract_ptr ())

external alloc_with_box_value : unit -> Int64.t = "__bench_alloc_box_value"
let interop_with_alloc_box_value =
  Test.create ~name:"Alloc with box value"
    (fun () -> alloc_with_box_value)

let () = 
  bench [ add_int
        ; add_int64
        ; add_float
        ; function_call
        ; function_call_unwraped
        ; bench_build_fn
        ; bench_build_fn'
        ; bench_build_fn_box
        ; interop_with_alloc
        ; interop_without_alloc
        ; interop_with_alloc_abstract_ptr
        ; interop_with_alloc_box_value ]