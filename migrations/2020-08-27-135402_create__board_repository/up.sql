create table board_repository (
    id serial primary key,
    board_id bigint not null,
    repository_id bigint not null,
    constraint uq_board_repository
        unique (board_id, repository_id)
);
