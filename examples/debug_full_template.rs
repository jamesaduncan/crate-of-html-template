use html_template::HtmlTemplate;

fn main() {
    let html = r#"
        <template>
            <div class="company">
                <h1 itemprop="name"></h1>
                <div itemprop="departments[]" class="department">
                    <h2 itemprop="name"></h2>
                    <div itemprop="teams[]" class="team">
                        <h3 itemprop="name"></h3>
                        <p>Lead: <span itemprop="lead" itemscope><span itemprop="name"></span></span></p>
                        <ul>
                            <li itemprop="members[]">
                                <span itemprop="name"></span> - <span itemprop="role"></span>
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </template>
    "#;

    let template = HtmlTemplate::from_str(html, None).unwrap();
    println!("Parsed template: {:#?}", template);
}