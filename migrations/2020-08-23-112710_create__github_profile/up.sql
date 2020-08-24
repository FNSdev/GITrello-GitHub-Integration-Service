create table github_profile (
    id serial primary key,
    user_id bigint not null unique,
    github_user_id bigint not null,
    github_login varchar(128) not null,
    access_token varchar(128) not null
);
