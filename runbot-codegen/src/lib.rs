extern crate proc_macro;

use std::collections::HashMap;

use convert_case::{Case, Casing};
use proc_macro::{TokenStream, TokenTree};
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
pub fn processor(_args: TokenStream, input: TokenStream) -> TokenStream {
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

    let (trait_name, trait_fn_name, processor_type) =
        if second_param_type == &syn::parse_quote!(&Message) {
            (
                quote! {MessageProcessor},
                quote! {process_message},
                quote! {Message},
            )
        } else if second_param_type == &syn::parse_quote!(&Notice) {
            (
                quote! {NoticeProcessor},
                quote! {process_notice},
                quote! {Notice},
            )
        } else if second_param_type == &syn::parse_quote!(&Request) {
            (
                quote! {RequestProcessor},
                quote! {process_request},
                quote! {Request},
            )
        } else if second_param_type == &syn::parse_quote!(&Post) {
            (quote! {PostProcessor}, quote! {process_post}, quote! {Post})
        } else {
            abort!(
                &second_param.span(),
                "second parameter must be &Message or &Notice or &Request or &Post"
            );
        };

    let vis = method.vis;
    let asyncness = method.sig.asyncness;
    let fn_name = method.sig.ident.clone();
    let return_type = &method.sig.output;
    let struct_name = &fn_name.to_string().to_case(Case::UpperCamel);
    let struct_name = proc_macro2::Ident::new(&struct_name, proc_macro2::Span::call_site());
    let static_name = &fn_name.to_string().to_case(Case::UpperSnake);
    let static_name = proc_macro2::Ident::new(&static_name, proc_macro2::Span::call_site());
    let first_param_ident = match first_param {
        FnArg::Typed(t) => match &*t.pat {
            syn::Pat::Ident(ident) => ident.ident.clone(),
            _ => abort!(&t.pat, "first parameter must be a parameter"),
        },
        _ => abort!(&first_param.span(), "first parameter must be a parameter"),
    };
    let second_param_ident = match second_param {
        FnArg::Typed(t) => match &*t.pat {
            syn::Pat::Ident(ident) => ident.ident.clone(),
            _ => abort!(&t.pat, "second parameter must be a parameter"),
        },
        _ => abort!(&second_param.span(), "second parameter must be a parameter"),
    };
    emit!(quote::quote! {
        #[derive(Copy, Clone, Default, Debug)]
        #vis struct #struct_name;

        #[::runbot::re_export::async_trait::async_trait]
        impl #trait_name for #struct_name {
            fn id(&self) -> &'static str {
                concat!(
                    env!("CARGO_PKG_NAME"),
                    "::",
                    module_path!(),
                    "::",
                    stringify!(#fn_name)
                )
            }

            #asyncness fn #trait_fn_name(&self, #first_param, #second_param) #return_type {
                #fn_name(#first_param_ident, #second_param_ident).await
            }
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
                let is_unknown =
                    ident == "Unknown" && quote!(#ty).to_string().contains("serde_json :: Value");
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

#[derive(Default, Debug)]
struct CommandAttributes {
    pattern: Option<syn::LitStr>,
}

impl CommandAttributes {
    fn parse(&mut self, meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        if meta.path.is_ident("pattern") {
            self.pattern = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn command(args: TokenStream, input: TokenStream) -> TokenStream {
    // method
    let method = parse_macro_input!(input as syn::ItemFn);
    let span = method.span();
    let method_clone = method.clone();
    // attrs
    let mut attrs = CommandAttributes::default();
    let command_parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(args with command_parser);
    let bot_command_pattern_str = if let Some(lit_str) = attrs.pattern {
        lit_str.value()
    } else {
        abort!(&method.span(), "command pattren missing");
    };
    let plain_text_regex =
        regex::Regex::new(r#"^[A-Za-z0-9_/\p{Han}\p{Hiragana}\p{Katakana}]+$"#).unwrap();
    let mut bot_command_items = vec![];
    for fragment in bot_command_pattern_str.split_ascii_whitespace() {
        if plain_text_regex.is_match(fragment) {
            bot_command_items.push(BotCommandItem::PlainText(
                false,
                false,
                false,
                fragment.to_owned(),
            ));
            continue;
        }
        for item in parse_template(fragment) {
            bot_command_items.push(item);
        }
    }
    if !is_text_to_end_at_most_one_and_last(&bot_command_items) {
        abort!(&method.span(), "text to end must be at most one and last");
    }

    // method
    if method.sig.asyncness.is_none() {
        abort!(&method.sig.span(), "method must be async");
    }
    let sig_params = &method.sig.inputs;
    if sig_params.len() < 2 {
        abort!(
            &method.sig.span(),
            "method must have 2 parameters Arc<BotContext>,&Message"
        );
    }
    let first_param = &sig_params[0];
    let first_param_type = match first_param {
        FnArg::Receiver(_) => {
            abort!(&first_param.span(), "first parameter must be a parameter");
        }
        FnArg::Typed(t) => t,
    };
    let first_param_type = &first_param_type.ty;
    if first_param_type != &syn::parse_quote!(Arc<BotContext>) {
        abort!(
            &first_param.span(),
            "first parameter must be Arc<BotContext>"
        );
    }
    let second_param = &sig_params[1];
    let second_param_type = match second_param {
        FnArg::Receiver(_) => {
            abort!(&second_param.span(), "second parameter must be a parameter");
        }
        FnArg::Typed(t) => t,
    };
    let second_param_type = &second_param_type.ty;
    if second_param_type != &syn::parse_quote!(&Message) {
        abort!(&second_param.span(), "second parameter must be &Message");
    }

    let paramed_bot_command_items = bot_command_items
        .iter()
        .filter(|item| match item {
            BotCommandItem::NumberParam(_, _, _, _) => true,
            BotCommandItem::TextToSpaceParam(_, _, _, _) => true,
            BotCommandItem::EnumParam(_, _, _, _, _) => true,
            BotCommandItem::TextToEnd(_, _, _, _) => true,
            _ => false,
        })
        .collect::<Vec<_>>();
    if sig_params.len() != paramed_bot_command_items.len() + 2 {
        abort!(
            &method.sig.span(),
            "method must have {} parameters, but got {}",
            paramed_bot_command_items.len() + 2,
            sig_params.len()
        );
    }

    for i in 0..paramed_bot_command_items.len() {
        let param_name_command_item = match paramed_bot_command_items[i] {
            BotCommandItem::NumberParam(_, _, _, name) => name,
            BotCommandItem::TextToSpaceParam(_, _, _, name) => name,
            BotCommandItem::EnumParam(_, _, _, name, _) => name,
            BotCommandItem::TextToEnd(_, _, _, name) => name,
            _ => continue,
        };
        let param_name_sig_param = match &sig_params[i + 2] {
            FnArg::Typed(t) => match &*t.pat {
                syn::Pat::Ident(ident) => ident.ident.to_string(),
                _ => continue,
            },
            _ => continue,
        };
        if param_name_command_item != &param_name_sig_param {
            abort!(
                &method.sig.span(),
                "param name mismatch, command item: {}, sig param: {}",
                param_name_command_item,
                param_name_sig_param
            );
        }

        let type_is_option_command_item = match paramed_bot_command_items[i] {
            BotCommandItem::NumberParam(optional, _, _, _) => optional,
            BotCommandItem::TextToSpaceParam(optional, _, _, _) => optional,
            BotCommandItem::EnumParam(optional, _, _, _, _) => optional,
            BotCommandItem::TextToEnd(optional, _, _, _) => optional,
            _ => continue,
        };
        let type_is_option_sig_param = match &sig_params[i + 2] {
            FnArg::Typed(t) => {
                // Check if the type is Option<T>
                if let syn::Type::Path(type_path) = &*t.ty {
                    if let Some(seg) = type_path.path.segments.last() {
                        seg.ident == "Option"
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => continue,
        };
        if type_is_option_command_item != &type_is_option_sig_param {
            abort!(
                &method.sig.span(),
                "param type option mismatch, command item: {}, sig param: {}",
                param_name_command_item,
                param_name_sig_param
            );
        }

        let vec_type_command_item = match paramed_bot_command_items[i] {
            BotCommandItem::NumberParam(_, a, b, _) => *a || *b,
            BotCommandItem::Number(_, a, b) => *a || *b,
            BotCommandItem::TextToSpace(_, a, b) => *a || *b,
            BotCommandItem::TextToSpaceParam(_, a, b, _) => *a || *b,
            BotCommandItem::PlainText(_, a, b, _) => *a || *b,
            BotCommandItem::Enum(_, a, b, _) => *a || *b,
            BotCommandItem::EnumParam(_, a, b, _, _) => *a || *b,
            BotCommandItem::TextToEnd(_, a, b, _) => *a || *b,
        };
        let vec_type_sig_param = match &sig_params[i + 2] {
            FnArg::Typed(t) => {
                if let syn::Type::Path(type_path) = &*t.ty {
                    if let Some(seg) = type_path.path.segments.last() {
                        seg.ident == "Vec"
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => continue,
        };
        if vec_type_command_item != vec_type_sig_param {
            abort!(
                &method.sig.span(),
                "param type vec mismatch, command item: {}, sig param: {}",
                param_name_command_item,
                param_name_sig_param
            );
        }
    }

    let vis = method.vis;
    let asyncness = method.sig.asyncness;
    let fn_name = method.sig.ident.clone();
    let return_type = &method.sig.output;
    let struct_name = &fn_name.to_string().to_case(Case::UpperCamel);
    let struct_name = proc_macro2::Ident::new(&struct_name, proc_macro2::Span::call_site());
    let static_name = &fn_name.to_string().to_case(Case::UpperSnake);
    let static_name = proc_macro2::Ident::new(&static_name, proc_macro2::Span::call_site());
    let first_param_ident = match first_param {
        FnArg::Typed(t) => match &*t.pat {
            syn::Pat::Ident(ident) => ident.ident.clone(),
            _ => abort!(&t.pat, "first parameter must be a parameter"),
        },
        _ => abort!(&first_param.span(), "first parameter must be a parameter"),
    };
    let second_param_ident = match second_param {
        FnArg::Typed(t) => match &*t.pat {
            syn::Pat::Ident(ident) => ident.ident.clone(),
            _ => abort!(&t.pat, "second parameter must be a parameter"),
        },
        _ => abort!(&second_param.span(), "second parameter must be a parameter"),
    };

    let mut command_item_ident_stream = quote! {};
    for item in paramed_bot_command_items {
        let command_item_ident = match item {
            BotCommandItem::NumberParam(_, _, _, name) => name,
            BotCommandItem::TextToSpaceParam(_, _, _, name) => name,
            BotCommandItem::EnumParam(_, _, _, name, _) => name,
            BotCommandItem::TextToEnd(_, _, _, name) => name,
            BotCommandItem::Number(_, _, _) => abort!(&span, "number param not support"),
            BotCommandItem::PlainText(_, _, _, _) => abort!(&span, "plain text param not support"),
            BotCommandItem::TextToSpace(_, _, _) => {
                abort!(&span, "text to space param not support")
            }
            BotCommandItem::Enum(_, _, _, _) => abort!(&span, "enum param not support"),
        };
        let command_item_ident =
            proc_macro2::Ident::new(&command_item_ident, proc_macro2::Span::call_site());

        command_item_ident_stream.extend(quote::quote! {
            , #command_item_ident
        });
    }

    let define_command_lopper = quote::quote! {
        let mut runbot_command_string_buffer = String::new();
        for message_data in &message.message {
            // message_data : MessageData
            match message_data {
                MessageData::Text(MessageText { text }) => {
                    if text.is_empty() {
                        continue;
                    }
                    if runbot_command_string_buffer.is_empty() {
                        runbot_command_string_buffer.push_str(text);
                    } else {
                        runbot_command_string_buffer.push(' ');
                        runbot_command_string_buffer.push_str(text);
                    }
                }
                MessageData::At(MessageAt { qq, .. }) => {
                    if runbot_command_string_buffer.is_empty() {
                        runbot_command_string_buffer.push_str(qq);
                    } else {
                        runbot_command_string_buffer.push(' ');
                        runbot_command_string_buffer.push_str(qq);
                    }
                }
                _ => return Ok(false),
            }
        }
        let mut runbot_command_looper = ::runbot::command::CommandLopper::new(runbot_command_string_buffer.trim().split_ascii_whitespace().collect::<Vec<&str>>());
    };

    let mut define_lopper_value = quote::quote! {};

    if std::env::var_os("RUNBOT_CODEGEN_DEBUG").is_some() {
        eprintln!("bot_command_items : {:?}", bot_command_items)
    }

    for item in bot_command_items {
        match item {
            BotCommandItem::Number(optional, repat_less_one, repat_zero_or_more) => {
                if optional {
                    define_lopper_value.extend(quote::quote! {
                        runbot_command_looper.next_number();
                    });
                } else if repat_less_one {
                    define_lopper_value.extend(quote::quote! {
                        let runbot_command_number = runbot_command_looper.next_number();
                        if runbot_command_number.is_none() {
                            return Ok(false);
                        }
                        while let Some(_) = runbot_command_looper.next_number() {
                            continue;
                        }
                    });
                } else if repat_zero_or_more {
                    define_lopper_value.extend(quote::quote! {
                        while let Some(_) = runbot_command_looper.next_number() {
                            continue;
                        }
                    });
                } else {
                    define_lopper_value.extend(quote::quote! {
                        let runbot_command_number = runbot_command_looper.next_number();
                        if runbot_command_number.is_none() {
                            return Ok(false);
                        }
                    });
                }
            }
            BotCommandItem::NumberParam(optional, repat_less_one, repat_zero_or_more, name) => {
                let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
                if optional {
                    define_lopper_value.extend(quote::quote! {
                        let runbot_command_number = runbot_command_looper.next_number();
                        let #ident = if let Some(number) = runbot_command_number {
                            if let Ok(p) = std::str::FromStr::from_str(number.as_str()) {
                                Some(p)
                            } else {
                                return Ok(false);
                            }
                        } else {
                            None
                        };
                    });
                } else if repat_less_one {
                    define_lopper_value.extend(quote::quote! {
                        let mut runbot_command_numbers = Vec::new();
                        while let Some(number) = runbot_command_looper.next_number() {
                            if let Ok(p) = std::str::FromStr::from_str(number.as_str()) {
                                runbot_command_numbers.push(p);
                            } else {
                                return Ok(false);
                            }
                        }
                        if runbot_command_numbers.is_empty() {
                            return Ok(false);
                        }
                        let #ident = runbot_command_numbers;
                    })
                } else if repat_zero_or_more {
                    define_lopper_value.extend(quote::quote! {
                        let mut runbot_command_numbers = Vec::new();
                        while let Some(number) = runbot_command_looper.next_number() {
                            if let Ok(p) = std::str::FromStr::from_str(number.as_str()) {
                                runbot_command_numbers.push(p);
                            } else {
                                return Ok(false);
                            }
                        }
                        let #ident = runbot_command_numbers;
                    });
                } else {
                    define_lopper_value.extend(quote::quote! {
                        let runbot_command_number = runbot_command_looper.next_number();
                        if runbot_command_number.is_none() {
                            return Ok(false);
                        }
                        let #ident = if let Ok(p) = std::str::FromStr::from_str(runbot_command_number.unwrap().as_str()) {
                            p
                        } else {
                            return Ok(false);
                        };
                    });
                }
            }
            BotCommandItem::PlainText(optional, repat_less_one, repat_zero_or_more, text) => {
                if optional {
                    define_lopper_value.extend(quote::quote! {
                        runbot_command_looper.cut_plain_text(#text);
                    });
                } else if repat_less_one {
                    define_lopper_value.extend(quote::quote! {
                        if !runbot_command_looper.cut_plain_text(#text) {
                            return Ok(false);
                        }
                        while let Some(_) = runbot_command_looper.cut_plain_text(#text) {
                            continue;
                        }
                    });
                } else if repat_zero_or_more {
                    define_lopper_value.extend(quote::quote! {
                        while let Some(_) = runbot_command_looper.cut_plain_text(#text) {
                            continue;
                        }
                    });
                } else {
                    define_lopper_value.extend(quote::quote! {
                        if !runbot_command_looper.cut_plain_text(#text) {
                            return Ok(false);
                        }
                    });
                }
            }
            BotCommandItem::TextToSpace(optional, repat_less_one, repat_zero_or_more) => {
                if optional {
                    define_lopper_value.extend(quote::quote! {
                        runbot_command_looper.cut_text_to_space();
                    });
                } else if repat_less_one {
                    define_lopper_value.extend(quote::quote! {
                        if !runbot_command_looper.cut_text_to_space() {
                            return Ok(false);
                        }
                        while let Some(_) = runbot_command_looper.cut_text_to_space() {
                            continue;
                        }
                    });
                } else if repat_zero_or_more {
                    define_lopper_value.extend(quote::quote! {
                        while let Some(_) = runbot_command_looper.cut_text_to_space() {
                            continue;
                        }
                    });
                } else {
                    define_lopper_value.extend(quote::quote! {
                        if !runbot_command_looper.cut_text_to_space() {
                            return Ok(false);
                        }
                    });
                }
            }
            BotCommandItem::TextToSpaceParam(
                optional,
                repat_less_one,
                repat_zero_or_more,
                name,
            ) => {
                let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
                if optional {
                    define_lopper_value.extend(quote::quote! {
                        let runbot_command_text = runbot_command_looper.cut_text_to_space();
                        let #ident = if let Some(text) = runbot_command_text {
                            if let Ok(p) = std::str::FromStr::from_str(text.as_str()) {
                                Some(p)
                            } else {
                                return Ok(false);
                            }
                        } else {
                            None
                        };
                    });
                } else if repat_less_one {
                    define_lopper_value.extend(quote::quote! {
                        let mut runbot_command_texts = vec![];
                        runbot_command_texts.push(if let Some(text) = runbot_command_looper.cut_text_to_space() {
                            if let Ok(p) = std::str::FromStr::from_str(text.as_str()) {
                                p
                            } else {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        });
                        while let Some(text) = runbot_command_looper.cut_text_to_space() {
                            runbot_command_texts.push(if let Ok(p) = std::str::FromStr::from_str(text.as_str()) {
                                p
                            } else {
                                return Ok(false);
                            });
                        }
                        let #ident = runbot_command_texts;
                    });
                } else if repat_zero_or_more {
                    define_lopper_value.extend(quote::quote! {
                        let mut runbot_command_texts = vec![];
                        while let Some(text) = runbot_command_looper.cut_text_to_space() {
                            runbot_command_texts.push(if let Ok(p) = std::str::FromStr::from_str(text.as_str()) {
                                p
                            } else {
                                return Ok(false);
                            });
                        }
                        let #ident = runbot_command_texts;
                    });
                } else {
                    define_lopper_value.extend(quote::quote! {
                        if !runbot_command_looper.cut_text_to_space() {
                            return Ok(false);
                        }
                    });
                }
            }
            BotCommandItem::Enum(optional, repat_less_one, repat_zero_or_more, options) => {
                if optional {
                    define_lopper_value.extend(quote::quote! {
                        runbot_command_looper.next_enum(&[#(#options),*]);
                    });
                } else if repat_less_one {
                    define_lopper_value.extend(quote::quote! {
                        let runbot_command_option = runbot_command_looper.next_enum(&[#(#options),*]);
                        if runbot_command_option.is_none() {
                            return Ok(false);
                        }
                        while let Some(_) = runbot_command_looper.next_enum(&[#(#options),*]) {
                            continue;
                        }
                    });
                } else if repat_zero_or_more {
                    define_lopper_value.extend(quote::quote! {
                        while let Some(_) = runbot_command_looper.next_enum(&[#(#options),*]) {
                            continue;
                        }
                    });
                } else {
                    define_lopper_value.extend(quote::quote! {
                        if runbot_command_looper.next_enum(&[#(#options),*]).is_none() {
                            return Ok(false);
                        }
                    });
                }
            }
            BotCommandItem::EnumParam(
                optional,
                repat_less_one,
                repat_zero_or_more,
                name,
                options,
            ) => {
                let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
                if optional {
                    define_lopper_value.extend(quote::quote! {
                        let #ident = runbot_command_looper.next_enum(&[#(#options),*]);
                    });
                } else if repat_less_one {
                    define_lopper_value.extend(quote::quote! {
                        let mut runbot_command_option = vec![];
                        runbot_command_option.push(if let Some(text) = runbot_command_looper.next_enum(&[#(#options),*]) {
                            if let Ok(text) = std::str::FromStr::from_str(text.as_str()) {
                                text
                            } else {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        });
                        while let Some(text) = runbot_command_looper.next_enum(&[#(#options),*]) {
                            runbot_command_option.push(if let Ok(text) = std::str::FromStr::from_str(text.as_str()) {
                                text
                            } else {
                                return Ok(false);
                            });
                        }
                        let #ident = runbot_command_option;
                    });
                } else if repat_zero_or_more {
                    define_lopper_value.extend(quote::quote! {
                        let mut runbot_command_option = vec![];
                        while let Some(text) = runbot_command_looper.next_enum(&[#(#options),*]) {
                            runbot_command_option.push(if let Ok(text) = std::str::FromStr::from_str(text.as_str()) {
                                text
                            } else {
                                return Ok(false);
                            });
                        }
                        let #ident = runbot_command_option;
                    });
                } else {
                    define_lopper_value.extend(quote::quote! {
                        let runbot_command_option = if let Some(text) = runbot_command_looper.next_enum(&[#(#options),*]) {
                            if let Ok(text) = std::str::FromStr::from_str(text.as_str()) {
                                text
                            } else {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        };
                        let #ident = runbot_command_option;
                    });
                }
            }
            BotCommandItem::TextToEnd(optional, repat_less_one, repat_zero_or_more, name) => {
                let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
                if optional {
                    define_lopper_value.extend(quote::quote! {
                        let #ident = if let Some(text) = runbot_command_looper.cut_text_to_end() {
                            if let Ok(text) = std::str::FromStr::from_str(text.as_str()) {
                                Some(text)
                            } else {
                                return Ok(false);
                            }
                        };
                    });
                } else if repat_less_one {
                    define_lopper_value.extend(quote::quote! {
                        let mut runbot_command_texts = vec![];
                        runbot_command_texts.push(if let Some(text) = runbot_command_looper.cut_text_to_end() {
                            if let Ok(text) = std::str::FromStr::from_str(text.as_str()) {
                                text
                            } else {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        });
                    });
                } else if repat_zero_or_more {
                    define_lopper_value.extend(quote::quote! {
                        let mut runbot_command_texts = vec![];
                        while let Some(text) = runbot_command_looper.cut_text_to_end() {
                            runbot_command_texts.push(if let Ok(text) = std::str::FromStr::from_str(text.as_str()) {
                                text
                            } else {
                                return Ok(false);
                            });
                        }
                        let #ident = runbot_command_texts;
                    });
                } else {
                    define_lopper_value.extend(quote::quote! {
                        let #ident = if let Some(text) = runbot_command_looper.cut_text_to_end() {
                            if let Ok(text) = std::str::FromStr::from_str(text.as_str()) {
                                text
                            } else {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        };
                    });
                }
            }
        }
    }

    emit!(quote::quote! {
        #[derive(Copy, Clone, Default, Debug)]
        #vis struct #struct_name;

        #[::runbot::re_export::async_trait::async_trait]
        impl MessageProcessor for #struct_name {
            fn id(&self) -> &'static str {
                concat!(
                    env!("CARGO_PKG_NAME"),
                    "::",
                    module_path!(),
                    "::",
                    stringify!(#fn_name)
                )
            }
            #asyncness fn process_message(&self, #first_param, #second_param) #return_type {
                #define_command_lopper
                #define_lopper_value
                #fn_name(#first_param_ident, #second_param_ident #command_item_ident_stream).await
            }
        }

        #vis static #static_name: #struct_name = #struct_name;

        #method_clone

        impl Into<Processor> for #struct_name {
            fn into(self) -> Processor {
                Processor::Message(Box::new(self))
            }
        }
    })
}

fn is_text_to_end_at_most_one_and_last(items: &[BotCommandItem]) -> bool {
    let count = items
        .iter()
        .filter(|x| matches!(x, BotCommandItem::TextToEnd(_, _, _, _)))
        .count();
    match count {
        0 => true,                                                                // 没有也可以
        1 => matches!(items.last(), Some(BotCommandItem::TextToEnd(_, _, _, _))), // 有且在最后
        _ => false,                                                               // 超过一个
    }
}

////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
enum BotCommandItem {
    Number(bool, bool, bool),              // {:n}? | {:n}* | {:n}+ | 匹配数字
    NumberParam(bool, bool, bool, String), // {time:n}? | {time:n}* | {time:n}+ | 匹配数字并命名
    PlainText(bool, bool, bool, String),   // hi? | hello* | world+ | 纯文本
    TextToSpace(bool, bool, bool),         // {:s}? | {:s}* | {:s}+ | 直到空白
    TextToSpaceParam(bool, bool, bool, String), // {text:s}? | {text:s}* | {text:s}+ | 直到空白并命名
    Enum(bool, bool, bool, Vec<String>),        // [a|b]? | [a|b]* | [a|b]+
    EnumParam(bool, bool, bool, String, Vec<String>), // [name:a|b]? | [name:a|b]* | [name:a|b]+
    TextToEnd(bool, bool, bool, String),        // {text:e}? | {text:e}* | {text:e}+ | 直到结尾
}

fn get_repeat_flags_from_char(ch: char) -> Option<(bool, bool, bool)> {
    match ch {
        '?' => Some((true, false, false)),
        '+' => Some((false, true, false)),
        '*' => Some((false, false, true)),
        _ => None,
    }
}

fn parse_template(template: &str) -> Vec<BotCommandItem> {
    let mut result = Vec::new();
    let mut i = 0;
    let template_len = template.len();

    while i < template_len {
        let rest = &template[i..];

        // 匹配 {...}
        if rest.starts_with('{') {
            if let Some(j) = rest.find('}') {
                let body = &rest[1..j];

                // 查看紧跟其后的重复标志，仅当为 ?/*/+ 时才前进
                let mut next_index = i + j + 1;
                let mut flags = (false, false, false);
                if let Some(ch) = rest[j + 1..].chars().next() {
                    if let Some(f) = get_repeat_flags_from_char(ch) {
                        flags = f;
                        next_index += ch.len_utf8();
                    }
                }

                if body == ":n" {
                    result.push(BotCommandItem::Number(flags.0, flags.1, flags.2));
                    i = next_index;
                    continue;
                } else if body == ":s" {
                    result.push(BotCommandItem::TextToSpace(flags.0, flags.1, flags.2));
                    i = next_index;
                    continue;
                } else if let Some(colon) = body.find(':') {
                    let name = &body[..colon];
                    let typ = &body[colon + 1..];
                    match typ {
                        "n" => result.push(BotCommandItem::NumberParam(
                            flags.0,
                            flags.1,
                            flags.2,
                            name.to_string(),
                        )),
                        "s" => result.push(BotCommandItem::TextToSpaceParam(
                            flags.0,
                            flags.1,
                            flags.2,
                            name.to_string(),
                        )),
                        "e" => result.push(BotCommandItem::TextToEnd(
                            flags.0,
                            flags.1,
                            flags.2,
                            name.to_string(),
                        )),
                        _ => {}
                    }
                    i = next_index;
                    continue;
                }
                // 如果 body 不匹配已知类型，降级到纯文本处理
            }
        }

        // 匹配 [枚举]
        if rest.starts_with('[') {
            if let Some(j) = rest.find(']') {
                let body = &rest[1..j];

                // 查看紧跟其后的重复标志，仅当为 ?/*/+ 时才前进
                let mut next_index = i + j + 1;
                let mut flags = (false, false, false);
                if let Some(ch) = rest[j + 1..].chars().next() {
                    if let Some(f) = get_repeat_flags_from_char(ch) {
                        flags = f;
                        next_index += ch.len_utf8();
                    }
                }

                if let Some(colon) = body.find(':') {
                    let name = &body[..colon];
                    let options: Vec<String> = body[colon + 1..]
                        .split('|')
                        .map(|s| s.to_string())
                        .collect();
                    result.push(BotCommandItem::EnumParam(
                        flags.0,
                        flags.1,
                        flags.2,
                        name.to_string(),
                        options,
                    ));
                } else {
                    let options: Vec<String> = body.split('|').map(|s| s.to_string()).collect();
                    result.push(BotCommandItem::Enum(flags.0, flags.1, flags.2, options));
                }
                i = next_index;
                continue;
            }
        }

        // 匹配纯文本（包括空白）以及 ?/*/+ 标记
        let mut j = 0;
        while i + j < template_len {
            let c = template[i + j..].chars().next().unwrap();
            if c == '{' || c == '[' {
                break;
            }
            j += c.len_utf8();
        }
        if j > 0 {
            let text = &template[i..i + j];
            let mut last_idx = 0;
            let mut chars_iter = text.char_indices().peekable();

            while let Some((idx, ch)) = chars_iter.next() {
                if let Some((opt, rep0, rep1)) = get_repeat_flags_from_char(ch) {
                    let part = &text[last_idx..idx];
                    // 避免空切片导致的额外空项（例如末尾符号后）
                    if !part.is_empty() {
                        let trimmed = part.trim();
                        if trimmed.is_empty() {
                            // 对纯空白片段保留一个空字符串 PlainText，用于表示空白占位
                            result.push(BotCommandItem::PlainText(
                                false,
                                false,
                                false,
                                "".to_string(),
                            ));
                        } else {
                            result.push(BotCommandItem::PlainText(
                                opt,
                                rep0,
                                rep1,
                                trimmed.to_string(),
                            ));
                        }
                    }
                    last_idx = idx + ch.len_utf8();
                }
            }
            // 剩余部分（没有跟随重复标志）
            if last_idx < text.len() {
                let part = &text[last_idx..];
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    // 对仅包含空白的剩余片段，也保留一个空字符串 PlainText
                    result.push(BotCommandItem::PlainText(
                        false,
                        false,
                        false,
                        "".to_string(),
                    ));
                } else {
                    result.push(BotCommandItem::PlainText(
                        false,
                        false,
                        false,
                        trimmed.to_string(),
                    ));
                }
            }

            i += j;
        } else {
            // 无法识别的字符，跳过一个字符以避免死循环
            if let Some(ch) = template[i..].chars().next() {
                i += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    result
}

#[derive(Default, Debug)]
struct ModuleAttributes {
    name: Option<syn::LitStr>,
    help: Option<syn::LitStr>,
    processors: Option<syn::LitStr>,
}

impl ModuleAttributes {
    fn parse(&mut self, ts: &TokenStream) -> syn::Result<()> {
        if ts.is_empty() {
            return Ok(());
        }
        let error_msg = "All attributes should be `ident = \"value\"` joined by ,";
        let tree_vec = ts.clone().into_iter().collect::<Vec<_>>();
        let mut idx: usize = 0;
        loop {
            if idx >= tree_vec.len() {
                break;
            }
            let ident = &tree_vec[idx];
            idx += 1;
            let ident = match ident {
                TokenTree::Ident(ident) => ident,
                _ => {
                    return Err(syn::Error::new(ident.span().into(), error_msg));
                }
            };
            if idx >= tree_vec.len() {
                return Err(syn::Error::new(ident.span().into(), error_msg));
            }
            let punct = &tree_vec[idx];
            idx += 1;
            match punct {
                TokenTree::Punct(punct) => {
                    let equals_char: char = '=';
                    if equals_char != punct.as_char() {
                        return Err(syn::Error::new(punct.span().into(), error_msg));
                    }
                }
                _ => return Err(syn::Error::new(punct.span().into(), error_msg)),
            }
            if idx >= tree_vec.len() {
                return Err(syn::Error::new(punct.span().into(), error_msg));
            }
            let literal = &tree_vec[idx];
            idx += 1;
            let literal = match literal {
                TokenTree::Literal(literal) => {
                    let literal_str = literal.to_string();
                    syn::parse_str::<syn::LitStr>(&literal_str)?
                }
                _ => return Err(syn::Error::new(literal.span().into(), error_msg)),
            };
            match ident.to_string().as_str() {
                "name" => {
                    if self.name.is_some() {
                        return Err(syn::Error::new(ident.span().into(), "duplicate 'name'"));
                    }
                    self.name = Some(literal);
                }
                "help" => {
                    if self.help.is_some() {
                        return Err(syn::Error::new(ident.span().into(), "duplicate 'help'"));
                    }
                    self.help = Some(literal);
                }
                "processors" => {
                    if self.processors.is_some() {
                        return Err(syn::Error::new(
                            ident.span().into(),
                            "duplicate 'processors'",
                        ));
                    }
                    self.processors = Some(literal);
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span().into(),
                        format!("not allowed idnet '{}'", ident.to_string()),
                    ));
                }
            }
            if idx >= tree_vec.len() {
                break;
            }
            let punct = &tree_vec[idx];
            idx += 1;
            match punct {
                TokenTree::Punct(punct) => {
                    let equals_char: char = ',';
                    if equals_char != punct.as_char() {
                        return Err(syn::Error::new(punct.span().into(), error_msg));
                    }
                }
                _ => return Err(syn::Error::new(punct.span().into(), error_msg)),
            }
        }
        Ok(())
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn module(args: TokenStream, input: TokenStream) -> TokenStream {
    //// attrs
    // []
    // [ Ident Punct(=) Literal Punct(,) ]
    let mut attrs = ModuleAttributes::default();
    match attrs.parse(&args) {
        Ok(_) => {}
        Err(err) => {
            abort!(&args.into_iter().take(1).last().unwrap().span(), err);
        }
    };
    //// impl
    let module_impl = parse_macro_input!(input as syn::ItemImpl);
    let module_impl_span = module_impl.span();
    let struct_ident = match *module_impl.self_ty {
        syn::Type::Path(ref type_path) => &type_path.path.segments.last().unwrap().ident,
        _ => abort!(&module_impl_span, "Expected a struct type for impl block"),
    };
    let struct_name = struct_ident.to_string();
    let functions = module_impl
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(m) => Some(m),
            _ => None,
        })
        .collect::<Vec<&syn::ImplItemFn>>();
    //// collect attrs and impl functions
    let mut function_map = functions
        .iter()
        .map(|f| (f.sig.ident.to_string(), *f))
        .collect::<HashMap<String, &syn::ImplItemFn>>();
    let function_names = functions
        .iter()
        .map(|f| f.sig.ident.to_string())
        .collect::<Vec<String>>();
    // id
    let id_function_defined = function_names.contains(&"id".to_owned());
    let id_tokens = if id_function_defined {
        let id_function = function_map.remove(&"id".to_owned()).unwrap();
        quote! {
            #id_function
        }
    } else {
        quote! {
            fn id() -> &'static str {
                concat!(env!("CARGO_PKG_NAME"), "::", module_path!(), "::", #struct_name)
            }
        }
    };
    // name
    let name_function_defined = function_names.contains(&"name".to_owned());
    let name_tokens = if let Some(name) = attrs.name {
        if name_function_defined {
            abort!(
                &module_impl_span,
                "both define `attribute name` and `function name`, only one can be defined"
            );
        }
        let lit_str = name.value();
        if lit_str.ends_with(")") {
            let expr: syn::Expr = match syn::parse_str(lit_str.as_str()) {
                Ok(expr) => expr,
                Err(error) => abort!(&module_impl_span, error),
            };
            quote! {
                fn name() -> &'static str {
                    #expr
                }
            }
        } else {
            quote! {
                fn name() -> &'static str {
                    #name
                }
            }
        }
    } else {
        if !name_function_defined {
            quote! {
                fn name() -> &'static str {
                    #struct_name
                }
            }
        } else {
            let name_function = function_map.remove(&"name".to_owned()).unwrap();
            quote! {
                #name_function
            }
        }
    };
    // help
    let help_function_defined = function_names.contains(&"help".to_owned());
    let help_tokens = if let Some(help) = attrs.help {
        if help_function_defined {
            abort!(
                &module_impl_span,
                "both define `attribute help` and `function help`, only one can be defined"
            );
        }
        let lit_str = help.value();
        if lit_str.ends_with(")") {
            let expr: syn::Expr = match syn::parse_str(lit_str.as_str()) {
                Ok(expr) => expr,
                Err(error) => abort!(&module_impl_span, error),
            };
            quote! {
                fn help() -> &'static str {
                    #expr
                }
            }
        } else {
            quote! {
                fn help() -> &'static str {
                    #help
                }
            }
        }
    } else {
        if !help_function_defined {
            quote! {
                fn help() -> &'static str {
                    #struct_name
                }
            }
        } else {
            let help_function = function_map.remove(&"help".to_owned()).unwrap();
            quote! {
                #help_function
            }
        }
    };
    // processors
    let processors_function_defined = function_names.contains(&"processors".to_owned());
    let processors_tokens = if let Some(processors) = attrs.processors {
        if processors_function_defined {
            abort!(
                &module_impl_span,
                "both define `attribute processors` and `function processors`, only one can be defined"
            );
        }
        let lit_str = processors.value();
        let lit_str_array = lit_str
            .split("+")
            .into_iter()
            .filter_map(|s| {
                let s = s.trim();
                if s.is_empty() { None } else { Some(s) }
            })
            .map(|sn| {
                if sn.ends_with(")") {
                    let expr: syn::Expr = match syn::parse_str(sn) {
                        Ok(expr) => expr,
                        Err(error) => abort!(&module_impl_span, error),
                    };
                    quote! {
                        #expr
                    }
                } else {
                    let ident = proc_macro2::Ident::new(
                        &sn.to_case(Case::UpperSnake),
                        proc_macro2::Span::call_site(),
                    );
                    quote! {
                        #ident.into()
                    }
                }
            })
            .reduce(|a, b| {
                let mut t = proc_macro2::TokenStream::from(a);
                let com = quote! {,};
                t.extend(com);
                t.extend(b);
                t
            });
        let lit_str_array = if let Some(lit_str_array) = lit_str_array {
            lit_str_array
        } else {
            quote! {}
        };
        quote! {
            fn processors() -> Vec<Processor> {
                vec![#lit_str_array]
            }
        }
    } else {
        if !processors_function_defined {
            quote! {
                fn processors() -> Vec<Processor> {
                    vec![]
                }
            }
        } else {
            let processors_function = function_map.remove(&"processors".to_owned()).unwrap();
            quote! {
                #processors_function
            }
        }
    };
    // surplus_functions
    let mut surplus_functions_tokens = quote! {};
    for (_, function) in function_map {
        surplus_functions_tokens.extend(quote! {
            #function
        });
    }
    // into processor
    let struct_define = quote! {
        #[derive(Debug, Copy, Clone)]
        pub struct #struct_ident();
    };
    let into_process = quote! {
        impl Into<Processor> for #struct_ident {
            fn into(self) -> Processor {
                Processor::Module(Box::new(ProcessModule {
                    id: Self::id(),
                    name: Self::name(),
                    help: Self::help(),
                    processors: Self::processors().into(),
                }))
            }
        }
    };
    // static
    let static_name = proc_macro2::Ident::new(
        &struct_ident.to_string().to_case(Case::UpperSnake),
        proc_macro2::Span::call_site(),
    );
    let static_instance = quote! {
        pub static #static_name: #struct_ident = #struct_ident();
    };
    //// output
    let output = quote! {
        #struct_define
        #[::runbot::re_export::async_trait::async_trait]
        impl Module for #struct_ident {
            #id_tokens
            #name_tokens
            #help_tokens
            #processors_tokens
            #surplus_functions_tokens
        }
        #into_process
        #static_instance
    };
    emit!(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template_repeat() {
        let template = "{:n}{time:n}?{:n}*{time:n}+";
        let items = parse_template(template);
        assert_eq!(
            items,
            vec![
                BotCommandItem::Number(false, false, false),
                BotCommandItem::NumberParam(true, false, false, "time".to_string()),
                BotCommandItem::Number(false, false, true),
                BotCommandItem::NumberParam(false, true, false, "time".to_string()),
            ]
        );
    }

    #[test]
    fn test_plain_text_repeat() {
        let template = "hi?hello*world+";
        let items = parse_template(template);
        assert_eq!(
            items,
            vec![
                BotCommandItem::PlainText(true, false, false, "hi".to_string()),
                BotCommandItem::PlainText(false, false, true, "hello".to_string()),
                BotCommandItem::PlainText(false, true, false, "world".to_string()),
            ]
        );
    }

    #[test]
    fn test_plain_text_chinese_optional() {
        let template = "我是Rust软件工程师?你好吗";
        let items = parse_template(template);
        assert_eq!(
            items,
            vec![
                BotCommandItem::PlainText(true, false, false, "我是Rust软件工程师".to_string()),
                BotCommandItem::PlainText(false, false, false, "你好吗".to_string()),
            ]
        );
    }

    #[test]
    fn test_plain_text_chinese_repeat() {
        let template = "你好*世界+";
        let items = parse_template(template);
        assert_eq!(
            items,
            vec![
                BotCommandItem::PlainText(false, false, true, "你好".to_string()),
                BotCommandItem::PlainText(false, true, false, "世界".to_string()),
            ]
        );
    }

    #[test]
    fn test_plain_text_mix() {
        let template = "a?b*c+d";
        let items = parse_template(template);
        assert_eq!(
            items,
            vec![
                BotCommandItem::PlainText(true, false, false, "a".to_string()),
                BotCommandItem::PlainText(false, false, true, "b".to_string()),
                BotCommandItem::PlainText(false, true, false, "c".to_string()),
                BotCommandItem::PlainText(false, false, false, "d".to_string()),
            ]
        );
    }

    #[test]
    fn test_enum_and_param_repeat() {
        let template = "[a|b][enum_name:a|b][a|b]? [enum_name:a|b]* [enum_name:a|b]+";
        let items = parse_template(template);
        assert_eq!(
            items,
            vec![
                BotCommandItem::Enum(false, false, false, vec!["a".to_string(), "b".to_string()]),
                BotCommandItem::EnumParam(
                    false,
                    false,
                    false,
                    "enum_name".to_string(),
                    vec!["a".to_string(), "b".to_string()]
                ),
                BotCommandItem::Enum(true, false, false, vec!["a".to_string(), "b".to_string()]),
                BotCommandItem::PlainText(false, false, false, "".to_string()),
                BotCommandItem::EnumParam(
                    false,
                    false,
                    true,
                    "enum_name".to_string(),
                    vec!["a".to_string(), "b".to_string()]
                ),
                BotCommandItem::PlainText(false, false, false, "".to_string()),
                BotCommandItem::EnumParam(
                    false,
                    true,
                    false,
                    "enum_name".to_string(),
                    vec!["a".to_string(), "b".to_string()]
                ),
            ]
        );
    }

    #[test]
    fn test_text_to_end_param_repeat() {
        let template = "{text:e}{text:e}?{text:e}*{text:e}+";
        let items = parse_template(template);
        assert_eq!(
            items,
            vec![
                BotCommandItem::TextToEnd(false, false, false, "text".to_string()),
                BotCommandItem::TextToEnd(true, false, false, "text".to_string()),
                BotCommandItem::TextToEnd(false, false, true, "text".to_string()),
                BotCommandItem::TextToEnd(false, true, false, "text".to_string()),
            ]
        );
    }

    #[test]
    fn test_mix_all() {
        let template = "提醒我{time:n}[单位:秒|分|时]?之后[通知|告诉]+{text:e}";
        let items = parse_template(template);
        assert_eq!(
            items,
            vec![
                BotCommandItem::PlainText(false, false, false, "提醒我".to_string()),
                BotCommandItem::NumberParam(false, false, false, "time".to_string()),
                BotCommandItem::EnumParam(
                    true,
                    false,
                    false,
                    "单位".to_string(),
                    vec!["秒".to_string(), "分".to_string(), "时".to_string()]
                ),
                BotCommandItem::PlainText(false, false, false, "之后".to_string()),
                BotCommandItem::Enum(
                    false,
                    true,
                    false,
                    vec!["通知".to_string(), "告诉".to_string()]
                ),
                BotCommandItem::TextToEnd(false, false, false, "text".to_string()),
            ]
        );
    }

    #[test]
    fn test_plain_text_with_no_special() {
        let template = "纯文本无特殊符号";
        let items = parse_template(template);
        assert_eq!(
            items,
            vec![BotCommandItem::PlainText(
                false,
                false,
                false,
                "纯文本无特殊符号".to_string()
            ),]
        );
    }
}
