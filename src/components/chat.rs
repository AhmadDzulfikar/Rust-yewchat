use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss.tx.clone().try_send(serde_json::to_string(&message).unwrap()) {
            log::debug!("message sent successfully");
        }

        Self {
            users: Vec::new(),
            messages: Vec::new(),
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                if let Ok(msg) = serde_json::from_str::<WebSocketMessage>(&s) {
                    match msg.message_type {
                        MsgTypes::Users => {
                            let users_from_message = msg.data_array.unwrap_or_default();
                            self.users = users_from_message
                                .iter()
                                .map(|u| UserProfile {
                                    name: u.clone(),
                                    avatar: format!(
                                        "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                        u
                                    ),
                                })
                                .collect();
                            true
                        }
                        MsgTypes::Message => {
                            if let Some(data) = msg.data {
                                if let Ok(message_data) = serde_json::from_str::<MessageData>(&data) {
                                    self.messages.push(message_data);
                                    return true;
                                }
                            }
                            false
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Msg::SubmitMessage => {
                if let Some(input) = self.chat_input.cast::<HtmlInputElement>() {
                    let message_text = input.value().trim().to_string();
                    if !message_text.is_empty() {
                        let message = WebSocketMessage {
                            message_type: MsgTypes::Message,
                            data: Some(message_text.clone()),
                            data_array: None,
                        };
                        if let Err(e) = self
                            .wss
                            .tx
                            .clone()
                            .try_send(serde_json::to_string(&message).unwrap())
                        {
                            log::debug!("error sending to channel: {:?}", e);
                        }
                        input.set_value("");
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
            <div class="flex w-screen h-screen font-sans text-gray-800">
                // Sidebar Users List
                <aside class="flex-none w-60 bg-gray-50 border-r border-gray-200 overflow-y-auto">
                    <h2 class="text-2xl font-semibold p-4 border-b border-gray-200">{"Users"}</h2>
                    <ul class="divide-y divide-gray-200">
                        { for self.users.iter().map(|u| html! {
                            <li class="flex items-center p-3 hover:bg-gray-100 cursor-pointer">
                                <img
                                    class="w-12 h-12 rounded-full mr-4"
                                    src={u.avatar.clone()}
                                    alt={format!("Avatar of {}", u.name)}
                                />
                                <div class="flex flex-col">
                                    <span class="font-medium">{ &u.name }</span>
                                    <span class="text-xs text-gray-500">{"Online"}</span>
                                </div>
                            </li>
                        })}
                    </ul>
                </aside>

                // Chat Area
                <main class="flex flex-col flex-grow bg-white">
                    <header class="flex items-center justify-between p-4 border-b border-gray-200 bg-gray-100">
                        <h1 class="text-xl font-semibold">{"💬 Chat!"}</h1>
                    </header>

                    <section class="flex-grow overflow-auto p-4 space-y-4 bg-gray-50">
                        { for self.messages.iter().map(|m| {
                            let user = self.users.iter().find(|u| u.name == m.from);

                            html! {
                                <div class="flex items-start space-x-3 max-w-xl">
                                    {
                                        if let Some(user) = user {
                                            html! {
                                                <img
                                                    class="w-10 h-10 rounded-full"
                                                    src={user.avatar.clone()}
                                                    alt={format!("Avatar of {}", user.name)}
                                                />
                                            }
                                        } else {
                                            html! {
                                                <div class="w-10 h-10 rounded-full bg-gray-300 flex items-center justify-center text-gray-600">
                                                    {"?"}
                                                </div>
                                            }
                                        }
                                    }

                                    <div>
                                        <div class="text-sm font-semibold">{ &m.from }</div>
                                        <div class="mt-1 text-gray-700 text-sm max-w-prose break-words">
                                            {
                                                if m.message.ends_with(".gif") {
                                                    html! {
                                                        <img class="rounded-md max-w-xs" src={m.message.clone()} alt="gif" />
                                                    }
                                                } else {
                                                    html! {
                                                        <p>{ &m.message }</p>
                                                    }
                                                }
                                            }
                                        </div>
                                    </div>
                                </div>
                            }
                        })}
                    </section>

                    <footer class="p-4 border-t border-gray-200 bg-white flex items-center space-x-3">
                        <input
                            ref={self.chat_input.clone()}
                            type="text"
                            placeholder="Type your message..."
                            class="flex-grow px-4 py-2 rounded-full border border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent"
                            autocomplete="off"
                        />
                        <button
                            onclick={submit}
                            class="bg-blue-600 hover:bg-blue-700 text-white rounded-full w-12 h-12 flex items-center justify-center shadow-md transition-colors duration-200"
                            aria-label="Send message"
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                class="w-6 h-6"
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10l9-6 9 6-9 6-9-6z" />
                            </svg>
                        </button>
                    </footer>
                </main>
            </div>
        }
    }
}
