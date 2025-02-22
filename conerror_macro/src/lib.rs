use proc_macro::TokenStream;

use quote::quote;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::visit_mut::{visit_expr_try_mut, visit_impl_item_fn_mut, VisitMut};
use syn::{parse_macro_input, parse_quote_spanned, ExprTry, ImplItemFn, ItemFn, ItemImpl, Type};

#[proc_macro_attribute]
pub fn conerror(_: TokenStream, input: TokenStream) -> TokenStream {
    match parse_macro_input!(input as Item) {
        Item::Fn(mut f) => {
            MapErr::new(None, Some(f.sig.ident.to_string())).visit_item_fn_mut(&mut f);
            quote!(#f).into()
        }
        Item::Impl(mut i) => {
            MapErr::new(Some(i.self_ty.clone()), None).visit_item_impl_mut(&mut i);
            quote!(#i).into()
        }
    }
}

enum Item {
    Fn(ItemFn),
    Impl(ItemImpl),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ahead = input.fork();
        match ahead.parse::<ItemFn>() {
            Ok(v) => {
                input.advance_to(&ahead);
                Ok(Item::Fn(v))
            }
            Err(e) => match input.parse::<ItemImpl>() {
                Ok(v) => Ok(Item::Impl(v)),
                Err(mut e1) => {
                    e1.combine(e);
                    Err(e1)
                }
            },
        }
    }
}

struct MapErr {
    self_ty: Option<Box<Type>>,
    ident: Option<String>,
}

impl MapErr {
    fn new(self_ty: Option<Box<Type>>, ident: Option<String>) -> Self {
        Self { self_ty, ident }
    }
}

impl VisitMut for MapErr {
    fn visit_expr_try_mut(&mut self, i: &mut ExprTry) {
        let ident = self.ident.as_ref().unwrap();
        let module = match self.self_ty {
            Some(ref v) => quote!(std::any::type_name::<#v>()),
            None => quote!(module_path!()),
        };
        let expr = &i.expr;
        *i.expr = parse_quote_spanned! {expr.span() =>
            #expr.map_err(|err| conerror::Error::chain(err, file!(), line!(), #ident, #module))
        };
        visit_expr_try_mut(self, i);
    }

    fn visit_impl_item_fn_mut(&mut self, i: &mut ImplItemFn) {
        let mut indices = vec![];
        for (i, attr) in i.attrs.iter().enumerate() {
            if attr.path().is_ident("conerror") {
                indices.push(i);
            }
        }
        if indices.is_empty() {
            return;
        }

        for idx in indices {
            i.attrs.remove(idx);
        }
        self.ident = Some(i.sig.ident.to_string());
        visit_impl_item_fn_mut(self, i);
    }
}
