use std::path::{Path};
use wasmer::{Store, Module, imports, Instance, Exports, Function};

unsafe extern "C" fn finalizer<T>(value: ocaml::Raw) {
    value.as_pointer::<T>().drop_in_place()
}

#[ocaml::func]
pub fn make_store_default() -> ocaml::Pointer<Store> {
    let store = Store::default();
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
pub fn make_instance(module: ocaml::Pointer<Module>) -> ocaml::Pointer<'static, Instance> {
    let module = module.as_ref();
    let import_object = imports! {};

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
            .into_vec()
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
            |list, item| unsafe { list.add(gc, item) }
        )
}