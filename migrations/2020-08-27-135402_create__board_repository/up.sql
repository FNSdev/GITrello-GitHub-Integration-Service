create table board_repository (
    id serial primary key,
    github_profile_id int not null references github_profile(id) on delete cascade,
    board_id bigint not null,
    repository_name varchar(100) not null,
    repository_owner varchar(100) not null,
    constraint uq_board_repository
        unique (board_id, repository_name, repository_owner)
);
