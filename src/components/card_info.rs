use crate::logic::types::{CardEntry, CardType, Rarity};
use chrono::{DateTime, Local};
use float_pretty_print::PrettyPrintFloat;
use serde_derive::{Deserialize, Serialize};
use std::cmp;
use std::iter::Filter;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, ToString};
use wasm_bindgen::prelude::*;
use yew::events::KeyboardEvent;
use yew::format::Json;
use yew::prelude::*;
use yew::web_sys::HtmlInputElement as InputElement;
use yew::{html, Component, ComponentLink, Href, Html, InputData, NodeRef, ShouldRender};

/// The root component of cr-tools
pub struct CardInfo {
    pub props: Props,
    link: ComponentLink<Self>,
    clean: bool,
    card_backup: CardEntry,
}

pub enum Msg {
    Update,
    Cancel,
    UpdateName(String),
    UpdateLevel(usize),
    UpdateHave(usize),
    UpdateRarity(Rarity),
}

#[derive(Properties, Clone)]
pub struct Props {
    pub card: CardEntry,
    pub on_update: Callback<CardEntry>,
}

impl Component for CardInfo {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            card_backup: props.card.clone(),
            props,
            link,
            clean: true,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.clean = false;

        match msg {
            Msg::UpdateName(name) => self.props.card.name = name,
            Msg::UpdateLevel(level) => self.props.card.level = level,
            Msg::UpdateHave(have) => self.props.card.have = have,
            Msg::UpdateRarity(rarity) => self.props.card.rarity = rarity,
            Msg::Update => {
                // Give the new card to the listing component
                self.props.on_update.emit(self.props.card.clone());

                // Set as clean
                self.clean = true;
            }
            Msg::Cancel => {
                // Restore the card from the backup
                self.props.card = self.card_backup.clone();

                // Set as clean
                self.clean = true;
            }
        }

        // Re-render
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        if self.clean {
            if let Some(data) = &self.props.card.computed {
                // Handle non-legendary cards

                let get_date = |date: DateTime<Local>| date.date().format("%F");

                html! {
                    <>

                    // The input fields for new cards
                    { self.view_inputs() }

                    // The calculated outputs for the card
                    <span>{"Need: "} {self.props.card.get_needed()}</span>
                    <span>{"Remaining: "} {data.cards_remaining}</span>
                    <span>{"Requests: "} {data.requests_remaining}</span>
                    <span>{"Weeks: "} {Self::simple_round(data.weeks_remaining.clone())}</span>
                    <span>{"Days: "} {Self::simple_round(data.days_remaining.clone())}</span>
                    <span>{"Days in order: "} {Self::simple_round(data.days_in_order.unwrap().clone())}</span>
                    <span>{"Done on: "} {get_date(data.done_on)}</span>
                    <span>{"Done in order: "} {get_date(data.done_in_order_on.unwrap())}</span>

                    </>
                }
            } else {
                // Handle legendary cards

                let cards_remaining = if self.props.card.get_needed() < self.props.card.have {
                    0
                } else {
                    self.props.card.get_needed() - self.props.card.have
                };

                html! {
                    <>

                    // The input fields for new cards
                    { self.view_inputs() }

                    // The calculated outputs for the card
                    <span>{"Need: "} {self.props.card.get_needed()}</span>
                    <span>{"Remaining: "} { cards_remaining }</span>
                    <span>{"Requests: n/a"}</span>
                    <span>{"Weeks: n/a"}</span>
                    <span>{"Days: n/a"}</span>
                    <span>{"Days in order: n/a"}</span>
                    <span>{"Done on: n/a"}</span>
                    <span>{"Done in order: n/a"}</span>

                    </>
                }
            }
        } else {
            // Handle editing
            html! {
                <>

                // The input fields for new cards
                { self.view_inputs() }

                // Save edits button
                <button onclick=self.link.callback(|_| Msg::Update)> {"Save"} </button>

                // Cancel button
                <button onclick=self.link.callback(|_| Msg::Cancel)> {"Cancel"} </button>

                // Padding
                <span/>
                <span/>
                <span/>
                <span/>
                <span/>
                <span/>

                </>
            }
        }
    }
}

impl CardInfo {
    fn simple_round(number: f64) -> String {
        format!("{:.3}", PrettyPrintFloat(number))
    }

    fn get_rarities(&self, card: Option<&CardEntry>) -> Html {
        // TODO cache/memoize this

        Rarity::iter()
            .map(|rarity| {
                let name = format!("{:?}", rarity);

                let should_select = if let Some(c) = card {
                    c.rarity == rarity
                } else {
                    false
                };

                html! {<option value=name selected={should_select}> {name} </option>}
            })
            .collect::<Html>()
    }

    /// Renders the input elements
    fn view_inputs(&self) -> Html {
        html! {
            <>

            <input
                type="text"
                placeholder="name"
                value={self.props.card.name.to_owned()}
                oninput=self.link.callback(|i: InputData| Msg::UpdateName(i.value))
            />

            <input
                type="number"
                placeholder="level"
                value={self.props.card.level}
                oninput=self.link.callback(|i: InputData| Msg::UpdateLevel(i.value.parse::<usize>().unwrap()))
            />

            <input
                type="number"
                placeholder="have"
                value={self.props.card.have}
                oninput=self.link.callback(|i: InputData| Msg::UpdateHave(i.value.parse::<usize>().unwrap()))
            />

            <select onchange=self.link.callback(|event: ChangeData| {
                if let yew::events::ChangeData::Select(data) = event {
                    Msg::UpdateRarity(Rarity::from_str(&data.value()).unwrap())
                } else {
                    panic!("Big oof");
                }
            }) >
                { self.get_rarities(Some(&self.props.card)) }
            </select>

            </>
        }
    }
}
