use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
	Data, DeriveInput, Error, Expr, ExprLit, Field, Fields, FieldsUnnamed, Index, Lit, Meta,
	MetaNameValue, Result, parse_macro_input,
};

/// Derive macro that generates a `dissolve(self)` method for structs.
///
/// For named structs, returns a struct with public fields named `{OriginalName}Dissolved`.
/// For tuple structs, returns a tuple with the included fields.
///
/// # Attributes
///
/// - `#[dissolved(skip)]` - Skip this field in the dissolved struct/tuple
/// - `#[dissolved(rename = "new_name")]` - Rename this field in the dissolved struct
#[proc_macro_derive(Dissolve, attributes(dissolved))]
pub fn derive_dissolve(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	match generate_dissolve_impl(&input) {
		Ok(tokens) => tokens.into(),
		Err(err) => err.to_compile_error().into(),
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DissolvedOption {
	Skip,
	Rename(syn::Ident),
}

#[derive(Debug, Clone)]
struct FieldInfo {
	should_skip: bool,
	renamed_to: Option<syn::Ident>,
}

impl DissolvedOption {
	const IDENT: &str = "dissolved";

	const SKIP_IDENT: &str = "skip";

	const RENAME_IDENT: &str = "rename";

	fn from_meta(meta: &Meta) -> Result<Self> {
		let unknown_attribute_err = |path: &syn::Path| {
			let path_str = path
				.segments
				.iter()
				.map(|seg| seg.ident.to_string())
				.collect::<Vec<_>>()
				.join("::");

			Error::new_spanned(
				path,
				format!(
					"unknown dissolved attribute option '{}'; supported options: {}, {} = \"new_name\"",
					Self::SKIP_IDENT,
					Self::RENAME_IDENT,
					path_str,
				),
			)
		};

		let opt = match meta {
			Meta::Path(path) => {
				if !path.is_ident(Self::SKIP_IDENT) {
					return Err(unknown_attribute_err(path));
				}

				DissolvedOption::Skip
			},
			Meta::NameValue(MetaNameValue { path, value, .. }) => {
				if !path.is_ident(Self::RENAME_IDENT) {
					return Err(unknown_attribute_err(path));
				}

				match value {
					Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) => {
						syn::parse_str::<syn::Ident>(&lit_str.value())
							.map(DissolvedOption::Rename)?
					},
					_ => {
						return Err(Error::new_spanned(
							value,
							format!("{} value must be a string literal", Self::RENAME_IDENT),
						));
					},
				}
			},
			Meta::List(_) => {
				return Err(Error::new_spanned(
					meta,
					"nested lists are not supported in dissolved attributes",
				));
			},
		};

		Ok(opt)
	}
}

impl FieldInfo {
	fn new() -> Self {
		Self { should_skip: false, renamed_to: None }
	}
}

fn generate_dissolve_impl(input: &DeriveInput) -> Result<proc_macro2::TokenStream> {
	let struct_name = &input.ident;

	let Data::Struct(data_struct) = &input.data else {
		return Err(Error::new_spanned(
			input,
			"Dissolve can only be derived for structs",
		));
	};

	match &data_struct.fields {
		Fields::Named(fields) => generate_named_struct_impl(struct_name, fields),
		Fields::Unnamed(fields) => generate_tuple_struct_impl(struct_name, fields),
		Fields::Unit => Err(Error::new_spanned(
			input,
			"Dissolve cannot be derived for unit structs",
		)),
	}
}

fn generate_named_struct_impl(
	struct_name: &syn::Ident,
	fields: &syn::FieldsNamed,
) -> Result<proc_macro2::TokenStream> {
	let included_fields: Vec<_> = fields
		.named
		.iter()
		.map(|field| {
			let info = get_field_info(field)?;
			if info.should_skip {
				Ok((None, info))
			} else {
				Ok((Some(field), info))
			}
		})
		.filter_map(|res| match res {
			Ok((Some(field), info)) => Some(Ok((field, info))),
			Err(e) => Some(Err(e)),
			_ => None,
		})
		.collect::<Result<_>>()?;

	if included_fields.is_empty() {
		return Err(Error::new_spanned(
			struct_name,
			"cannot create dissolved struct with no fields (all fields are skipped)",
		));
	}

	let field_definitions = included_fields.iter().map(|(field, info)| {
		// unwrap is safe because struct has named fields
		let original_name = field.ident.as_ref().unwrap();
		let ty = &field.ty;

		let dissolved_field_name = match &info.renamed_to {
			Some(new_name) => new_name,
			None => original_name,
		};

		quote! { pub #dissolved_field_name: #ty }
	});

	let field_moves = included_fields.iter().map(|(field, info)| {
		// unwrap is safe because struct has named fields
		let original_name = field.ident.as_ref().unwrap();

		let dissolved_field_name = match &info.renamed_to {
			Some(new_name) => new_name,
			None => original_name,
		};

		quote! { #dissolved_field_name: self.#original_name }
	});

	let dissolved_struct_name = format_ident!("{}Dissolved", struct_name);

	Ok(quote! {
		pub struct #dissolved_struct_name {
			#(#field_definitions),*
		}

		impl #struct_name {
			/// Dissolve this struct into its public-field equivalent.
			///
			/// This method consumes the original struct and returns a new struct where all included
			/// fields are made public and optionally renamed.
			pub fn dissolve(self) -> #dissolved_struct_name {
				#dissolved_struct_name {
					#(#field_moves),*
				}
			}
		}
	})
}

