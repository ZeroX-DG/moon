mod painter;

use css::cssom::css_rule::CSSRule;
use layout::{build_layout_tree, layout_box::LayoutBox, ContainingBlock};
use painter::SkiaPainter;
use painting::paint;
use skulpin::winit;
use style::render_tree::build_render_tree;
use style::value_processing::{CSSLocation, CascadeOrigin, ContextualRule};

fn print_layout_tree(root: &LayoutBox, level: usize) {
    let child_nodes = &root.children;
    println!(
        "{}{:#?}({:#?})(x: {} | y: {} | width: {} | height: {})",
        "    ".repeat(level),
        root.box_type,
        root.render_node.borrow().node,
        root.dimensions.content.x,
        root.dimensions.content.y,
        root.dimensions.content.width,
        root.dimensions.content.height
    );
    for node in child_nodes {
        print_layout_tree(node, level + 1);
    }
}

fn main() {
    // parse HTML
    let html = include_str!("../fixtures/test.html");
    let tokenizer = html::tokenizer::Tokenizer::new(html.to_owned());
    let tree_builder = html::tree_builder::TreeBuilder::new(tokenizer);
    let document = tree_builder.run();

    // parse CSS
    let css = r#"
        * { display: block; }
        div { padding: 12px; }
        .a { background-color: rgb(52, 152, 219); }
        .b { background-color: rgb(155, 89, 182); }
        .c { background-color: rgb(52, 73, 94); }
        .d { background-color: rgb(231, 76, 60); }
        .e { background-color: rgb(230, 126, 34); }
        .f { background-color: rgb(241, 196, 15); }
        .g { background-color: rgb(64, 64, 122); }
    "#;

    let tokenizer = css::tokenizer::Tokenizer::new(css.to_string());
    let mut parser = css::parser::Parser::<css::tokenizer::token::Token>::new(tokenizer.run());
    let stylesheet = parser.parse_a_css_stylesheet();

    let rules = stylesheet
        .iter()
        .map(|rule| match rule {
            CSSRule::Style(style) => ContextualRule {
                inner: style,
                location: CSSLocation::Embedded,
                origin: CascadeOrigin::User,
            },
        })
        .collect::<Vec<ContextualRule>>();

    // layout
    let render_tree = build_render_tree(document, &rules);
    let mut layout_tree = build_layout_tree(render_tree.root.unwrap()).unwrap();

    let logical_size = winit::dpi::LogicalSize::new(500.0, 300.0);

    layout::layout(
        &mut layout_tree,
        &mut ContainingBlock {
            offset_x: 0.,
            offset_y: 0.,
            x: 0.,
            y: 0.,
            width: logical_size.width,
            height: logical_size.height,
            previous_margin_bottom: 0.0,
            collapsed_margins_vertical: 0.0,
        },
    );

    print_layout_tree(&layout_tree, 0);

    // window creation
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    let visible_range = skulpin::skia_safe::Rect {
        left: 0.0,
        right: logical_size.width as f32,
        top: 0.0,
        bottom: logical_size.height as f32,
    };
    let scale_to_fit = skulpin::skia_safe::matrix::ScaleToFit::Center;

    // Create a single window
    let winit_window = winit::window::WindowBuilder::new()
        .with_title("Moon")
        .with_inner_size(logical_size)
        .build(&event_loop)
        .expect("Failed to create window");

    let window = skulpin::WinitWindow::new(&winit_window);

    let renderer = skulpin::RendererBuilder::new()
        .use_vulkan_debug_layer(false)
        .coordinate_system(skulpin::CoordinateSystem::VisibleRange(
            visible_range,
            scale_to_fit,
        ))
        .build(&window);

    // Check if there were error setting up vulkan
    if let Err(e) = renderer {
        println!("Error during renderer construction: {:?}", e);
        return;
    }

    let mut renderer = renderer.unwrap();
    let mut painter = SkiaPainter::new();

    event_loop.run(move |event, _, control_flow| {
        let window = skulpin::WinitWindow::new(&winit_window);

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            //
            // Request a redraw any time we finish processing events
            //
            winit::event::Event::MainEventsCleared => {
                // Queue a RedrawRequested event.
                winit_window.request_redraw();
            }
            //
            // Redraw
            //
            winit::event::Event::RedrawRequested(_window_id) => {
                if let Err(e) = renderer.draw(&window, |canvas, _| {
                    paint(&layout_tree, &mut painter, canvas);
                }) {
                    println!("Error during draw: {:?}", e);
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
            }
            _ => {}
        }
    });
}
