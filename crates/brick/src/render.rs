use super::Brick;
use crate::BrickProps;
use minijinja::Environment;

impl Brick {
    pub fn render(&mut self, env: &Environment) {
        if let Brick::render(r) = self {
            let n = &r.name;
            let cx = &r.data;
            let n = env
                .get_template(n)
                .map_err(|e| e.to_string())
                .and_then(|t| {
                    t.render(cx)
                        .map_err(|e| format!("render failed: {} => {:#?}", e, &cx))
                })
                .and_then(|t| {
                    serde_json::from_str::<Brick>(&t)
                        .map_err(|e| format!("deserialize failed: {} => {}", e, &t))
                });
            match n {
                Ok(x) => {
                    *self = x;
                }
                Err(x) => {
                    #[cfg(feature = "dioxus")]
                    dioxus::logger::tracing::info!("{x:?}");
                }
            }
        }
        if let Some(cs) = self.borrow_children_mut() {
            for c in cs {
                c.render(env);
            }
        }
    }
}
