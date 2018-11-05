pub use self::artists::*;
pub use self::assets::*;
pub use self::codes::*;
pub use self::comps::*;
pub use self::domain_events::*;
pub use self::enums::*;
pub use self::event_artists::*;
pub use self::event_interest::*;
pub use self::events::*;
pub use self::external_logins::*;
pub use self::fee_schedule_ranges::*;
pub use self::fee_schedules::*;
pub use self::for_display::*;
pub use self::holds::*;
pub use self::order_items::*;
pub use self::orders::*;
pub use self::organization_invites::*;
pub use self::organization_users::*;
pub use self::organizations::*;
pub use self::paging::*;
pub use self::payment_methods::*;
pub use self::payments::*;
pub use self::redeemable_ticket::*;
pub use self::regions::*;
pub use self::scopes::*;
pub use self::ticket_instances::RedeemResults;
pub use self::ticket_instances::*;
pub use self::ticket_pricing::*;
pub use self::ticket_type_codes::*;
pub use self::ticket_types::*;
pub use self::users::*;
pub use self::venues::*;
pub use self::wallets::*;
use serde::{Deserialize, Deserializer};

pub mod concerns;

mod artists;
mod assets;
mod codes;
mod comps;
mod domain_events;
mod enums;
mod event_artists;
mod event_interest;
mod events;
mod external_logins;
mod fee_schedule_ranges;
mod fee_schedules;
mod for_display;
mod holds;
mod order_items;
mod orders;
mod organization_invites;
mod organization_users;
mod organizations;
mod paging;
mod payment_methods;
mod payments;
mod redeemable_ticket;
mod regions;
pub mod scopes;
mod ticket_instances;
mod ticket_pricing;
mod ticket_type_codes;
mod ticket_types;
mod users;
mod venues;
mod wallets;

pub fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}
