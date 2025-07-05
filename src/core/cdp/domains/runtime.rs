use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type ExecutionContextId = u64;
pub type RemoteObjectId = String;
pub type ScriptId = String;
pub type TimeDelta = f64;
pub type Timestamp = f64;

///Primitive value which cannot be JSON-stringified. Includes values -0, NaN, Infinity, -Infinity, and bigint literals.
pub type UnserializableValue = String;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CallArgument {
    pub value: Option<Value>,
    pub unserializable_value: Option<UnserializableValue>,
    pub object_id: Option<RemoteObjectId>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CallFrame {
    pub function_name: String,
    pub script_id: ScriptId,
    pub url: String,
    pub line_number: i32,
    pub column_number: i32,
}

///https://chromedevtools.github.io/devtools-protocol/tot/Runtime/#type-DeepSerializedValue
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeepSerializedValue {
    pub type_name: String, // Allowed Values: undefined, null, string, number, boolean, bigint, regexp, date, symbol, array, object, function, map, set, weakmap, weakset, error, proxy, promise, typedarray, arraybuffer, node, window, generator
    pub value: Option<Value>,
    pub object_id: Option<RemoteObjectId>,
    pub weak_local_object_reference: Option<u64>, //CDP docs says it's number (??)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteObject {
    #[serde(rename = "type")]
    pub type_name: String, // Allowed Values: object, function, undefined, string, number, boolean, symbol, bigint
    pub subtype: Option<String>, // Allowed Values: array, null, node, regexp, date, map, set, weakmap, weakset, iterator, generator, error, proxy, promise, typedarray, arraybuffer, dataview, webassemblymemory, wasmvalue
    pub class_name: Option<String>, // Object class (constructor) name. Specified for object type values only.
    pub value: Option<Value>, // Remote object value in case of primitive values or JSON values (if it was requested).
    pub unserializable_value: Option<UnserializableValue>, // Primitive value which can not be JSON-stringified does not have value, but gets this property.
    pub description: Option<String>,                       // String representation of the object.
    pub deep_serialized_value: Option<DeepSerializedValue>,
    pub object_id: Option<RemoteObjectId>,
    // pub custom_preview: Option<CustomPreview>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PropertyDescriptor {
    pub name: String,
    pub value: Option<RemoteObject>,
    pub writable: Option<bool>,
    pub get: Option<RemoteObject>,
    pub set: Option<RemoteObject>,
    pub configurable: bool,
    pub enumerable: bool,
    pub was_thrown: Option<bool>,
    pub is_own: Option<bool>,
    pub symbol: Option<RemoteObject>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InternalPropertyDescriptor {
    pub name: String,
    pub value: Option<RemoteObject>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrivatePropertyDescriptor {
    pub name: String,
    pub value: Option<RemoteObject>,
    pub get: Option<RemoteObject>,
    pub set: Option<RemoteObject>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SerializationOptions {
    pub serialization: String, // Allowed Values: deep, json, idOnly
    pub max_depth: Option<i32>,
    pub additional_parameters: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionDetails {
    pub exception_id: i32,
    pub text: String,
    pub line_number: i32,
    pub column_number: i32,
    // pub script_id: ScriptId,
    // pub url: Option<String>,
    // pub stack_trace: Option<StackTrace>,
    // pub exception: Option<RemoteObject>,
    // pub execution_context_id: Option<i32>,
    // pub exception_meta_data: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateResponse {
    pub result: Option<RemoteObject>,
    pub exception_details: Option<ExceptionDetails>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPropertiesResponse {
    pub result: Vec<PropertyDescriptor>,
    // pub internal_properties: Option<Vec<InternalPropertyDescriptor>>,
    // pub private_properties: Option<Vec<PrivatePropertyDescriptor>>,
    pub exception_details: Option<ExceptionDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EvaluateResult {
    Primitive(Value),
    Array(Vec<EvaluateResult>),
    Object(Value),
    Node(RemoteObjectId),
    Error(String),
}

impl EvaluateResult {
    pub fn from_remote_object(
        obj: RemoteObject,
        props: Option<Vec<PropertyDescriptor>>,
    ) -> Result<Self> {
        match obj.type_name.as_str() {
            "undefined" => Ok(Self::Primitive(Value::Bool(false))),
            "boolean" | "string" | "number" => {
                Ok(Self::Primitive(obj.value.unwrap_or(Value::Null)))
            }
            "bigint" => {
                let val = obj
                    .unserializable_value
                    .map(Value::String)
                    .unwrap_or(Value::Null);
                Ok(Self::Primitive(val))
            }
            "object" => match obj.subtype.as_deref() {
                Some("null") => Ok(Self::Primitive(Value::Bool(false))),
                Some("array") | Some("typedarray") => {
                    let mut items = Vec::new();
                    if let Some(descriptors) = props {
                        for prop in descriptors {
                            if prop.name.parse::<usize>().is_ok() {
                                if let Some(value) = prop.value {
                                    items.push(Self::from_remote_object(value, None)?);
                                }
                            }
                        }
                    }
                    Ok(Self::Array(items))
                }
                Some("node") | Some("element") => {
                    let id = obj
                        .object_id
                        .ok_or_else(|| anyhow!("Missing object_id for node/element"))?;
                    Ok(Self::Node(id))
                }
                Some("date") => {
                    let val = obj
                        .value
                        .clone()
                        .or_else(|| {
                            obj.unserializable_value
                                .as_ref()
                                .map(|s| Value::String(s.clone()))
                        })
                        .or_else(|| obj.description.as_ref().map(|s| Value::String(s.clone())))
                        .ok_or_else(|| anyhow!("Date object has no valid string representation"))?;
                    Ok(Self::Primitive(val))
                }
                Some("error") => {
                    let msg = obj
                        .description
                        .clone()
                        .or_else(|| {
                            obj.value
                                .as_ref()
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                        })
                        .unwrap_or_else(|| "Unknown error".to_string());
                    Ok(Self::Error(msg))
                }
                None => {
                    if let Some(descriptors) = props {
                        let mut map = serde_json::Map::new();
                        for p in descriptors {
                            let val = p
                                .value
                                .ok_or_else(|| anyhow!("Missing value in property"))?;
                            let nested = Self::from_remote_object(val, None)?;
                            map.insert(p.name, nested.into_inner());
                        }
                        Ok(Self::Object(Value::Object(map)))
                    } else {
                        Ok(Self::Object(Value::Null))
                    }
                }
                Some(other) => Ok(Self::Error(format!(
                    "Unsupported object subtype: {:?}",
                    other
                ))),
            },
            other => Ok(Self::Error(format!("Unsupported type: {}", other))),
        }
    }

    pub fn into_inner(self) -> Value {
        match self {
            Self::Primitive(val) => val,
            Self::Array(vec) => Value::Array(vec.into_iter().map(|v| v.into_inner()).collect()),
            Self::Object(obj) => obj,
            Self::Node(id) => Value::String(id),
            Self::Error(msg) => Value::String(msg),
        }
    }
}

#[derive(Serialize)]
pub struct AddBinding<'a> {
    #[serde(rename = "name")]
    pub name: &'a str,
    #[serde(rename = "executionContextId", skip_serializing_if = "Option::is_none")]
    pub execution_context_id: Option<&'a ExecutionContextId>,
    #[serde(rename = "binding", skip_serializing_if = "Option::is_none")]
    pub binding: Option<&'a str>,
}

impl<'a> AddBinding<'a> {
    // pub fn default(name: &'a str) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         name,
    //         execution_context_id: None,
    //         binding: None,
    //     })
    //     .unwrap()
    // }

    pub fn default(name: &'a str) -> Self {
        Self {
            name,
            execution_context_id: None,
            binding: None,
        }
    }

    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            execution_context_id: None,
            binding: None,
        }
    }

    pub fn execution_context_id(mut self, value: &'a ExecutionContextId) -> Self {
        self.execution_context_id = Some(value);
        self
    }

    pub fn binding(mut self, value: &'a str) -> Self {
        self.binding = Some(value);
        self
    }

    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct RemoveBinding<'a> {
    #[serde(rename = "name")]
    pub name: &'a str,
}

impl<'a> RemoveBinding<'a> {
    // pub fn default(name: &'a str) -> serde_json::Value {
    //     serde_json::to_value(Self { name }).unwrap()
    // }

    pub fn default(name: &'a str) -> Self {
        Self { name }
    }
}

#[derive(Serialize)]
pub struct RuntimeEnable {}

impl RuntimeEnable {
    // pub fn default() -> serde_json::Value {
    //     serde_json::to_value(Self {}).unwrap()
    // }

    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct RuntimeDisable {}

impl RuntimeDisable {
    // pub fn default() -> serde_json::Value {
    //     serde_json::to_value(Self {}).unwrap()
    // }

    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct CallFunctionOn<'a> {
    #[serde(rename = "functionDeclaration")]
    pub function_declaration: &'a str,
    #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<&'a RemoteObjectId>,
    #[serde(rename = "arguments", skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<&'a CallArgument>>,
    #[serde(rename = "silent", skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
    #[serde(rename = "returnByValue", skip_serializing_if = "Option::is_none")]
    pub return_by_value: Option<bool>,
    #[serde(rename = "generatePreview", skip_serializing_if = "Option::is_none")]
    pub generate_preview: Option<bool>,
    #[serde(rename = "userGesture", skip_serializing_if = "Option::is_none")]
    pub user_gesture: Option<bool>,
    #[serde(rename = "awaitPromise", skip_serializing_if = "Option::is_none")]
    pub await_promise: Option<bool>,
    #[serde(rename = "executionContextId", skip_serializing_if = "Option::is_none")]
    pub execution_context_id: Option<&'a ExecutionContextId>,
    #[serde(rename = "objectGroup", skip_serializing_if = "Option::is_none")]
    pub object_group: Option<&'a str>,
}

impl<'a> CallFunctionOn<'a> {
    // pub fn default(function_declaration: &'a str) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         function_declaration,
    //         object_id: None,
    //         arguments: None,
    //         silent: None,
    //         return_by_value: None,
    //         generate_preview: None,
    //         user_gesture: Some(true),
    //         await_promise: Some(true),
    //         execution_context_id: None,
    //         object_group: None,
    //     })
    //     .unwrap()
    // }

    pub fn default(function_declaration: &'a str) -> Self {
        Self {
            function_declaration,
            object_id: None,
            arguments: None,
            silent: None,
            return_by_value: None,
            generate_preview: None,
            user_gesture: Some(true),
            await_promise: Some(true),
            execution_context_id: None,
            object_group: None,
        }
    }

    pub fn new(function_declaration: &'a str) -> Self {
        Self {
            function_declaration,
            object_id: None,
            arguments: None,
            silent: None,
            return_by_value: None,
            generate_preview: None,
            user_gesture: Some(true),
            await_promise: Some(true),
            execution_context_id: None,
            object_group: None,
        }
    }

    pub fn object_id(mut self, value: &'a RemoteObjectId) -> Self {
        self.object_id = Some(value);
        self
    }
    pub fn arguments(mut self, value: Vec<&'a CallArgument>) -> Self {
        self.arguments = Some(value);
        self
    }
    pub fn silent(mut self, value: bool) -> Self {
        self.silent = Some(value);
        self
    }
    pub fn return_by_value(mut self, value: bool) -> Self {
        self.return_by_value = Some(value);
        self
    }
    pub fn generate_preview(mut self, value: bool) -> Self {
        self.generate_preview = Some(value);
        self
    }
    pub fn user_gesture(mut self, value: bool) -> Self {
        self.user_gesture = Some(value);
        self
    }

    pub fn await_promise(mut self, value: bool) -> Self {
        self.await_promise = Some(value);
        self
    }
    pub fn execution_context_id(mut self, value: &'a ExecutionContextId) -> Self {
        self.execution_context_id = Some(value);
        self
    }
    pub fn object_group(mut self, value: &'a str) -> Self {
        self.object_group = Some(value);
        self
    }
    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct Evaluate<'a> {
    #[serde(rename = "expression")]
    pub expression: &'a str,
    #[serde(rename = "objectGroup", skip_serializing_if = "Option::is_none")]
    pub object_group: Option<&'a str>,
    #[serde(
        rename = "includeCommandLineAPI",
        skip_serializing_if = "Option::is_none"
    )]
    pub include_command_line_api: Option<bool>,
    #[serde(rename = "silent", skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
    #[serde(rename = "contextId", skip_serializing_if = "Option::is_none")]
    pub context_id: Option<&'a ExecutionContextId>,
    #[serde(rename = "returnByValue", skip_serializing_if = "Option::is_none")]
    pub return_by_value: Option<bool>,
    #[serde(rename = "generatePreview", skip_serializing_if = "Option::is_none")]
    pub generate_preview: Option<bool>,
    #[serde(rename = "userGesture", skip_serializing_if = "Option::is_none")]
    pub user_gesture: Option<bool>,
    #[serde(rename = "awaitPromise", skip_serializing_if = "Option::is_none")]
    pub await_promise: Option<bool>,
    #[serde(rename = "timeout", skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
    #[serde(
        rename = "allowUnsafeEvalBlockedByCSP",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_unsafe_eval_blocked_by_csp: Option<bool>,
}

impl<'a> Evaluate<'a> {
    // pub fn default(expression: &'a str) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         expression,
    //         object_group: None,
    //         include_command_line_api: None,
    //         silent: None,
    //         context_id: None,
    //         return_by_value: None,
    //         generate_preview: None,
    //         user_gesture: Some(true),
    //         await_promise: Some(true),
    //         timeout: None,
    //         allow_unsafe_eval_blocked_by_csp: Some(true),
    //     })
    //     .unwrap()
    // }

    pub fn default(expression: &'a str) -> Self {
        Self {
            expression,
            object_group: None,
            include_command_line_api: None,
            silent: None,
            context_id: None,
            return_by_value: None,
            generate_preview: None,
            user_gesture: Some(true),
            await_promise: Some(true),
            timeout: None,
            allow_unsafe_eval_blocked_by_csp: Some(true),
        }
    }

    pub fn new(expression: &'a str) -> Self {
        Self {
            expression,
            object_group: None,
            include_command_line_api: None,
            silent: None,
            context_id: None,
            return_by_value: None,
            generate_preview: None,
            user_gesture: Some(true),
            await_promise: Some(true),
            timeout: None,
            allow_unsafe_eval_blocked_by_csp: Some(true),
        }
    }

    pub fn object_group(mut self, object_group: &'a str) -> Self {
        self.object_group = Some(object_group);
        self
    }
    pub fn include_command_line_api(mut self, value: bool) -> Self {
        self.include_command_line_api = Some(value);
        self
    }
    pub fn silent(mut self, value: bool) -> Self {
        self.silent = Some(value);
        self
    }
    pub fn context_id(mut self, context_id: &'a ExecutionContextId) -> Self {
        self.context_id = Some(context_id);
        self
    }
    pub fn return_by_value(mut self, value: bool) -> Self {
        self.return_by_value = Some(value);
        self
    }
    pub fn generate_preview(mut self, value: bool) -> Self {
        self.generate_preview = Some(value);
        self
    }
    pub fn user_gesture(mut self, value: bool) -> Self {
        self.user_gesture = Some(value);
        self
    }
    pub fn await_promise(mut self, value: bool) -> Self {
        self.await_promise = Some(value);
        self
    }
    pub fn timeout(mut self, value: f64) -> Self {
        self.timeout = Some(value);
        self
    }
    pub fn allow_unsafe_eval_blocked_by_csp(mut self, value: bool) -> Self {
        self.allow_unsafe_eval_blocked_by_csp = Some(value);
        self
    }
    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct GetProperties<'a> {
    #[serde(rename = "objectId")]
    pub object_id: &'a RemoteObjectId,
    #[serde(rename = "ownProperties", skip_serializing_if = "Option::is_none")]
    pub own_properties: Option<bool>,
    #[serde(
        rename = "accessorPropertiesOnly",
        skip_serializing_if = "Option::is_none"
    )]
    pub accessor_properties_only: Option<bool>,
    #[serde(rename = "generatePreview", skip_serializing_if = "Option::is_none")]
    pub generate_preview: Option<bool>,
    #[serde(
        rename = "nonIndexedPropertiesOnly",
        skip_serializing_if = "Option::is_none"
    )]
    pub non_indexed_properties_only: Option<bool>,
}

