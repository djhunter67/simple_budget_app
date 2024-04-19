use askama::Template;

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate<'a> {
    /// The title of the page.
    pub title: &'a str,
    /// The index of the page.
    pub index: &'a str,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    /// The title of the page.
    pub title: &'a str,
    pub current_amount: f32,
    pub total_expenses: f32,
    pub total_income: f32,
}
