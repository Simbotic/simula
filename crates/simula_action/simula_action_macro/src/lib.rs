use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(InputDerive)]
pub fn input_action_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_input_action_macro(&ast)
}

fn impl_input_action_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl PepeDerive for #name {
            fn hello_input_action() {
                println!("Hello, Macro! My name is {}!", stringify!(#name));
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(KeyboardInput)]
pub fn keyboard_intput_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_keyboard_input_macro(&ast)
}

fn impl_keyboard_input_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl Key for #name {
            const KEYCODE: KeyCode = KeyCode::#name;
        }
    };
    gen.into()
}
