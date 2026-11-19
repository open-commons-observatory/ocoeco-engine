# ocoeco-engine

ocoeco's rendering engine - the face of the [Open Commons Observatory](https://open-commons-observatory.github.io/): its website,
rendered from the data that [ocoeco](https://github.com/open-commons-observatory/ocoeco) - the
observatory's information ecosystem - gathers.

    oco site --data DIR --out DIR

Every word and number on the site comes from the data: the observatory and its family, the grand
projects with their views (how close they are, tables ranked best first, devices, families, tests,
backlog, landscape), every project's path on three horizons, and how the commons works. One
stylesheet; graphics as inline SVG; no JavaScript for a view that does not need it, and a page
readable without it where possible (the table is the default of a page that also has a graph). The
mark, discovery and the harvest live in ocoeco; this repository is only the face.

## A view that needs a script

A lens may give a view `tabs`: more views of the same subject, one page, a strip of links
(`#table`, `#graph`) above them. The view itself is the first tab and the default; the strip
works from the address alone (CSS `:target`), so without JavaScript the first tab shows. A tab of
kind `embed` names, in the lens, what the engine copies from the data directory next to the page
and wires in:

    [[view]]
    id = "steps"  kind = "matrix"  ...  tab_title = "Table"      # tab_id defaults to "table"
    [[view.tabs]]
    id = "graph"
    title = "Graph"
    kind = "embed"
    script = "graph/graph.js"                  # paths in the data directory
    data = "graph/bringup.json"
    libs = ["https://cdn.jsdelivr.net/npm/cytoscape@3.30.2/dist/cytoscape.min.js"]   # pinned, https

The page gets the libraries' script tags, then `<div class="embed" data-data="graph/bringup.json">`
(with a note for readers without JavaScript), then the script, which finds its container as the
element just before it. The container may be hidden when the script runs (another tab shows): a
script that needs a size waits until it has one. A view of kind `embed` alone, with no tabs, is the
same container under the view's head.
