//! Tests for the Renderable derive macro
//!
//! These tests verify that the derive macro correctly generates RenderValue
//! implementations for various struct patterns.

use html_template::*;

#[cfg(feature = "derive")]
mod derive_tests {
    use super::*;
    use html_template_macros::Renderable;
    use serde_json::json;

    #[derive(Renderable)]
    struct SimplePerson {
        name: String,
        age: u32,
        email: String,
    }

    #[test]
    fn test_simple_struct_property_access() {
        let person = SimplePerson {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        };

        // Test simple property access
        assert_eq!(
            person.get_property(&["name".to_string()]).unwrap().as_ref(),
            "Alice"
        );
        assert_eq!(
            person.get_property(&["age".to_string()]).unwrap().as_ref(),
            "30"
        );
        assert_eq!(
            person
                .get_property(&["email".to_string()])
                .unwrap()
                .as_ref(),
            "alice@example.com"
        );

        // Test missing property
        assert!(person.get_property(&["missing".to_string()]).is_none());
    }

    #[test]
    fn test_simple_struct_type_and_id() {
        let person = SimplePerson {
            name: "Bob".to_string(),
            age: 25,
            email: "bob@example.com".to_string(),
        };

        // Test type
        assert_eq!(person.get_type(), Some("SimplePerson"));

        // Test id (should be None since no field is marked as id)
        assert_eq!(person.get_id(), None);

        // Test array behavior
        assert!(!person.is_array());
        assert!(person.as_array().is_none());
    }

    #[derive(Renderable)]
    struct PersonWithCustomAttributes {
        #[renderable(id)]
        user_id: String,

        #[renderable(rename = "fullName")]
        name: String,

        age: u32,

        #[renderable(skip)]
        password: String,
    }

    #[test]
    fn test_custom_attributes() {
        let person = PersonWithCustomAttributes {
            user_id: "123".to_string(),
            name: "Charlie".to_string(),
            age: 35,
            password: "secret123".to_string(),
        };

        // Test renamed property
        assert_eq!(
            person
                .get_property(&["fullName".to_string()])
                .unwrap()
                .as_ref(),
            "Charlie"
        );

        // Original name should not work
        assert!(person.get_property(&["name".to_string()]).is_none());

        // Test regular property
        assert_eq!(
            person.get_property(&["age".to_string()]).unwrap().as_ref(),
            "35"
        );

        // Test skipped property
        assert!(person.get_property(&["password".to_string()]).is_none());

        // Test id field
        assert_eq!(person.get_id(), Some("123"));

        // User ID should still be accessible as property
        assert_eq!(
            person
                .get_property(&["user_id".to_string()])
                .unwrap()
                .as_ref(),
            "123"
        );
    }

    #[derive(Renderable)]
    struct PersonWithId {
        id: String,
        name: String,
    }

    #[test]
    fn test_automatic_id_field() {
        let person = PersonWithId {
            id: "person-456".to_string(),
            name: "Diana".to_string(),
        };

        // Test automatic id detection
        assert_eq!(person.get_id(), Some("person-456"));

        // ID should also be accessible as property
        assert_eq!(
            person.get_property(&["id".to_string()]).unwrap().as_ref(),
            "person-456"
        );
    }

    #[derive(Renderable)]
    struct Blog {
        title: String,
        posts: Vec<String>,
        author: String,
    }

    #[test]
    fn test_vec_fields() {
        let blog = Blog {
            title: "Tech Blog".to_string(),
            posts: vec![
                "Rust is awesome".to_string(),
                "WebAssembly tutorial".to_string(),
                "Async programming".to_string(),
            ],
            author: "Eve".to_string(),
        };

        // Test array detection
        assert!(blog.is_array());

        // Test regular properties
        assert_eq!(
            blog.get_property(&["title".to_string()]).unwrap().as_ref(),
            "Tech Blog"
        );
        assert_eq!(
            blog.get_property(&["author".to_string()]).unwrap().as_ref(),
            "Eve"
        );

        // Test array length access
        assert_eq!(
            blog.get_property(&["posts".to_string()]).unwrap().as_ref(),
            "3"
        );

        // Test array item access (when implemented)
        // assert_eq!(
        //     blog.get_property(&["posts".to_string(), "[0]".to_string()]).unwrap().as_ref(),
        //     "Rust is awesome"
        // );
    }

    #[derive(Renderable)]
    struct OptionalFields {
        name: String,
        nickname: Option<String>,
        age: Option<u32>,
    }

    #[test]
    fn test_optional_fields() {
        let person_with_nickname = OptionalFields {
            name: "Frank".to_string(),
            nickname: Some("Frankie".to_string()),
            age: Some(28),
        };

        let person_without_nickname = OptionalFields {
            name: "Grace".to_string(),
            nickname: None,
            age: None,
        };

        // Test present optional fields
        assert_eq!(
            person_with_nickname
                .get_property(&["nickname".to_string()])
                .unwrap()
                .as_ref(),
            "Frankie"
        );
        assert_eq!(
            person_with_nickname
                .get_property(&["age".to_string()])
                .unwrap()
                .as_ref(),
            "28"
        );

        // Test absent optional fields (should return empty string)
        assert_eq!(
            person_without_nickname
                .get_property(&["nickname".to_string()])
                .unwrap()
                .as_ref(),
            ""
        );
        assert_eq!(
            person_without_nickname
                .get_property(&["age".to_string()])
                .unwrap()
                .as_ref(),
            ""
        );
    }

    // Note: Nested struct support requires more complex implementation
    // For now, focusing on basic functionality

    #[test]
    fn test_template_rendering_with_derived_struct() {
        let person = SimplePerson {
            name: "Isabel".to_string(),
            age: 42,
            email: "isabel@example.com".to_string(),
        };

        // Create a simple template
        let template_html = r#"
            <template>
                <div class="person">
                    <h2 itemprop="name"></h2>
                    <p>Age: <span itemprop="age"></span></p>
                    <p>Email: <span itemprop="email"></span></p>
                </div>
            </template>
        "#;

        let template = HtmlTemplate::from_str(template_html, Some("div.person")).unwrap();
        let output = template.render(&person).unwrap();

        assert!(output.contains("Isabel"));
        assert!(output.contains("42"));
        assert!(output.contains("isabel@example.com"));
    }

    #[test]
    fn test_template_rendering_with_custom_attributes() {
        let person = PersonWithCustomAttributes {
            user_id: "789".to_string(),
            name: "Jack".to_string(),
            age: 33,
            password: "hidden".to_string(),
        };

        // Template uses the renamed property
        let template_html = r#"
            <template>
                <div class="person">
                    <h2 itemprop="fullName"></h2>
                    <p>Age: <span itemprop="age"></span></p>
                    <p>ID: <span itemprop="user_id"></span></p>
                </div>
            </template>
        "#;

        let template = HtmlTemplate::from_str(template_html, Some("div.person")).unwrap();
        let output = template.render(&person).unwrap();

        assert!(output.contains("Jack"));
        assert!(output.contains("33"));
        assert!(output.contains("789"));
        // Password should not appear anywhere since it's skipped
        assert!(!output.contains("hidden"));
    }

    // Note: Generic struct support requires trait bounds
    // Will be implemented in a future iteration
}
