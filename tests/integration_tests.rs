#![allow(dead_code)]

use core::f64;

use dissolve_derive::Dissolve;

#[test]
fn test_basic_dissolve() {
	#[derive(Dissolve)]
	struct Basic {
		name: String,
		age: u32,
	}

	// Arrange
	let s = Basic { name: "alice".into(), age: 30 };

	// Act
	let BasicDissolved { name, age } = s.dissolve();

	// Assert
	assert_eq!(name, "alice");
	assert_eq!(age, 30);
}

#[test]
fn test_skip_field() {
	#[derive(Dissolve)]
	struct WithSkip {
		name: String,

		#[dissolved(skip)]
		password: String,
	}

	// Arrange
	let s = WithSkip { name: "bob".into(), password: "secret".into() };

	// Act
	let WithSkipDissolved { name } = s.dissolve();

	// Assert
	assert_eq!(name, "bob");
}

#[test]
fn test_rename_field() {
	#[derive(Dissolve)]
	struct WithRename {
		#[dissolved(rename = "full_name")]
		name: String,

		#[dissolved(rename = "email_address")]
		email: String,
	}

	// Arrange
	let s = WithRename { name: "charlie".into(), email: "charlie@example.com".into() };

	// Act
	let WithRenameDissolved { full_name, email_address } = s.dissolve();

	// Assert
	assert_eq!(full_name, "charlie");
	assert_eq!(email_address, "charlie@example.com");
}

#[test]
fn test_complex_struct() {
	#[derive(Dissolve)]
	struct Complex {
		pub id: u64,

		#[dissolved(rename = "user_name")]
		name: String,

		#[dissolved(skip)]
		password_hash: String,

		email: Option<String>,
	}

	// Arrange
	let s = Complex {
		id: 123,
		name: "dave".into(),
		password_hash: "hash123".into(),
		email: Some("dave@example.com".into()),
	};

	// Act
	let ComplexDissolved { id, user_name, email } = s.dissolve();

	// Assert
	assert_eq!(id, 123);
	assert_eq!(user_name, "dave");
	assert_eq!(email.as_deref(), Some("dave@example.com"));
}

