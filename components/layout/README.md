# Layout

The layout process calculate the position of each elements on the screen.

## Box tree generation

- In the layout process, the box tree is a tree structure of "boxes". These boxes are rectangular regions that each element generate to calculate size and position.
- Each element must generate 1 principle box and optionally many marker-box. Unless the element's `display` value is `none`.
- Each box has a formatting context, which is an environment that decide how the box should layout its contents.
- Each layout support different types of formatting contexts. For example:
  - Flow layout (block-and-inline layout):
    - Block formatting context
    - Inline formatting context
  - Flex layout:
    - Flex formatting context
  - Grid layout:
    - Grid formatting context
- A box can either establish a new formatting context, which in this case is called "independent formatting context" or continue the formatting context of the parent box.
- A formatting context of a box is decided by the `display` property of the element that generate that box.
- A `display` property consists of 2 parts:
  - **Inner display type:** Determines which formatting context to use.
  - **Outer display type:** Determines how the box participate in the flow layout.
- The value of the `display` property can either be in `short` form or `full` form. The `full` form syntax is (`Outer display type` `<space>` `Inner display type`).
  - Short: `block` | Full: `block flow`
  - Short: `inline` | Full: `inline flow`
  - Short: `flex` | Full: `block flex`
- A full table of what type of box to generate base on `display` value is available [here][boxgentable].

[boxgentable]: https://drafts.csswg.org/css-display-3/#the-display-properties
