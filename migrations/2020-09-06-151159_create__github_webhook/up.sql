create table github_webhook (
    id serial primary key,
    webhook_id bigint not null,
    url varchar(100) not null,
    board_repository_id int not null references board_repository(id) on delete cascade,
    constraint uq_github_webhook unique (board_repository_id)
);