#[test]
fn test_tuple_struct() {
	#[derive(Dissolve)]
	struct MultiField(String, i32, #[dissolved(skip)] bool);

	// Arrange
	let t = MultiField("test".into(), 42, true);

	// Act
	let (field_0, field_1) = t.dissolve();

	// Assert
	assert_eq!(field_0, "test");
	assert_eq!(field_1, 42);
}

#[test]
fn test_single_field_tuple_after_skip() {
	#[derive(Dissolve)]
	struct SingleField(#[dissolved(skip)] String, i32);

	// Arrange
	let t = SingleField("skipped".into(), 100);

	// Act
	let (field_0,) = t.dissolve();

	// Assert
	assert_eq!(field_0, 100);
}

#[test]
fn test_nested_types() {
	use std::collections::HashMap;

	#[derive(Dissolve)]
	struct WithComplexTypes {
		#[dissolved(rename = "user_data")]
		data: HashMap<String, Vec<i32>>,

		#[dissolved(skip)]
		internal_map: HashMap<u64, String>,
	}

	// Arrange
	let data = HashMap::from([
		("key1".into(), vec![1, 2, 3]),
		("key2".into(), vec![42, 1729]),
	]);
	let internal_map = HashMap::from([(1, "value".into())]);

	let s = WithComplexTypes { data: data.clone(), internal_map };

	// Act
	let WithComplexTypesDissolved { user_data } = s.dissolve();

	// Assert
	assert_eq!(user_data, data);
}

#[test]
fn test_generic_struct() {
	#[derive(Dissolve)]
	struct GenericStruct<T> {
		first_field: String,
		second_field: u32,
		extra: T,
	}

	// Arrange
	let s = GenericStruct { first_field: "test".into(), second_field: 42, extra: [1729; 10] };

	// Act
	let GenericStructDissolved { first_field, second_field, extra } = s.dissolve();

	// Assert
	assert_eq!(first_field, "test");
	assert_eq!(second_field, 42);
	assert_eq!(extra, [1729; 10]);
}

#[test]
fn test_multiple_generics() {
	#[derive(Dissolve)]
	struct MultiGeneric<T, U> {
		value_t: T,
		value_u: U,
	}

	// Arrange
	let s = MultiGeneric { value_t: "hello".to_string(), value_u: 42u64 };

	// Act
	let MultiGenericDissolved { value_t, value_u } = s.dissolve();

	// Assert
	assert_eq!(value_t, "hello");
	assert_eq!(value_u, 42u64);
}

#[test]
fn test_generic_with_lifetime() {
	#[derive(Dissolve)]
	struct WithLifetime<'a> {
		data: &'a str,
		count: usize,
	}

	// Arrange
	let text = "borrowed".to_string();
	let s = WithLifetime { data: text.as_str(), count: 5 };

	// Act
	let WithLifetimeDissolved { data, count } = s.dissolve();

	// Assert
	assert_eq!(data, text);
	assert_eq!(count, 5);
}

#[test]
fn test_generic_with_where_clause() {
	#[derive(Dissolve)]
	struct WithWhereClause<T>
	where
		T: Clone,
	{
		value: T,
		name: String,
	}

	// Arrange
	let s = WithWhereClause { value: vec![1, 2, 3], name: "series".into() };

	// Act
	let WithWhereClauseDissolved { value, name } = s.dissolve();

	// Assert
	assert_eq!(value, vec![1, 2, 3]);
	assert_eq!(name, "series");
}

#[test]
fn test_visibility_pub_crate() {
	#[derive(Dissolve)]
	#[dissolve(visibility = "pub(crate)")]
	struct VisibilityCrate {
		data: String,
	}

	// Arrange
	let s = VisibilityCrate { data: "test".into() };

	// Act

	let VisibilityCrateDissolved { data } = s.dissolve(); // should be accessible as pub(crate)

	// Assert
	assert_eq!(data, "test");
}

mod test_pub_super_visibility {
	use dissolve_derive::Dissolve;

	#[derive(Dissolve)]
	#[dissolve(visibility = "pub(super)")]
	struct VisibilitySuper {
		value: i32,
	}

	#[test]
	fn test_visibility_pub_super() {
		// Arrange
		let s = VisibilitySuper { value: 42 };

		// Act
		let VisibilitySuperDissolved { value } = s.dissolve();

		// Assert
		assert_eq!(value, 42);
	}
}

#[test]
fn test_visibility_pub_self() {
	#[derive(Dissolve)]
	#[dissolve(visibility = "pub(self)")]
	struct VisibilitySelf {
		name: String,
	}

	// Arrange
	let s = VisibilitySelf { name: "private".into() };

	// Act
	let VisibilitySelfDissolved { name } = s.dissolve();

	// Assert
	assert_eq!(name, "private");
}

#[test]
fn test_visibility_private() {
	#[derive(Dissolve)]
	#[dissolve(visibility = "")]
	struct VisibilityPrivate {
		secret: u64,
	}

	// Arrange
	let s = VisibilityPrivate { secret: 123456 };

	// Act
	let VisibilityPrivateDissolved { secret } = s.dissolve();

	// Assert
	assert_eq!(secret, 123456);
}

#[test]
fn test_visibility_default_pub() {
	// Without visibility attribute, should default to pub
	#[derive(Dissolve)]
	struct DefaultVisibility {
		field: String,
	}

	// Arrange
	let s = DefaultVisibility { field: "public".into() };

	// Act
	let DefaultVisibilityDissolved { field } = s.dissolve();

	// Assert
	assert_eq!(field, "public");
}

#[test]
fn test_visibility_with_skip_and_rename() {
	#[derive(Dissolve)]
	#[dissolve(visibility = "pub(crate)")]
	struct VisibilityWithAttributes {
		#[dissolved(rename = "renamed_field")]
		original_field: String,

		#[dissolved(skip)]
		hidden: bool,

		normal: i32,
	}

	// Arrange
	let s = VisibilityWithAttributes { original_field: "value".into(), hidden: true, normal: 99 };

	// Act
	let VisibilityWithAttributesDissolved { renamed_field, normal } = s.dissolve();

	// Assert
	assert_eq!(renamed_field, "value");
	assert_eq!(normal, 99);
}

#[test]
fn test_visibility_tuple_struct() {
	#[derive(Dissolve)]
	#[dissolve(visibility = "pub(crate)")]
	struct VisibilityTuple(String, i32, #[dissolved(skip)] f64);

	// Arrange
	let t = VisibilityTuple("tuple".into(), 42, f64::consts::PI);

	// Act
	let (field_0, field_1) = t.dissolve();

	// Assert
	assert_eq!(field_0, "tuple");
	assert_eq!(field_1, 42);
}

#[test]
fn test_visibility_with_generics() {
	#[derive(Dissolve)]
	#[dissolve(visibility = "pub(crate)")]
	struct VisibilityGeneric<T> {
		value: T,
		count: usize,
	}

	// Arrange
	let s = VisibilityGeneric { value: vec![1, 2, 3], count: 3 };

	// Act
	let VisibilityGenericDissolved { value, count } = s.dissolve();

	// Assert
	assert_eq!(value, vec![1, 2, 3]);
	assert_eq!(count, 3);
}
