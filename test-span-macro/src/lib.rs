use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use syn::parse_macro_input;
use syn::AttributeArgs;
use syn::ExprAssign;
use syn::ItemFn;
use syn::Path;
use syn::ReturnType;

#[proc_macro_attribute]
pub fn test_span(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as AttributeArgs);
    let test_fn = parse_macro_input!(item as ItemFn);

    let macro_attrs = if attrs.as_slice().is_empty() {
        quote! { test }
    } else {
        quote! {#(#attrs)*}
    };

    let fn_attrs = &test_fn.attrs;

    let mut level = quote!(::test_span::reexports::tracing::Level::INFO);

    let mut target_directives: Vec<_> = Vec::new();

    // Get tracing level from #[level(tracing::Level::INFO)]
    let fn_attrs = fn_attrs
        .iter()
        .filter(|attr| {
            let path = &attr.path;
            match quote!(#path).to_string().as_str() {
                "level" => {
                    let value: Path = attr.parse_args().expect(
                        "wrong level attribute syntax. Example: #[level(tracing::Level::INFO)]",
                    );
                    level = quote!(#value);
                    false
                }
                "target" => {
                    let value: ExprAssign = attr.parse_args().expect("each targetFilter directive expects a single assignment expression. example: #[targetFilter(apollo_router=debug)]");
                    // foo = Level::INFO => .with_target("foo".to_string(), Level::INFO)
                    let name = value.left;
                    let mut target_name = quote!(#name).to_string();
                    target_name.retain(|c| !c.is_whitespace());

                    let target_value = value.right;

                    target_directives.push(quote!(.with_target(#target_name .to_string(), #target_value)));

                    false
                }
                _ => true,
            }
        })
        .collect::<Vec<_>>();

    let maybe_async = &test_fn.sig.asyncness;

    let body = &test_fn.block;
    let test_name = &test_fn.sig.ident;
    let output_type = &test_fn.sig.output;

    let maybe_semicolon = if let ReturnType::Default = output_type {
        quote! {;}
    } else {
        quote! {}
    };

    let run_test = if maybe_async.is_some() {
        async_test(test_name)
    } else {
        sync_test(test_name)
    };

    let ret = quote! {#output_type};

    let subscriber_boilerplate = subscriber_boilerplate(level, target_directives);

    quote! {
      #[#macro_attrs]
      #(#fn_attrs)*
      #maybe_async fn #test_name() #ret {
        use ::test_span::reexports::tracing::Instrument;
        #maybe_async fn #test_name(get_telemetry: impl Fn() -> (::test_span::Span, ::test_span::Records), get_logs: impl Fn() -> ::test_span::Records, get_spans: impl Fn() -> ::test_span::Span) #ret
          #body


        #subscriber_boilerplate

        #run_test #maybe_semicolon
      }
    }
    .into()
}

fn async_test(test_name: &Ident) -> TokenStream2 {
    quote! {
        #test_name(get_telemetry, get_logs, get_spans)
            .instrument(root_span).await
    }
}

fn sync_test(test_name: &Ident) -> TokenStream2 {
    quote! {
        root_span.in_scope(|| {
            #test_name(get_telemetry, get_logs, get_spans)
        });
    }
}
fn subscriber_boilerplate(
    level: TokenStream2,
    target_directives: Vec<TokenStream2>,
) -> TokenStream2 {
    quote! {
        let filter = ::test_span::Filter::new(#level) #(#target_directives)*;

        ::test_span::init();

        let root_span = ::test_span::reexports::tracing::span!(#level, "root");

        let root_id = root_span.id().clone().expect("couldn't get root span id; this cannot happen.");

        #[allow(unused)]
        let get_telemetry = || ::test_span::get_telemetry_for_root(&root_id, &filter);

        #[allow(unused)]
        let get_logs = || ::test_span::get_logs_for_root(&root_id, &filter);


        #[allow(unused)]
        let get_spans = || ::test_span::get_spans_for_root(&root_id, &filter);
    }
}
