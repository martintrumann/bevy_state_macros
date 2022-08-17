mod state;

use proc_macro::TokenStream;
use proc_macro2::{Punct, Spacing};

use quote::{__private::mk_ident, format_ident, quote, ToTokens, TokenStreamExt};
use syn::{bracketed, parse::Parse, parse_macro_input, Ident, Token, Type};

struct System {
    name: Ident,
    generics: Vec<Type>,
}

impl Parse for System {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;

        if input.is_empty() || input.peek(Token!(,)) {
            return Ok(Self {
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

        Ok(Self { name, generics })
    }
}
impl ToTokens for System {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        tokens.append(self.name.clone());

        if !self.generics.is_empty() {
            tokens.append(Punct::new(':', Spacing::Joint));
            tokens.append(Punct::new(':', Spacing::Alone));

            tokens.append(Punct::new('<', Spacing::Alone));

            tokens.append_separated(self.generics.clone(), Punct::new(',', Spacing::Alone));

            tokens.append(Punct::new('>', Spacing::Alone));
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
            systems: systems
                .into_iter()
                .map(|mut s| {
                    s.name = format_ident!("add_{}", s.name);
                    s
                })
                .collect(),
        })
    }
}

#[proc_macro]
pub fn add_systems(input: TokenStream) -> TokenStream {
    let AddSystems { app, systems } = parse_macro_input!(input as AddSystems);

    quote!({
        let mut sets = std::collections::HashMap::new();
        #( #systems (&mut sets); )*
        for (_, v) in sets.into_iter() {
            #app.add_system_set(v);
        }
    })
    .into()
}

fn inner(num: u8, input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let state = parse_macro_input!(input as state::State);

    let system = parse_macro_input!(annotated_item as syn::ItemFn);

    let name = system.sig.ident.clone();
    let add_name = format_ident!("add_{}", name);

    let state_name = state.name;
    let state_variant = state.variant;
    let extra = state.extra;

    let func = mk_ident(
        match num {
            0 => "on_update",
            1 => "on_enter",
            2 => "on_exit",
            3 => "on_pause",
            4 => "on_resume",
            _ => unreachable!(),
        },
        None,
    );

    let generics = system.sig.generics.clone();

    let gen_tokens = generics.type_params().map(|p| &p.ident);

    let out = quote!(
        fn #add_name #generics (map: &mut std::collections::HashMap<(#state_name, u8), SystemSet>) {
            let ss = map
                .remove(&(#state_name::#state_variant, #num))
                .unwrap_or_else(|| SystemSet::#func(#state_name::#state_variant));

            map.insert(
                (#state_name::#state_variant, #num),
                ss.with_system(#name::< #(#gen_tokens),* > #extra)
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
    inner(0, input, annotated_item)
}

#[proc_macro_attribute]
pub fn on_enter(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    inner(1, input, annotated_item)
}

#[proc_macro_attribute]
pub fn on_exit(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    inner(2, input, annotated_item)
}

#[proc_macro_attribute]
pub fn on_pause(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    inner(3, input, annotated_item)
}

#[proc_macro_attribute]
pub fn on_resume(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    inner(4, input, annotated_item)
}
