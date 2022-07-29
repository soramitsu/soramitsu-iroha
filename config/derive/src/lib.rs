//! Contains the `#[derive(Configurable)]` macro definition.

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Data, DataStruct, DeriveInput, Fields, GenericArgument, Ident, Lit, LitStr, Meta,
    PathArguments, Token, Type, TypePath,
};

struct EnvPrefix {
    _ident: Ident,
    _eq: Token![=],
    prefix: LitStr,
}

mod attrs {
    pub const ENV_PREFIX: &str = "env_prefix";
    pub const SERDE_AS_STR: &str = "serde_as_str";
    pub const INNER: &str = "inner";
}

fn get_type_argument<'sl, 'tl>(s: &'sl str, ty: &'tl Type) -> Option<&'tl GenericArgument> {
    let path = if let Type::Path(r#type) = ty {
        r#type
    } else {
        return None;
    };
    let segments = &path.path.segments;
    if segments.len() != 1 || segments[0].ident != s {
        return None;
    }

    if let PathArguments::AngleBracketed(bracketed_arguments) = &segments[0].arguments {
        if bracketed_arguments.args.len() == 1 {
            return Some(&bracketed_arguments.args[0]);
        }
    }
    None
}

fn is_arc_rwlock(ty: &Type) -> bool {
    #[allow(clippy::shadow_unrelated)]
    let dearced_ty = get_type_argument("Arc", ty)
        .and_then(|ty| {
            if let GenericArgument::Type(r#type) = ty {
                Some(r#type)
            } else {
                None
            }
        })
        .unwrap_or(ty);
    get_type_argument("RwLock", dearced_ty).is_some()
}

// TODO: make it const generic type once it will be stabilized
fn parse_const_ident(input: ParseStream, ident: &'static str) -> syn::Result<Ident> {
    let parse_ident: Ident = input.parse()?;
    if parse_ident == ident {
        Ok(parse_ident)
    } else {
        Err(syn::Error::new_spanned(parse_ident, "Unknown ident"))
    }
}

impl Parse for EnvPrefix {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _ident: parse_const_ident(input, attrs::ENV_PREFIX)?,
            _eq: input.parse()?,
            prefix: input.parse()?,
        })
    }
}

struct Inner {
    _ident: Ident,
}

impl Parse for Inner {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _ident: parse_const_ident(input, attrs::INNER)?,
        })
    }
}

struct SerdeAsStr {
    _ident: Ident,
}

impl Parse for SerdeAsStr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _ident: parse_const_ident(input, attrs::SERDE_AS_STR)?,
        })
    }
}

/// Derive for config. Check other doc in `iroha_config` reexport
#[proc_macro_derive(Configurable, attributes(config))]
pub fn configurable_derive(input: TokenStream) -> TokenStream {
    let ast = match syn::parse(input) {
        Ok(ast) => ast,
        Err(err) => {
            abort_call_site!("Failed to parse input Token Stream: {}", err)
        }
    };
    impl_configurable(&ast)
}

