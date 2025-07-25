use html_template::HtmlTemplate;

fn main() {
    let html = r#"
        <template>
            <div class="users">
                <article itemprop="users[]" class="user">
                    <h3 itemprop="name"></h3>
                    <p>Email: <a href="mailto:${email}" itemprop="email"></a></p>
                    <p>Age: <span itemprop="age"></span></p>
                    <div class="tags">
                        Tags: <span itemprop="tags"></span>
                    </div>
                </article>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();

    // Print parsed elements
    println!("Parsed elements:");
    let template_info = format!("{:#?}", template);
    println!("{}", template_info);
}
