//! Example demonstrating the Renderable derive macro
//!
//! This example shows how to use the #[derive(Renderable)] macro to automatically
//! implement the RenderValue trait for custom structs.

#[cfg(feature = "derive")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use html_template::*;
    use html_template_macros::Renderable;

    // Define a struct with the Renderable derive macro
    #[derive(Renderable)]
    struct BlogPost {
        id: String,
        title: String,

        #[renderable(rename = "authorName")]
        author: String,

        content: String,
        tags: Vec<String>,

        #[renderable(skip)]
        internal_notes: String,
    }

    // Create some sample data
    let post = BlogPost {
        id: "post-123".to_string(),
        title: "Introduction to Rust Templates".to_string(),
        author: "Jane Developer".to_string(),
        content: "Learn how to create powerful templates with Rust...".to_string(),
        tags: vec![
            "rust".to_string(),
            "templates".to_string(),
            "web".to_string(),
        ],
        internal_notes: "This is for internal use only".to_string(),
    };

    // Create an HTML template
    let template_html = r#"
        <template>
            <article class="blog-post">
                <header>
                    <h1 itemprop="title"></h1>
                    <p class="author">By <span itemprop="authorName"></span></p>
                    <p class="post-id">ID: <span itemprop="id"></span></p>
                </header>
                <div class="content" itemprop="content"></div>
                <footer>
                    <p>Tags: <span itemprop="tags"></span> tag(s)</p>
                </footer>
            </article>
        </template>
    "#;

    // Compile the template
    let template = HtmlTemplate::from_str(template_html, Some("article.blog-post"))?;

    // Render using the derived struct
    let rendered = template.render(&post)?;

    println!("Rendered blog post:");
    println!("{}", rendered);

    // Demonstrate property access
    println!("\nProperty access examples:");
    println!("Title: {:?}", post.get_property(&["title".to_string()]));
    println!(
        "Author (renamed): {:?}",
        post.get_property(&["authorName".to_string()])
    );
    println!("ID: {:?}", post.get_id());
    println!("Tag count: {:?}", post.get_property(&["tags".to_string()]));
    println!("Is array: {}", post.is_array());

    // The internal_notes field should not be accessible
    println!(
        "Internal notes (should be None): {:?}",
        post.get_property(&["internal_notes".to_string()])
    );

    Ok(())
}

#[cfg(not(feature = "derive"))]
fn main() {
    println!("This example requires the 'derive' feature to be enabled.");
    println!("Run with: cargo run --example derive_example --features derive");
}
