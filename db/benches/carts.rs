use bigneon_db::dev::*;
use bigneon_db::models::*;
use criterion::Criterion;
use diesel::prelude::*;
use dotenv::dotenv;
use rand;
use std::env;
use uuid::Uuid;

pub fn benchmark_carts(c: &mut Criterion) {
    let project = TestProject::new_without_rollback();
    let connection = project.get_connection();
    let organization = project.create_organization().finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_a_specific_number_of_tickets(1_000_000)
        .with_ticket_pricing()
        .finish();

    let ticket_type_id = event.ticket_types(true, None, connection).unwrap()[0].id;

    let mut users = Vec::<Uuid>::new();

    for _ in 0..1_000 {
        let user = project.create_user().finish();
        users.push(user.id);
    }

    dotenv().ok();
    let conn_str = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be defined.");

    c.bench_function_over_inputs(
        "Add to cart",
        move |b, &&max_purchases| {
            b.iter(|| {
                let connection = PgConnection::establish(&conn_str).unwrap();
                let x: usize = rand::random();
                let y: usize = rand::random();
                let cart = Order::create(users[x % 1_000], OrderTypes::Cart)
                    .commit(&connection)
                    .unwrap();
                cart.add_tickets(ticket_type_id, (y % max_purchases) as i64, &connection)
                    .unwrap();
            })
        },
        &[1, 4, 50],
    );
}
