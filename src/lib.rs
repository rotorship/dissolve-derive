//! # Dissolve Derive
//!
//! A procedural macro for safely taking ownership of inner fields from a struct without exposing
//! those fields publicly.
//!
//! ## Motivation
//!
//! The `dissolve-derive` proc macro solves a specific problem: when you have a struct with
//! private fields and need to transfer ownership of those fields to another part of your code,
//! you often face two undesirable choices:
//!
//! 1. **Make fields public**: This exposes your internal state and allows arbitrary mutation,
//!    breaking encapsulation.
//! 2. **Write accessor methods**: This requires boilerplate code and may involve cloning data,
//!    which is inefficient for large structures.
//!
//! The `Dissolve` derive macro provides a `dissolve(self)` method that consumes the struct and
//! returns its fields in a type-safe manner. This approach:
//!
//! - **Preserves encapsulation**: Fields remain private in the original struct
//! - **Enables efficient ownership transfer**: No cloning required, fields are moved
//! - **Prevents misuse**: The dissolved struct is a different type, preventing it from being used
//!   where the original struct is expected
//! - **Provides flexibility**: Control which fields are exposed and rename them if needed
//! - **Allows custom visibility**: Configure the visibility of the `dissolve` method itself
//!
//! ## Use Cases
//!
//! ### 1. API Boundaries
//!
//! When building a library, you want to keep internal structure private but allow consumers
//! to extract owned data when they're done with the struct's instance:
//!
//! ```rust
//! use dissolve_derive::Dissolve;
//!
//! #[derive(Dissolve)]
//! pub struct Connection {
//!     // Private: users can't modify the socket directly
//!     socket: std::net::TcpStream,
//!
//!     // Private: internal state
//!     buffer: Vec<u8>,
//!
//!     // Skip: purely internal, never exposed
//!     #[dissolved(skip)]
//!     statistics: ConnectionStats,
//! }
//!
//! # struct ConnectionStats;
//!
//! // Users can dissolve the connection to reclaim the socket
//! // without having public access to it during normal operation
//! # fn example(conn: Connection) {
//! let ConnectionDissolved { socket, buffer } = conn.dissolve();
//! # }
//! ```
//!
//! ### 2. Builder Pattern Finalization
//!
//! Use dissolve to finalize a builder and extract components with controlled visibility:
//!
//! ```rust
//! use dissolve_derive::Dissolve;
//!
//! #[derive(Dissolve)]
//! #[dissolve(visibility = "pub(crate)")]
//! pub struct ConfigBuilder {
//!     database_url: String,
//!     max_connections: u32,
//!
//!     #[dissolved(skip)]
//!     validated: bool,
//! }
//!
//! impl ConfigBuilder {
//!     pub fn build(mut self) -> Config {
//!         self.validated = true;
//!         // Only accessible within the crate due to pub(crate)
//!         let ConfigBuilderDissolved { database_url, max_connections } = self.dissolve();
//!         Config { database_url, max_connections }
//!     }
//! }
//! # pub struct Config { database_url: String, max_connections: u32 }
//! ```
//!
//! ### 3. State Machine Transitions
//!
//! Safely transition between states by dissolving one state struct and constructing the next:
//!
//! ```rust
//! use core::time::Instant;
//!
//! use dissolve_derive::Dissolve;
//!
//! #[derive(Dissolve)]
//! struct PendingRequest {
//!     request_id: u64,
//!     payload: Vec<u8>,
//!
//!     #[dissolved(skip)]
//!     timestamp: Instant,
//! }
//!
//! #[derive(Dissolve)]
//! struct ProcessedRequest {
//!     request_id: u64,
//!     response: Vec<u8>,
//! }
//!
//! impl PendingRequest {
//!     fn process(self) -> ProcessedRequest {
//!         let PendingRequestDissolved { request_id, payload } = self.dissolve();
//!         let response = process_payload(payload);
//!         ProcessedRequest { request_id, response }
//!     }
//! }
//!
//! # fn process_payload(p: Vec<u8>) -> Vec<u8> { p }
//! ```
//!
//! ### 4. Zero-Cost Abstraction Unwrapping
//!
//! When wrapping types for compile-time guarantees, use dissolve for efficient unwrapping:
//!
//! ```rust
//! use dissolve_derive::Dissolve;
//!
//! #[derive(Dissolve)]
//! pub struct Validated<T> {
//!     inner: T,
//!
//!     #[dissolved(skip)]
//!     validation_token: ValidationToken,
//! }
//!
//! impl<T> Validated<T> {
//!     pub fn into_inner(self) -> T {
//!         self.dissolve().inner
//!     }
//! }
//!
//! # struct ValidationToken;
//! ```
//!
//! ## Attributes
//!
//! ### Container Attributes (on structs)
//!
//! - `#[dissolve(visibility = "...")]` - Set the visibility of the `dissolve` method
//!   - Supported values: `"pub"`, `"pub(crate)"`, `"pub(super)"`, `"pub(self)"`, or empty string for private
//!   - Default: `"pub"` if not specified
//!
//! ### Field Attributes
//!
//! - `#[dissolved(skip)]` - Skip this field in the dissolved output
//! - `#[dissolved(rename = "new_name")]` - Rename this field in the dissolved struct (named structs only)
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use dissolve_derive::Dissolve;
//!
//! #[derive(Dissolve)]
//! struct User {
//!     name: String,
//!     email: String,
//! }
//!
//! let user = User {
//!     name: "alice".to_string(),
//!     email: "alice@example.com".to_string(),
//! };
//!
//! let UserDissolved { name, email } = user.dissolve();
//! assert_eq!(name, "alice");
//! ```
//!
//! ### With Custom Visibility
//!
//! ```rust
//! use dissolve_derive::Dissolve;
//!
//! #[derive(Dissolve)]
//! #[dissolve(visibility = "pub(crate)")]
//! pub struct InternalData {
//!     value: i32,
//! }
//!
//! // The dissolve method is only accessible within the same crate
//! ```
//!
//! ### Skipping Fields
//!
//! ```rust
//! use dissolve_derive::Dissolve;
//!
//! #[derive(Dissolve)]
//! struct Credentials {
//!     username: String,
//!
//!     #[dissolved(skip)]
//!     password: String,  // Never exposed, even through dissolve
//! }
//! ```
//!
//! ### Renaming Fields
//!
//! ```rust
//! use dissolve_derive::Dissolve;
//!
//! #[derive(Dissolve)]
//! struct ApiResponse {
//!     #[dissolved(rename = "user_id")]
//!     id: u64,
//!
//!     #[dissolved(rename = "user_name")]
//!     name: String,
//! }
//! ```
//!
//! ### Tuple Structs
//!
//! ```rust
//! use dissolve_derive::Dissolve;
//!
//! #[derive(Dissolve)]
//! struct Coordinate(f64, f64, #[dissolved(skip)] String);
//!
//! let coord = Coordinate(1.0, 2.0, "label".to_string());
//! let (x, y) = coord.dissolve();
//! ```

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
#[proc_macro_derive(Dissolve, attributes(dissolve, dissolved))]
pub fn derive_dissolve(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	match generate_dissolve_impl(&input) {
		Ok(tokens) => tokens.into(),
		Err(err) => err.to_compile_error().into(),
	}
}