impl<'a> GetProperties<'a> {
    // pub fn default(object_id: &'a RemoteObjectId) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         object_id,
    //         own_properties: Some(true),
    //         accessor_properties_only: None,
    //         generate_preview: None,
    //         non_indexed_properties_only: None,
    //     })
    //     .unwrap()
    // }

    pub fn default(object_id: &'a RemoteObjectId) -> Self {
        Self {
            object_id,
            own_properties: Some(true),
            accessor_properties_only: None,
            generate_preview: None,
            non_indexed_properties_only: None,
        }
    }

    pub fn new(object_id: &'a RemoteObjectId) -> Self {
        Self {
            object_id,
            own_properties: Some(true),
            accessor_properties_only: None,
            generate_preview: None,
            non_indexed_properties_only: None,
        }
    }

    pub fn own_properties(mut self, value: bool) -> Self {
        self.own_properties = Some(value);
        self
    }

    pub fn accessor_properties_only(mut self, value: bool) -> Self {
        self.accessor_properties_only = Some(value);
        self
    }

    pub fn generate_preview(mut self, value: bool) -> Self {
        self.generate_preview = Some(value);
        self
    }

    pub fn non_indexed_properties_only(mut self, value: bool) -> Self {
        self.non_indexed_properties_only = Some(value);
        self
    }

    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct ReleaseObject<'a> {
    #[serde(rename = "objectId")]
    pub object_id: &'a RemoteObjectId,
}

impl<'a> ReleaseObject<'a> {
    // pub fn default(object_id: &'a RemoteObjectId) -> serde_json::Value {
    //     serde_json::to_value(Self { object_id }).unwrap()
    // }

    pub fn default(object_id: &'a RemoteObjectId) -> Self {
        Self { object_id }
    }
}
