module Store = struct
  type t
  external default : unit -> t = "make_store_default"
end

module Module = struct
  type t
  external make : Store.t -> string -> t = "make_module"
  external of_file : Store.t -> string -> t = "make_module_from_file"
end

module Value = struct
  type t =
    | I32 of int32
    | I64 of int64
    | F32 of float
    | ExternalRef
    | FuncRef
    | U128

  let to_string t =
    match t with
    | I32 v -> Printf.sprintf "I32(%ld)" v
    | I64 v -> Printf.sprintf "I64(%Ld)" v
    | F32 v -> Printf.sprintf "F32(%f)" v
    | _ -> failwith("Other types are not supported")

  let pp fmt t =
    Format.fprintf fmt "%s" (to_string t)
end

module Function = struct
  type t
  external call : t -> Value.t list -> Value.t list = "call"
end

module Exports = struct
  type t
  external get_function : t -> string -> Function.t = "get_function"
end

module Instance = struct
  type t
  external make : Module.t -> t = "make_instance"
  external exports : t -> Exports.t = "exports"
end
