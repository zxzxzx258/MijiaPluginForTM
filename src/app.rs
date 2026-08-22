use crate::{
    config::{self, AppConfig},
    metrics::{self, CounterSnapshot, NetworkSnapshot},
    miio,
};
use cosmic::{
    Element, Task, app,
    applet::{
        cosmic_panel_config::PanelAnchor, token::subscription::activation_token_subscription,
    },
    iced::{
        Alignment, Length, Rectangle, Subscription,
        futures::{SinkExt, channel::mpsc},
        platform_specific::shell::wayland::commands::popup::destroy_popup,
        stream, window,
    },
    surface,
    widget::{
        Id, autosize, button, column, container, dropdown::popup_dropdown, icon,
        rectangle_tracker::*, row, text,
    },
};
use std::{collections::HashMap, hash::Hash, sync::LazyLock, time::Duration};

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("monitor-autosize-main"));

#[derive(Hash)]
struct TickSettings(u64);

pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<MonitorApplet>(())
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Power {
        device_id: String,
        result: Result<f64, String>,
    },
    InterfaceSelected(usize),
    Surface(surface::Action),
    TogglePopup,
    Rectangle(RectangleUpdate<u32>),
    PopupClosed(window::Id),
    Token,
    Close(window::Id),
}

#[derive(Clone)]
struct MonitorApplet {
    core: app::Core,
    config: AppConfig,
    config_error: Option<String>,
    counters: CounterSnapshot,
    network: NetworkSnapshot,
    interfaces: Vec<String>,
    devices: HashMap<String, DeviceState>,
    rectangle_tracker: Option<RectangleTracker<u32>>,
    rectangle: Rectangle,
    popup: Option<window::Id>,
}

#[derive(Debug, Clone, Default)]
struct DeviceState {
    watts: Option<f64>,
    error: Option<String>,
    in_flight: bool,
}

impl MonitorApplet {
    fn refresh_interfaces(&mut self) {
        self.interfaces = metrics::active_physical_interfaces();
        if let Some(interface) = &self.config.interface
            && !self.interfaces.contains(interface)
        {
            self.interfaces.push(interface.clone());
            self.interfaces.sort();
        }
    }

    fn device_power_text(&self, device: &config::MiJiaConfig) -> String {
        let watts = self
            .devices
            .get(&device.stable_id())
            .and_then(|state| state.watts)
            .map_or_else(|| "--W".into(), |watts| format!("{watts:.0}W"));
        format!("{}：{watts}", device.name)
    }

    fn panel_content(&self) -> Element<'_, Message> {
        let download = self
            .core
            .applet
            .text(format!(
                "↓ {}",
                metrics::format_rate_compact(self.network.rate.rx_bytes_per_second)
            ))
            .size(12);
        let upload = self
            .core
            .applet
            .text(format!(
                "↑ {}",
                metrics::format_rate_compact(self.network.rate.tx_bytes_per_second)
            ))
            .size(12);
        let mut content = row![
            icon::from_name("io.github.cosmic.Monitor-symbolic")
                .size(self.core.applet.suggested_size(true).0)
                .symbolic(true)
        ]
        .spacing(14)
        .align_y(Alignment::Center);
        if self.config.show_network {
            content = content.push(download).push(upload);
        }
        if self.config.show_mijia {
            for device in &self.config.mijia {
                content = content.push(
                    self.core
                        .applet
                        .text(self.device_power_text(device))
                        .size(12),
                );
            }
        }
        content.into()
    }

    fn popup_content(&self, id: window::Id) -> Element<'_, Message> {
        let config_status = self
            .config_error
            .as_deref()
            .unwrap_or("configuration loaded");
        let mut details = column![
            text("COSMIC Monitor").size(18),
            text("网卡选择"),
            popup_dropdown(
                std::iter::once(format!(
                    "自动（当前：{}）",
                    metrics::automatic_interface().unwrap_or_else(|| "无".into())
                ))
                .chain(self.interfaces.iter().cloned())
                .collect::<Vec<_>>(),
                Some(
                    self.config
                        .interface
                        .as_ref()
                        .and_then(|selected| {
                            self.interfaces
                                .iter()
                                .position(|interface| interface == selected)
                        })
                        .map_or(0, |index| index + 1),
                ),
                Message::InterfaceSelected,
                id,
                Message::Surface,
                |message| message,
            )
            .width(Length::Fixed(260.0)),
            text(format!("网络接口：{}", self.network.interface)),
            text(format!(
                "下载：{}",
                metrics::format_rate(self.network.rate.rx_bytes_per_second)
            )),
            text(format!(
                "上传：{}",
                metrics::format_rate(self.network.rate.tx_bytes_per_second)
            )),
        ]
        .spacing(8)
        .padding(16);
        if self.config.mijia.is_empty() {
            details = details.push(text("米家设备：未配置"));
        } else {
            for device in &self.config.mijia {
                let state = self.devices.get(&device.stable_id());
                let power = state
                    .and_then(|state| state.watts)
                    .map_or_else(|| "-- W".into(), |watts| format!("{watts:.0} W"));
                let status = state
                    .and_then(|state| state.error.as_deref())
                    .unwrap_or("已连接");
                details = details.push(text(format!("{}：{power}（{status}）", device.name)));
            }
        }
        details = details
            .push(text(config_status).size(11))
            .push(button::standard("关闭").on_press(Message::Close(id)));
        self.core
            .applet
            .popup_container(container(details).width(Length::Fill))
            .into()
    }
}

