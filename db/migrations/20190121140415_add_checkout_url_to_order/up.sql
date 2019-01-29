ALTER TABLE orders
    add checkout_url text;

ALTER TABLE orders
    add checkout_url_expires timestamp;

alter table payments
    alter column created_by drop not null;
