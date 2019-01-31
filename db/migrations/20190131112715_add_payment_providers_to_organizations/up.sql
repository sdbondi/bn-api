alter table organizations
    add allowed_payment_providers Text[] not null DEFAULT '{"stripe"}'

