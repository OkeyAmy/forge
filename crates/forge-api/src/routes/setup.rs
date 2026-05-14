use axum::response::{Html, IntoResponse};

pub async fn handler() -> impl IntoResponse {
    let install_url = std::env::var("GITHUB_APP_PUBLIC_URL").ok();
    let install_link = install_url
        .as_deref()
        .map(|url| format!(r#"<a class="button" href="{url}">Install Forge on GitHub</a>"#))
        .unwrap_or_else(|| {
            r#"<span class="button disabled">GitHub App install URL not configured</span>"#
                .to_string()
        });

    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Forge GitHub App</title>
  <style>
    body {{ margin: 0; font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color: #171717; background: #f7f8fa; }}
    main {{ max-width: 780px; margin: 0 auto; padding: 72px 24px; }}
    h1 {{ font-size: 44px; line-height: 1.05; margin: 0 0 16px; letter-spacing: 0; }}
    p {{ font-size: 17px; line-height: 1.65; color: #444; margin: 0 0 18px; }}
    .steps {{ display: grid; gap: 12px; margin: 32px 0; }}
    .step {{ border: 1px solid #d9dde5; background: #fff; border-radius: 8px; padding: 18px; }}
    .step strong {{ display: block; margin-bottom: 6px; color: #111; }}
    .button {{ display: inline-flex; align-items: center; min-height: 44px; padding: 0 18px; border-radius: 8px; background: #111; color: #fff; text-decoration: none; font-weight: 650; }}
    .button.disabled {{ background: #d7dae0; color: #555; }}
    code {{ background: #eceff3; border-radius: 6px; padding: 2px 6px; }}
  </style>
</head>
<body>
  <main>
    <h1>Forge</h1>
    <p>Connect Forge to a repository, label an issue <code>forge</code>, review the generated plan, then approve execution in an E2B sandbox.</p>
    {install_link}
    <section class="steps" aria-label="Setup steps">
      <div class="step"><strong>1. Install</strong> Add the GitHub App to the repositories Forge should work on.</div>
      <div class="step"><strong>2. Plan</strong> Add the <code>forge</code> label or comment <code>/forge plan</code> on an issue.</div>
      <div class="step"><strong>3. Approve</strong> Comment <code>/forge approve</code> after reviewing the plan.</div>
      <div class="step"><strong>4. Review PRs</strong> Comment <code>/forge review</code> on a pull request for native Forge review.</div>
    </section>
  </main>
</body>
</html>"#
    ))
}
