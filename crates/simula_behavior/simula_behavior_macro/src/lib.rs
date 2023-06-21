use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(BehaviorFactory, attributes(BehaviorAttributes))]
pub fn behavior_factory_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_behavior_factory(&ast)
}

fn impl_behavior_factory(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let attributes_type = get_attributes_type(&ast.attrs).unwrap();

    if let syn::Data::Enum(data_enum) = &ast.data {
        let insert_variant_impls: Vec<_> = data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                quote! {
                    Self::#variant_ident(data) => BehaviorInfo::insert_with(commands, data),
                }
            })
            .collect();

        let label_variant_impls: Vec<_> = data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                let variant_ident_label = format!("{}", variant_ident);
                quote! {
                    Self::#variant_ident(_) => #variant_ident_label,
                }
            })
            .collect();

        let typ_variant_impls: Vec<_> = data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                let variant_argument = get_variant_argument(&variant.fields).unwrap();
                quote! {
                    Self::#variant_ident(_) => <#variant_argument as BehaviorInfo>::TYPE,
                }
            })
            .collect();

        let reflect_variant_impls: Vec<_> = data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                quote! {
                    Self::#variant_ident(data) => data,
                }
            })
            .collect();

        let copy_from_variant_impls: Vec<_> = data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                let variant_argument = get_variant_argument(&variant.fields).unwrap();
                quote! {
                    Self::#variant_ident(data) => *data = world.get::<#variant_argument>(entity).ok_or(BehaviorMissing)?.clone(),
                }
            })
            .collect();

        let list_variant_impls: Vec<_> = data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                quote! {
                    Self::#variant_ident(Default::default()),
                }
            })
            .collect();

        let gen = quote! {
            impl BehaviorFactory for #name {
                type Attributes = #attributes_type;

                fn insert(&self, commands: &mut EntityCommands) {
                    match self {
                        #(#insert_variant_impls)*
                    }
                }

                fn label(&self) -> &str {
                    match self {
                        #(#label_variant_impls)*
                    }
                }

                fn typ(&self) -> BehaviorType {
                    match self {
                        #(#typ_variant_impls)*
                    }
                }

                fn reflect(&self) -> &dyn Reflect {
                    match self {
                        #(#reflect_variant_impls)*
                    }
                }

                fn reflect_mut(&mut self) -> &mut dyn Reflect {
                    match self {
                        #(#reflect_variant_impls)*
                    }
                }

                fn copy_from(&mut self, entity: Entity, world: &World) -> Result<(), BehaviorMissing> {
                    match self {
                        #(#copy_from_variant_impls)*
                    }
                    Ok(())
                }

                fn list() -> Vec<Self> {
                    vec![
                        #(#list_variant_impls)*
                    ]
                }
            }
        };

        gen.into()
    } else {
        panic!("BehaviorFactory can only be derived for enums");
    }
}

fn get_attributes_type(attrs: &[syn::Attribute]) -> syn::Result<syn::Type> {
    for attr in attrs {
        if attr.path.is_ident("BehaviorAttributes") {
            return attr.parse_args();
        }
    }
    Err(syn::Error::new_spanned(
        attrs.first().unwrap(),
        "Expected #[BehaviorAttributes(Type)] attribute",
    ))
}

fn get_variant_argument(fields: &syn::Fields) -> Option<&syn::Type> {
    match fields {
        syn::Fields::Unnamed(fields_unnamed) => {
            // Assuming that variant has exactly one field
            let field = fields_unnamed.unnamed.first()?;
            Some(&field.ty)
        }
        _ => None,
    }
}
