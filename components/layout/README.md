# Layout process

The layout process decides where the elements on the web page will be displayed.It contains these steps:

1. Box tree construction
2. Layout boxes
  1. Compute size of the box
  2. Compute position of the box
  3. Layout children box of this box
  3. Apply explicit sizing to the box

## Box tree construction

This is the process build a tree of `LayoutBox` from the render tree. The algorithm for this step is roughly like this:

```rust
pub fn build_layout_tree(node: RenderNode) {
  // 1. Create the box for the node

  // 2. For each child of the node
    // 1. Create the sub tree for the child node
    // 2. Find a place to insert into the created box
    // 3. Insert the node into the location

  // 3. Return the node
}
```

A sample HTML input might looks like this:

```html
<html>
  <body>
    <h1>Title</h1>
    <div>
      <span>Text</span>
    </div>
    <span>
      This is some text
      <div>Inner</div>
      This is another text
    </span>
  </body>
</html>
```

A layout tree should looks like this:

```
[BlockLevelBox] - Element("HTML")/
├─ [BlockLevelBox] - Element("body")/
│  ├─ [InlineLevelBox Anonymous]/
│  │  ├─ [TextRun] - Text("Title")
│  ├─ [BlockLevelBox] - Element("div")/
│  │  ├─ [InlineLevelBox] - Element("span")/
│  │  │  ├─ [TextRun] - Text("Text")
│  ├─ [BlockLevelBox Anonymous]/
│  │  ├─ [InlineLevelBox] - Element("span")/
│  │  │  ├─ [TextRun] - Text("This is some text")
│  ├─ [BlockLevelBox] - Element("div")/
│  │  ├─ [InlineLevelBox Anonymous]/
│  │  │  ├─ [TextRun] - Text("Inner")
│  ├─ [BlockLevelBox - Anonymous]/
│  │  ├─ [InlineLevelBox]/
│  │  │  ├─ [TextRun] - Text("This is another text")
```

The algorithm for box creation:

```rust
fn create_box(node: RenderNode) -> LayoutBox {
  // 1. If the render node is a text node return a text run
  // 2. Match the box outer display type
  //   1. If Block => return a BlockLevelBox
  //   2. If inline => return an InlineLevelBox
}
```

The algorithm for finding insertion place

```rust
fn find_insertion_place(parent_stack: &mut Vec<LayoutBox>, layout_box: &LayoutBox) -> &mut LayoutBox {
  // 1. If the layout_box is an InlineLevelBox
  //   1. If the last box in the parent stack doesn't have all inline children
  //     1. Create an anonymous BlockLevelBox and mark as inline children only
  //     2. Insert the anonymous box as child of the last box in the parent stack
  //     3. return the anonymous box
  //   2. return the last box in parent_stack
  // 2. If the layout_box is an BlockLevelBox
  //   1. While the last box in the parent_stack is not a BlockLevelBox
  //     1. Pop the last box in the parent_stack
  //   2. If the last box in the parent_stack contains all inline children
  //     1. Create an anonymous BlockLevelBox
  //     2. Mark the anonymous box as contains only inline children
  //     3. Move all inline boxes of the last box in parent_stack into the anonymous box
  //     4. Insert the anonymous box into the last box in parent_stack
  //     5. Return the last box in the parent_stack
  //   3. Return the last box in the parent_stack
}
```

## Layout boxes

This process decides the size and position of each box.

```rust
impl LayoutBox {
  fn layout(&mut self) {
    let mut context = self.establish_formatting_context();
    context.layout(self.children.as_mut());
  }

  fn establish_formatting_context(&self) -> Box<dyn FormattingContext> {
    // 1. Match box inner display type
    //   If Flow => Match box only contains inline boxes
    //     If true => return InlineFormattingContext
    //     Else => return BlockFormattingContext
    //   If Flex => return FlexFormattingContext
    //   If Grid => return GridFormattingContext
  }
}
```

Each formatting should share these behavior:

```rust
pub struct BaseFormattingContext {
  offset_x: f32,
  offset_y: f32,
  width: f32,
  height: f32
}

pub trait FormattingContext {
  fn layout(&mut self, boxes: Vec<&mut LayoutBox>) {
    for layout_box in boxes.iter_mut() {
      self.calculate_size(layout_box);
      self.calculate_position(layout_box);
      layout_box.layout();
      self.update_new_data(layout_box);
      self.apply_explicit_sizes(layout_box);
    }
  }

  fn calculate_size(&mut self, layout_box: &mut LayoutBox);

  fn base(&self) -> &BaseFormattingContext;

  fn update_new_data(&self, layout_box: &LayoutBox);

  fn calculate_position(&mut self, layout_box: &mut LayoutBox) {
    // 1. Set the x = offset x of the context + box's margin left + padding left + border left
    // 2. Set the y = offset y of the context + box's margin top + padding top + border top
    // 3. Set the position of the box to the calculated x, y
  }

  fn apply_explicit_sizes(&mut self, layout_box: &mut LayoutBox) {
    // 1. If the layout_box is not an anonymous
    //   1. Get the computed style for width and height of the associated render node
    //   2. If the width or height is not `auto` and the box is not an inline box
    //     1. Assign those value for the size of the box
  }
}
```

Each formatting context has it own way to calculate size and position:

### Block formatting context

In block formatting context every box is placed in a vertical direction one after each other.

```rust
impl FormattingContext for BlockFormattingContext {
  fn base(&self) -> &BaseFormattingContext {
    &self.base
  }
  
  fn calculate_size(&mut self, layout_box: &mut LayoutBox) {
    // Follow the rule in the spec
  }

  fn update_new_data(&mut self, layout_box: &LayoutBox) {
    let rect = layout_box.dimensions.margin_box();
    self.base.height += rect.height;
    self.base.offset_y += rect.height;

    if self.base.width < rect.width {
      self.base.width = rect.width;
    }
  }
}
```

### Inline formatting context

In inline formatting context every box is placed in a horizontal direction one after each other.

```rust
impl FormattingContext for InlineFormattingContext {
  fn base(&self) -> &BaseFormattingContext {
    &self.base
  }
  
  fn calculate_size(&mut self, layout_box: &mut LayoutBox) {
    // Follow the rule in the spec
  }

  fn update_new_data(&mut self, layout_box: &LayoutBox) {
    let rect = child.dimensions.margin_box();
    self.base.width += rect.width;
    self.base.offset_x += rect.width;

    if self.base.height < rect.height {
      self.base.height = rect.height;
    }
  }
}
```