#[derive(Debug, Clone)]
struct ContainerAttributes {
	visibility: syn::Visibility,
}

impl ContainerAttributes {
	const IDENT: &str = "dissolve";

	const VISIBILITY_IDENT: &str = "visibility";

	fn from_derive_input(input: &DeriveInput) -> Result<Self> {
		let mut visibility = syn::parse_str::<syn::Visibility>("pub").unwrap();

		for attr in input.attrs.iter().filter(|attr| attr.path().is_ident(Self::IDENT)) {
			match &attr.meta {
				Meta::List(_) => {
					let nested_metas = attr.parse_args_with(
						syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
					)?;

					for nested_meta in nested_metas {
						match &nested_meta {
							Meta::NameValue(MetaNameValue { path, value, .. }) => {
								if path.is_ident(Self::VISIBILITY_IDENT) {
									match value {
										Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) => {
											let vis_str = lit_str.value();
											visibility = syn::parse_str::<syn::Visibility>(&vis_str)
												.map_err(|e| {
													Error::new_spanned(
														value,
														format!(
															"invalid visibility: {e}. Supported: 'pub', 'pub(crate)', 'pub(super)', 'pub(self)' or empty for private",
														),
													)
												})?;
										},
										_ => {
											return Err(Error::new_spanned(
												value,
												"visibility value must be a string literal",
											));
										},
									}
								} else {
									return Err(Error::new_spanned(
										path,
										format!(
											"unknown dissolve attribute option '{}'; supported option: {}",
											path.get_ident()
												.map(|i| i.to_string())
												.unwrap_or_default(),
											Self::VISIBILITY_IDENT,
										),
									));
								}
							},
							_ => {
								return Err(Error::new_spanned(
									nested_meta,
									"dissolve container attribute must use name-value syntax: #[dissolve(visibility = \"...\")]",
								));
							},
						}
					}
				},
				_ => {
					return Err(Error::new_spanned(
						attr,
						"dissolve attribute must use list syntax: #[dissolve(visibility = \"...\")]",
					));
				},
			}
		}