fn generate_tuple_struct_impl(
	struct_name: &syn::Ident,
	fields: &FieldsUnnamed,
) -> Result<proc_macro2::TokenStream> {
	// For tuple structs, only `skip` is supported (`rename` does not make sense)
	let included_fields: Vec<_> = fields
		.unnamed
		.iter()
		.enumerate()
		.filter_map(|(index, field)| {
			match get_field_info(field) {
				Ok(info) => {
					if info.should_skip {
						None
					} else {
						// Check if rename was attempted on tuple struct
						if info.renamed_to.is_some() {
							Some(Err(Error::new_spanned(
								field,
								format!(
									"{} is unsupported for tuple struct fields, only {} is allowed",
									DissolvedOption::RENAME_IDENT,
									DissolvedOption::SKIP_IDENT,
								),
							)))
						} else {
							Some(Ok((index, field)))
						}
					}
				},
				Err(err) => Some(Err(err)),
			}
		})
		.collect::<Result<_>>()?;

	if included_fields.is_empty() {
		return Err(Error::new_spanned(
			struct_name,
			"cannot create dissolved tuple with no fields (all fields are skipped)",
		));
	}

	let tuple_types = included_fields.iter().map(|(_, field)| &field.ty);
	let tuple_type = if included_fields.len() == 1 {
		// Single element tuple needs trailing comma
		let ty = &included_fields[0].1.ty;
		quote! { (#ty,) }
	} else {
		quote! { (#(#tuple_types),*) }
	};

	let field_moves = included_fields.iter().map(|(original_index, _)| {
		let index = Index::from(*original_index);
		quote! { self.#index }
	});

	let tuple_construction = if included_fields.len() == 1 {
		// Single element tuple needs trailing comma
		quote! { (#(#field_moves,)*) }
	} else {
		quote! { (#(#field_moves),*) }
	};

	Ok(quote! {
		impl #struct_name {
			/// Dissolve this tuple struct into a tuple of its included non-skipped fields.
			pub fn dissolve(self) -> #tuple_type {
				#tuple_construction
			}
		}
	})
}

fn get_field_info(field: &Field) -> Result<FieldInfo> {
	let mut field_info = FieldInfo::new();

	for attr in field.attrs.iter().filter(|attr| attr.path().is_ident(DissolvedOption::IDENT)) {
		match attr.meta.clone() {
			Meta::List(_) => {
				// Parse #[dissolved(skip)] or #[dissolved(rename = "new_name")]
				let nested_metas = attr.parse_args_with(
					syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
				)?;

				for nested_meta in nested_metas {
					let option = DissolvedOption::from_meta(&nested_meta)?;
					match option {
						DissolvedOption::Skip => {
							if field_info.renamed_to.is_some() {
								return Err(Error::new_spanned(
									attr,
									format!(
										"cannot use {} on skipped field",
										DissolvedOption::RENAME_IDENT,
									),
								));
							}

							field_info.should_skip = true;
						},
						DissolvedOption::Rename(new_ident) => {
							if field_info.should_skip {
								return Err(Error::new_spanned(
									attr,
									format!(
										"cannot use {} on skipped field",
										DissolvedOption::RENAME_IDENT,
									),
								));
							}

							if field_info.renamed_to.is_some() {
								return Err(Error::new_spanned(
									attr,
									format!(
										"cannot specify multiple {} options on the same field",
										DissolvedOption::RENAME_IDENT,
									),
								));
							}

							field_info.renamed_to = Some(new_ident);
						},
					}
				}
			},
			Meta::Path(_) => {
				return Err(Error::new_spanned(
					attr,
					format!(
						"dissolved attribute requires options, use #[dissolved({})] or #[dissolved({} = \"new_name\")] instead",
						DissolvedOption::SKIP_IDENT,
						DissolvedOption::RENAME_IDENT,
					),
				));
			},
			Meta::NameValue(_) => {
				return Err(Error::new_spanned(
					attr,
					format!(
						"dissolved attribute should use list syntax: #[dissolved({} = \"new_name\")] instead of #[dissolved = ...]",
						DissolvedOption::RENAME_IDENT,
					),
				));
			},
		}
	}

	Ok(field_info)
}
