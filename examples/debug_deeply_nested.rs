use html_template::HtmlTemplate;
use serde_json::json;

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
    let data = json!({
        "name": "Tech Corp",
        "departments": [
            {
                "name": "Engineering",
                "teams": [
                    {
                        "name": "Backend Team",
                        "lead": {"name": "Alice"},
                        "members": [
                            {"name": "Bob", "role": "Senior Developer"},
                            {"name": "Carol", "role": "Developer"}
                        ]
                    },
                    {
                        "name": "Frontend Team",
                        "lead": {"name": "David"},
                        "members": [
                            {"name": "Eve", "role": "UI Designer"},
                            {"name": "Frank", "role": "Developer"}
                        ]
                    }
                ]
            }
        ]
    });

    let result = template.render(&data).unwrap();

    println!("Result:\n{}", result);

    println!("\nChecks:");
    println!("contains 'Tech Corp': {}", result.contains("Tech Corp"));
    println!("contains 'Engineering': {}", result.contains("Engineering"));
    println!(
        "contains 'Backend Team': {}",
        result.contains("Backend Team")
    );
    println!(
        "contains 'Frontend Team': {}",
        result.contains("Frontend Team")
    );
    println!("contains 'Lead: Alice': {}", result.contains("Lead: Alice"));
    println!("contains 'Lead: David': {}", result.contains("Lead: David"));
    println!(
        "contains 'Bob - Senior Developer': {}",
        result.contains("Bob - Senior Developer")
    );
    println!(
        "contains 'Eve - UI Designer': {}",
        result.contains("Eve - UI Designer")
    );
}
