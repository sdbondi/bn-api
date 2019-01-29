alter table orders drop column checkout_url;
alter table orders drop column checkout_url_expires;
alter table payments
    alter column created_by set not null;