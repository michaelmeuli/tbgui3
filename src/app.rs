use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
    env, process,
};

use crate::actions::Action;
use crate::actions::ApplicationAction;
use crate::content::{self, Content};
use crate::context::ContextPage;
use crate::dialog::DialogPage;
use crate::fl;
use crate::model::Sample;
use crate::views::nav::{get_nav_model, NavPage};
use crate::app::key_bind::key_binds;
use async_ssh2_tokio::client::Client;
use config::TbguiConfig;
use cosmic::app::context_drawer;
use cosmic::app::Core;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::{self, nav_bar};
use cosmic::iced::keyboard::Modifiers;
use cosmic::widget::menu::Action as _;
use futures_util::SinkExt;
use ssh::create_client;
use types::AppError;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

pub mod config;
pub mod icons;
pub mod localize;
pub mod menu;
pub mod settings;
pub mod ssh;
pub mod types;
pub mod utils;
pub mod key_bind;

pub struct Tbgui {
    core: Core,
    nav_model: nav_bar::Model,
    client: Option<Client>,
    content: Content,
    config_handler: Option<cosmic_config::Config>,
    config: TbguiConfig,
    app_themes: Vec<String>,
    context_page: ContextPage,
    key_binds: HashMap<KeyBind, Action>,
    modifiers: Modifiers,
    dialog_pages: VecDeque<DialogPage>,
    dialog_text_input: widget::Id,
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateClient,
    CreatedClient(Result<Client, AppError>),
    LoadRemoteState,
    LoadedRemoteState(Vec<Sample>),

    Content(content::Message),
    Application(ApplicationAction),

    RunTbProfiler,
    OpenRepositoryUrl,
    SubscriptionChannel,
    UpdateConfig(TbguiConfig),
    LaunchUrl(String),
    Error(AppError),
    DialogCancel,
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: TbguiConfig,
}

impl Tbgui {
    fn settings(&self) -> Element<Message> {
        widget::scrollable(widget::settings::section().title(fl!("appearance")).add(
            widget::settings::item::item(
                fl!("theme"),
                widget::dropdown(
                    &self.app_themes,
                    Some(self.config.app_theme.into()),
                    |theme| Message::Application(ApplicationAction::AppTheme(theme)),
                ),
            ),
        ))
        .into()
    }

    // fn create_nav_item(&mut self, list: &List) -> EntityMut<SingleSelect> {
    //     self.nav_model
    //         .insert()
    //         .text(format!(
    //             "{} {}",
    //             list.icon
    //                 .clone()
    //                 .unwrap_or(emojis::get_by_shortcode("pencil").unwrap().to_string()),
    //             list.name.clone()
    //         ))
    //         .data(list.clone())
    // }
}

impl cosmic::Application for Tbgui {
    type Executor = cosmic::executor::Default;
    type Flags = Flags;
    type Message = Message;
    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "ch.uzh.michael.tbgui";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let mut commands = vec![];

        // Construct the app model with the runtime's core.
        let mut app = Tbgui {
            core,
            context_page: ContextPage::About,
            nav_model: get_nav_model(&flags),
            client: None,
            content: Content::new(),
            config_handler: flags.config_handler,
            config: flags.config,
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            dialog_pages: VecDeque::new(),
            dialog_text_input: widget::Id::unique(),
        };

        println!("CreateClient");
        commands.push(Task::done(cosmic::Action::App(Message::CreateClient)));

        app.core.nav_bar_set_toggled(false);

        let command = app.update_title();
        commands.push(command);

