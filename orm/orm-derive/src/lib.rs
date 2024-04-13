use proc_macro::TokenStream;
use syn::{parse_macro_input, Attribute, Data, DeriveInput};

#[proc_macro_derive(Object, attributes(table_name, column_name))]
pub fn derive_object(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let struct_name = input.ident.to_string();

    let struct_ = match input.data {
        Data::Struct(struct_) => struct_,
        _ => panic!("Expected Data::Struct"),
    };

    fn get_attr(attrs: &[Attribute]) -> Option<String> {
        let code = attrs.iter().next()?;
        let token = code.tokens.clone().into_iter().next()?;
        let x = token.to_string();
        let attr = x.as_str().strip_prefix("(\"")?.strip_suffix("\")")?;
        if attr.is_empty() {
            return None;
        }
        Some(String::from(attr))
    }

    let table_name = match get_attr(&input.attrs) {
        Some(s) => s,
        None => struct_name.clone(),
    };

    let DeriveInput { ident, .. } = input;

    let syn::Fields::Named(fields) = struct_.fields else {
            let impl_ = quote::quote! {
                impl orm::object::Object for #ident {
                    const SCHEMA : orm::object::Schema = orm::object::Schema {
                        object_name: #struct_name,
                        table_name: #table_name,
                        fields: &[],
                    };

                    fn to_row(&self) -> orm::storage::Row {
                        vec![]
                    }

                    fn from_row(row : &orm::storage::RowSlice) -> Self {
                        Self {}
                    }
            
                    fn schema() -> orm::object::Schema {
                        <#ident>::SCHEMA
                    }
                }
            };
            return TokenStream::from(impl_);
    };

    let mut fields_name = Vec::new();

    let impl_fields = fields.named.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        fields_name.push(field_ident);
        let field_name = field_ident.to_string();
        let field_column = match get_attr(&field.attrs) {
            None => field_name.clone(),
            Some(s) => s,
        };
        let syn::Type::Path(path) = &field.ty else {
            panic!("Expected Type::Path");
        };
        let field_type = path.path
            .clone()
            .segments
            .into_iter()
            .next()
            .unwrap()
            .ident
            .to_string();
        let field_type_ident = quote::format_ident!("{}", 
            match field_type.as_str() {
                "String" => "String",
                "Vec" => "Bytes",
                "i64" => "Int64",
                "f64" => "Float64",
                "bool" => "Bool",
                _ => panic!("Unexepted type for deraiving implementation trait Object"),
            }
        );
        quote::quote! { 
            orm::object::Field {
                feild_name: #field_name,
                column_name: #field_column,
                feild_type: orm::data::DataType::#field_type_ident,
            } 
        }
    }).collect::<Vec<_>>();

    let fields_to_row = fields.named.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        quote::quote! {
            orm::data::Value::from(&self.#field_ident)
        }
    }).collect::<Vec<_>>();

    let impl_ = quote::quote! { 
        impl orm::object::Object for #ident {
            const SCHEMA : orm::object::Schema = orm::object::Schema {
                object_name: #struct_name,
                table_name: #table_name,
                fields: &[#(#impl_fields),*],
            };

            fn to_row(&self) -> orm::storage::Row {
                vec![#(#fields_to_row),*]
            }

            fn from_row(row: &orm::storage::RowSlice) -> Self {
                let mut values: std::slice::Iter<orm::data::Value> = row.iter();
                Self {
                    #(
                        #fields_name: values.next().unwrap().into()
                    ),*
                }
            }

            fn schema() -> orm::object::Schema {
                <#ident>::SCHEMA
            }
        }
    };
    
    impl_.into()
}

