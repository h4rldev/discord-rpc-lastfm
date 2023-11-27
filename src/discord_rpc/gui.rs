use anyhow::Result;
use home::home_dir;
use iced::theme;
use iced::widget::{checkbox, column, container, svg};
use iced::{color, Element, Length, Sandbox, Settings};
use image::{GenericImageView, ImageBuffer, ImageFormat, Rgba};
use std::thread;
use std::{io::Cursor, path::Path};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
//use tray::start_tray;
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};

mod tray;

pub fn init() {
    let event_loop = EventLoopBuilder::new().build();
    let quit_i = MenuItem::new("Quit", true, None);
    let menu_items = vec![quit_i.clone()];

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    let icon = load_icon();

    let tray_menu = Menu::new();
    tray_menu
        .append_items(&[
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("tao".to_string()),
                    copyright: Some("Copyright tao".to_string()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &menu_items[0],
        ])
        .expect("Failed to add menu items");

    let mut tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("tao - awesome windowing lib")
            .with_icon(icon)
            .build()
            .unwrap(),
    );

    match Tiger::run(Settings::default()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    };

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_i.id() {
                tray_icon.take();
                *control_flow = ControlFlow::Exit;
            }
            println!("{event:?}");
        }

        if let Ok(event) = tray_channel.try_recv() {
            println!("{event:?}");
        }
    })
}

fn load_icon() -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let icon_data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/icon.ico"));
        let img =
            image::load(Cursor::new(icon_data), ImageFormat::Ico).expect("Failed to load icon");
        let (width, height) = img.dimensions();
        //let key_combo: Accelerator = "Shift+Alt+Ctrl+D".parse()?;
        let rgba = img.to_rgba8().into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

#[derive(Debug, Default)]
struct Tiger {
    apply_color_filter: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    ToggleColorFilter(bool),
}

impl Sandbox for Tiger {
    type Message = Message;

    fn new() -> Self {
        Tiger::default()
    }

    fn title(&self) -> String {
        String::from("SVG - Iced")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::ToggleColorFilter(apply_color_filter) => {
                self.apply_color_filter = apply_color_filter;
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let apply_color_filter = checkbox(
            "Apply a color filter",
            self.apply_color_filter,
            Message::ToggleColorFilter,
        );

        container(
            column![container(apply_color_filter).width(Length::Fill).center_x()]
                .spacing(20)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
    }
}