        (app, Task::batch(commands))
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu::menu_bar(&self.key_binds)]
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }
        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::Application(ApplicationAction::ToggleContextDrawer),
            )
            .title(self.context_page.title()),
            ContextPage::Settings => context_drawer::context_drawer(
                self.settings(),
                Message::Application(ApplicationAction::ToggleContextDrawer),
            )
            .title(self.context_page.title()),
        })
    }

    fn dialog(&self) -> Option<Element<Message>> {
        let dialog_page = self.dialog_pages.front()?;
        let dialog = dialog_page.view(&self.dialog_text_input);
        Some(dialog.into())
    }

    fn view(&self) -> Element<Self::Message> {
        let page_view = match self.nav_model.active_data::<NavPage>() {
            Some(NavPage::RunTbProfiler) => self.content.view().map(Message::Content),
            Some(NavPage::DownloadResults) => self.view_settings(),
            Some(NavPage::DeleteResults) => self.view_settings(),
            Some(NavPage::Settings) => self.view_settings(),
            None => cosmic::widget::text("Unkown page selected.").into(),
        };
        page_view
            .apply(widget::container)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        struct MySubscription;

        Subscription::batch(vec![
            // Create a subscription which emits updates through a channel.
            Subscription::run_with_id(
                std::any::TypeId::of::<MySubscription>(),
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    _ = channel.send(Message::SubscriptionChannel).await;

                    futures_util::future::pending().await
                }),
            ),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<TbguiConfig>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        let mut commands = vec![];
        match message {
            Message::CreateClient => {
                println!("CreateClient2");
                let config = self.config.clone();
                let command = Task::perform(
                    async move {
                        println!("CreateClient3");
                        create_client(&config).await
                    },
                    |client| match client {
                        Ok(client) => cosmic::Action::App(Message::CreatedClient(Ok(client))),
                        Err(err) => cosmic::Action::App(Message::CreatedClient(Err(
                            AppError::Network(err.to_string()),
                        ))),
                    },
                );
                commands.push(command);
            }
            Message::CreatedClient(result) => {
                println!("Created client");
                match result {
                    Ok(client) => {
                        self.client = Some(client);
                    }
                    Err(err) => {
                        eprintln!("Error creating client: {}", err);
                        self.dialog_pages
                            .push_back(DialogPage::Info(AppError::Network(err.to_string())));
                    }
                }
                commands.push(Task::done(cosmic::Action::App(Message::LoadRemoteState)));
            }
            Message::LoadRemoteState => {
                let client = self.client.clone();
                let config = self.config.clone();
                let command = Task::perform(
                    async move {
                        if let Some(client) = client {
                            Sample::get_raw_reads(&client, &config).await
                        } else {
                            Err(AppError::Network("Client not initialized".to_string()))
                        }
                    },
                    |result| match result {
                        Ok(remote_state) => {
                            cosmic::Action::App(Message::LoadedRemoteState(remote_state))
                        }
                        Err(err) => {
                            cosmic::Action::App(Message::Error(AppError::Network(err.to_string())))
                        }
                    },
                );
                commands.push(command);
            }
            Message::LoadedRemoteState(result) => {
                let items = result.clone();
                //commands.push(Task::done(cosmic::Action::App(Message::Content(content::Message::SetItems(Vec::new())))));
                //commands.push(Task::done(cosmic::Action::App(Message::Content(content::Message::SetItems(items)))));
                let message = Message::Content(content::Message::SetItems(items));
                return self.update(message);
            }
            Message::Content(message) => {
                let content_items = self.content.update(message);
                for content_item in content_items {
                    match content_item {
                        content::TaskMessage::Get(list_id) => {}
                        content::TaskMessage::Update(task) => {}
                    }
                }
            }
            Message::RunTbProfiler => {
                //TODO: fetch raw sequences first
            }
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }

            Message::SubscriptionChannel => {
                // For example purposes only.
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
            Message::Error(err) => {
                eprintln!("Error: {}", err);
                self.dialog_pages.pop_front();
                self.dialog_pages.push_back(DialogPage::Info(err));
            }
            Message::DialogCancel => {
                self.dialog_pages.pop_front();
            }
            Message::Application(application_action) => match application_action {
                ApplicationAction::WindowClose => {
                    if let Some(window_id) = self.core.main_window_id() {
                        commands.push(cosmic::iced::window::close(window_id));
                    }
                }
                ApplicationAction::WindowNew => match env::current_exe() {
                    Ok(exe) => match process::Command::new(&exe).spawn() {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("failed to execute {exe:?}: {err}");
                        }
                    },
                    Err(err) => {
                        eprintln!("failed to get current executable path: {err}");
                    }
                },

                ApplicationAction::AppTheme(theme) => {
                    if let Some(handler) = &self.config_handler {
                        if let Err(err) = self.config.set_app_theme(&handler, theme.into()) {
                            tracing::error!("{err}")
                        }
                    }
                }
                ApplicationAction::SystemThemeModeChange => {}  //TODO: implement this
                ApplicationAction::Key(modifiers, key) => {
                    for (key_bind, action) in &self.key_binds {
                        if key_bind.matches(modifiers, &key) {
                            return self.update(action.message());
                        }
                    }
                }
                ApplicationAction::Modifiers(modifiers) => {
                    self.modifiers = modifiers;
                }
                ApplicationAction::Focus(_) => {}

                ApplicationAction::ToggleContextPage(context_page) => {
                    if self.context_page == context_page {
                        self.core.window.show_context = !self.core.window.show_context;
                    } else {
                        self.context_page = context_page;
                        self.core.window.show_context = true;
                    }
                }
                ApplicationAction::ToggleContextDrawer => {
                    self.core.window.show_context = !self.core.window.show_context
                }
            },
        }
        Task::batch(commands)
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.nav_model.activate(id);
        self.update_title()
    }
}

impl Tbgui {
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav_model.text(self.nav_model.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    fn save_config(&mut self) -> Task<Message> {
        if let Some(ref config_handler) = self.config_handler {
            if let Err(err) = self.config.write_entry(config_handler) {
                log::error!("failed to save config: {}", err);
            }
        }

        Task::none()
    }

    fn save_theme(&self) -> Task<Message> {
        Task::none()
        //cosmic::app::command::set_theme(self.config.app_theme.theme())
        //TODO: use the above command to set the theme in the app
    }
}
