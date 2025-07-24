use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, DeriveInput, Data, Fields, FieldsNamed,
    Attribute, Meta, Type, 
};

/// Derive macro for automatically implementing the RenderValue trait
/// 
/// # Examples
/// 
/// ```ignore
/// use html_template::{RenderValue, Renderable};
/// 
/// #[derive(Renderable)]
/// struct Person {
///     name: String,
///     age: u32,
///     #[renderable(rename = "emailAddress")]
///     email: String,
///     #[renderable(skip)]
///     password: String,
/// }
/// ```
#[proc_macro_derive(Renderable, attributes(renderable))]
pub fn derive_renderable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    match generate_renderable_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_renderable_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    let data = match &input.data {
        Data::Struct(data) => data,
        _ => return Err(syn::Error::new_spanned(input, "Renderable can only be derived for structs")),
    };
    
    let fields = match &data.fields {
        Fields::Named(fields) => fields,
        Fields::Unnamed(_) => return Err(syn::Error::new_spanned(input, "Renderable requires named fields")),
        Fields::Unit => return Err(syn::Error::new_spanned(input, "Renderable cannot be derived for unit structs")),
    };
    
    let field_handlers = generate_field_handlers(fields)?;
    let array_check = generate_array_check(fields);
    let array_impl = generate_array_impl(fields);
    let type_impl = generate_type_impl(input);
    let id_impl = generate_id_impl(fields);
    let get_value_impl = generate_get_value_impl(fields);
    
    Ok(quote! {
        impl #impl_generics html_template::RenderValue for #name #ty_generics #where_clause {
            fn get_property(&self, path: &[String]) -> Option<std::borrow::Cow<str>> {
                if path.is_empty() {
                    return None;
                }
                
                #field_handlers
                
                None
            }
            
            #array_check
            
            #array_impl
            
            #type_impl
            
            #id_impl
            
            #get_value_impl
        }
    })
}

fn generate_field_handlers(fields: &FieldsNamed) -> syn::Result<TokenStream2> {
    let mut field_matches = Vec::new();
    
    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        
        let attrs = parse_field_attributes(&field.attrs)?;
        
        // Skip fields marked with #[renderable(skip)]
        if attrs.skip {
            continue;
        }
        
        // Use custom name if provided, otherwise use field name
        let property_name = attrs.rename.unwrap_or(field_name_str);
        
        let field_type = &field.ty;
        
        // Handle different types of fields
        let value_expr = if is_option_type(field_type) {
            quote! {
                if let Some(ref value) = self.#field_name {
                    if path.len() == 1 {
                        Some(value.to_string().into())
                    } else {
                        // For nested access, try to call get_property if it implements RenderValue
                        None
                    }
                } else {
                    Some("".into())
                }
            }
        } else if is_vec_type(field_type) {
            quote! {
                if path.len() == 1 {
                    Some(self.#field_name.len().to_string().into())
                } else {
                    // Array access not implemented in this simplified version
                    None
                }
            }
        } else if is_string_type(field_type) {
            quote! {
                if path.len() == 1 {
                    Some(self.#field_name.clone().into())
                } else {
                    None
                }
            }
        } else {
            quote! {
                if path.len() == 1 {
                    Some(self.#field_name.to_string().into())
                } else {
                    // For nested access, we'd need the field to implement RenderValue
                    None
                }
            }
        };
        
        field_matches.push(quote! {
            if path[0] == #property_name {
                return #value_expr;
            }
        });
    }
    
    Ok(quote! {
        #(#field_matches)*
    })
}

fn generate_array_check(fields: &FieldsNamed) -> TokenStream2 {
    // Check if any field is a Vec type
    let has_array_field = fields.named.iter().any(|field| {
        let attrs = parse_field_attributes(&field.attrs).unwrap_or_default();
        !attrs.skip && is_vec_type(&field.ty)
    });
    
    quote! {
        fn is_array(&self) -> bool {
            #has_array_field
        }
    }
}

