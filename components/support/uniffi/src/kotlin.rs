/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::prelude::*;
use std::{
    env,
    collections::HashMap,
    convert::TryFrom, convert::TryInto,
    fs::File,
    iter::IntoIterator,
    fmt::Display,
    path::{Path, PathBuf},
};

use anyhow::bail;
use anyhow::Result;
use askama::Template;

use super::types;

#[derive(Template)]
#[template(ext="kt", escape="none", source=r#"
// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

// TODO: how to declare package namespace? Probably when calling the generator.
package mozilla.appservices.example;

import com.sun.jna.Library
import com.sun.jna.Pointer
import mozilla.appservices.support.native.RustBuffer
import mozilla.appservices.support.native.loadIndirect
import org.mozilla.appservices.fxaclient.BuildConfig

internal typealias Handle = Long
//TODO error type specific to this component.

@Suppress("FunctionNaming", "FunctionParameterNaming", "LongParameterList", "TooGenericExceptionThrown")
internal interface LibTODOName : Library {
    companion object {
        internal var INSTANCE: LibTODOName =
            loadIndirect(componentName = "name", componentVersion = BuildConfig.LIBRARY_VERSION)
    }
    {% for m in self.members() -%}
    {{ m.render().unwrap() }}
    {% endfor -%}
}
"#)]
pub struct ComponentInterfaceKotlinWrapper<'a> {
  ci: &'a types::ComponentInterface,
}
impl<'a> ComponentInterfaceKotlinWrapper<'a> {
    pub fn new(ci: &'a types::ComponentInterface) -> Self {
        Self { ci }
    }

    // XXX TODO: I'd like to make this return `impl Iterator` but then askama takes a reference to it
    // and tries to call `into_iter()` and that doesn't work becaue it tries to do a move. Bleh. More to learn...
    fn members(&self) -> Vec<Box<dyn askama::Template + 'a>> {
        self.ci.members.iter().map(|m| {
            match m {
                types::InterfaceMember::Object(obj) => ObjectKotlinWrapper::boxed(self.ci, obj),
                types::InterfaceMember::Record(rec) => RecordKotlinWrapper::boxed(self.ci, rec),
                types::InterfaceMember::Namespace(n) => NamespaceKotlinWrapper::boxed(self.ci, n),
                types::InterfaceMember::Enum(e) => EnumKotlinWrapper::boxed(self.ci, e),
            }
        }).collect()
    }
}

#[derive(Template)]
#[template(ext="kt", escape="none", source=r#"

fun {{ self.struct_name() }}_free(hande: Handle, err: RustError.ByReference)

{%- for m in self.members() %}
    {{ m.render().unwrap() }}
{% endfor -%}
"#)]
struct ObjectKotlinWrapper<'a> {
    ci: &'a types::ComponentInterface,
    obj: &'a types::ObjectType,
}
impl<'a> ObjectKotlinWrapper<'a> {
    fn boxed(ci: &'a types::ComponentInterface, obj: &'a types::ObjectType) -> Box<dyn askama::Template + 'a> {
        Box::new(Self { ci, obj })
    }

    fn struct_name(&self) -> &'a str {
        &self.obj.name
    }

    fn members(&self) -> Vec<Box<dyn askama::Template + 'a>> {
        self.obj.members.iter().map(|m|{
            match m {
                types::ObjectTypeMember::Constructor(cons) => ObjectConstructorKotlinWrapper::boxed(self.ci, self.obj, cons),
                types::ObjectTypeMember::Method(meth) => ObjectMethodKotlinWrapper::boxed(self.ci, self.obj, meth),
            }
        }).collect()
    }
}
#[derive(Template)]
#[template(ext="kt", escape="none", source=r#"
fun {{ self.ffi_name() }}(
  {%- for arg in cons.argument_types %}
    {{ arg.ffi_name() }}: {{ arg.typ.resolve(ci)|type_decl }},
  {%- endfor %}
  err: RustError.ByReference,
): Handle
"#)]
struct ObjectConstructorKotlinWrapper<'a> {
    ci: &'a types::ComponentInterface,
    obj: &'a types::ObjectType,
    cons: &'a types::ObjectTypeConstructor,
}

impl<'a> ObjectConstructorKotlinWrapper<'a> {
    fn boxed(ci: &'a types::ComponentInterface, obj: &'a types::ObjectType, cons: &'a types::ObjectTypeConstructor) -> Box<dyn askama::Template + 'a> {
        Box::new(Self { ci, obj, cons })
    }

    fn ffi_name(&self) -> String {
        format!("{}_{}", self.obj.name, self.cons.name)
    }
}

#[derive(Template)]
#[template(ext="rs", escape="none", source=r#"
fun {{ self.ffi_name() }}(
  handle: Handle,
  {%- for arg in meth.argument_types %}
    {{ arg.ffi_name() }}: {{ arg.typ.resolve(ci)|type_decl }},
  {%- endfor %}
  err: RustError.ByReference,
)
  {%- match meth.return_type -%}
  {%- when Some with (typ) %}
    : {{ typ.resolve(ci)|type_decl }}
  {% when None -%}
  {%- endmatch %}
"#)]
struct ObjectMethodKotlinWrapper<'a> {
    ci: &'a types::ComponentInterface,
    obj: &'a types::ObjectType,
    meth: &'a types::ObjectTypeMethod,
}

