use std::path::{Path};
use wasmer::{Store, Module, Instance, Exports, Function, ImportObject};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_engine_universal::Universal;

unsafe extern "C" fn finalizer<T>(value: ocaml::Raw) {
    value.as_pointer::<T>().drop_in_place()
}

#[ocaml::func]
pub fn make_store_default() -> ocaml::Pointer<Store> {
    let compiler = Singlepass::default();
    let store = Store::new(&Universal::new(compiler).engine());
    ocaml::Pointer::alloc_final(store, Some(finalizer::<Store>), None)
}

#[ocaml::func]
pub fn make_module(store: ocaml::Pointer<Store>, code: String) -> ocaml::Pointer<'static, Module> {
    let module = Module::new(store.as_ref(), code).expect("Could not create module");
    ocaml::Pointer::alloc_final(module, Some(finalizer::<Module>), None)
}

#[ocaml::func]
pub fn make_module_from_file(store: ocaml::Pointer<Store>, filename: String) -> ocaml::Pointer<'static, Module> {
    let path = Path::new(filename.as_str());
    let module =
        Module::from_file(store.as_ref(), path)
            .expect("Could not create module from file");
    ocaml::Pointer::alloc_final(module, Some(finalizer::<Module>), None)
}

#[ocaml::func]
pub fn make_instance(import_object: ocaml::Pointer<ImportObject>, module: ocaml::Pointer<Module>) -> ocaml::Pointer<'static, Instance> {
    let module = module.as_ref();
    let import_object = import_object.as_ref();

    let instance = Instance::new(&module, &import_object).expect("Could not create instance");
    ocaml::Pointer::alloc_final(instance, Some(finalizer::<Instance>), None)
}

#[ocaml::func]
pub fn exports(instance: ocaml::Pointer<Instance>) -> ocaml::Pointer<'static, Exports> {
    let exports = instance.as_ref().exports.clone();
    ocaml::Pointer::alloc_final(exports, Some(finalizer::<Exports>), None)
}

#[ocaml::func]
pub fn get_function(exports: ocaml::Pointer<Exports>, name: String) -> ocaml::Pointer<'static, Function> {
    let function =
        exports
            .as_ref()
            .get_function(name.as_str())
            .expect("Could not find function")
            .clone();
    ocaml::Pointer::alloc_final(function, Some(finalizer::<Function>), None)
}

#[derive(ocaml::IntoValue, ocaml::FromValue, Clone, Copy)]
enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    ExternalRef,
    FuncRef,
    U128,
}

impl Into<wasmer::Value> for Value {
    fn into(self) -> wasmer::Value {
        match self {
            Value::I32(v) => wasmer::Value::I32(v),
            Value::I64(v) => wasmer::Value::I64(v),
            Value::F32(v) => wasmer::Value::F32(v),
            _ => panic!("Not supported"),
        }
    }
}

impl From<&wasmer::Value> for Value {
    fn from(value: &wasmer::Value) -> Self {
        match value {
            wasmer::Value::I32(v) => Value::I32(*v),
            wasmer::Value::I64(v) => Value::I64(*v),
            wasmer::Value::F32(v) => Value::F32(*v),
            _ => panic!("Not supported")
        }
    }
}

#[ocaml::func]
pub fn call(function: ocaml::Pointer<Function>, params: ocaml::List<Value>) -> ocaml::List<'static, Value> {
    let params: Vec<wasmer::Value> =
        params
            .into_vec() // TODO: `ListIterator` does not provide proper variable types
            .into_iter()
            .map(|value| value.into())
            .collect();

    let return_values =
        function
            .as_ref()
            .call(&params)
            .expect("Function call was not successful");

    return_values
        .into_iter()
        .map(|value| value.into())
        .fold(
            ocaml::List::empty(),
            // TODO: Better ways to convert iterators to OCaml lists.
            |list, item| unsafe { list.add(gc, item) }
        )
}

#[ocaml::func]
pub fn make_imports() -> ocaml::Pointer<ImportObject> {
    let imports = ImportObject::new();
    ocaml::Pointer::alloc_final(imports, Some(finalizer::<ImportObject>), None)
}

#[ocaml::func]
pub fn register_export_object(imports: ocaml::Raw, name: String, exports: ocaml::Pointer<Exports>) {
    let mut ptr = unsafe { imports.as_pointer::<ImportObject>() };
    let imports = ptr.as_mut();
    let exports = exports.as_ref();

    imports.register(name, exports.clone());
}

#[ocaml::func]
pub fn exports_from_list(exports_list: ocaml::List<(String, ocaml::Raw)>) -> ocaml::Pointer<'static, Exports> {
    let mut exports = Exports::new();

    for (name, export) in exports_list.into_linked_list() {
        let export = unsafe { export.as_pointer::<wasmer::Function>() }.as_ref().clone();
        exports.insert(name, export);
    }

    ocaml::Pointer::alloc_final(exports, Some(finalizer::<Exports>), None)
}

