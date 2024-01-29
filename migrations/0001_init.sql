create table if not exists records (
    id integer primary key not null,
    slug text not null,
    created_at integer not null,
    messages text not null,
    done boolean not null default false,
    count integer not null default 0
);
