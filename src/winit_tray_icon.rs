use colour::yellow_ln_bold;
use std::io::Write;
use std::process;
use std::sync::{Arc, Mutex};
use tray_icon::menu::{IsMenuItem, MenuEvent, MenuItem};
use tray_icon::{menu::Menu, Icon};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use winit::application::ApplicationHandler;
use winit::event_loop::EventLoop;
use winit::platform::windows::EventLoopBuilderExtWindows;

#[derive(Debug)]
enum UserEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

struct Application {
    stop_tracker: Option<Arc<Mutex<bool>>>,
    tray_icon: Option<TrayIcon>,
}

impl Application {
    fn new() -> Application {
        Application { stop_tracker: None, tray_icon: None }
    }

    fn new_tray_icon() -> TrayIcon {
        TrayIconBuilder::new()
            .with_menu(Box::new(Self::new_tray_menu()))
            .with_tooltip("RL Hours Tracker")
            .with_icon(
                Icon::from_path("images/rl-hours-tracker-logo.ico", Some((128, 128))).unwrap(),
            )
            .build()
            .unwrap()
    }

    fn new_tray_menu() -> Menu {
        let tray_menu = Menu::new();
        let menu_item1 = MenuItem::new("Exit", true, None);
        let menu_item2 = MenuItem::new("Stop Tracker", true, None);
        let items: Vec<&dyn IsMenuItem> = vec![&menu_item1, &menu_item2];

        if let Err(e) = tray_menu.append_items(&items) {
            println!("{e:?}");
        }

        tray_menu
    }

    fn set_stop_tracker(&mut self, stop_tracker: &Arc<Mutex<bool>>) -> &mut Self {
        self.stop_tracker = Some(stop_tracker.clone());

        self
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if winit::event::StartCause::Init == cause {
            self.tray_icon = Some(Self::new_tray_icon())
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::MenuEvent(menu) => {
                if menu.id == "1001" {
                    _event_loop.exit();
                    print!("{}[2K\r", 27 as char);
                    std::io::stdout()
                        .flush()
                        .expect("could not flush the output stream");
                    yellow_ln_bold!("Goodbye!");
                    process::exit(0);
                } else if menu.id == "1002" {
                    *self.stop_tracker.as_mut().unwrap().try_lock().unwrap() = true;
                }
            }
            UserEvent::TrayIconEvent(_tray) => {}
        }
    }
}

pub fn initialize_tray_icon(stop_tracker: Arc<Mutex<bool>>) {
    std::thread::spawn(move || {
        let event_loop = EventLoop::<UserEvent>::with_user_event()
            .with_any_thread(true)
            .build()
            .unwrap();

        let proxy = event_loop.create_proxy();
        TrayIconEvent::set_event_handler(Some(move |event| {
            let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
        }));

        let proxy = event_loop.create_proxy();
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = proxy.send_event(UserEvent::MenuEvent(event));
        }));

        let mut app = Application::new();
        app.set_stop_tracker(&stop_tracker);

        if let Err(e) = event_loop.run_app(&mut app) {
            println!("Error: {e:?}")
        }
    });
}
