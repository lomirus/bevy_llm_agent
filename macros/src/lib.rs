use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Error, Expr, ExprLit, Ident, ItemStruct, Lit, Meta, MetaNameValue, Path, Result, Token,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct AgentToolArgs {
    system: Path,
}

impl Parse for AgentToolArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let key: Ident = input.parse()?;
        if key != "system" {
            return Err(Error::new(key.span(), "expected `system`"));
        }
        input.parse::<Token![=]>()?;
        let system = input.parse()?;
        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }
        Ok(Self { system })
    }
}

/// Declares an LLM agent tool backed by a Bevy system.
///
/// The system takes `In<ToolInput<Args>>` and returns the result sent back to the model.
/// The tool is named after the struct in snake case, and its doc comment is sent to the
/// model as the tool description.
///
/// This also derives `SceneComponent`, `Clone`, and `Default`, so the tool can be
/// attached in `bsn!` with `AgentTools [ @MyTool ]`.
#[proc_macro_attribute]
pub fn agent_tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AgentToolArgs);
    let item = parse_macro_input!(item as ItemStruct);

    expand_agent_tool(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_agent_tool(args: AgentToolArgs, item: ItemStruct) -> Result<TokenStream2> {
    let runtime = dependency_crate("bevy-llm-agent", &item)?;
    let bevy = dependency_crate("bevy", &item)?;
    let AgentToolArgs { system } = args;

    let name = &item.ident;
    let tool_name = snake_case(&name.unraw().to_string());
    let description = doc_comment(&item)?;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    Ok(quote! {
        #[derive(
            #bevy::prelude::SceneComponent,
            ::core::clone::Clone,
            ::core::default::Default,
        )]
        #item

        impl #impl_generics #name #type_generics #where_clause {
            fn scene() -> impl #bevy::prelude::Scene {
                #bevy::scene::InsertTemplate {
                    type_id: ::core::any::TypeId::of::<#runtime::tool::Tool>(),
                    template: ::std::boxed::Box::new(
                        #runtime::tool::Tool::new(#tool_name, #description, #system)
                    ),
                }
            }
        }
    })
}

fn doc_comment(item: &ItemStruct) -> Result<String> {
    let lines: Vec<String> = item
        .attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            Meta::NameValue(MetaNameValue {
                path,
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(line),
                        ..
                    }),
                ..
            }) if path.is_ident("doc") => Some(line.value()),
            _ => None,
        })
        .map(|line| line.strip_prefix(' ').map(str::to_owned).unwrap_or(line))
        .collect();
    let description = lines.join("\n").trim().to_owned();
    if description.is_empty() {
        return Err(Error::new(
            item.ident.span(),
            "agent tools need a doc comment, which is sent to the model as the tool description",
        ));
    }
    Ok(description)
}

/// `GetHTTPStatus` -> `get_http_status`.
fn snake_case(ident: &str) -> String {
    let chars: Vec<char> = ident.chars().collect();
    let mut name = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev = i.checked_sub(1).map(|i| chars[i]);
            let next = chars.get(i + 1);
            let word_start = prev.is_some_and(|p| p.is_lowercase() || p.is_ascii_digit())
                || (prev.is_some_and(char::is_uppercase) && next.is_some_and(|n| n.is_lowercase()));
            if word_start {
                name.push('_');
            }
            name.extend(c.to_lowercase());
        } else {
            name.push(c);
        }
    }
    name
}

fn dependency_crate(package: &str, item: &ItemStruct) -> Result<TokenStream2> {
    match crate_name(package) {
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(FoundCrate::Name(name)) => {
            let name = format_ident!("{name}");
            Ok(quote!(::#name))
        }
        Err(error) => Err(Error::new(item.ident.span(), error.to_string())),
    }
}
