mod painter;

use css::cssom::css_rule::CSSRule;
use style::value_processing::{ContextualRule, CSSLocation, CascadeOrigin};
use style::render_tree::build_render_tree;
use layout::{build_layout_tree, ContainingBlock, layout_box::LayoutBox};
use painting::paint;
use painter::SkiaPainter;
use skulpin::winit;

pub fn print_layout_tree(root: &LayoutBox, level: usize) {
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
        * { display: block; padding: 12px; }
        .a { background-color: rgb(52, 152, 219); }
        .b { background-color: rgb(155, 89, 182); }
        .c { background-color: rgb(52, 73, 94); }
        .d { background-color: rgb(231, 76, 60); }
        .e { background-color: rgb(230, 126, 34); }
        .f { background-color: rgb(241, 196, 15); }
        .g { background-color: rgb(127, 140, 141); height: 100px; }
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

    // return print_layout_tree(&layout_tree, 0);

    // window creation
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

    let logical_size = winit::dpi::LogicalSize::new(500.0, 400.0);
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

    layout::layout(&mut layout_tree, &mut ContainingBlock {
        offset_x: 0.,
        offset_y: 0.,
        x: 0.,
        y: 0.,
        width: logical_size.width,
        height: logical_size.height,
        previous_margin_bottom: 0.0,
        collapsed_margins_vertical: 0.0
    }, winit_window.scale_factor() as f32);

    println!("{}", winit_window.scale_factor());

    // Create the renderer, which will draw to the window
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

    event_loop.run(move |event, _window_target, control_flow| {
        let window = skulpin::WinitWindow::new(&winit_window);

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            //
            // Close if the escape key is hit
            //
            winit::event::Event::WindowEvent {
                event:
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
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
                    let painter = SkiaPainter::new(canvas);
                    // f*ck yeah! paint this shit!
                    paint(&layout_tree, painter);
                }) {
                    println!("Error during draw: {:?}", e);
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
            }

            _ => {}
        }
    });
}
