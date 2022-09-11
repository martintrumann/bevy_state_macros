//! This is a simple macro crate to allow bevy systems to be added using a macro.
//!
//! # Example
//! ```
#![doc = include_str!("../examples/simple.rs")]
//! ```

mod state;

use proc_macro::TokenStream;

use quote::{__private::mk_ident, format_ident, quote};
use state::State;
use syn::{bracketed, parenthesized, parse::Parse, parse_macro_input, Ident, Token, Type};

const ON_UPDATE: u8 = 0;
const ON_ENTER: u8 = 1;
const ON_EXIT: u8 = 2;
const ON_PAUSE: u8 = 3;
const ON_RESUME: u8 = 4;

macro_rules! add_name( () => {"_add_{}"});

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

    let systems = systems.iter().map(|s| {
        if let Some((
            num,
            State {
                name: state_name,
                variant: state_variant,
                extra: state_extra,
            },
        )) = &s.attr
        {
            let func = mk_ident(
                match *num {
                    ON_UPDATE => "add_system_on_update",
                    ON_ENTER => "add_system_on_enter",
                    ON_EXIT => "add_system_on_exit",
                    ON_PAUSE => "add_system_on_pause",
                    ON_RESUME => "add_system_on_resume",
                    _ => unreachable!(),
                },
                None,
            );
            let name = &s.name;
            let extra = (!s.generics.is_empty()).then(|| {
                let generics = &s.generics;
                quote!( ::< #(#generics),* > )
            });

            quote! {
                <App as bevy_state_stack::AppStateStackExt>::#func(
                    &mut #app,
                    #state_name::#state_variant,
                    #name #state_extra #extra
                );
            }
        } else {
            let name = format_ident!(add_name!(), s.name);
            quote!( #name(&mut #app); )
        }
    });

    quote!({
        #( #systems )*
    })
    .into()
}

fn inner(num: u8, input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let state = parse_macro_input!(input as state::State);

    let system = parse_macro_input!(annotated_item as syn::ItemFn);

    let name = &system.sig.ident;
    let add_name = format_ident!(add_name!(), name);

    let func = mk_ident(
        match num {
            ON_UPDATE => "add_system_on_update",
            ON_ENTER => "add_system_on_enter",
            ON_EXIT => "add_system_on_exit",
            ON_PAUSE => "add_system_on_pause",
            ON_RESUME => "add_system_on_resume",
            _ => unreachable!(),
        },
        None,
    );

    let State {
        name: state_name,
        variant: state_variant,
        extra: state_extra,
    } = &state;

    let out = quote!(
        fn #add_name(app: &mut bevy::prelude::App) {
            <App as bevy_state_stack::AppStateStackExt>::#func(
                app,
                #state_name::#state_variant,
                #name #state_extra
            );
        }

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
