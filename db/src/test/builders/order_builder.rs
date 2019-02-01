use diesel::prelude::*;
use models::*;
use test::builders::*;
use uuid::Uuid;

pub struct OrderBuilder<'a> {
    user: Option<User>,
    ticket_type_id: Option<Uuid>,
    connection: &'a PgConnection,
    quantity: u32,
    is_paid: bool,
    with_free_items: bool,
    on_behalf_of_user: Option<User>,
}

impl<'a> OrderBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> OrderBuilder<'a> {
        OrderBuilder {
            connection,
            user: None,
            ticket_type_id: None,
            quantity: 10,
            is_paid: false,
            with_free_items: false,
            on_behalf_of_user: None,
        }
    }

    pub fn for_user(mut self, user: &User) -> OrderBuilder<'a> {
        self.user = Some(user.clone());
        self
    }

    pub fn on_behalf_of_user(mut self, user: &User) -> OrderBuilder<'a> {
        self.on_behalf_of_user = Some(user.clone());
        self
    }

    pub fn for_event(mut self, event: &Event) -> OrderBuilder<'a> {
        self.ticket_type_id = Some(event.ticket_types(true, None, &self.connection).unwrap()[0].id);
        self
    }

    pub fn quantity(mut self, quantity: u32) -> OrderBuilder<'a> {
        self.quantity = quantity;
        self
    }

    pub fn is_paid(mut self) -> OrderBuilder<'a> {
        self.is_paid = true;
        self
    }

    pub fn with_free_items(mut self) -> OrderBuilder<'a> {
        self.with_free_items = true;
        self
    }

    pub fn finish(mut self) -> Order {
        if self.user.is_none() {
            let user = UserBuilder::new(self.connection).finish();
            self.user = Some(user);
        }
        if self.ticket_type_id.is_none() {
            let event = EventBuilder::new(self.connection)
                .with_ticket_pricing()
                .finish();
            self.ticket_type_id =
                Some(event.ticket_types(true, None, &self.connection).unwrap()[0].id);
        }

        let mut cart =
            Order::find_or_create_cart(self.user.as_ref().unwrap(), self.connection).unwrap();

        let redemption_code = if self.with_free_items {
            let comp = HoldBuilder::new(self.connection)
                .with_ticket_type_id(self.ticket_type_id.unwrap())
                .with_hold_type(HoldTypes::Comp)
                .finish();
            Some(comp.redemption_code)
        } else {
            None
        };

        let user = self.user.unwrap();

        cart.update_quantities(
            user.id,
            &[UpdateOrderItem {
                ticket_type_id: self.ticket_type_id.unwrap(),
                quantity: self.quantity,
                redemption_code,
            }],
            false,
            false,
            self.connection,
        )
        .unwrap();

        if let Some(on_behalf_of_user) = self.on_behalf_of_user {
            cart.set_behalf_of_user(on_behalf_of_user, user.id, self.connection)
                .unwrap();
        }

        let total = cart.calculate_total(self.connection).unwrap();

        let mut cart = cart;
        if self.is_paid {
            cart.add_external_payment(Some("blah".to_string()), user.id, total, self.connection)
                .unwrap();
        }

        cart
    }
}