		Ok(Self { visibility })
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
	let generics = &input.generics;
	let container_attrs = ContainerAttributes::from_derive_input(input)?;

	let Data::Struct(data_struct) = &input.data else {
		return Err(Error::new_spanned(
			input,
			"Dissolve can only be derived for structs",
		));
	};

	match &data_struct.fields {
		Fields::Named(fields) => {
			generate_named_struct_impl(struct_name, generics, fields, &container_attrs)
		},
		Fields::Unnamed(fields) => {
			generate_tuple_struct_impl(struct_name, generics, fields, &container_attrs)
		},
		Fields::Unit => Err(Error::new_spanned(
			input,
			"Dissolve cannot be derived for unit structs",
		)),
	}
}

fn generate_named_struct_impl(
	struct_name: &syn::Ident,
	generics: &syn::Generics,
	fields: &syn::FieldsNamed,
	container_attrs: &ContainerAttributes,
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

		// Extract doc comments from the original field
		let doc_attrs = field.attrs.iter().filter(|attr| attr.path().is_ident("doc"));

		quote! {
			#(#doc_attrs)*
			pub #dissolved_field_name: #ty
		}
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

	// Split generics for use in different positions
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let dissolved_struct_doc = format!(
		"Dissolved struct for [`{struct_name}`].\n\n\
		This struct contains all non-skipped fields from the original struct with public visibility. \
		Fields may be renamed according to `#[dissolved(rename = \"...\")]` attributes.",
	);

	let visibility = &container_attrs.visibility;

	Ok(quote! {
		#[doc = #dissolved_struct_doc]
		pub struct #dissolved_struct_name #impl_generics #where_clause {
			#(#field_definitions),*
		}

		impl #impl_generics #struct_name #ty_generics #where_clause {
			/// Dissolve this struct into its public-field equivalent.
			///
			/// This method consumes the original struct and returns a new struct where all included
			/// fields are made public and optionally renamed.
			#visibility fn dissolve(self) -> #dissolved_struct_name #ty_generics {
				#dissolved_struct_name {
					#(#field_moves),*
				}
			}
		}
	})
}

fn generate_tuple_struct_impl(
	struct_name: &syn::Ident,
	generics: &syn::Generics,
	fields: &FieldsUnnamed,
	container_attrs: &ContainerAttributes,
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

	// Split generics for use in different positions
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
	let visibility = &container_attrs.visibility;

	Ok(quote! {
		impl #impl_generics #struct_name #ty_generics #where_clause {
			/// Dissolve this tuple struct into a tuple of its included non-skipped fields.
			#visibility fn dissolve(self) -> #tuple_type {
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
