extern crate proc_macro;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{FnArg, parse_macro_input, spanned::Spanned};

/// 如果设置RUNBOT_CODEGEN_DEBUG变量，编译时将会以note方式打印RUNBOT_CODEGEN的生成结果

macro_rules! emit {
    ($tokens:expr) => {{
        use proc_macro2_diagnostics::SpanDiagnosticExt;
        let mut tokens = $tokens;
        if std::env::var_os("RUNBOT_CODEGEN_DEBUG").is_some() {
            let debug_tokens = proc_macro2::Span::call_site()
                .note("emitting RUNBOT_CODEGEN_DEBUG code generation debug output")
                .note(tokens.to_string())
                .emit_as_item_tokens();
            tokens.extend(debug_tokens);
        }
        tokens.into()
    }};
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn processor(
    _args: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let method = parse_macro_input!(input as syn::ItemFn);
    let method_clone = method.clone();
    if method.sig.asyncness.is_none() {
        abort!(&method.sig.span(), "method must be async");
    }

    // async fn(bot_ctx: Arc<BotContext>, message: Message) -> anyhow::Result<bool>

    // async
    let sig_params = &method.sig.inputs;
    if sig_params.len() != 2 {
        abort!(&method.sig.span(), "method must have 2 parameters");
    }

    // bot_ctx: Arc<BotContext>
    let first_param = &sig_params[0];
    let t = match first_param {
        FnArg::Receiver(_) => {
            abort!(&first_param.span(), "first parameter must be a parameter");
        }
        FnArg::Typed(t) => t,
    };
    let first_param_type = &t.ty;
    if first_param_type != &syn::parse_quote!(Arc<BotContext>) {
        abort!(
            &first_param.span(),
            "first parameter must be Arc<BotContext>"
        );
    }

    // message: Message
    let second_param = &sig_params[1];
    let second_param_type = match second_param {
        FnArg::Receiver(_) => {
            abort!(&second_param.span(), "second parameter must be a parameter");
        }
        FnArg::Typed(t) => t,
    };
    let second_param_type = &second_param_type.ty;

    let (trait_name, trait_fn_name, processor_type) = if second_param_type == &syn::parse_quote!(&Message) {
        (quote! {MessageProcessor}, quote! {process_message}, quote! {Message})
    } else if second_param_type == &syn::parse_quote!(&Notice) {
        (quote! {NoticeProcessor}, quote! {process_notice}, quote! {Notice})
    } else if second_param_type == &syn::parse_quote!(&Request) {
        (quote! {RequestProcessor}, quote! {process_request}, quote! {Request})
    } else if second_param_type == &syn::parse_quote!(&Post) {
        (quote! {PostProcessor}, quote! {process_post}, quote! {Post})
    } else {
        abort!(&second_param.span(), "second parameter must be &Message or &Notice or &Request or &Post");
    };

    let vis = method.vis;
    let asyncness = method.sig.asyncness;
    let fn_name = method.sig.ident.clone();
    let block = &method.block;
    let return_type = &method.sig.output;
    let struct_name = &fn_name.to_string().to_case(Case::UpperCamel);
    let struct_name = proc_macro2::Ident::new(&struct_name, proc_macro2::Span::call_site());
    let static_name = &fn_name.to_string().to_case(Case::UpperSnake);
    let static_name = proc_macro2::Ident::new(&static_name, proc_macro2::Span::call_site());

    emit!(quote::quote! {
        #[derive(Copy, Clone, Default, Debug)]
        #vis struct #struct_name;

        #[::runbot::re_export::async_trait::async_trait]
        impl #trait_name for #struct_name {
            #asyncness fn #trait_fn_name(&self, #first_param, #second_param) #return_type #block
        }

        #vis static #static_name: #struct_name = #struct_name;

        #method_clone

        impl Into<Processor> for #struct_name {
            fn into(self) -> Processor {
                Processor::#processor_type(Box::new(self))
            }
        }
    })
}

#[proc_macro_derive(ParseJson)]
pub fn parse_json_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl #name {
            pub fn parse(json: &serde_json::Value) -> Result<Self> {
                Ok(serde_json::from_value(json.clone())?)
            }
        }
    };

    TokenStream::from(expanded)
}


