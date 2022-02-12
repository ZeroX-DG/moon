# Layout process

After creating a render tree, the browser will need to create a layout tree from that render tree and compute the size & location of each elements visible on the page. This whole process is called layout process.

## Box tree construction

This is the process build a tree of `LayoutBox` from the render tree. `LayoutBox` comes in these flavors:

- BlockBox: Boxes that participate in a Block Formatting Context (BFC).
- InlineBox: Boxes that participate in an Inline Formatting Context (IFC).

The InlineBox however, can contains text, which in this case the box will be refered as TextRun.

> **Note:** LineBox is mentioned in the spec as a box that used to contains InlineBox. However, with the current layout algorithm we don't consider LineBox as a LayoutBox but merely a simple Box that is used to store InlineBoxes as part of a BlockBox after the IFC is run.

The box tree construction create a `LayoutBox` for a render node & establish a formatting context based on the children. However, the actual LayoutBox children will be inserted to the parent based on the formatting context that it establish instead. For example, a render node can contains a mixture of inline & block box, but those inline box will be inserted to an anonymous box if the parent layout box establish a BFC.

This part of the tree construction process is described in CSS Visual formatting model spec: https://www.w3.org/TR/CSS22/visuren.html

### Formatting context establishing

The formatting context currently established based on the css `display` property of render node or the children of the render node. If a node is a block node that contains only inline nodes, the box will establish an IFC. Refer to [`get_formatting_context_type()`][1] for more info.

> **Note:** In our current algorithm, when a formatting context is run on a node. It mainly perform the layout on the node's children but relies on some information resided in the current node.

## LayoutBox offset

The offset of the layout box is relative to it's [containing block][2]. Thus, the calculation of the LayoutBox offset should not be misunderstood as the calculation of the absolute position of the box on the page.

[1]: https://github.com/ZeroX-DG/moon/blob/7b8424e87c518a9aa0a1d025fb3f3d8a46ee97e0/components/layout/src/formatting_context.rs#L62
[2]: https://www.w3.org/TR/CSS22/visuren.html#containing-block