fn generate_array_impl(fields: &FieldsNamed) -> TokenStream2 {
    let mut array_handling = Vec::new();
    
    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let attrs = parse_field_attributes(&field.attrs).unwrap_or_default();
        
        if attrs.skip || !is_vec_type(&field.ty) {
            continue;
        }
        
        let property_name = attrs.rename.unwrap_or_else(|| field_name.to_string());
        
        array_handling.push(quote! {
            if !items.is_empty() {
                // Check if this is an array property
                if let Some(first_item) = items.first() {
                    if let Some(property) = first_item.get_property(&[#property_name.to_string()]) {
                        let result: Vec<&dyn html_template::RenderValue> = self.#field_name.iter()
                            .map(|item| item as &dyn html_template::RenderValue)
                            .collect();
                        return Some(result);
                    }
                }
            }
        });
    }
    
    quote! {
        fn as_array(&self) -> Option<Vec<&dyn html_template::RenderValue>> {
            let items: Vec<&dyn html_template::RenderValue> = vec![self];
            #(#array_handling)*
            None
        }
    }
}

fn generate_type_impl(input: &DeriveInput) -> TokenStream2 {
    let type_name = input.ident.to_string();
    
    quote! {
        fn get_type(&self) -> Option<&str> {
            Some(#type_name)
        }
    }
}

fn generate_id_impl(fields: &FieldsNamed) -> TokenStream2 {
    // Look for a field named "id" or marked with #[renderable(id)]
    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let attrs = parse_field_attributes(&field.attrs).unwrap_or_default();
        
        if attrs.skip {
            continue;
        }
        
        if attrs.id || field_name == "id" {
            // For string fields, we can return a reference directly
            if is_string_type(&field.ty) {
                return quote! {
                    fn get_id(&self) -> Option<&str> {
                        Some(&self.#field_name)
                    }
                };
            } else {
                // For non-string fields, we can't return a reference to a temporary
                return quote! {
                    fn get_id(&self) -> Option<&str> {
                        None // Non-string IDs not supported yet
                    }
                };
            }
        }
    }
    
    quote! {
        fn get_id(&self) -> Option<&str> {
            None
        }
    }
}

fn generate_get_value_impl(_fields: &FieldsNamed) -> TokenStream2 {
    quote! {
        fn get_value(&self, path: &[String]) -> Option<&dyn html_template::RenderValue> {
            if path.is_empty() {
                Some(self)
            } else {
                None
            }
        }
    }
}

#[derive(Default)]
struct FieldAttributes {
    skip: bool,
    rename: Option<String>,
    id: bool,
}

fn parse_field_attributes(attrs: &[Attribute]) -> syn::Result<FieldAttributes> {
    let mut result = FieldAttributes::default();
    
    for attr in attrs {
        if !attr.path().is_ident("renderable") {
            continue;
        }
        
        match &attr.meta {
            Meta::List(meta_list) => {
                // Parse the meta list manually
                let content = meta_list.tokens.to_string();
                
                // Simple parsing for common cases
                if content.trim() == "skip" {
                    result.skip = true;
                } else if content.trim() == "id" {
                    result.id = true;
                } else if content.starts_with("rename") {
                    // Parse rename = "value"
                    if let Some(start) = content.find('"') {
                        if let Some(end) = content.rfind('"') {
                            if start < end {
                                let value = &content[start + 1..end];
                                result.rename = Some(value.to_string());
                            }
                        }
                    }
                } else {
                    // Try to parse as comma-separated attributes
                    for part in content.split(',') {
                        let part = part.trim();
                        if part == "skip" {
                            result.skip = true;
                        } else if part == "id" {
                            result.id = true;
                        } else if part.starts_with("rename") {
                            if let Some(start) = part.find('"') {
                                if let Some(end) = part.rfind('"') {
                                    if start < end {
                                        let value = &part[start + 1..end];
                                        result.rename = Some(value.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Meta::Path(path) if path.is_ident("skip") => {
                result.skip = true;
            }
            _ => {
                return Err(syn::Error::new_spanned(attr, "Invalid renderable attribute"));
            }
        }
    }
    
    Ok(result)
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn is_vec_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Vec";
        }
    }
    false
}

fn is_string_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "String";
        }
    }
    false
}