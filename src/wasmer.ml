module Store = struct
  type t
  external default : unit -> t = "make_store_default"
end

module Module = struct
  type t
  external make : Store.t -> string -> t = "make_module"
  external of_file : Store.t -> string -> t = "make_module_from_file"
end

module Type = struct
  type t =
    | I32
    | I64
    | F32
    | F64
    | V128
    | ExternalRef
    | FuncRef
end

module Value = struct
  type t =
    | I32 of int32
    | I64 of int64
    | F32 of float
    | ExternalRef
    | FuncRef
    | V128

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
  external make : Store.t -> Type.t list * Type.t list -> (Value.t list -> Value.t list) -> t = "make_function"
end

module Exports = struct
  type t
  external get_function : t -> string -> Function.t = "get_function"
  external of_list : (string * Function.t) list -> t = "exports_from_list"
end

module Imports = struct
  type t
  external make : unit -> t = "make_imports"
  external register : t -> string -> Exports.t -> unit = "register_export_object"
end

module Instance = struct
  type t
  external make : Imports.t -> Module.t -> t = "make_instance"
  external exports : t -> Exports.t = "exports"
end
