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
pub fn message_processor(_args: TokenStream, input: TokenStream) -> TokenStream {
    common_processor(
        _args,
        input,
        Box::new(syn::parse_quote!(Arc<Message>)),
        quote! {MessageProcessor},
        quote! {process_message},
    )
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn notice_processor(_args: TokenStream, input: TokenStream) -> TokenStream {
    common_processor(
        _args,
        input,
        Box::new(syn::parse_quote!(Arc<Notice>)),
        quote! {NoticeProcessor},
        quote! {process_notice},
    )
}

fn common_processor(
    _args: TokenStream,
    input: TokenStream,
    mtp: Box<syn::Type>,
    trait_name: proc_macro2::TokenStream,
    trait_fn_name: proc_macro2::TokenStream,
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
    if second_param_type != &mtp {
        abort!(&second_param.span(), "second parameter must be &Message");
    }

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

        #[allow(non_upper_case_globals)]
        #vis static #static_name: #struct_name = #struct_name;

        #method_clone
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
    r#gen.into()
}
