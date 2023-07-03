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
                    Self::#variant_ident(data) => BehaviorSpec::insert_with(commands, data),
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

        let icon_variant_impls: Vec<_> = data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                let variant_argument = get_variant_argument(&variant.fields).unwrap();
                quote! {
                    Self::#variant_ident(_) => <#variant_argument as BehaviorSpec>::ICON,
                }
            })
            .collect();

        let desc_variant_impls: Vec<_> = data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                let variant_argument = get_variant_argument(&variant.fields).unwrap();
                quote! {
                    Self::#variant_ident(_) => <#variant_argument as BehaviorSpec>::DESC,
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
                    Self::#variant_ident(_) => <#variant_argument as BehaviorSpec>::TYPE,
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

        let ui_variant_impls: Vec<_> = data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                quote! {
                    Self::#variant_ident(data) => data.ui(state, ui, type_registry),
                }
            })
            .collect();

        let ui_readonly_variant_impls: Vec<_> = data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                quote! {
                    Self::#variant_ident(data) => data.ui_readonly(state, ui, type_registry),
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

                fn icon(&self) -> &str {
                    match self {
                        #(#icon_variant_impls)*
                    }
                }

                fn desc(&self) -> &str {
                    match self {
                        #(#desc_variant_impls)*
                    }
                }

                fn typ(&self) -> BehaviorType {
                    match self {
                        #(#typ_variant_impls)*
                    }
                }

                fn inner_reflect(&self) -> &dyn Reflect {
                    match self {
                        #(#reflect_variant_impls)*
                    }
                }

                fn inner_reflect_mut(&mut self) -> &mut dyn Reflect {
                    match self {
                        #(#reflect_variant_impls)*
                    }
                }

                fn ui(
                    &mut self,
                    state: Option<protocol::BehaviorState>,
                    ui: &mut bevy_inspector_egui::egui::Ui,
                    type_registry: &bevy::reflect::TypeRegistry,
                ) -> bool {
                    match self {
                        #(#ui_variant_impls)*
                    }
                }

                fn ui_readonly(
                    &self,
                    state: Option<protocol::BehaviorState>,
                    ui: &mut bevy_inspector_egui::egui::Ui,
                    type_registry: &bevy::reflect::TypeRegistry,
                ) {
                    match self {
                        #(#ui_readonly_variant_impls)*
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
