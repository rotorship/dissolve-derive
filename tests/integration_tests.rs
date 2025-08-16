#![allow(dead_code)]

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
