use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Pair, FnArg, ItemTrait, ReturnType, TraitItem, TraitItemMethod,
};
use tonic_build::{Method, Service};

struct MyMethod {
    pub name: String,
    pub identifier: String,
    pub client_streaming: bool,
    pub server_streaming: bool,
    pub request: proc_macro2::TokenStream,
    pub response: proc_macro2::TokenStream,
    pub generated_request: syn::Ident,
    pub generated_response: syn::Ident,
}

impl Method for MyMethod {
    const CODEC_PATH: &'static str = "tonic_rpc::json_codec::MyCodec";
    type Comment = String;

    fn name(&self) -> &str {
        &self.name
    }
    fn identifier(&self) -> &str {
        &self.identifier
    }
    fn comment(&self) -> &[Self::Comment] {
        &[]
    }
    fn client_streaming(&self) -> bool {
        self.client_streaming
    }
    fn server_streaming(&self) -> bool {
        self.server_streaming
    }
    fn request_response_name(
        &self,
        _: &str,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let request = self.generated_request.clone();
        let response = self.generated_response.clone();
        (quote! {super::#request}, quote! {super::#response})
    }
}

struct MyService {
    pub name: String,
    pub package: String,
    pub identifier: String,
    pub methods: Vec<MyMethod>,
}

impl Service for MyService {
    const CODEC_PATH: &'static str = "tonic_rpc::json_codec::MyCodec";
    type Comment = String;
    type Method = MyMethod;

    fn name(&self) -> &str {
        &self.name
    }
    fn package(&self) -> &str {
        &self.package
    }
    fn identifier(&self) -> &str {
        &self.identifier
    }
    fn comment(&self) -> &[Self::Comment] {
        &[]
    }
    fn methods(&self) -> &[Self::Method] {
        &self.methods
    }
}

fn make_method(method: TraitItemMethod, trait_name: &str) -> MyMethod {
    let name = method.sig.ident.to_string();
    let server_streaming = method
        .attrs
        .iter()
        .any(|attr| attr.path.is_ident("server_streaming"));
    let mut args: Vec<_> = method.sig.inputs.into_pairs().collect();
    if args.len() != 1 {
        panic!("Invalid rpc argument type");
    }
    let request = match args.pop() {
        Some(Pair::End(FnArg::Typed(pat))) => pat.ty.to_token_stream(),
        _ => panic!("Invalid rpc argument type"),
    };
    let response = match method.sig.output {
        ReturnType::Default => quote! { "()" },
        ReturnType::Type(_arrow, ty) => ty.to_token_stream(),
    };
    MyMethod {
        identifier: name.clone(),
        name: name.clone(),
        client_streaming: false,
        server_streaming,
        request,
        response,
        generated_request: quote::format_ident!(
            "__tonic_generated_{}_{}_request",
            trait_name,
            name.clone()
        ),
        generated_response: quote::format_ident!(
            "__tonic_generated_{}_{}_response",
            trait_name,
            name.clone()
        ),
    }
}

#[proc_macro_attribute]
pub fn tonic_rpc(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    let trait_ = parse_macro_input!(item as ItemTrait);
    let name = trait_.ident.to_string();
    let methods: Vec<_> = trait_
        .items
        .into_iter()
        .filter_map(|item| match item {
            TraitItem::Method(method) => Some(make_method(method, &name)),
            _ => None,
        })
        .collect();
    let service = MyService {
        package: name.clone(),
        identifier: name.clone(),
        name,
        methods,
    };
    let client = tonic_build::client::generate(&service, "");
    let server = tonic_build::server::generate(&service, "");
    let types = service.methods.iter().map(|m| {
        let request_name = m.generated_request.clone();
        let request_type = m.request.clone();
        let response_name = m.generated_response.clone();
        let response_type = m.response.clone();
        quote! {
            type #request_name = #request_type;
            type #response_name = #response_type;
        }
    });
    let types = quote! { #( #types )*};
    (quote! {
        #types
        #client
        #server
    })
    .into()
}
