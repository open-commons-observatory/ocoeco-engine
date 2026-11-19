//! Views that need a script, and the tabs that put two views of one subject on one page.
//!
//! A lens names, for a view of kind `embed`, its `script` and its `data` file (paths in the data
//! directory) and the `libs` the script stands on (pinned https addresses). The engine copies the
//! two files next to the page and writes the container and the script tags; what is drawn inside
//! is the script's. A view may also carry `tabs`: more views of the same subject beside it, the
//! view itself the first and the default. A tab is a link to a pane (`#id`), chosen by the page's
//! address alone, so the strip works without JavaScript - the first pane shows and the others wait.

use crate::site::esc;
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::fmt::Write as _;
use std::path::{Component, Path};

/// Where the first pane of a tabbed view is linked, unless the lens names another.
pub const FIRST_TAB: &str = "table";

/// One pane of a page with tabs: its address, its label, what it holds.
pub struct Pane {
    pub id: String,
    pub title: String,
    pub html: String,
}

/// The extra tabs a view declares, in order.
pub fn tabs_of(v: &Value) -> Vec<&Value> {
    v["tabs"].as_array().into_iter().flatten().collect()
}

/// The panes of one page, each opening with the strip that names them all (its own marked), so the
/// pane that shows always shows the strip too. Only CSS (`:target`) chooses the pane.
pub fn panes(items: &[Pane]) -> String {
    let mut b = String::from(r#"<div class="panes">"#);
    for (i, pane) in items.iter().enumerate() {
        let _ = write!(
            b,
            r#"<section class="pane" id="{}"><nav class="strip" aria-label="⟦tabs-label⟧">"#,
            esc(&pane.id)
        );
        for (j, tab) in items.iter().enumerate() {
            let _ = write!(
                b,
                r##"<a href="#{}"{}>{}</a>"##,
                esc(&tab.id),
                if i == j {
                    r#" class="on" aria-current="true""#
                } else {
                    ""
                },
                esc(&tab.title)
            );
        }
        let _ = write!(b, "</nav>{}</section>", pane.html);
    }
    b + "</div>"
}

/// The container the script draws in, with the script tags: the libraries first, then the
/// container, then the script - which finds its container as the element just before it.
pub fn embed(v: &Value) -> String {
    let mut b = String::new();
    for lib in v["libs"].as_array().into_iter().flatten() {
        let _ = write!(
            b,
            r#"<script src="{}"></script>"#,
            esc(lib.as_str().unwrap_or_default())
        );
    }
    let _ = write!(
        b,
        r#"<div class="embed" data-data="{}"><noscript><p class="mute">⟦embed-needs-js⟧</p></noscript></div><script src="{}"></script>"#,
        esc(v["data"].as_str().unwrap_or_default()),
        esc(v["script"].as_str().unwrap_or_default())
    );
    b
}

/// The views of a view that embed: itself, when it does, and each tab that does.
fn embedding(v: &Value) -> Vec<&Value> {
    std::iter::once(v)
        .chain(tabs_of(v))
        .filter(|x| x["kind"] == "embed")
        .collect()
}

/// A path in the data directory: relative, going nowhere but down.
fn inside(path: &str) -> Result<&Path> {
    let p = Path::new(path);
    if path.ends_with(".html") {
        bail!("{path:?} would overwrite a page");
    }
    if path.is_empty() || !p.components().all(|c| matches!(c, Component::Normal(_))) {
        bail!("{path:?} is not a path inside the data directory");
    }
    Ok(p)
}

/// A tab is an embed, has an address of its own, and no two panes of a page share one.
fn check_tabs(v: &Value) -> Result<()> {
    let mut seen = vec![v["tab_id"].as_str().unwrap_or(FIRST_TAB)];
    for t in tabs_of(v) {
        let id = t["id"].as_str().unwrap_or_default();
        if t["kind"] != "embed" {
            bail!("tab {id:?} is not of kind embed: only an embed may be a tab");
        }
        if id.is_empty() || seen.contains(&id) {
            bail!("tab {id:?} is empty or named twice on one page");
        }
        seen.push(id);
    }
    Ok(())
}

/// Copies what a view's embeds name - script and data - from the data directory to `dir`, at the
/// same paths, and refuses a lens that names a file outside the directory or a library that is not
/// a pinned https address.
pub fn publish(root: &Path, v: &Value, dir: &Path) -> Result<()> {
    check_tabs(v)?;
    for e in embedding(v) {
        let id = e["id"].as_str().unwrap_or("?");
        for lib in e["libs"].as_array().into_iter().flatten() {
            let lib = lib.as_str().unwrap_or_default();
            if !lib.starts_with("https://") {
                bail!("view {id}: library {lib:?} is not an https address");
            }
        }
        for key in ["script", "data"] {
            let path = inside(e[key].as_str().unwrap_or_default())
                .with_context(|| format!("view {id}: its {key}"))?;
            let to = dir.join(path);
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(root.join(path), &to)
                .with_context(|| format!("view {id}: copying {}", root.join(path).display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn panes_each_open_with_the_strip_and_mark_their_own_tab() {
        let h = panes(&[
            Pane {
                id: "table".into(),
                title: "Table".into(),
                html: "<p>t</p>".into(),
            },
            Pane {
                id: "graph".into(),
                title: "Graph".into(),
                html: "<p>g</p>".into(),
            },
        ]);
        assert!(h.contains(r#"<section class="pane" id="table"><nav class="strip""#));
        assert!(h.contains(r##"<a href="#table" class="on" aria-current="true">Table</a><a href="#graph">Graph</a></nav><p>t</p></section>"##));
        assert!(h.contains(r##"<a href="#table">Table</a><a href="#graph" class="on" aria-current="true">Graph</a></nav><p>g</p></section>"##));
    }

    #[test]
    fn an_embed_is_libraries_then_container_then_script_and_says_it_needs_javascript() {
        let h = embed(
            &json!({"kind":"embed","libs":["https://cdn.example/a@1.js","https://cdn.example/b@2.js"],
            "script":"graph/graph.js","data":"graph/bringup.json"}),
        );
        let at = |x: &str| h.find(x).unwrap_or_else(|| panic!("{x} in {h}"));
        assert!(at("a@1.js") < at("b@2.js") && at("b@2.js") < at(r#"<div class="embed""#));
        assert!(at(r#"<div class="embed""#) < at(r#"<script src="graph/graph.js">"#));
        assert!(h.contains(r#"data-data="graph/bringup.json""#));
        assert!(h.contains("<noscript><p class=\"mute\">⟦embed-needs-js⟧</p></noscript>"));
    }

    #[test]
    fn a_path_outside_the_data_directory_is_refused() {
        for bad in ["../x.js", "/etc/passwd", "a/../../x", ""] {
            assert!(inside(bad).is_err(), "{bad:?}");
        }
        assert!(inside("graph/graph.js").is_ok());
    }

    #[test]
    fn a_tab_must_be_an_embed_with_an_address_of_its_own() {
        let root = std::env::temp_dir();
        let out = root.join("oco-never-written");
        let tab = |id: &str, kind: &str| {
            json!({"id":"steps","kind":"matrix","tabs":[{"id":id,"kind":kind,
                "script":"a.js","data":"a.json"}]})
        };
        for (id, kind) in [("graph", "matrix"), ("", "embed"), ("table", "embed")] {
            let e = publish(&root, &tab(id, kind), &out)
                .unwrap_err()
                .to_string();
            assert!(e.contains("tab"), "{id:?} {kind}: {e}");
        }
        let page = json!({"id":"g","kind":"embed","script":"steps.html","data":"a.json"});
        assert!(
            publish(&root, &page, &out).is_err(),
            "a script may not be a page"
        );
    }
    #[test]
    fn publishing_copies_script_and_data_and_names_what_is_wrong() {
        let base = std::env::temp_dir().join(format!("oco-embed-{}", std::process::id()));
        let (root, out) = (base.join("data"), base.join("out"));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(root.join("graph")).unwrap();
        std::fs::write(root.join("graph/graph.js"), "js").unwrap();
        std::fs::write(root.join("graph/b.json"), "{}").unwrap();
        let v = json!({"id":"steps","kind":"matrix","tabs":[{"id":"graph","kind":"embed",
            "script":"graph/graph.js","data":"graph/b.json","libs":["https://cdn.example/a.js"]}]});
        publish(&root, &v, &out).unwrap();
        assert_eq!(
            std::fs::read_to_string(out.join("graph/graph.js")).unwrap(),
            "js"
        );
        assert_eq!(
            std::fs::read_to_string(out.join("graph/b.json")).unwrap(),
            "{}"
        );
        let missing =
            json!({"id":"g","kind":"embed","script":"graph/none.js","data":"graph/b.json"});
        let e = format!("{:#}", publish(&root, &missing, &out).unwrap_err());
        assert!(e.contains("view g") && e.contains("none.js"), "{e}");
        let http = json!({"id":"g","kind":"embed","script":"graph/graph.js","data":"graph/b.json","libs":["http://x/a.js"]});
        assert!(publish(&root, &http, &out).is_err());
        let plain = json!({"id":"p","kind":"matrix"});
        publish(&root, &plain, &out).unwrap();
        std::fs::remove_dir_all(&base).unwrap();
    }
}
