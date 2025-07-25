use html_template::HtmlTemplate;

fn main() {
    let html = r#"
        <template>
            <div class="company">
                <h1 itemprop="name"></h1>
                <div itemprop="departments[]" class="department">
                    <h2 itemprop="name"></h2>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    println!("Parsed template: {:#?}", template);
}