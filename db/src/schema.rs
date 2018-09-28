table! {
    artists (id) {
        id -> Uuid,
        organization_id -> Nullable<Uuid>,
        is_private -> Bool,
        name -> Text,
        bio -> Text,
        image_url -> Nullable<Text>,
        thumb_image_url -> Nullable<Text>,
        website_url -> Nullable<Text>,
        youtube_video_urls -> Array<Text>,
        facebook_username -> Nullable<Text>,
        instagram_username -> Nullable<Text>,
        snapchat_username -> Nullable<Text>,
        soundcloud_username -> Nullable<Text>,
        bandcamp_username -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    assets (id) {
        id -> Uuid,
        ticket_type_id -> Uuid,
        blockchain_name -> Text,
        blockchain_asset_id -> Nullable<Text>,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    event_artists (id) {
        id -> Uuid,
        event_id -> Uuid,
        artist_id -> Uuid,
        rank -> Int4,
        set_time -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    event_interest (id) {
        id -> Uuid,
        event_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    events (id) {
        id -> Uuid,
        name -> Text,
        organization_id -> Uuid,
        venue_id -> Nullable<Uuid>,
        created_at -> Timestamp,
        event_start -> Nullable<Timestamp>,
        door_time -> Nullable<Timestamp>,
        status -> Text,
        publish_date -> Nullable<Timestamp>,
        promo_image_url -> Nullable<Text>,
        additional_info -> Nullable<Text>,
        age_limit -> Nullable<Int4>,
        cancelled_at -> Nullable<Timestamp>,
        updated_at -> Timestamp,
    }
}

table! {
    external_logins (id) {
        id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamp,
        site -> Text,
        access_token -> Text,
        external_user_id -> Text,
        updated_at -> Timestamp,
    }
}

table! {
    fee_schedule_ranges (id) {
        id -> Uuid,
        fee_schedule_id -> Uuid,
        min_price -> Int8,
        fee_in_cents -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    fee_schedules (id) {
        id -> Uuid,
        name -> Text,
        version -> Int2,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    order_items (id) {
        id -> Uuid,
        order_id -> Uuid,
        item_type -> Text,
        quantity -> Int8,
        unit_price_in_cents -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        ticket_pricing_id -> Nullable<Uuid>,
        fee_schedule_range_id -> Nullable<Uuid>,
        parent_id -> Nullable<Uuid>,
    }
}

table! {
    orders (id) {
        id -> Uuid,
        user_id -> Uuid,
        status -> Text,
        order_type -> Text,
        expires_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    organization_invites (id) {
        id -> Uuid,
        organization_id -> Uuid,
        inviter_id -> Uuid,
        user_email -> Text,
        created_at -> Timestamp,
        security_token -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
        accepted -> Nullable<Int2>,
        updated_at -> Timestamp,
    }
}

table! {
    organizations (id) {
        id -> Uuid,
        owner_user_id -> Uuid,
        name -> Text,
        address -> Nullable<Text>,
        city -> Nullable<Text>,
        state -> Nullable<Text>,
        country -> Nullable<Text>,
        postal_code -> Nullable<Text>,
        phone -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        fee_schedule_id -> Uuid,
    }
}

table! {
    organization_users (id) {
        id -> Uuid,
        organization_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    payments (id) {
        id -> Uuid,
        order_id -> Uuid,
        created_by -> Uuid,
        status -> Text,
        payment_method -> Text,
        amount -> Int8,
        provider -> Text,
        external_reference -> Text,
        raw_data -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    regions (id) {
        id -> Uuid,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    ticket_holdings (id) {
        id -> Uuid,
        name -> Text,
        asset_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    ticket_instances (id) {
        id -> Uuid,
        asset_id -> Uuid,
        token_id -> Int4,
        ticket_holding_id -> Nullable<Uuid>,
        order_item_id -> Nullable<Uuid>,
        reserved_until -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    ticket_pricing (id) {
        id -> Uuid,
        ticket_type_id -> Uuid,
        name -> Text,
        status -> Text,
        price_in_cents -> Int8,
        start_date -> Timestamp,
        end_date -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    ticket_types (id) {
        id -> Uuid,
        event_id -> Uuid,
        name -> Text,
        status -> Text,
        start_date -> Timestamp,
        end_date -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Uuid,
        first_name -> Text,
        last_name -> Text,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        profile_pic_url -> Nullable<Text>,
        thumb_profile_pic_url -> Nullable<Text>,
        cover_photo_url -> Nullable<Text>,
        hashed_pw -> Text,
        password_modified_at -> Timestamp,
        created_at -> Timestamp,
        last_used -> Nullable<Timestamp>,
        active -> Bool,
        role -> Array<Text>,
        password_reset_token -> Nullable<Uuid>,
        password_reset_requested_at -> Nullable<Timestamp>,
        updated_at -> Timestamp,
    }
}

table! {
    venues (id) {
        id -> Uuid,
        region_id -> Nullable<Uuid>,
        organization_id -> Nullable<Uuid>,
        is_private -> Bool,
        name -> Text,
        address -> Nullable<Text>,
        city -> Nullable<Text>,
        state -> Nullable<Text>,
        country -> Nullable<Text>,
        postal_code -> Nullable<Text>,
        phone -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    wallets (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        organization_id -> Nullable<Uuid>,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(artists -> organizations (organization_id));
joinable!(assets -> ticket_types (ticket_type_id));
joinable!(event_artists -> artists (artist_id));
joinable!(event_artists -> events (event_id));
joinable!(event_interest -> events (event_id));
joinable!(event_interest -> users (user_id));
joinable!(events -> organizations (organization_id));
joinable!(events -> venues (venue_id));
joinable!(external_logins -> users (user_id));
joinable!(fee_schedule_ranges -> fee_schedules (fee_schedule_id));
joinable!(order_items -> fee_schedule_ranges (fee_schedule_range_id));
joinable!(order_items -> orders (order_id));
joinable!(order_items -> ticket_pricing (ticket_pricing_id));
joinable!(orders -> users (user_id));
joinable!(organization_invites -> organizations (organization_id));
joinable!(organization_users -> organizations (organization_id));
joinable!(organization_users -> users (user_id));
joinable!(organizations -> fee_schedules (fee_schedule_id));
joinable!(organizations -> users (owner_user_id));
joinable!(payments -> orders (order_id));
joinable!(payments -> users (created_by));
joinable!(ticket_holdings -> assets (asset_id));
joinable!(ticket_instances -> assets (asset_id));
joinable!(ticket_instances -> order_items (order_item_id));
joinable!(ticket_instances -> ticket_holdings (ticket_holding_id));
joinable!(ticket_pricing -> ticket_types (ticket_type_id));
joinable!(ticket_types -> events (event_id));
joinable!(venues -> organizations (organization_id));
joinable!(venues -> regions (region_id));
joinable!(wallets -> organizations (organization_id));
joinable!(wallets -> users (user_id));

allow_tables_to_appear_in_same_query!(
    artists,
    assets,
    event_artists,
    event_interest,
    events,
    external_logins,
    fee_schedule_ranges,
    fee_schedules,
    order_items,
    orders,
    organization_invites,
    organizations,
    organization_users,
    payments,
    regions,
    ticket_holdings,
    ticket_instances,
    ticket_pricing,
    ticket_types,
    users,
    venues,
    wallets,
);
