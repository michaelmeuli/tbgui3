use crate::content::{self, Content};
use crate::fl;
use crate::model::sample::Item;
use crate::model::sample::ProfilerTask;
use crate::model::sample::RemoteState;
use crate::views::nav::{get_nav_model, Action, ContextPage, NavPage};
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
use futures_util::SinkExt;
use ssh::create_client;
use std::collections::{HashMap, VecDeque};
use types::{AppError, DialogPage};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

pub mod config;
pub mod icon_cache;
pub mod localize;
pub mod menu;
pub mod settings;
pub mod ssh;
pub mod types;
pub mod utils;

pub struct App {
    core: Core,
    context_page: ContextPage,
    nav_model: nav_bar::Model,
    client: Option<Client>,
    pub items: Vec<Item>,
    content: Content,
    key_binds: HashMap<KeyBind, Action>,
    config: TbguiConfig,
    config_handler: Option<cosmic_config::Config>,
    dialog_pages: VecDeque<DialogPage>,
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateClient,
    CreatedClient(Result<Client, AppError>),
    LoadRemoteState,
    LoadedRemoteState(RemoteState),
    LoadRemoteState2,
    LoadedRemoteState2(Vec<ProfilerTask>),
    Content(content::Message),
    RunTbProfiler,
    OpenRepositoryUrl,
    SubscriptionChannel,
    ToggleContextPage(ContextPage),
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

impl cosmic::Application for App {
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
        let mut app = App {
            core,
            context_page: ContextPage::default(),
            nav_model: get_nav_model(&flags),
            client: None,
            items: Vec::new(),
            content: Content::new(),
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, TbguiConfig::VERSION)
                .map(|context| match TbguiConfig::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            config_handler: flags.config_handler,
            dialog_pages: VecDeque::new(),
        };

        println!("CreateClient");
        commands.push(Task::done(cosmic::Action::App(Message::CreateClient)));

        app.core.nav_bar_set_toggled(false);

        // TODO: remove as replaced by content
        // if app.items.is_empty() {
        //     commands.push(app.update_rawreads_data().map(cosmic::Action::App));
        // }

        // Create a startup command that sets the window title.  //TODO?
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
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
        })
    }

    fn dialog(&self) -> Option<Element<Message>> {
        let dialog_page = match self.dialog_pages.front() {
            Some(some) => some,
            None => return None,
        };

        let dialog = match dialog_page {
            DialogPage::Info(app_errored) => {
                let mut content = widget::column::with_capacity(2).spacing(12);

                match app_errored {
                    AppError::Network(body) => {
                        let title = widget::text::title4("Network error");
                        content = content.push(title);
                        content = content.push(widget::text(body));
                    }
                    AppError::NoItemsChecked(body) => {
                        let title =
                            widget::text::title4("Cannot run tbprofiler with zero items checked");
                        content = content.push(title);
                        content = content.push(widget::text::body(body));
                    }
                    AppError::IO(body) => {
                        let title = widget::text::title4("IO error");
                        content = content.push(title);
                        content = content.push(widget::text(body));
                    }
                }
                widget::dialog()
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(content)
            }
        };
        Some(dialog.into())
    }

    fn view(&self) -> Element<Self::Message> {
        let page_view = match self.nav_model.active_data::<NavPage>() {
            Some(NavPage::RunTbProfiler) => self.view_raw_sequences(),
            Some(NavPage::DownloadResults) => self.content.view().map(Message::Content),
            Some(NavPage::DeleteResults) => self.view_raw_sequences2(),
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
                        Err(err) => cosmic::Action::App(Message::CreatedClient(Err(AppError::Network(err.to_string())))),
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
                        self.dialog_pages.push_back(DialogPage::Info(AppError::Network(err.to_string())));
                    }
                }
                commands.push(Task::done(cosmic::Action::App(Message::LoadRemoteState))); 
                commands.push(Task::done(cosmic::Action::App(Message::LoadRemoteState2)));
            }
            Message::LoadRemoteState => {
                let client = self.client.clone();
                let config = self.config.clone();
                let command = Task::perform(
                    async move {
                        if let Some(client) = client {
                            Item::get_raw_reads(&client, &config).await
                        } else {
                            Err(AppError::Network("Client not initialized".to_string()))
                        }
                    },
                    |result| match result {
                        Ok(remote_state) => cosmic::Action::App(Message::LoadedRemoteState(remote_state)),
                        Err(err) => cosmic::Action::App(Message::Error(AppError::Network(err.to_string()))),
                    },
                );
                commands.push(command);
                
            }
            Message::LoadRemoteState2 => {
                let client = self.client.clone();
                let config = self.config.clone();
                let command = Task::perform(
                    async move {
                        if let Some(client) = client {
                            ProfilerTask::get_raw_reads(&client, &config).await
                        } else {
                            Err(AppError::Network("Client not initialized".to_string()))
                        }
                    },
                    |result| match result {
                        Ok(remote_state) => cosmic::Action::App(Message::LoadedRemoteState2(remote_state)),
                        Err(err) => cosmic::Action::App(Message::Error(AppError::Network(err.to_string()))),
                    },
                );
                commands.push(command);
                
            }
            Message::LoadedRemoteState(result) => {
                println!("Loaded remote state: {:?}", result);
                self.items = result.items;
            }
            Message::LoadedRemoteState2(result) => {
                let items = result.clone();
                //commands.push(Task::done(cosmic::Action::App(Message::Content(content::Message::SetItems(Vec::new())))));
                //commands.push(Task::done(cosmic::Action::App(Message::Content(content::Message::SetItems(items)))));
                let message = Message::Content(content::Message::SetItems(items));
                return self.update(message)
            }
            Message::Content(message) => {
                let content_items = self.content.update(message);
                for content_item in content_items {
                    match content_item {
                        content::TaskMessage::Get(list_id) => {

                        }
                        content::TaskMessage::Update(task) => {

                        }
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

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
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
        }
        Task::batch(commands)
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        self.nav_model.activate(id);
        self.update_title()
    }
}

impl App {
    // TODO: remove as replaced by content
    pub fn update_rawreads_data(&self) -> Task<Message> {
        let client = self.client.clone();
        let config = self.config.clone();
        if let Some(client) = client {
            Task::perform(
                async move { Item::get_raw_reads(&client, &config).await },
                |result| match result {
                    Ok(remote_state) => Message::LoadedRemoteState(remote_state),
                    Err(err) => Message::Error(AppError::Network(err.to_string())),
                },
            )
        } else {
            Task::none()
        }
    }

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