#[derive(ocaml::IntoValue, ocaml::FromValue, Clone, Copy)]
enum Type {
    I32,
    I64,
    F32,
    F64,
    V128,
    ExternRef,
    FuncRef,
}

impl Into<wasmer::Type> for Type {
    fn into(self: Type) -> wasmer::Type {
        match self {
            Type::I32 => wasmer::Type::I32,
            Type::I64 => wasmer::Type::I64,
            Type::F32 => wasmer::Type::F32,
            Type::F64 => wasmer::Type::F64,
            Type::V128 => wasmer::Type::V128,
            Type::ExternRef => wasmer::Type::ExternRef,
            Type::FuncRef => wasmer::Type::FuncRef,
        }
    }
}

impl From<wasmer::Type> for Type {
    fn from(ty: wasmer::Type) -> Self {
        match ty {
            wasmer::Type::I32 => Type::I32,
            wasmer::Type::I64 => Type::I64,
            wasmer::Type::F32 => Type::F32,
            wasmer::Type::F64 => Type::F64,
            wasmer::Type::V128 => Type::V128,
            wasmer::Type::ExternRef => Type::ExternRef,
            wasmer::Type::FuncRef => Type::FuncRef,
        }
    }
}

#[derive(wasmer::WasmerEnv, Clone)]
struct Env { value: ocaml::Raw }

#[ocaml::func]
pub fn make_function(store: ocaml::Pointer<Store>, signature: (ocaml::List<Type>, ocaml::List<Type>), f: ocaml::Raw) -> ocaml::Pointer<'static, Function> {
    let signature = {
        let mut params: Vec<wasmer::Type> = vec![];
        for ty in signature.0.into_linked_list() {
            params.push(ty.into())
        }

        let mut return_type: Vec<wasmer::Type> = vec![];
        for ty in signature.1.into_linked_list() {
            return_type.push(ty.into())
        }

        wasmer::FunctionType::new(params, return_type)
    };

    let function = Function::new_with_env(store.as_ref(), signature, Env { value: f }, |env: &Env, args| {
        // TODO: The oficial ocaml-rs docs recomment not to use `recover_handle` outside the scope of tests
        // need a better way to do this.
        let gc = unsafe { ocaml::Runtime::recover_handle() };

        let args: ocaml::List<Value> =
            args
                .into_iter()
                .fold(
                    ocaml::List::empty(),
                    |list, item| unsafe { list.add(gc, item.into()) }
                );

        let result = unsafe {
            env
                .value
                .as_value()
                .call(gc, args)
                .expect("Could not call OCaml function")
        };

        let return_value: Vec<wasmer::Value> = {
            let return_value: ocaml::List<Value> = ocaml::FromValue::from_value(result);
            let mut ret: Vec<wasmer::Value> = vec![];
            for value in return_value.into_vec() {
                ret.push(value.into())
            }

            ret
        };
        
        Ok(return_value)
    });

    ocaml::Pointer::alloc_final(function, Some(finalizer::<Function>), None)
}

/* This should be only used for benchmarking */

#[ocaml::func]
pub fn __bench_alloc() -> ocaml::Pointer<i64> {
    let value = 42;
    ocaml::Pointer::alloc_final(value, None, None)
}

#[ocaml::func]
pub fn __bench_alloc_abstract_ptr() -> ocaml::Value {
    let value = Box::new(42);
    let ptr: *mut i64 = Box::into_raw(value);
    unsafe { ocaml::Value::alloc_abstract_ptr(ptr) }
}

#[ocaml::func]
pub fn __bench_alloc_box_value() -> ocaml::OCaml<ocaml::interop::DynBox<Box<i64>>> {
    let value = Box::new(42);
    ocaml::OCaml::box_value(gc, value)
}

#[ocaml::func]
pub fn __bench_no_alloc() -> ocaml::Int {
    let value = 42;
    value
}

#[ocaml::func]
pub fn __call_for_bench(function: ocaml::Pointer<Function>) {
    let params = [wasmer::Value::I32(42), wasmer::Value::I32(10)];
    function.as_ref().call(&params).unwrap();
}

fn build_function_from_code(code: String, function_name: String) -> Function {
    let compiler = Singlepass::default();
    let store = Store::new(&Universal::new(compiler).engine());
    let module = Module::new(&store, code).expect("Could not create module");
    let imports = ImportObject::new();
    let instance = Instance::new(&module, &imports).expect("Could not create instance");

    instance.exports.get_function(&function_name).unwrap().clone()
}

#[ocaml::func]
pub fn __function_from_code(code: String, function_name: String) -> ocaml::Pointer<Function> {
    let function = build_function_from_code(code, function_name);
    ocaml::Pointer::alloc_final(function, Some(finalizer::<Function>), None)
}

#[ocaml::func]
pub fn __function_from_code_box_value(code: String, function_name: String) -> ocaml::OCaml<ocaml::interop::DynBox<Function>> {
    let function = build_function_from_code(code, function_name);
    ocaml::OCaml::box_value(gc, function)
}