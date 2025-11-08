mod event;

use std::cell::RefCell;
use std::rc::Rc;
use web_time::Instant;

use rand::prelude::{SmallRng, *};
use ratatui::{prelude::Terminal, Frame};
use ratzilla::{backend::canvas::CanvasBackendOptions, CanvasBackend, WebRenderer};
use web_sys::wasm_bindgen::JsCast;

use solitaire::ui;
use solitaire::ui::component::Component;

const TICK_RATE: web_time::Duration = web_time::Duration::from_millis(100);
const PARENT_ELEMENT_ID: &str = "tui";

fn get_parent() -> web_sys::Element {
    let doc = web_sys::window().unwrap().document().unwrap();
    doc.get_element_by_id(PARENT_ELEMENT_ID).unwrap()
}

fn get_canvas(parent: &web_sys::Element) -> web_sys::HtmlCanvasElement {
    parent.first_child().unwrap().dyn_into().unwrap()
}

fn update_font(canvas: &web_sys::HtmlCanvasElement) {
    let ctx: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();
    ctx.set_font("19px 'JetBrains Mono', monospace");
}

fn create_terminal(parent: &web_sys::Element) -> ui::error::Result<Terminal<CanvasBackend>> {
    // Firstly, delete the existing canvas if it exists
    match parent.first_child() {
        Some(canvas) => {
            parent.remove_child(&canvas).unwrap();
        }
        _ => {}
    }

    // Create the backend
    let backend =
        // Provide a parent element ID so we can grab the canvas ourselves
        CanvasBackend::new_with_options(CanvasBackendOptions::new().grid_id(PARENT_ELEMENT_ID))
            .unwrap();
    // Update the font
    update_font(&get_canvas(parent));

    Ok(Terminal::new(backend)?)
}

fn on_resize(terminal: Rc<RefCell<Terminal<CanvasBackend>>>) {
    let on_resize = web_sys::wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
        // It's a bit inefficient but the canvas backend really doesn't like being resized,
        // so on resize just recreate the terminal
        *terminal.borrow_mut() = create_terminal(&get_parent()).unwrap();
    });
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())
        .unwrap();
    on_resize.forget();
}

fn request_animation_frame(f: &web_sys::wasm_bindgen::closure::Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

fn main() -> ui::error::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let terminal = Rc::new(RefCell::new(create_terminal(&get_parent())?));

    let rng = SmallRng::from_rng(thread_rng()).unwrap();
    let app = Rc::new(RefCell::new(ui::component::app::AppComponent::new(&rng)));

    on_resize(terminal.clone());

    terminal.clone().borrow().on_key_event({
        let app = app.clone();
        move |event| {
            app.borrow_mut()
                .handle_event(&event::convert_key_event(event))
                .unwrap();
        }
    });

    terminal.clone().borrow().on_mouse_event({
        let app = app.clone();
        move |event| {
            app.borrow_mut()
                .handle_event(&event::convert_mouse_event(event))
                .unwrap();
        }
    });

    // todo implement autosave

    let mut last_tick_instant = Instant::now();
    let mut on_render = move |frame: &mut Frame| {
        let mut app = app.borrow_mut();

        // Add ticks
        let now = Instant::now();
        let mut dt = now.duration_since(last_tick_instant);
        while dt >= TICK_RATE {
            last_tick_instant = now;
            app.handle_tick(&dt).unwrap();
            dt = now.duration_since(last_tick_instant);
        }

        app.render(frame, frame.area());
    };

    // Can't use Ratzilla's draw_web because it takes ownership of the terminal
    // but the on resize callback also needs ownership so we can update it,
    // so this is just a reimplementation of what they do (but with a borrow for the Rc)
    let callback = Rc::new(RefCell::new(None));
    *callback.borrow_mut() = Some(web_sys::wasm_bindgen::closure::Closure::wrap(Box::new({
        let cb = callback.clone();
        move || {
            terminal
                .borrow_mut()
                .draw(|frame| on_render(frame))
                .unwrap();
            request_animation_frame(cb.borrow().as_ref().unwrap());
        }
    })
        as Box<dyn FnMut()>));
    request_animation_frame(callback.borrow().as_ref().unwrap());

    Ok(())
}