fn impl_load_env(
    field_idents: &[&Ident],
    inner: &[bool],
    lvalue: &[proc_macro2::TokenStream],
    as_str: &[bool],
    field_ty: &[Type],
    field_environment: &[String],
) -> proc_macro2::TokenStream {
    let set_field = field_ty
        .iter()
        .zip(field_idents.iter())
        .zip(as_str.iter())
        .zip(lvalue.iter())
        .map(|(((ty, ident), &as_str_attr), l_value)| {
            let is_string = if let Type::Path(TypePath { path, .. }) = ty {
                path.is_ident("String")
            } else {
                false
            };
            let set_field = if is_string {
                quote! { #l_value = var }
            } else if as_str_attr {
                quote! {
                    #l_value = serde_json::from_value(var.into())
                        .map_err(|e| iroha_config::derive::Error::field_error(stringify!(#ident), e))?
                }
            } else {
                quote! {
                    #l_value = serde_json::from_str(&var)
                        .map_err(|e| iroha_config::derive::Error::field_error(stringify!(#ident), e))?
                }
            };
            (set_field, l_value)
        })
        .zip(field_environment.iter())
        .zip(inner.iter())
        .map(|(((set_field, l_value), field_env), &inner_thing)| {
            let inner_thing2 = if inner_thing {
                quote! {
                    #l_value.load_environment()?;
                }
            } else {
                quote! {}
            };
            quote! {
                if let Ok(var) = std::env::var(#field_env) {
                    #set_field;
                }
                #inner_thing2
            }
        });

    quote! {
        fn load_environment(
            &'_ mut self
        ) -> core::result::Result<(), iroha_config::derive::Error> {
            #(#set_field)*
            Ok(())
        }
    }
}

fn impl_get_doc_recursive(
    field_ty: &[Type],
    field_idents: &[&Ident],
    inner: Vec<bool>,
    docs: Vec<LitStr>,
) -> proc_macro2::TokenStream {
    if field_idents.is_empty() {
        return quote! {
            fn get_doc_recursive<'a>(
                inner_field: impl AsRef<[&'a str]>,
            ) -> core::result::Result<std::option::Option<String>, iroha_config::derive::Error>
            {
                Err(iroha_config::derive::Error::UnknownField(
                    inner_field.as_ref().iter().map(ToString::to_string).collect()
                ))
            }
        };
    }
    let variants = field_idents
        .iter()
        .zip(inner)
        .zip(docs)
        .zip(field_ty)
        .map(|(((ident, inner_thing), documentation), ty)| {
            if inner_thing {
                quote! {
                    [stringify!(#ident)] => {
                        let curr_doc = #documentation;
                        let inner_docs = <#ty as iroha_config::Configurable>::get_inner_docs();
                        let total_docs = format!("{}\n\nHas following fields:\n\n{}\n", curr_doc, inner_docs);
                        Some(total_docs)
                    },
                    [stringify!(#ident), rest @ ..] => <#ty as iroha_config::Configurable>::get_doc_recursive(rest)?,
                }
            } else {
                quote! { [stringify!(#ident)] => Some(#documentation.to_owned()), }
            }
        })
        // XXX: Workaround
        //Decription of issue is here https://stackoverflow.com/a/65353489
        .fold(quote! {}, |acc, new| quote! { #acc #new });

    quote! {
        fn get_doc_recursive<'a>(
            inner_field: impl AsRef<[&'a str]>,
        ) -> core::result::Result<std::option::Option<String>, iroha_config::derive::Error>
        {
            let inner_field = inner_field.as_ref();
            let doc = match inner_field {
                #variants
                field => return Err(iroha_config::derive::Error::UnknownField(
                    field.iter().map(ToString::to_string).collect()
                )),
            };
            Ok(doc)
        }
    }
}

fn impl_get_inner_docs(
    field_ty: &[Type],
    field_idents: &[&Ident],
    inner: Vec<bool>,
    docs: Vec<LitStr>,
) -> proc_macro2::TokenStream {
    let inserts = field_idents
        .iter()
        .zip(inner)
        .zip(docs)
        .zip(field_ty)
        .map(|(((ident, inner_thing), documentation), ty)| {
            let doc = if inner_thing {
                quote!{ <#ty as iroha_config::Configurable>::get_inner_docs().as_str() }
            } else {
                quote!{ #documentation.into() }
            };

            quote! {
                inner_docs.push_str(stringify!(#ident));
                inner_docs.push_str(": ");
                inner_docs.push_str(#doc);
                inner_docs.push_str("\n\n");
            }
        })
        // XXX: Workaround
        //Description of issue is here https://stackoverflow.com/a/65353489
        .fold(quote! {}, |acc, new| quote! { #acc #new });

    quote! {
        fn get_inner_docs() -> String {
            let mut inner_docs = String::new();
            #inserts
            inner_docs
        }
    }
}

fn impl_get_docs(
    field_ty: &[Type],
    field_idents: &[&Ident],
    inner: Vec<bool>,
    docs: Vec<LitStr>,
) -> proc_macro2::TokenStream {
    let inserts = field_idents
        .iter()
        .zip(inner)
        .zip(docs)
        .zip(field_ty)
        .map(|(((ident, inner_thing), documentation), ty)| {
            let doc = if inner_thing {
                quote!{ <#ty as iroha_config::Configurable>::get_docs().into() }
            } else {
                quote!{ #documentation.into() }
            };

            quote! { map.insert(stringify!(#ident).to_owned(), #doc); }
        })
        // XXX: Workaround
        //Decription of issue is here https://stackoverflow.com/a/65353489
        .fold(quote! {}, |acc, new| quote! { #acc #new });

    quote! {
        fn get_docs() -> serde_json::Value {
            let mut map = serde_json::Map::new();
            #inserts
            map.into()
        }
    }
}

fn impl_get_recursive(
    field_idents: &[&Ident],
    inner: Vec<bool>,
    lvalue: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    if field_idents.is_empty() {
        return quote! {
            fn get_recursive<'a, T>(
                &self,
                inner_field: T,
            ) -> iroha_config::BoxedFuture<'a, core::result::Result<serde_json::Value, Self::Error>>
            where
                T: AsRef<[&'a str]> + Send + 'a,
            {
                Err(iroha_config::derive::Error::UnknownField(
                    inner_field.as_ref().iter().map(ToString::to_string).collect()
                ))
            }
        };
    }
    let variants = field_idents
        .iter()
        .zip(inner)
        .zip(lvalue.iter())
        .map(|((ident, inner_thing), l_value)| {
            let inner_thing2 = if inner_thing {
                quote! {
                    [stringify!(#ident), rest @ ..] => {
                        #l_value.get_recursive(rest)?
                    },
                }
            } else {
                quote! {}
            };
            quote! {
                [stringify!(#ident)] => {
                    serde_json::to_value(&#l_value)
                        .map_err(|e| iroha_config::derive::Error::field_error(stringify!(#ident), e))?
                }
                #inner_thing2
            }
        })
        // XXX: Workaround
        //Decription of issue is here https://stackoverflow.com/a/65353489
        .fold(quote! {}, |acc, new| quote! { #acc #new });

    quote! {
        fn get_recursive<'a, T>(
            &self,
            inner_field: T,
        ) -> core::result::Result<serde_json::Value, Self::Error>
        where
            T: AsRef<[&'a str]> + Send + 'a,
        {
            let inner_field = inner_field.as_ref();
            let value = match inner_field {
                #variants
                field => return Err(iroha_config::derive::Error::UnknownField(
                    field.iter().map(ToString::to_string).collect()
                )),
            };
            Ok(value)
        }
    }
}

#[allow(clippy::too_many_lines, clippy::str_to_string)]
fn impl_configurable(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let prefix = ast
        .attrs
        .iter()
        .find_map(|attr| attr.parse_args::<EnvPrefix>().ok())
        .map(|pref| pref.prefix.value())
        .unwrap_or_default();

    let fields = if let Data::Struct(DataStruct {
        fields: Fields::Named(fields),
        ..
    }) = &ast.data
    {
        &fields.named
    } else {
        abort!(ast, "Only structs are supported")
    };
    let field_idents = fields
        .iter()
        .map(|field| {
            #[allow(clippy::expect_used)]
            field
                .ident
                .as_ref()
                .expect("Should always be set for named structures")
        })
        .collect::<Vec<_>>();
    let field_attrs = fields.iter().map(|field| &field.attrs).collect::<Vec<_>>();
    let field_ty = fields
        .iter()
        .map(|field| field.ty.clone())
        .collect::<Vec<_>>();

    let inner = field_attrs
        .iter()
        .map(|attrs| attrs.iter().any(|attr| attr.parse_args::<Inner>().is_ok()))
        .collect::<Vec<_>>();

    let as_str = field_attrs
        .iter()
        .map(|attrs| {
            attrs
                .iter()
                .any(|attr| attr.parse_args::<SerdeAsStr>().is_ok())
        })
        .collect::<Vec<_>>();

    let field_environment = field_idents
        .iter()
        .into_iter()
        .map(|ident| prefix.clone() + &ident.to_string().to_uppercase())
        .collect::<Vec<_>>();
    let docs = field_attrs
        .iter()
        .zip(field_environment.iter())
        .zip(field_ty.iter())
        .map(|((attrs, env), field_type)| {
            let real_doc = attrs
                .iter()
                .filter_map(|attr| attr.parse_meta().ok())
                .find_map(|metadata| {
                    if let Meta::NameValue(meta) = metadata {
                        if meta.path.is_ident("doc") {
                            if let Lit::Str(s) = meta.lit {
                                return Some(s);
                            }
                        }
                    }
                    None
                });
            let real_doc = real_doc.map(|doc| doc.value() + "\n\n").unwrap_or_default();
            let docs = format!(
                "{}Has type `{}`. Can be configured via environment variable `{}`",
                real_doc,
                quote! { #field_type }.to_string().replace(' ', ""),
                env
            );
            LitStr::new(&docs, Span::mixed_site())
        })
        .collect::<Vec<_>>();
    let lvalue = field_ty.iter().map(is_arc_rwlock).zip(field_idents.iter());
    let lvalue_read = lvalue
        .clone()
        .map(|(is_arc_rwlock, ident)| {
            if is_arc_rwlock {
                quote! { self.#ident.read().await }
            } else {
                quote! { self.#ident }
            }
        })
        .collect::<Vec<_>>();
    let lvalue_write = lvalue
        .clone()
        .map(|(is_arc_rwlock, ident)| {
            if is_arc_rwlock {
                quote! { self.#ident.write().await }
            } else {
                quote! { self.#ident }
            }
        })
        .collect::<Vec<_>>();

    let load_environment = impl_load_env(
        &field_idents,
        &inner,
        &lvalue_write,
        &as_str,
        &field_ty,
        &field_environment,
    );
    let get_recursive = impl_get_recursive(&field_idents, inner.clone(), &lvalue_read);
    let get_doc_recursive =
        impl_get_doc_recursive(&field_ty, &field_idents, inner.clone(), docs.clone());
    let get_inner_docs = impl_get_inner_docs(&field_ty, &field_idents, inner.clone(), docs.clone());
    let get_docs = impl_get_docs(&field_ty, &field_idents, inner, docs);

    let out = quote! {
        impl iroha_config::Configurable for #name {
            type Error = iroha_config::derive::Error;

            #get_recursive
            #get_doc_recursive
            #get_docs
            #get_inner_docs
            #load_environment
        }
    };
    out.into()
}
