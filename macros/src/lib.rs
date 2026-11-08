use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Error, Ident, ItemStruct, Path, Result, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct AgentToolArgs {
    args: Type,
    output: Type,
    system: Path,
}

impl Parse for AgentToolArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut args = None;
        let mut output = None;
        let mut system = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "args" => {
                    if args.is_some() {
                        return Err(Error::new(key.span(), "duplicate `args`"));
                    }
                    args = Some(input.parse()?);
                }
                "output" => {
                    if output.is_some() {
                        return Err(Error::new(key.span(), "duplicate `output`"));
                    }
                    output = Some(input.parse()?);
                }
                "system" => {
                    if system.is_some() {
                        return Err(Error::new(key.span(), "duplicate `system`"));
                    }
                    system = Some(input.parse()?);
                }
                _ => {
                    return Err(Error::new(
                        key.span(),
                        "expected `args`, `output`, or `system`",
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            args: args.ok_or_else(|| input.error("missing `args`"))?,
            output: output.ok_or_else(|| input.error("missing `output`"))?,
            system: system.ok_or_else(|| input.error("missing `system`"))?,
        })
    }
}

/// Declares an LLM agent tool and binds its argument, output, and Bevy system types.
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
    let AgentToolArgs {
        args,
        output,
        system,
    } = args;

    let name = &item.ident;
    let (impl_generics, type_generics, where_clause) = item.generics.split_for_impl();

    Ok(quote! {
        #item

        impl #impl_generics #name #type_generics #where_clause {
            fn scene() -> impl #bevy::prelude::Scene {
                #bevy::scene::InsertTemplate {
                    type_id: ::std::any::TypeId::of::<#runtime::tool::Tool>(),
                    template: ::std::boxed::Box::new(#runtime::tool::Tool::of::<Self>()),
                }
            }
        }

        impl #impl_generics #runtime::tool::ToolTrait for #name #type_generics #where_clause {
            type Args = #args;
            type Output = #output;

            fn boxed_system() -> #bevy::ecs::system::BoxedSystem {
                ::std::boxed::Box::new(
                    #bevy::ecs::system::IntoSystem::into_system(#system)
                )
            }
        }
    })
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