impl<'a> ObjectMethodKotlinWrapper<'a> {
    fn boxed(ci: &'a types::ComponentInterface, obj: &'a types::ObjectType, meth: &'a types::ObjectTypeMethod) -> Box<dyn askama::Template + 'a> {
        Box::new(Self {ci, obj, meth })
    }

    fn ffi_name(&self) -> String {
        format!("{}_{}", self.obj.name, self.meth.name)
    }
}

#[derive(Template)]
#[template(ext="rs", escape="none", source=r#"
TODO: RECORD {{ rec.name }}
"#)]
struct RecordKotlinWrapper<'a> {
    ci: &'a types::ComponentInterface,
    rec: &'a types::RecordType,
}

impl<'a> RecordKotlinWrapper<'a> {
    fn boxed(ci: &'a types::ComponentInterface, rec: &'a types::RecordType) -> Box<dyn askama::Template + 'a> {
        Box::new(Self { ci, rec })
    }
}

#[derive(Template)]
#[template(ext="rs", escape="none", source=r#"
{%- for m in self.members() %}
    {{ m.render().unwrap() }}
{% endfor -%}
"#)]
struct NamespaceKotlinWrapper<'a> {
    ci: &'a types::ComponentInterface,
    ns: &'a types::NamespaceType,
}
impl<'a> NamespaceKotlinWrapper<'a> {
    fn boxed(ci: &'a types::ComponentInterface, ns: &'a types::NamespaceType) -> Box<dyn askama::Template + 'a> {
        Box::new(Self { ci, ns })
    }

    fn members(&self) -> Vec<Box<dyn askama::Template + 'a>> {
        self.ns.members.iter().map(|m|{
            match m {
                types::NamespaceTypeMember::Function(f) => NamespaceFunctionKotlinWrapper::boxed(self.ci, self.ns, f),
            }
        }).collect()
    }
}

#[derive(Template)]
#[template(ext="rs", escape="none", source=r#"
fun {{ f.ffi_name() }}(
  {%- for arg in f.argument_types %}
    {{ arg.ffi_name() }}: {{ arg.typ.resolve(ci)|type_decl }},
  {%- endfor %}
  // TODO: error param
)
  {%- match f.return_type -%}
  {%- when Some with (typ) %}
    : {{ typ.resolve(ci)|type_decl }}
  {% when None -%}
  {%- endmatch %}
"#)]
struct NamespaceFunctionKotlinWrapper<'a> {
    ci: &'a types::ComponentInterface,
    ns: &'a types::NamespaceType,
    f: &'a types::NamespaceTypeFunction,
}

impl<'a> NamespaceFunctionKotlinWrapper<'a> {
    fn boxed(ci: &'a types::ComponentInterface, ns: &'a types::NamespaceType, f: &'a types::NamespaceTypeFunction) -> Box<dyn askama::Template + 'a> {
        Box::new(Self { ci, ns, f })
    }
}

#[derive(Template)]
#[template(ext="rs", escape="none", source=r#"
TODO: enum {{e.name}}
"#)]
struct EnumKotlinWrapper<'a> {
    ci: &'a types::ComponentInterface,
    e: &'a types::EnumType,
}

impl<'a> EnumKotlinWrapper<'a> {
    fn boxed(ci: &'a types::ComponentInterface, e: &'a types::EnumType) -> Box<dyn askama::Template + 'a> {
        Box::new(Self { ci, e })
    }
}

mod filters {
    use std::fmt;
    use crate::types;
    use anyhow::Error;

    pub fn type_decl(typ: &Result<types::TypeReference, Error>) -> Result<String, askama::Error> {
        // TODO: how to return a nice askama::Error here?
        let typ = typ.as_ref().unwrap();
        Ok(match typ {
            types::TypeReference::Boolean => "u8".to_string(),
            types::TypeReference::U64 => "u64".to_string(),
            types::TypeReference::U32 => "u32".to_string(),
            types::TypeReference::Enum(_) => "u32".to_string(),
            types::TypeReference::String => "FfiStr<'_>".to_string(), // XXX TODO: not suitable for use in return position?
            _ => format!("[TODO: DECL {:?}]", typ),
        })
    }

    pub fn type_lift(nm: &dyn fmt::Display, typ: &Result<types::TypeReference, Error>) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        let typ = typ.as_ref().unwrap();
        Ok(match typ {
            types::TypeReference::Boolean => format!("({} != 0)", nm),
            types::TypeReference::U64 => nm,
            types::TypeReference::U32 => nm,
            // XXX TODO: error handling if the conversion fails.
            types::TypeReference::Enum(_) => format!("({}).try_into().unwrap()", nm),
            types::TypeReference::String => format!("({}).as_str()", nm),
            _ => format!("[TODO: LIFT {:?}]", typ),
        })
    }

    pub fn type_lower( nm: &dyn fmt::Display, typ: &Result<types::TypeReference, Error>) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        let typ = typ.as_ref().unwrap();
        Ok(match typ {
            types::TypeReference::Boolean => format!("({} as u8)", nm),
            types::TypeReference::U64 => nm,
            types::TypeReference::U32 => nm,
            types::TypeReference::Enum(_) => format!("({} as u32)", nm),
            types::TypeReference::String => format!("({}).as_str()", nm),
            _ => format!("[TODO: LOWER {:?}]", typ),
        })
    }
}