impl cosmic::Application for MonitorApplet {
    type Message = Message;
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    const APP_ID: &str = "io.github.cosmic.Monitor";

    fn init(core: app::Core, _flags: ()) -> (Self, app::Task<Message>) {
        let (config, config_error) = config::load();
        let mut applet = Self {
            core,
            config,
            config_error,
            counters: CounterSnapshot::default(),
            network: NetworkSnapshot::default(),
            interfaces: Vec::new(),
            devices: HashMap::new(),
            rectangle_tracker: None,
            rectangle: Rectangle::default(),
            popup: None,
        };
        applet.refresh_interfaces();
        (applet, Task::done(cosmic::Action::App(Message::Tick)))
    }
    fn core(&self) -> &app::Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut app::Core {
        &mut self.core
    }
    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            rectangle_tracker_subscription(0).map(|event| Message::Rectangle(event.1)),
            Subscription::run_with(
                TickSettings(self.config.refresh_seconds.max(1)),
                |settings| {
                    let refresh = settings.0;
                    stream::channel(1, move |mut output: mpsc::Sender<Message>| async move {
                        let mut timer = tokio::time::interval(Duration::from_secs(refresh));
                        loop {
                            timer.tick().await;
                            if output.send(Message::Tick).await.is_err() {
                                break;
                            }
                        }
                    })
                },
            ),
            activation_token_subscription(0).map(|_| Message::Token),
        ])
    }

    fn update(&mut self, message: Message) -> app::Task<Message> {
        match message {
            Message::Tick => {
                self.refresh_interfaces();
                let interface = self.config.interface.clone();
                if let Ok(snapshot) = self.counters.sample(interface.as_deref()) {
                    self.network = snapshot;
                }
                let pending = self
                    .config
                    .mijia
                    .iter()
                    .filter(|device| {
                        !self
                            .devices
                            .get(&device.stable_id())
                            .is_some_and(|state| state.in_flight)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                for device in &pending {
                    self.devices
                        .entry(device.stable_id())
                        .or_default()
                        .in_flight = true;
                }
                return Task::batch(pending.into_iter().map(|device| {
                    let device_id = device.stable_id();
                    Task::future(async move {
                        Message::Power {
                            device_id,
                            result: miio::read_power(
                                device.ip,
                                &device.token,
                                &device.model,
                                device.power_siid,
                                device.power_piid,
                            )
                            .await
                            .map(|r| r.watts)
                            .map_err(|e| e.to_string()),
                        }
                    })
                    .map(cosmic::Action::App)
                }));
            }
            Message::Power { device_id, result } => match result {
                Ok(watts) => {
                    self.devices.insert(
                        device_id,
                        DeviceState {
                            watts: Some(watts),
                            error: None,
                            in_flight: false,
                        },
                    );
                }
                Err(error) => {
                    let state = self.devices.entry(device_id).or_default();
                    state.error = Some(error);
                    state.in_flight = false;
                }
            },
            Message::InterfaceSelected(index) => {
                self.config.interface = if index == 0 {
                    None
                } else {
                    self.interfaces.get(index - 1).cloned()
                };
                self.counters = CounterSnapshot::default();
                self.config_error = config::save(&self.config).err();
                return Task::done(cosmic::Action::App(Message::Tick));
            }
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }
            Message::TogglePopup => {
                if let Some(id) = self.popup.take() {
                    return destroy_popup(id);
                }
                return surface::surface_task(surface::action::app_popup(
                    |_| Default::default(),
                    |app: &mut Self| {
                        let id = window::Id::unique();
                        app.popup = Some(id);
                        let mut settings = app.core.applet.get_popup_settings(
                            app.core.main_window_id().unwrap(),
                            id,
                            None,
                            None,
                            None,
                        );
                        let Rectangle {
                            x,
                            y,
                            width,
                            height,
                        } = app.rectangle;
                        settings.positioner.anchor_rect = Rectangle::<i32> {
                            x: x.max(1.0) as i32,
                            y: y.max(1.0) as i32,
                            width: width.max(1.0) as i32,
                            height: height.max(1.0) as i32,
                        };
                        settings.positioner.size = None;
                        settings
                    },
                    None,
                ));
            }
            Message::Rectangle(update) => match update {
                RectangleUpdate::Rectangle(rectangle) => self.rectangle = rectangle.1,
                RectangleUpdate::Init(tracker) => self.rectangle_tracker = Some(tracker),
            },
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Token => {}
            Message::Close(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                    return destroy_popup(id);
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let horizontal = matches!(
            self.core.applet.anchor,
            PanelAnchor::Top | PanelAnchor::Bottom
        );
        let button = button::custom(self.panel_content())
            .padding(if horizontal {
                [0, self.core.applet.suggested_padding(true).0]
            } else {
                [self.core.applet.suggested_padding(true).0, 0]
            })
            .on_press_down(Message::TogglePopup)
            .class(cosmic::theme::Button::AppletIcon);

        autosize::autosize(
            if let Some(tracker) = self.rectangle_tracker.as_ref() {
                Element::from(tracker.container(0, button).ignore_bounds(true))
            } else {
                button.into()
            },
            AUTOSIZE_MAIN_ID.clone(),
        )
        .into()
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Message> {
        self.popup_content(id)
    }
}
