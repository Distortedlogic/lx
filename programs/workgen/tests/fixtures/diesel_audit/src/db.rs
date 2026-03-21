use diesel::prelude::*;

fn get_user_orders(conn: &mut PgConnection, user_id: i32) -> Vec<Order> {
    let user = users::table.filter(users::id.eq(user_id)).first::<User>(conn).unwrap();
    let orders = orders::table.filter(orders::user_id.eq(user.id)).load::<Order>(conn).unwrap();
    orders
}

fn update_status(conn: &mut PgConnection, id: i32, new_status: &str) {
    diesel::update(items::table.filter(items::id.eq(id)))
        .set(items::status.eq(new_status))
        .execute(conn)
        .unwrap();
    diesel::update(items::table.filter(items::id.eq(id)))
        .set(items::updated_at.eq(diesel::dsl::now))
        .execute(conn)
        .unwrap();
}

fn batch_upsert(conn: &mut PgConnection, records: Vec<NewRecord>) {
    for record in &records {
        diesel::insert_into(records::table)
            .values(record)
            .on_conflict(records::key)
            .do_update()
            .set(record)
            .execute(conn)
            .unwrap();
    }
}

fn get_active_items(conn: &mut PgConnection) -> Vec<Item> {
    let all_items = items::table.load::<Item>(conn).unwrap();
    let active: Vec<Item> = all_items.into_iter().filter(|i| i.status == "active").collect();
    active
}

fn fetch_report(conn: &mut PgConnection, query: &str) -> Vec<ReportRow> {
    diesel::sql_query(format!(
        "SELECT u.name, COUNT(o.id) as order_count FROM users u JOIN orders o ON o.user_id = u.id WHERE u.status = '{}' GROUP BY u.name",
        query
    ))
    .load::<ReportRow>(conn)
    .unwrap()
}
