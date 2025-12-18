//! This modules contains the functionality for creating the tray icon for the program
//! and creating the thread for the event loop to run in.
use colour::yellow_ln_bold;
use image::{ImageFormat, ImageReader};
use log::{error, info};
use std::error::Error;
use std::io::{Cursor, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tray_icon::menu::{IsMenuItem, MenuEvent, MenuItem};
use tray_icon::{menu::Menu, Icon};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use winit::application::ApplicationHandler;
use winit::event_loop::EventLoop;

pub const IMAGE_BYTES: &[u8] = include_bytes!("../images/rl-hours-tracker-logo.ico");

#[derive(Debug)]
enum UserEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
    QuitApp(AtomicBool),
}

struct Application {
    currently_tracking: Option<Arc<Mutex<AtomicBool>>>,
    stop_tracker: Option<Arc<Mutex<AtomicBool>>>,
    tray_icon: Option<TrayIcon>,
}

impl Application {
    fn new() -> Application {
        Application {
            currently_tracking: None,
            stop_tracker: None,
            tray_icon: None,
        }
    }

    pub fn new_tray_icon() -> TrayIcon {
        let image = load_image(IMAGE_BYTES).unwrap_or_else(|e| {
            error!("error occurred when loading image: {e}");
            panic!("could not load image for tray icon");
        });
        TrayIconBuilder::new()
            .with_menu(Box::new(Self::new_tray_menu()))
            .with_tooltip("RL Hours Tracker")
            .with_icon(image)
            .build()
            .unwrap_or_else(|e| {
                error!("error occurred creating tray icon: {e}");
                panic!("could not create tray icon");
            })
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

    fn set_currently_tracking(&mut self, currently_tracking: &Arc<Mutex<AtomicBool>>) -> &mut Self {
        self.currently_tracking = Some(currently_tracking.clone());

        self
    }

    fn set_stop_tracker(&mut self, stop_tracker: &Arc<Mutex<AtomicBool>>) -> &mut Self {
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
            #[cfg(not(target_os = "linux"))]
            {
                self.tray_icon = Some(Self::new_tray_icon())
            }
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        info!("Cleaning up system tray...");
        self.tray_icon.take();
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
                } else if menu.id == "1002"
                    && self
                        .currently_tracking
                        .as_ref()
                        .unwrap_or_else(|| {
                            error!("currently_tracking is None");
                            panic!("currently_tracking is None");
                        })
                        .try_lock()
                        .unwrap_or_else(|e| {
                            error!(
                                "error when attempting to access lock for currently_tracking: {e}"
                            );
                            panic!("error when attempting to access lock for currently_tracking");
                        })
                        .fetch_and(true, Ordering::SeqCst)
                {
                    *self
                        .stop_tracker
                        .as_mut()
                        .unwrap_or_else(|| {
                            error!("currently_tracking is None");
                            panic!("currently_tracking is None");
                        })
                        .try_lock()
                        .unwrap_or_else(|e| {
                            error!(
                                "error when attempting to access lock for currently_tracking: {e}"
                            );
                            panic!("error when attempting to access lock for currently_tracking");
                        }) = true.into();

                    *self
                        .currently_tracking
                        .as_mut()
                        .unwrap_or_else(|| {
                            error!("currently_tracking is None");
                            panic!("currently_tracking is None");
                        })
                        .try_lock()
                        .unwrap_or_else(|e| {
                            error!(
                                "error when attempting to access lock for currently_tracking: {e}"
                            );
                            panic!("error when attempting to access lock for currently_tracking");
                        }) = false.into();
                }
            }
            UserEvent::QuitApp(quit) => {
                if quit.fetch_and(true, Ordering::SeqCst) {
                    _event_loop.exit();
                }
            }
            UserEvent::TrayIconEvent(_tray) => {}
        }
    }
}

pub fn initialize_tray_icon(
    stop_tracker: Arc<Mutex<AtomicBool>>,
    currently_tracking: Arc<Mutex<AtomicBool>>,
) {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .unwrap_or_else(|e| {
            error!("error occurred creating event loop: {e}");
            panic!("could not create event loop for tray icon");
        });

    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
    }));

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let proxy = event_loop.create_proxy();
    ctrlc::set_handler(move || {
        info!("Interrupting program");
        print!("{}[2K\r", 27 as char);
        std::io::stdout()
            .flush()
            .expect("could not flush the output stream");
        yellow_ln_bold!("Goodbye!");
        let _ = proxy.send_event(UserEvent::QuitApp(AtomicBool::new(true)));
    })
    .unwrap_or_else(|e| {
        error!("could not create handler for ctrlc: {e}");
    });

    let mut app = Application::new();
    app.set_stop_tracker(&stop_tracker);
    app.set_currently_tracking(&currently_tracking);

    if let Err(e) = event_loop.run_app(&mut app) {
        error!("Error: {e:?}");
        colour::e_red_ln!("Error: {e:?}");
    }
}

pub fn load_image(image_bytes: &[u8]) -> Result<Icon, Box<dyn Error>> {
    let mut image_reader = ImageReader::new(Cursor::new(image_bytes));
    image_reader.set_format(ImageFormat::Ico);

    let image = image_reader.decode()?;
    let (icon_rgba, icon_width, icon_height) = {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height)?;

    Ok(icon)
}
