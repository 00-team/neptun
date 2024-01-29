
PRAGMA foreign_keys=off;

begin transaction;

create table if not exists records_new (
    id integer primary key not null,
    slug text not null,
    created_at integer not null,
    messages text not null default '{}',
    done boolean not null default false,
    count integer not null default 0
);

insert into records_new select * from records;

drop table if exists records;

alter table records_new rename to records;

commit;

PRAGMA foreign_keys=on;

