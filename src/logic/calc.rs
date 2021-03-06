use super::types::{get_request_size, Arena, CardEntry, Rarity, REQUEST_FREQUENCY};
use anyhow::{bail, Result};
use chrono::{DateTime, Duration, Local};
use std::cmp;
use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("One or more cards have missing values")]
    MissingCalculatedValues,
}

#[derive(PartialEq, Clone)]
pub struct CardData {
    pub cards_remaining: usize,
    pub requests_remaining: usize,
    pub weeks_remaining: f64,
    pub days_remaining: f64,
    pub done_on: DateTime<Local>,
    pub days_in_order: Option<f64>,
    pub done_in_order_on: Option<DateTime<Local>>,
}

impl CardEntry {
    pub fn calc_remaining(&self, arena: Option<&Arena>) -> Option<CardData> {
        // Cannot request legendary cards
        if self.rarity == Rarity::Legendary {
            return None;
        }

        // The arena the user is in (default to the LegendaryArena)
        let request_size = get_request_size(&arena.unwrap_or(&Arena::LegendaryArena));

        let cards_remaining = if self.get_needed_cards() < self.have {
            0
        } else {
            self.get_needed_cards() - self.have
        };

        let requests_remaining = (cards_remaining as f64
            / if self.rarity == Rarity::Common {
                request_size.common as f64
            } else {
                request_size.rare as f64
            })
        .ceil() as usize;

        let weeks_remaining = requests_remaining as f64
            / match self.rarity {
                Rarity::Common => REQUEST_FREQUENCY.common,
                Rarity::Rare => REQUEST_FREQUENCY.rare,
                Rarity::Epic => REQUEST_FREQUENCY.epic,
                Rarity::Legendary => REQUEST_FREQUENCY.legendary, // Unreachable
            } as f64;

        let days_remaining = weeks_remaining * 7 as f64;

        let done_on =
            Local::now().checked_add_signed(Duration::days(days_remaining.ceil() as i64))?;

        Some(CardData {
            cards_remaining,
            requests_remaining,
            weeks_remaining,
            days_remaining,
            done_on,
            days_in_order: None,
            done_in_order_on: None,
        })
    }

    pub fn compute_all(list: &mut Vec<Self>, arena: Option<&Arena>) {
        for card in list {
            card.computed = card.calc_remaining(arena);
        }
    }

    /// Custom order algorithm for sorting CardEntries by days
    //  FnMut(&Self, &Self) -> cmp::Ordering
    pub fn sort_by_remaining(
        arena: Option<&Arena>,
    ) -> impl FnMut(&Self, &Self) -> cmp::Ordering + '_ {
        move |a: &Self, b: &Self| {
            // Handle legendary cards
            if a.rarity == Rarity::Legendary {
                if b.rarity == Rarity::Legendary {
                    return cmp::Ordering::Equal;
                } else {
                    return cmp::Ordering::Greater;
                }
            }
            if b.rarity == Rarity::Legendary {
                return cmp::Ordering::Less;
            }

            let get_remaining = |card: &CardEntry| {
                card.computed
                    .clone()
                    .unwrap_or_else(|| card.calc_remaining(arena).unwrap())
                    .days_remaining
            };

            // Compare the cards
            get_remaining(a).partial_cmp(&get_remaining(b)).unwrap()
        }
    }

    pub fn sum_all(list: &mut Vec<Self>) -> Result<()> {
        let mut prev_time_regular = 0.;
        let mut prev_time_epic = 0.;

        for card in list {
            // Handle cards according to their respective rarities
            let prev_time = match card.rarity {
                Rarity::Common | Rarity::Rare => &mut prev_time_regular,
                Rarity::Epic => &mut prev_time_epic,
                Rarity::Legendary => {
                    // Skip legendary cards
                    continue;
                }
            };

            if let Some(data) = &mut card.computed {
                let current_time = data.days_remaining + *prev_time;

                data.done_in_order_on = Some(
                    Local::now()
                        .checked_add_signed(Duration::days(current_time.ceil() as i64))
                        .ok_or(MyError::MissingCalculatedValues)?,
                );

                data.days_in_order = Some(current_time);
                *prev_time = current_time;
            } else {
                bail!(MyError::MissingCalculatedValues);
            }
        }

        Ok(())
    }
}
