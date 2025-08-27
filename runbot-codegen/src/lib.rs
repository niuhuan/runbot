extern crate proc_macro;

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
    let struct_name = snake_to_upper_camel(&fn_name.to_string());
    let struct_name = proc_macro2::Ident::new(&struct_name, proc_macro2::Span::call_site());
    let static_name = to_upper_snake(&fn_name.to_string());
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

fn snake_to_upper_camel(s: &str) -> String {
    s.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn to_upper_snake(s: &str) -> String {
    s.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_uppercase())
        .collect::<Vec<String>>()
        .join("_")
}
