use config::Config;
use mirror_mirror::type_info::{GetMeta, ScalarType, Type};
use mirror_mirror::{Reflect, TypeDescriptor};
use toml::Value;

const SPACE: &str = "";

fn main() {
    let config = Config::default();
    let ty_info = config.as_reflect().type_descriptor();

    let toml_value = toml::to_string(&config).unwrap();
    let toml_value = toml::from_str::<Value>(&toml_value).unwrap();

    let nix_options = generate_nix_options(ty_info.as_ref(), &toml_value, 1);
    let nix_options = format!("{{ lib, ... }}: with lib; {{\n{nix_options}}}\n");

    if let Some(file) = std::env::args().nth(1) {
        std::fs::write(file, nix_options).unwrap();
    } else {
        println!("{nix_options}");
    }
}

fn generate_nix_options(schema: &TypeDescriptor, toml_value: &Value, indent: usize) -> String {
    match schema.get_type() {
        Type::Struct(struct_info) => {
            let mut nix_code = String::new();
            for field in struct_info.field_types() {
                let field_name = field.name();
                let field_ty = field.get_type();
                let field_docs = field.docs();

                let value = Value::String(String::new());
                let field_value = toml_value.get(field_name).unwrap_or(&value);

                nix_code.push_str(&format!(
                    "{SPACE:indent$}{} = mkOption {{\n",
                    field_name,
                    indent = indent * 2
                ));
                nix_code.push_str(&format!(
                    "{SPACE:indent$}type = {};\n",
                    infer_nix_type(&field_ty, toml_value, indent + 1),
                    indent = (indent + 1) * 2,
                ));
                if let Type::Enum(info) = &field_ty {
                    let t = toml_to_nix(field_value, indent + 1);
                    if info.type_name().starts_with("core::option::Option") && !t.is_empty() {
                        nix_code.push_str(&format!(
                            "{SPACE:indent$}default = {t};\n",
                            indent = (indent + 1) * 2,
                        ));
                    }
                }
                if field_docs.len() > 1 {
                    nix_code.push_str(&format!(
                        "{SPACE:indent$}description = {docs};\n",
                        docs = format!(
                            "''{}''",
                            field_docs
                                .iter()
                                .map(|d| d.trim())
                                .collect::<Vec<_>>()
                                .join("\n")
                        ),
                        indent = (indent + 1) * 2,
                    ));
                } else if field_docs.len() == 1 {
                    nix_code.push_str(&format!(
                        "{SPACE:indent$}description = {docs};\n",
                        docs = format!("{:?}", field_docs.first().unwrap().trim()),
                        indent = (indent + 1) * 2,
                    ));
                }
                nix_code.push_str(&format!("{SPACE:indent$}}};\n", indent = indent * 2));
            }
            nix_code
        }
        Type::Enum(enum_info) => enum_info
            .variants()
            .map(|v| format!("\"{}\"", v.name()))
            .collect::<Vec<_>>()
            .join(" "),
        _ => panic!("Type not supported: {:?}", schema),
    }
}

fn infer_nix_type(type_info: &Type<'_>, toml_value: &Value, indent: usize) -> String {
    match type_info {
        Type::Scalar(s) => match s {
            ScalarType::usize
            | ScalarType::u8
            | ScalarType::u16
            | ScalarType::u32
            | ScalarType::u64
            | ScalarType::u128
            | ScalarType::i8
            | ScalarType::i16
            | ScalarType::i32
            | ScalarType::i64
            | ScalarType::i128 => "types.number".to_string(),
            ScalarType::bool => "types.bool".to_string(),
            ScalarType::f32 | ScalarType::f64 => "types.float".to_string(),
            ScalarType::char | ScalarType::String => "types.string".to_string(),
        },
        Type::Struct(s) => {
            let submodule_code =
                generate_nix_options(&s.into_type_descriptor(), toml_value, indent + 2);
            format!(
                "types.submodule {{\n{SPACE:i$}options = {{\n{submodule_code}{SPACE:i$}}};\n{SPACE:n$}}}",
                n = indent * 2,
                i = (indent + 1) * 2,
            )
        }
        Type::Enum(enum_info) => {
            if enum_info.type_name().starts_with("core::option::Option") {
                let inner_type = enum_info
                    .variants()
                    .last()
                    .unwrap()
                    .field_types()
                    .next()
                    .unwrap()
                    .get_type();
                let inner_type = infer_nix_type(&inner_type, toml_value, indent);
                format!("types.nullOr {inner_type}")
            } else {
                let enum_code = enum_info
                    .variants()
                    .map(|v| format!("\"{}\"", v.name()))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("types.enum [ {enum_code} ]")
            }
        }
        Type::List(list_info) => {
            let item_type = infer_nix_type(&list_info.element_type(), toml_value, indent);
            format!("types.listOf {item_type}")
        }
        Type::TupleStruct(tuple_struct_info) => infer_nix_type(
            &tuple_struct_info.field_type_at(0).unwrap().get_type(),
            toml_value,
            indent,
        ),
        Type::Map(_map_info) => {
            format!("types.attrs")
        }
        _ => panic!("Type not supported: {:?}", type_info),
    }
}

fn toml_to_nix(value: &Value, indent: usize) -> String {
    match value {
        Value::String(s) => {
            if s.is_empty() {
                "null".to_string()
            } else {
                format!("\"{s}\"")
            }
        }
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => {
            if f.is_nan() {
                "null".to_string()
            } else if f.is_infinite() {
                if f.is_sign_positive() {
                    "null".to_string()
                } else {
                    "null".to_string()
                }
            } else {
                format!("{f:.1}")
            }
        }
        Value::Boolean(b) => b.to_string(),
        Value::Datetime(dt) => format!("\"{dt}\""),
        Value::Array(arr) => {
            let items = arr
                .iter()
                .map(|v| toml_to_nix(v, indent + 1))
                .collect::<Vec<_>>()
                .join(" ");
            format!("[ {} ]", items)
        }
        Value::Table(map) => {
            let mut nix_code = String::new();
            for (key, val) in map {
                nix_code.push_str(&format!(
                    "{SPACE:indent$}{} = {};\n",
                    if key.chars().next().unwrap().is_ascii_digit() {
                        format!("\"{key}\"")
                    } else {
                        key.to_string()
                    },
                    toml_to_nix(val, indent + 1),
                    indent = (indent + 1) * 2,
                ));
            }
            format!("{{\n{nix_code}{SPACE:indent$}}}", indent = indent * 2)
        }
    }
}