#[proc_macro_derive(UnknownTypeSerde)]
pub fn notice_type_serde(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;

    let variants = match input.data {
        syn::Data::Enum(syn::DataEnum { ref variants, .. }) => variants,
        _ => panic!("NoticeTypeSerde can only be used with enums"),
    };

    let match_arms_ser = variants.iter().map(|v| {
        let ident = &v.ident;
        let ser_string = ident.to_string().to_case(Case::Snake);
        if ident == "Unknown" {
            quote! { #name::Unknown(s) => serializer.serialize_str(s), }
        } else {
            quote! { #name::#ident => serializer.serialize_str(#ser_string), }
        }
    });

    let match_arms_de = variants.iter().map(|v| {
        let ident = &v.ident;
        let de_string = ident.to_string().to_case(Case::Snake);
        if ident == "Unknown" {
            quote! { other => Ok(#name::Unknown(other.to_string())), }
        } else {
            quote! { #de_string => Ok(#name::#ident), }
        }
    });

    let r#gen = quote! {
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where S: serde::Serializer {
                match self {
                    #(#match_arms_ser)*
                }
            }
        }
        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where D: serde::Deserializer<'de> {
                let s = String::deserialize(deserializer)?;
                match s.as_str() {
                    #(#match_arms_de)*
                }
            }
        }
    };
    emit!(r#gen)
}


#[proc_macro_derive(UnknownEnumSerdeAndParse, attributes(enum_field))]
pub fn unknown_enum_serde_and_parse(input: TokenStream) -> TokenStream {
    // 解析输入
    let input = parse_macro_input!(input as syn::DeriveInput);
    let enum_name = &input.ident;

    // 默认分派字段
    let mut field_name = Option::<String>::None;


    // 支持 #[parse_json(field = "...")]
    for attr in &input.attrs {
        if attr.path().is_ident("enum_field") {
            if let Ok(nested) = attr.parse_args_with(|input: syn::parse::ParseStream| {
                syn::punctuated::Punctuated::<darling::ast::NestedMeta, syn::Token![,]>::parse_terminated(input)
            }) {
                for meta in &nested {
                    if let darling::ast::NestedMeta::Meta(syn::Meta::NameValue(nv)) = meta {
                        if nv.path.is_ident("name") {
                            if let syn::Expr::Lit(expr_lit) = &nv.value {
                                if let syn::Lit::Str(ref s) = expr_lit.lit {
                                    field_name = Some(s.value());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let field_name = if let Some(field_name) = field_name {
        field_name
    } else {
        abort!(&input.span(), "enum_field not define");
    };

    // 处理变体
    let variants = match &input.data {
        syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
        _ => panic!("ParseJson only supports enums"),
    };

    let mut arms = Vec::new();
    let mut unknown_arm = None;

    for variant in variants {
        let ident = &variant.ident;
        match &variant.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed.first().unwrap().ty;
                // 检查是不是 Unknown(serde_json::Value)
                let is_unknown = ident == "Unknown" && quote!(#ty).to_string().contains("serde_json :: Value");
                if is_unknown {
                    unknown_arm = Some(quote! {
                        _ => Ok(#enum_name::Unknown(value.clone()))
                    });
                } else {
                    let case_name = &ident.to_string().to_case(Case::Snake);
                    arms.push(quote! {
                        #case_name => Ok(#enum_name::#ident(<#ty>::parse(value)?))
                    });
                }
            }
            _ => {}
        }
    }

    let unknown_arm = unknown_arm.unwrap_or(quote! {
        _ => Err(Error::FieldError("Unknown type".to_string()))
    });

    let r#gen = quote! {
        impl #enum_name {
            pub fn parse(value: &serde_json::Value) -> Result<Self> {
                let request_type = value.get(#field_name)
                    .ok_or(Error::FieldError(format!("{} not found", #field_name)))?;
                let request_type = request_type.as_str()
                    .ok_or(Error::FieldError(format!("{} not is str", #field_name)))?;
                match request_type {
                    #(#arms,)*
                    #unknown_arm,
                }
            }
        }
    };
    emit!(r#gen)
}
