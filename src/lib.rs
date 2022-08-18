//! This is a simple macro crate to allow bevy systems to be added using a macro.
//!
//! # Example
//! ```
#![doc = include_str!("../examples/simple.rs")]
//! ```

mod state;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::{__private::mk_ident, format_ident, quote, ToTokens};
use syn::{bracketed, parenthesized, parse::Parse, parse_macro_input, Ident, Token, Type};

const ON_UPDATE: u8 = 0;
const ON_ENTER: u8 = 1;
const ON_EXIT: u8 = 2;
const ON_PAUSE: u8 = 3;
const ON_RESUME: u8 = 4;

struct System {
    attr: Option<(u8, state::State)>,
    name: Ident,
    generics: Vec<Type>,
}

impl Parse for System {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr = if input.peek(Token!(#)) {
            input.parse::<Token!(#)>().unwrap();

            let inner;
            bracketed!(inner in input);

            let name: Ident = inner.parse()?;

            let num = match name.to_string().as_str() {
                "on" => ON_UPDATE,
                "on_update" => ON_UPDATE,
                "on_enter" => ON_ENTER,
                "on_exit" => ON_EXIT,
                "on_pause" => ON_PAUSE,
                "on_resume" => ON_RESUME,
                _ => return Err(syn::Error::new(name.span(), "Not a bevy state trigger.")),
            };

            let state;
            parenthesized!(state in inner);
            let state = state.parse::<state::State>()?;

            Some((num, state))
        } else {
            None
        };

        let name = input.parse()?;

        if input.is_empty() || input.peek(Token!(,)) {
            return Ok(Self {
                attr,
                name,
                generics: Vec::new(),
            });
        }

        input.parse::<Token!(::)>()?;
        input.parse::<Token!(<)>()?;

        let mut generics = Vec::new();
        while !input.peek(Token!(>)) {
            generics.push(input.parse::<Type>()?);
            let _ = input.parse::<Token!(,)>();
        }

        input.parse::<Token!(>)>()?;

        Ok(Self {
            attr,
            name,
            generics,
        })
    }
}

impl ToTokens for System {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        if let Some((num, ref state)) = self.attr {
            let extra = (!self.generics.is_empty()).then(|| {
                let generics = &self.generics;
                quote!( ::< #(#generics),* > )
            });

            ss_ins(num, state, extra, &self.name).to_tokens(tokens)
        } else {
            let name = format_ident!("add_{}", self.name);

            quote!( #name(&mut sets); ).to_tokens(tokens)
        }
    }
}

struct AddSystems {
    app: Ident,
    systems: Vec<System>,
}

impl Parse for AddSystems {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let app = input.parse()?;

        let _ = input.parse::<Token!(,)>();

        let systems;
        bracketed!(systems in input);

        let systems = systems.parse_terminated::<_, Token![,]>(System::parse)?;

        Ok(Self {
            app,
            systems: systems.into_iter().collect(),
        })
    }
}

#[proc_macro]
/// Add systems to an app.
pub fn add_systems(input: TokenStream) -> TokenStream {
    let AddSystems { app, systems } = parse_macro_input!(input as AddSystems);

    quote!({
        let mut sets = std::collections::HashMap::new();

        #( #systems )*

        for (_, v) in sets.into_iter() {
            #app.add_system_set(v);
        }
    })
    .into()
}

fn ss_ins(
    num: u8,
    state: &state::State,
    extra: Option<TokenStream2>,
    name: &Ident,
) -> TokenStream2 {
    let func = mk_ident(
        match num {
            ON_UPDATE => "on_update",
            ON_ENTER => "on_enter",
            ON_EXIT => "on_exit",
            ON_PAUSE => "on_pause",
            ON_RESUME => "on_resume",
            _ => unreachable!(),
        },
        None,
    );

    let state_name = &state.name;
    let state_variant = &state.variant;
    let state_extra = &state.extra;

    quote!({
        let ss = sets
            .remove(&(#state_name::#state_variant, #num))
            .unwrap_or_else(|| SystemSet::#func(#state_name::#state_variant));

        sets.insert(
            (#state_name::#state_variant, #num),
            ss.with_system(#name #extra #state_extra)
        );
    })
}

fn inner(num: u8, input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let state = parse_macro_input!(input as state::State);

    let system = parse_macro_input!(annotated_item as syn::ItemFn);

    let name = &system.sig.ident;
    let add_name = format_ident!("add_{}", name);

    let state_name = state.name.clone();

    let inner = ss_ins(num, &state, None, name);

    let out = quote!(
        fn #add_name (sets: &mut std::collections::HashMap<(#state_name, u8), SystemSet>) #inner

        #system
    );

    out.into()
}

#[proc_macro_attribute]
pub fn on(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    on_update(input, annotated_item)
}

#[proc_macro_attribute]
pub fn on_update(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    inner(ON_UPDATE, input, annotated_item)
}

#[proc_macro_attribute]
pub fn on_enter(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    inner(ON_ENTER, input, annotated_item)
}

#[proc_macro_attribute]
pub fn on_exit(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    inner(ON_EXIT, input, annotated_item)
}

#[proc_macro_attribute]
pub fn on_pause(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    inner(ON_PAUSE, input, annotated_item)
}

#[proc_macro_attribute]
pub fn on_resume(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    inner(ON_RESUME, input, annotated_item)
}
